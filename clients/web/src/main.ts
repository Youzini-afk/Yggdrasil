import { renderAssistantDrawer } from "./drawer/assistant";
import { YggProtocolClient, type KernelEvent } from "./protocol/client";
import { renderShell, type RouteName } from "./shell/shell";
import { renderForgeSurface } from "./surfaces/forge";
import { renderPlaySurface } from "./surfaces/play";

const app = document.querySelector<HTMLDivElement>("#app");
const client = new YggProtocolClient(location.origin === "null" ? "http://127.0.0.1:8787" : location.origin);

let route: RouteName = "play";
let sessionId: string | undefined;
let events: KernelEvent[] = [];
let closeEvents: (() => void) | undefined;
let assistOpen = false;

async function render(error?: string) {
  if (!app) return;
  let body = "";
  let diagnostics: Record<string, unknown> = {};
  try {
    const [packages, capabilities, hostDiagnostics] = await Promise.all([
      client.packages().catch(() => []),
      client.capabilities().catch(() => []),
      client.diagnostics().catch(() => ({})),
    ]);
    diagnostics = hostDiagnostics;
    body = route === "play" ? renderPlaySurface(packages) : renderForgeSurface(capabilities, events, sessionId);
  } catch (caught) {
    error = caught instanceof Error ? caught.message : String(caught);
    body = route === "play" ? renderPlaySurface([]) : renderForgeSurface([], events, sessionId);
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
        render();
      });
      render();
    } catch (caught) {
      render(caught instanceof Error ? caught.message : String(caught));
    }
  });
}

render();
