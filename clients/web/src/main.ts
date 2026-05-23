import { renderAssistantDrawer, type TextProofView } from "./drawer/assistant";
import { YggProtocolClient, type AssetRecord, type KernelEvent, type PackageRecord, type ProjectRecord, type ProjectionRecord, type ProposalRecord, type RegisteredCapability, type SurfaceContributionRecord } from "./protocol/client";
import { renderShell, type RouteName } from "./shell/shell";
import { renderForgeSurface } from "./surfaces/forge";
import { resolveSurfaceBundle } from "./surfaces/bundle-resolver";
import { mountSurface, type SurfaceHostHandle } from "./surfaces/surface-host";
import { renderHomeSurface } from "./surfaces/home";
import { renderPlaySurface } from "./surfaces/play";
import { buildAgentObservability, filterAgentLikeCapabilities, renderAgentReadinessPanel } from "./agent/observability.js";
import { buildExternalProjectAggregation, renderAssistantExternalProjectHints, type ExternalProjectAggregation } from "./projects/external-projects.js";
import { buildStorageInspectorModel, renderAssistantStorageHints, type StorageInspectorModel } from "./storage/storage-inspector.js";
import { buildMockChunks, createStreamingBuffer, getActiveTextEngine, getActiveTextEngineName, getTextEngineDiagnostics, initializeTextEnginePreference, getInitializationResult } from "./text-layout/index.js";
import { escapeHtml } from "./utils/html";

const app = document.querySelector<HTMLDivElement>("#app");
const client = new YggProtocolClient(location.origin === "null" ? "http://127.0.0.1:8787" : location.origin);

let route: RouteName = "home";
let sessionId: string | undefined;
let events: KernelEvent[] = [];
let surfaceEntries: SurfaceContributionRecord[] = [];
let projects: ProjectRecord[] = [];
let homeError: string | undefined;
let homeLoading = true;
let closeProjectEvents: (() => void) | undefined;
let latestSequence = 0;
let closeEvents: (() => void) | undefined;
let assistOpen = false;
let scheduledRender: ReturnType<typeof setTimeout> | undefined;
let pendingRenderError: string | undefined;
let activeMountedSurface: SurfaceHostHandle | null = null;

// --- Text Surface Proof state ---
const STREAM_PROOF_FONT = "16px Inter, Helvetica Neue, Arial, sans-serif";
const STREAM_PROOF_LINE_HEIGHT = 24;
const STREAM_PROOF_MAX_WIDTH = 340;

const MOCK_CHUNKS = buildMockChunks();
let streamBuffer = createStreamingBuffer(STREAM_PROOF_FONT, STREAM_PROOF_LINE_HEIGHT, STREAM_PROOF_MAX_WIDTH);
let streamChunkCursor = 0;
let streamTimer: ReturnType<typeof setInterval> | null = null;

// T3: Track engine initialization state
let engineInitDone = false;

let textProofView: TextProofView = {
  text: "",
  state: "idle",
  lineCount: 0,
  height: 0,
  chunkIndex: 0,
  totalChunks: MOCK_CHUNKS.length,
  engineName: getActiveTextEngineName(),
  engineVersion: getActiveTextEngine().config.version,
  engineState: getActiveTextEngine().state,
};

function syncTextProof() {
  const measured = streamBuffer.measure();
  const engine = getActiveTextEngine();
  const initResult = getInitializationResult();
  textProofView = {
    text: streamBuffer.text,
    state: streamBuffer.state,
    lineCount: measured.lineCount,
    height: measured.height,
    chunkIndex: streamChunkCursor,
    totalChunks: MOCK_CHUNKS.length,
    engineName: getActiveTextEngineName(),
    engineVersion: engine.config.version,
    engineState: engine.state,
    // T3 diagnostics
    enginePreference: initResult?.preference,
    fallbackReason: initResult?.fallbackReason,
    pretextAvailable: initResult?.pretextAvailable,
  };
}

