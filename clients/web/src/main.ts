import { renderAssistantDrawer, type TextProofView } from "./drawer/assistant";
import { YggProtocolClient, type KernelEvent, type PackageRecord, type SurfaceContributionRecord } from "./protocol/client";
import { renderShell, type RouteName } from "./shell/shell";
import { renderForgeSurface } from "./surfaces/forge";
import { renderPlaySurface } from "./surfaces/play";
import { buildMockChunks, createStreamingBuffer, getActiveTextEngine, getActiveTextEngineName, getTextEngineDiagnostics } from "./text-layout/index.js";

const app = document.querySelector<HTMLDivElement>("#app");
const client = new YggProtocolClient(location.origin === "null" ? "http://127.0.0.1:8787" : location.origin);

let route: RouteName = "play";
let sessionId: string | undefined;
let events: KernelEvent[] = [];
let surfaceEntries: SurfaceContributionRecord[] = [];
let latestSequence = 0;
let closeEvents: (() => void) | undefined;
let assistOpen = false;

// --- Text Surface Proof state ---
const STREAM_PROOF_FONT = "16px Inter, Helvetica Neue, Arial, sans-serif";
const STREAM_PROOF_LINE_HEIGHT = 24;
const STREAM_PROOF_MAX_WIDTH = 340;

const MOCK_CHUNKS = buildMockChunks();
let streamBuffer = createStreamingBuffer(STREAM_PROOF_FONT, STREAM_PROOF_LINE_HEIGHT, STREAM_PROOF_MAX_WIDTH);
let streamChunkCursor = 0;
let streamTimer: ReturnType<typeof setInterval> | null = null;

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
  stage.innerHTML = escaped
    ? `<p class="text-proof-content">${escaped}</p>`
    : `<p class="text-proof-placeholder">Tap replay to start mock stream…</p>`;
  meta.innerHTML = `
    ${textProofView.engineName ? `<span class="text-proof-badge engine-badge">engine ${textProofView.engineName} v${textProofView.engineVersion ?? "?"} <span class="engine-state state-${textProofView.engineState}">${textProofView.engineState}</span></span>` : ""}
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

// --- Render ---

async function render(error?: string) {
  if (!app) return;
  let body = "";
  let diagnostics: Record<string, unknown> = {};
  try {
    const [entries, capabilities, hostDiagnostics] = await Promise.all([
      client.surfaceContributions("experience_entry").catch(() => []),
      client.capabilities().catch(() => []),
      client.diagnostics().catch(() => ({})),
    ]);
    const [assets, projections, proposals, forgeSurfaces, packages, allSurfaces] = route === "forge"
      ? await Promise.all([
          client.assets().catch(() => []),
          client.projections().catch(() => []),
          client.proposals().catch(() => []),
          client.surfaceContributions("forge_panel").catch(() => []),
          client.packages().catch(() => []),
          client.surfaceContributions().catch(() => []),
        ])
      : [[], [], [], [], [], []];
    surfaceEntries = entries;
    diagnostics = hostDiagnostics;
    body = route === "play"
      ? renderPlaySurface(surfaceEntries, sessionId)
      : renderForgeSurface({ capabilities, events, assets, projections, proposals, forgeSurfaces, packages, allSurfaces, sessionId });
  } catch (caught) {
    error = caught instanceof Error ? caught.message : String(caught);
    body = route === "play"
      ? renderPlaySurface([], sessionId)
      : renderForgeSurface({ capabilities: [], events, assets: [], projections: [], proposals: [], forgeSurfaces: [], packages: [], allSurfaces: [], sessionId });
  }
  app.innerHTML = renderShell(route, body, renderAssistantDrawer(diagnostics, assistOpen, textProofView), error);
  wireEvents();
}

function wireEvents() {
  document.querySelectorAll<HTMLButtonElement>("[data-route]").forEach((button) => {
    button.addEventListener("click", () => {
      route = button.dataset.route as RouteName;
      render();
    });
  });
  document.querySelector<HTMLButtonElement>("[data-action='toggle-assist']")?.addEventListener("click", () => {
    assistOpen = !assistOpen;
    render();
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
        render();
      });
      render();
    } catch (caught) {
      render(caught instanceof Error ? caught.message : String(caught));
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
          render();
        });
        render();
      } catch (caught) {
        render(caught instanceof Error ? caught.message : String(caught));
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
        render();
      });
      render();
    } catch (caught) {
      render(caught instanceof Error ? caught.message : String(caught));
    }
  });
  document.querySelectorAll<HTMLButtonElement>("[data-action='approve-proposal']").forEach((button) => {
    button.addEventListener("click", async () => {
      try {
        const proposalId = button.dataset.proposalId;
        if (proposalId) await client.approveProposal(proposalId);
        render();
      } catch (caught) {
        render(caught instanceof Error ? caught.message : String(caught));
      }
    });
  });
  document.querySelectorAll<HTMLButtonElement>("[data-action='apply-proposal']").forEach((button) => {
    button.addEventListener("click", async () => {
      try {
        const proposalId = button.dataset.proposalId;
        if (proposalId) await client.applyProposal(proposalId);
        render();
      } catch (caught) {
        render(caught instanceof Error ? caught.message : String(caught));
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

render();
