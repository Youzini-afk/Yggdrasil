import type { KernelEvent, PackageRecord, RegisteredCapability, SurfaceContributionRecord } from "../protocol/client";
import { renderForgeSurface } from "../surfaces/forge";

export interface RenderDiagnosticResult {
  event_count: number;
  html_bytes: number;
  elapsed_ms: number;
}

function nowMs(): number {
  return globalThis.performance && typeof globalThis.performance.now === "function"
    ? globalThis.performance.now()
    : Date.now();
}

export function buildMockKernelEvents(count: number): KernelEvent[] {
  return Array.from({ length: count }, (_, index) => ({
    id: `evt_mock_${index}`,
    session_id: "ses_render_diagnostics",
    sequence: index,
    writer_package_id: index % 3 === 0 ? "kernel" : "official/playable-creation-board",
    kind: index % 3 === 0 ? "kernel/event.mock" : "official/playable-creation-board/state.delta",
    payload: {
      index,
      summary: `Mock event ${index}`,
      nested: {
        status: index % 2 === 0 ? "ready" : "pending",
        markers: ["alpha", "beta", "gamma"],
      },
    },
    metadata: { diagnostic: true },
    created_at: new Date(0).toISOString(),
  }));
}

function mockPackages(): PackageRecord[] {
  return [
    { id: "official/playable-creation-board", version: "0.1.0", state: "ready", entry_kind: "rust_inproc", capability_count: 14, hook_count: 0 },
    { id: "official/experience-observability-lab", version: "0.1.0", state: "ready", entry_kind: "rust_inproc", capability_count: 8, hook_count: 0 },
  ];
}

function mockCapabilities(): RegisteredCapability[] {
  return [
    { capability_id: "official/playable-creation-board/launch", provider_package_id: "official/playable-creation-board", version: "0.1.0", streaming: false },
    { capability_id: "official/experience-observability-lab/session_health", provider_package_id: "official/experience-observability-lab", version: "0.1.0", streaming: false },
  ];
}

function mockSurfaces(): SurfaceContributionRecord[] {
  return [
    {
      package_id: "official/playable-creation-board",
      entry_kind: "rust_inproc",
      package_state: "ready",
      surface: {
        id: "official/playable-creation-board/forge-panel",
        version: "0.1.0",
        slot: "forge_panel",
        title: "Playable Board",
        description: "Render diagnostics fixture",
        activation: {},
        required_permissions: [],
        metadata: {},
      },
    },
  ];
}

export function runForgeRenderDiagnostics(eventCounts: number[] = [50, 500]): RenderDiagnosticResult[] {
  return eventCounts.map((eventCount) => {
    const started = nowMs();
    const html = renderForgeSurface({
      capabilities: mockCapabilities(),
      events: buildMockKernelEvents(eventCount),
      assets: [],
      projections: [],
      proposals: [],
      forgeSurfaces: mockSurfaces(),
      packages: mockPackages(),
      allSurfaces: mockSurfaces(),
      sessionId: "ses_render_diagnostics",
    });
    return {
      event_count: eventCount,
      html_bytes: html.length,
      elapsed_ms: nowMs() - started,
    };
  });
}