function updateTextProofDOM() {
  const stage = document.querySelector<HTMLDivElement>(".text-proof-stage");
  const meta = document.querySelector<HTMLDivElement>(".text-proof-meta");
  if (!stage || !meta) return;
  const escaped = textProofView.text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");

  // Build badges for engine, preference, pretext availability, fallback reason
  const engineBadge = textProofView.engineName
    ? `<span class="text-proof-badge engine-badge">engine ${textProofView.engineName} v${textProofView.engineVersion ?? "?"} <span class="engine-state state-${textProofView.engineState}">${textProofView.engineState}</span></span>`
    : "";
  const preferenceBadge = textProofView.enginePreference
    ? `<span class="text-proof-badge pref-badge">pref ${textProofView.enginePreference}</span>`
    : "";
  const pretextBadge = textProofView.pretextAvailable !== undefined
    ? `<span class="text-proof-badge pretext-badge ${textProofView.pretextAvailable ? "pretext-available" : "pretext-unavailable"}">pretext ${textProofView.pretextAvailable ? "available" : "unavailable"}</span>`
    : "";
  const fallbackBadge = textProofView.fallbackReason
    ? `<span class="text-proof-badge fallback-reason-badge" title="${textProofView.fallbackReason.replace(/&/g, "&amp;").replace(/"/g, "&quot;")}">fallback: ${textProofView.fallbackReason.length > 40 ? textProofView.fallbackReason.slice(0, 39) + "…" : textProofView.fallbackReason}</span>`
    : "";

  stage.innerHTML = escaped
    ? `<p class="text-proof-content">${escaped}</p>`
    : `<p class="text-proof-placeholder">Tap replay to start mock stream…</p>`;
  meta.innerHTML = `
    ${engineBadge}
    ${preferenceBadge}
    ${pretextBadge}
    ${fallbackBadge}
    <span class="text-proof-badge state-${textProofView.state}">${textProofView.state}</span>
    <span class="text-proof-badge">lines ${textProofView.lineCount}</span>
    <span class="text-proof-badge">height ${Math.round(textProofView.height)}px</span>
    <span class="text-proof-badge">chunks ${textProofView.chunkIndex}/${textProofView.totalChunks}</span>
  `;
  const replayBtn = document.querySelector<HTMLButtonElement>("[data-action='replay-stream-proof']");
  const resetBtn = document.querySelector<HTMLButtonElement>("[data-action='reset-stream-proof']");
  if (replayBtn) replayBtn.disabled = textProofView.state === "streaming";
  if (resetBtn) resetBtn.disabled = textProofView.state === "idle";
}

async function loadProjects() {
  try {
    homeLoading = true;
    homeError = undefined;
    scheduleRender();
    projects = await client.listProjects();
    homeLoading = false;
    scheduleRender();
  } catch (caught) {
    homeError = caught instanceof Error ? caught.message : String(caught);
    homeLoading = false;
    scheduleRender();
  }
}

function stopStreamProof() {
  if (streamTimer) {
    clearInterval(streamTimer);
    streamTimer = null;
  }
}

function startStreamProof() {
  stopStreamProof();
  streamBuffer.reset();
  streamChunkCursor = 0;
  streamBuffer.state = "streaming";
  syncTextProof();

  function tick() {
    if (streamChunkCursor >= MOCK_CHUNKS.length) {
      streamBuffer.end();
      syncTextProof();
      updateTextProofDOM();
      stopStreamProof();
      return;
    }
    streamBuffer.append(MOCK_CHUNKS[streamChunkCursor]!);
    streamChunkCursor++;
    syncTextProof();
    updateTextProofDOM();
  }

  tick();
  streamTimer = setInterval(tick, 120);
}

// --- T3: Async engine initialization ---

async function initEnginePreference() {
  if (engineInitDone) return;
  try {
    await initializeTextEnginePreference();
  } catch {
    // initializeTextEnginePreference handles its own fallback;
    // this catch is for any unexpected errors during the async flow.
  }
  engineInitDone = true;
  syncTextProof();
}

// --- Render ---

