import type { KernelEvent, RegisteredCapability } from "../protocol/client";
import { escapeHtml, formatJson } from "../utils/html";

export function renderForgeSurface(capabilities: RegisteredCapability[], events: KernelEvent[], sessionId?: string) {
  return `
    <section class="surface surface-forge" aria-labelledby="forge-title">
      <div class="workspace-column primary">
        <p class="eyebrow">Forge</p>
        <h1 id="forge-title">Agentic Creation Workspace</h1>
        <p class="session-chip">Session: ${sessionId ?? "not opened"}</p>
        <button type="button" data-action="open-session">Begin Experience Session</button>
        <div class="event-tail">
          <h2>Events</h2>
          ${events.length ? events.map(renderEvent).join("") : "<p class='empty'>Open a session to inspect events.</p>"}
        </div>
      </div>
      <aside class="workspace-column secondary">
        <h2>Capabilities</h2>
        <div class="capability-list">
          ${capabilities.length ? capabilities.map(renderCapability).join("") : "<p class='empty'>No capabilities discovered.</p>"}
        </div>
      </aside>
    </section>
  `;
}

function renderCapability(capability: RegisteredCapability) {
  return `
    <article class="capability-row">
      <strong>${escapeHtml(capability.capability_id)}</strong>
      <span>${escapeHtml(capability.provider_package_id)} · ${escapeHtml(capability.version)}</span>
    </article>
  `;
}

function renderEvent(event: KernelEvent) {
  return `
    <article class="event-row">
      <span>#${event.sequence}</span>
      <strong>${escapeHtml(event.kind)}</strong>
      <code>${formatJson(event.payload)}</code>
    </article>
  `;
}
