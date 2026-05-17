import type { AssetRecord, KernelEvent, ProjectionRecord, ProposalRecord, RegisteredCapability, SurfaceContributionRecord } from "../protocol/client";
import { escapeHtml, formatJson } from "../utils/html";

export function renderForgeSurface(input: {
  capabilities: RegisteredCapability[];
  events: KernelEvent[];
  assets: AssetRecord[];
  projections: ProjectionRecord[];
  proposals: ProposalRecord[];
  forgeSurfaces: SurfaceContributionRecord[];
  sessionId?: string;
}) {
  const { capabilities, events, assets, projections, proposals, forgeSurfaces, sessionId } = input;
  return `
    <section class="surface surface-forge" aria-labelledby="forge-title">
      <div class="workspace-column primary">
        <div class="forge-header">
          <div>
            <p class="eyebrow">Forge</p>
            <h1 id="forge-title">Agentic Creation Workspace</h1>
            <p class="session-chip">Session: ${sessionId ?? "not opened"}</p>
          </div>
          <button type="button" data-action="open-session">${sessionId ? `New session` : "Begin Experience Session"}</button>
        </div>
        <div class="inspector-grid">
          <section>
            <h2>Package Forge Panels</h2>
            ${forgeSurfaces.length ? forgeSurfaces.map(renderSurface).join("") : "<p class='empty'>No package-contributed Forge panels.</p>"}
          </section>
          <section>
            <h2>Proposals</h2>
            ${proposals.length ? proposals.map(renderProposal).join("") : "<p class='empty'>No proposals yet.</p>"}
          </section>
        </div>
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
        <h2>Assets</h2>
        <div class="capability-list">${assets.length ? assets.map(renderAsset).join("") : "<p class='empty'>No assets.</p>"}</div>
        <h2>Projections</h2>
        <div class="capability-list">${projections.length ? projections.map(renderProjection).join("") : "<p class='empty'>No projections.</p>"}</div>
      </aside>
    </section>
  `;
}

function renderSurface(record: SurfaceContributionRecord) {
  return `
    <article class="event-row">
      <strong>${escapeHtml(record.surface.title)}</strong>
      <span>${escapeHtml(record.package_id)} · ${escapeHtml(record.surface.slot)} · ${escapeHtml(record.surface.id)}</span>
      <p>${escapeHtml(record.surface.description ?? "No description supplied.")}</p>
      <details class="surface-metadata"><summary>Inspect descriptor</summary><code>${formatJson(record.surface)}</code></details>
    </article>
  `;
}

function renderProposal(proposal: ProposalRecord) {
  const operationCount = proposal.operations.length;
  return `
    <article class="event-row">
      <strong>${escapeHtml(proposal.id)} · ${escapeHtml(proposal.status)}</strong>
      <span>${operationCount} operation${operationCount === 1 ? "" : "s"}${proposal.target_session_id ? ` · ${escapeHtml(proposal.target_session_id.slice(0, 8))}` : ""}</span>
      <div class="proposal-timeline"><span class="${proposal.status === "created" ? "active" : ""}">created</span><span class="${proposal.status === "approved" ? "active" : ""}">approved</span><span class="${proposal.status === "applied" ? "active" : ""}">applied</span></div>
      <details class="surface-metadata"><summary>Inspect proposal</summary><code>${formatJson(proposal)}</code></details>
      <div class="proposal-actions">
        ${proposal.status === "created" ? `<button type="button" class="button-warn" data-action="approve-proposal" data-proposal-id="${escapeHtml(proposal.id)}">Approve</button>` : ""}
        ${proposal.status === "approved" ? `<button type="button" class="button-success" data-action="apply-proposal" data-proposal-id="${escapeHtml(proposal.id)}">Apply</button>` : ""}
      </div>
    </article>
  `;
}

function renderCapability(capability: RegisteredCapability) {
  return `
    <article class="capability-row">
      <strong>${escapeHtml(capability.capability_id)}</strong>
      <span>${escapeHtml(capability.provider_package_id)} · ${escapeHtml(capability.version)}${capability.streaming ? " · ● Live" : ""}</span>
    </article>
  `;
}

function renderAsset(asset: AssetRecord) {
  return `<article class="capability-row"><strong>${mimeIcon(asset.mime)} ${escapeHtml(asset.id)}</strong><span>${escapeHtml(asset.origin_package_id)} · ${escapeHtml(asset.mime)} · ${asset.size_bytes} bytes</span></article>`;
}

function renderProjection(projection: ProjectionRecord) {
  const state = projection.state as Record<string, unknown> | undefined;
  const hint = state && typeof state === "object" && "status" in state ? String(state.status) : projection.source_kind_prefix ?? "snapshot";
  return `<article class="capability-row"><strong>${escapeHtml(projection.id)}</strong><span>${escapeHtml(projection.session_id)} · ${escapeHtml(hint)}</span><code>${formatJson(projection.state)}</code></article>`;
}

function mimeIcon(mime: string) {
  if (mime.includes("json") || mime.includes("text")) return "◇";
  if (mime.startsWith("image/")) return "◈";
  if (mime.startsWith("audio/")) return "◉";
  return "◆";
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