async function render(error?: string) {
  if (!app) return;
  let body = "";
  let diagnostics: Record<string, unknown> = {};
  let packages: PackageRecord[] = [];
  let allSurfaces: SurfaceContributionRecord[] = [];
  let proposals: ProposalRecord[] = [];
  let capabilities: RegisteredCapability[] = [];
  let externalProjects: ExternalProjectAggregation | undefined;
  let storageInspector: StorageInspectorModel | undefined;
  try {
    const [entries, caps, hostDiagnostics] = await Promise.all([
      client.surfaceContributions("experience_entry").catch(() => []),
      client.capabilities().catch(() => []),
      client.diagnostics().catch(() => ({})),
    ]);
    capabilities = caps;
    externalProjects = await buildExternalProjectAggregation(client, capabilities).catch((caught) => ({
      available: false,
      missing_capabilities: [],
      demo_source_ref: "",
      demo_workspace_ref: "",
      errors: [caught instanceof Error ? caught.message : String(caught)],
    }));
    storageInspector = await buildStorageInspectorModel(client, capabilities).catch((caught) => ({
      available: false,
      missing_capabilities: [],
      errors: [caught instanceof Error ? caught.message : String(caught)],
    }));
    const [assets, projections, props, forgeSurfaces, pkgs, surfaces] = route === "forge"
      ? await Promise.all([
          client.assets().catch(() => []),
          client.projections().catch(() => []),
          client.proposals().catch(() => []),
          client.surfaceContributions("forge_panel").catch(() => []),
          client.packages().catch(() => []),
          client.surfaceContributions().catch(() => []),
        ])
      : [[], [], [], [], [], []];
    packages = pkgs;
    allSurfaces = surfaces;
    proposals = props;
    surfaceEntries = entries;
    diagnostics = hostDiagnostics;
    body = renderRoute({ capabilities, events, assets, projections, proposals, forgeSurfaces, packages, allSurfaces, externalProjects, storageInspector });
  } catch (caught) {
    error = caught instanceof Error ? caught.message : String(caught);
    body = route === "home"
      ? renderHomeSurface({ projects, loading: homeLoading, error: homeError })
      : route === "play"
        ? renderPlaySurface([], sessionId, externalProjects)
        : route === "project"
          ? renderProjectPlaceholder()
          : renderForgeSurface({ capabilities: [], events, assets: [], projections: [], proposals: [], forgeSurfaces: [], packages: [], allSurfaces: [], sessionId, externalProjects, storageInspector });
  }
  // Build agent readiness panel for Assistant Drawer (lightweight, no real model/network)
  const observability = buildAgentObservability(
    packages,
    allSurfaces,
    events,
    proposals,
    capabilities,
  );
  const agentReadinessHtml = renderAgentReadinessPanel(
    observability.agentSurfaces,
    filterAgentLikeCapabilities(capabilities),
  );
  const externalProjectHtml = renderAssistantExternalProjectHints(externalProjects);
  const storageHtml = renderAssistantStorageHints(storageInspector);
  app.innerHTML = renderShell(route, body, renderAssistantDrawer(diagnostics, assistOpen, textProofView, agentReadinessHtml, `${externalProjectHtml}${storageHtml}`), error);
  wireEvents();
}

interface RenderRouteData {
  capabilities: RegisteredCapability[];
  events: KernelEvent[];
  assets: AssetRecord[];
  projections: ProjectionRecord[];
  proposals: ProposalRecord[];
  forgeSurfaces: SurfaceContributionRecord[];
  packages: PackageRecord[];
  allSurfaces: SurfaceContributionRecord[];
  externalProjects?: ExternalProjectAggregation;
  storageInspector?: StorageInspectorModel;
}

function renderRoute(data: RenderRouteData): string {
  switch (route) {
    case "home":
      return renderHomeSurface({ projects, loading: homeLoading, error: homeError });
    case "play":
      return renderPlaySurface(surfaceEntries, sessionId, data.externalProjects);
    case "project":
      return renderProjectPlaceholder();
    case "forge":
      return renderForgeSurface({
        capabilities: data.capabilities,
        events: data.events,
        assets: data.assets,
        projections: data.projections,
        proposals: data.proposals,
        forgeSurfaces: data.forgeSurfaces,
        packages: data.packages,
        allSurfaces: data.allSurfaces,
        sessionId,
        externalProjects: data.externalProjects,
        storageInspector: data.storageInspector,
      });
  }
}

function renderProjectPlaceholder(): string {
  return `
    <section class="surface project-mounted-surface">
      <div id="project-surface-container" class="project-surface-container">
        <div class="project-placeholder">Project started. Native surface mounting is available when its bundle mapping exists; external workspace mounting is a placeholder for Wave 4.</div>
      </div>
    </section>
  `;
}

function scheduleRender(error?: string) {
  if (error) pendingRenderError = error;
  if (scheduledRender) return;
  scheduledRender = setTimeout(() => {
    scheduledRender = undefined;
    const errorToRender = pendingRenderError;
    pendingRenderError = undefined;
    void render(errorToRender);
  }, 16);
}

