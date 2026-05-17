import { renderAssistantDrawer } from "./drawer/assistant";
import { YggProtocolClient, type KernelEvent, type PackageRecord, type SurfaceContributionRecord } from "./protocol/client";
import { renderShell, type RouteName } from "./shell/shell";
import { renderForgeSurface } from "./surfaces/forge";
import { renderPlaySurface } from "./surfaces/play";

const app = document.querySelector<HTMLDivElement>("#app");
const client = new YggProtocolClient(location.origin === "null" ? "http://127.0.0.1:8787" : location.origin);

let route: RouteName = "play";
let sessionId: string | undefined;
let events: KernelEvent[] = [];
let surfaceEntries: SurfaceContributionRecord[] = [];
let latestSequence = 0;
let closeEvents: (() => void) | undefined;
let assistOpen = false;

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
  app.innerHTML = renderShell(route, body, renderAssistantDrawer(diagnostics, assistOpen), error);
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
}

render();
