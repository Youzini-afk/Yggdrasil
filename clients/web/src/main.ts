import { renderAssistantDrawer, type TextProofView } from "./drawer/assistant";
import { YggProtocolClient, type KernelEvent, type PackageRecord, type ProposalRecord, type RegisteredCapability, type SurfaceContributionRecord } from "./protocol/client";
import { renderShell, type RouteName } from "./shell/shell";
import { renderForgeSurface } from "./surfaces/forge";
import { renderPlaySurface } from "./surfaces/play";
import { buildAgentObservability, filterAgentLikeCapabilities, renderAgentReadinessPanel } from "./agent/observability.js";
import { buildExternalProjectAggregation, renderAssistantExternalProjectHints, type ExternalProjectAggregation } from "./projects/external-projects.js";
import { buildMockChunks, createStreamingBuffer, getActiveTextEngine, getActiveTextEngineName, getTextEngineDiagnostics, initializeTextEnginePreference, getInitializationResult } from "./text-layout/index.js";

const app = document.querySelector<HTMLDivElement>("#app");
const client = new YggProtocolClient(location.origin === "null" ? "http://127.0.0.1:8787" : location.origin);

let route: RouteName = "play";
let sessionId: string | undefined;
let events: KernelEvent[] = [];
let surfaceEntries: SurfaceContributionRecord[] = [];
let latestSequence = 0;
let closeEvents: (() => void) | undefined;
let assistOpen = false;
let scheduledRender: ReturnType<typeof setTimeout> | undefined;
let pendingRenderError: string | undefined;

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
    body = route === "play"
      ? renderPlaySurface(surfaceEntries, sessionId, externalProjects)
      : renderForgeSurface({ capabilities, events, assets, projections, proposals, forgeSurfaces, packages, allSurfaces, sessionId, externalProjects });
  } catch (caught) {
    error = caught instanceof Error ? caught.message : String(caught);
    body = route === "play"
      ? renderPlaySurface([], sessionId, externalProjects)
      : renderForgeSurface({ capabilities: [], events, assets: [], projections: [], proposals: [], forgeSurfaces: [], packages: [], allSurfaces: [], sessionId, externalProjects });
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
  app.innerHTML = renderShell(route, body, renderAssistantDrawer(diagnostics, assistOpen, textProofView, agentReadinessHtml, externalProjectHtml), error);
  wireEvents();
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

function wireEvents() {
  document.querySelectorAll<HTMLButtonElement>("[data-route]").forEach((button) => {
    button.addEventListener("click", () => {
      route = button.dataset.route as RouteName;
      scheduleRender();
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
initEnginePreference().then(() => render());