async function handleMountSurfaceClick(packageId: string, surfaceId: string) {
  if (activeMountedSurface) {
    await activeMountedSurface.unmount();
    activeMountedSurface = null;
  }

  const record = surfaceEntries.find((entry) => entry.package_id === packageId && entry.surface.id === surfaceId);
  const outlet = document.getElementById("surface-outlet");
  const listArea = document.getElementById("surface-list-area");
  const inner = document.getElementById("surface-outlet-inner");
  if (!outlet || !inner) return;

  listArea?.classList.add("hidden");
  outlet.classList.remove("hidden");
  inner.innerHTML = "";

  try {
    const resolved = await resolveSurfaceBundle(client, surfaceId);
    activeMountedSurface = await mountSurface({
      containerId: "surface-outlet-inner",
      surfaceId,
      bundleUrl: resolved.bundleUrl,
      exportName: resolved.exportName,
      wrapperClass: resolved.wrapperClass,
      stylesheets: resolved.stylesheets,
      initialProps: {
        surfaceId,
        packageId: record?.package_id ?? packageId,
        sessionId,
      },
      hostBridge: {
        async callRpc(method, params) {
          return client.invoke(method, params);
        },
      },
    });
  } catch (caught) {
    inner.innerHTML = `<div class="surface-mount-error">Failed to mount: ${escapeHtml(caught instanceof Error ? caught.message : String(caught))}</div>`;
  }
}

async function handleUnmountSurfaceClick() {
  if (activeMountedSurface) {
    await activeMountedSurface.unmount();
    activeMountedSurface = null;
  }
  const inner = document.getElementById("surface-outlet-inner");
  if (inner) inner.innerHTML = "";
  document.getElementById("surface-outlet")?.classList.add("hidden");
  document.getElementById("surface-list-area")?.classList.remove("hidden");
}

async function mountProjectSurface(projectId: string, startedSessionId?: string) {
  const project = await client.getProject(projectId);
  const projectSessionId = startedSessionId ?? project.running_session_id;
  route = "project";
  scheduleRender();
  setTimeout(async () => {
    const container = document.getElementById("project-surface-container");
    if (!container) return;
    if (!project.entry_surface_id || project.type === "external_workspace") {
      container.innerHTML = `<div class="project-placeholder">${escapeHtml(project.title)} is running. Workspace/native surface mounting is a Wave 4 placeholder until bundle metadata is available.</div>`;
      return;
    }
    if (activeMountedSurface) await activeMountedSurface.unmount();
    container.innerHTML = "";
    try {
      const resolved = await resolveSurfaceBundle(client, project.entry_surface_id);
      activeMountedSurface = await mountSurface({
        containerId: "project-surface-container",
        surfaceId: resolved.surfaceId,
        bundleUrl: resolved.bundleUrl,
        exportName: resolved.exportName,
        wrapperClass: resolved.wrapperClass,
        stylesheets: resolved.stylesheets,
        hostBridge: {
          callRpc: (method, params) => {
            const rpcParams = (typeof params === "object" && params !== null) ? params : {};
            return projectSessionId
              ? client.invokeWithSession(method, rpcParams, projectSessionId)
              : client.invoke(method, rpcParams);
          },
        },
        initialProps: { projectId, sessionId: projectSessionId },
      });
    } catch (caught) {
      container.innerHTML = `<div class="project-placeholder">${escapeHtml(project.title)} is running. No bundle mapping found for ${escapeHtml(project.entry_surface_id)}: ${escapeHtml(caught instanceof Error ? caught.message : String(caught))}</div>`;
    }
  }, 0);
}

async function handleProjectAction(action: string, projectId?: string) {
  try {
    if (action === "play" && projectId) {
      const startResult = await client.startProject(projectId);
      await loadProjects();
      await mountProjectSurface(projectId, startResult.session_id);
    } else if (action === "stop" && projectId) {
      await client.stopProject(projectId);
      await loadProjects();
    } else if (action === "install") {
      scheduleRender("Install project wizard is available from the install-lab CLI/capability; web dialog is pending.");
    }
  } catch (caught) {
    scheduleRender(caught instanceof Error ? caught.message : String(caught));
  }
}

function wireEvents() {
  document.querySelectorAll<HTMLButtonElement>("[data-route]").forEach((button) => {
    button.addEventListener("click", async () => {
      await handleUnmountSurfaceClick();
      route = button.dataset.route as RouteName;
      scheduleRender();
    });
  });
  document.querySelectorAll<HTMLElement>("[data-action='play'], [data-action='stop'], [data-action='install']").forEach((element) => {
    element.addEventListener("click", () => {
      void handleProjectAction(element.dataset.action ?? "", element.dataset.projectId);
    });
  });
  document.querySelector<HTMLButtonElement>("[data-action='toggle-assist']")?.addEventListener("click", () => {
    assistOpen = !assistOpen;
    scheduleRender();
  });
  document.querySelector<HTMLButtonElement>("[data-action='open-session']")?.addEventListener("click", async () => {
    try {
      const session = await client.openSession();
      sessionId = session.id;
      events = await client.listEvents(session.id);
      closeEvents?.();
      closeEvents = client.subscribeEvents(session.id, (event) => {
        events = [...events, event].slice(-50);
        latestSequence = event.sequence;
        scheduleRender();
      });
      scheduleRender();
    } catch (caught) {
      scheduleRender(caught instanceof Error ? caught.message : String(caught));
    }
  });
  document.querySelectorAll<HTMLButtonElement>("[data-action='launch-surface']").forEach((button) => {
    button.addEventListener("click", async () => {
      try {
        const surfaceId = button.dataset.surfaceId;
        const record = surfaceEntries.find((entry) => entry.surface.id === surfaceId);
        if (!record) return;
        const template = record.surface.activation.session_template ?? {};
        const labels = Array.isArray(template.labels) ? template.labels.filter((item): item is string => typeof item === "string") : [record.surface.id];
        const session = await client.openSession(labels, { ...template, surface_id: record.surface.id, package_id: record.package_id });
        sessionId = session.id;
        if (record.surface.activation.launch_capability_id) {
          await client.invokeCapability(record.surface.activation.launch_capability_id, {}, record.package_id);
        }
        events = await client.listEvents(session.id);
        latestSequence = events.at(-1)?.sequence ?? 0;
        closeEvents?.();
        closeEvents = client.subscribeEvents(session.id, (event) => {
          events = [...events, event].slice(-50);
          latestSequence = event.sequence;
          scheduleRender();
        });
        scheduleRender();
      } catch (caught) {
        scheduleRender(caught instanceof Error ? caught.message : String(caught));
      }
    });
  });
  document.querySelectorAll<HTMLButtonElement>("[data-action='mount-surface']").forEach((button) => {
    button.addEventListener("click", async () => {
      try {
        const surfaceId = button.dataset.surfaceId;
        const packageId = button.dataset.packageId;
        if (!surfaceId || !packageId) return;
        await handleMountSurfaceClick(packageId, surfaceId);
      } catch (caught) {
        scheduleRender(caught instanceof Error ? caught.message : String(caught));
      }
    });
  });
  document.querySelector<HTMLButtonElement>("[data-action='unmount-surface']")?.addEventListener("click", () => {
    void handleUnmountSurfaceClick();
  });
  document.querySelector<HTMLButtonElement>("[data-action='fork-session']")?.addEventListener("click", async () => {
    if (!sessionId) return;
    try {
      const forked = await client.forkSession(sessionId, latestSequence || events.at(-1)?.sequence || 0, { forked_from: sessionId });
      sessionId = forked.id;
      events = await client.listEvents(forked.id);
      latestSequence = events.at(-1)?.sequence ?? 0;
      closeEvents?.();
      closeEvents = client.subscribeEvents(forked.id, (event) => {
        events = [...events, event].slice(-50);
        latestSequence = event.sequence;
        scheduleRender();
      });
      scheduleRender();
    } catch (caught) {
      scheduleRender(caught instanceof Error ? caught.message : String(caught));
    }
  });
  document.querySelectorAll<HTMLButtonElement>("[data-action='approve-proposal']").forEach((button) => {
    button.addEventListener("click", async () => {
      try {
        const proposalId = button.dataset.proposalId;
        if (proposalId) await client.approveProposal(proposalId);
        scheduleRender();
      } catch (caught) {
        scheduleRender(caught instanceof Error ? caught.message : String(caught));
      }
    });
  });
  document.querySelectorAll<HTMLButtonElement>("[data-action='apply-proposal']").forEach((button) => {
    button.addEventListener("click", async () => {
      try {
        const proposalId = button.dataset.proposalId;
        if (proposalId) await client.applyProposal(proposalId);
        scheduleRender();
      } catch (caught) {
        scheduleRender(caught instanceof Error ? caught.message : String(caught));
      }
    });
  });
  // Text Surface Proof controls
  document.querySelector<HTMLButtonElement>("[data-action='replay-stream-proof']")?.addEventListener("click", () => {
    if (textProofView.state !== "streaming") startStreamProof();
  });
  document.querySelector<HTMLButtonElement>("[data-action='reset-stream-proof']")?.addEventListener("click", () => {
    stopStreamProof();
    streamBuffer.reset();
    streamChunkCursor = 0;
    syncTextProof();
    updateTextProofDOM();
  });
}

// T3: Initialize engine preference on startup, then render.
// The fallback engine is available synchronously; the async init
// may switch to Pretext if available.
void loadProjects();
closeProjectEvents = client.subscribeEvents(undefined, (event) => {
  if (event.kind?.startsWith("kernel/v1/project.")) {
    void loadProjects();
  }
});
window.addEventListener("beforeunload", () => {
  closeEvents?.();
  closeProjectEvents?.();
});

initEnginePreference().then(() => render());
