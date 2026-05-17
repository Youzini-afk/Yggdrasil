import type { SurfaceContributionRecord } from "../protocol/client";
import { escapeHtml } from "../utils/html";

export function renderPlaySurface(entries: SurfaceContributionRecord[], sessionId?: string) {
  const cards = entries.length ? entries.map((entry) => experienceCard(entry)).join("") : placeholderCards();
  return `
    <section class="surface surface-play" aria-labelledby="play-title">
      <div class="hero-panel">
        <p class="eyebrow">Home / Play</p>
        <h1 id="play-title">Choose an Experience</h1>
        <p>Launcher-first shell discovered from package-declared <code>experience_entry</code> surfaces.</p>
        <div class="hero-actions">
          ${sessionId ? `<button type="button" data-action="fork-session">Fork active session</button><span class="session-chip">${escapeHtml(sessionId)}</span>` : `<span class="session-chip">No active session</span>`}
        </div>
      </div>
      <section class="rail" aria-label="Continue experiences">
        <div class="rail-header">
          <h2>Experience Entries</h2>
          <span>${entries.length} package surface${entries.length === 1 ? "" : "s"}</span>
        </div>
        <div class="experience-grid">${cards}</div>
      </section>
    </section>
  `;
}

function experienceCard(entry: SurfaceContributionRecord) {
  const surface = entry.surface;
  return `
    <article class="experience-card">
      <div class="card-glow"></div>
      <p class="eyebrow">${escapeHtml(entry.package_id)} · ${escapeHtml(entry.entry_kind)} · ${escapeHtml(entry.package_state)}</p>
      <h3>${escapeHtml(surface.title)} <small>${escapeHtml(surface.version)}</small></h3>
      <p>${escapeHtml(surface.description ?? "No package description supplied.")}</p>
      <span class="approval-policy">${escapeHtml(surface.approval_policy ?? "none")}</span>
      <div class="permission-strip">${surface.required_permissions.map(permissionBadge).join("")}</div>
      <details class="surface-metadata"><summary>Surface metadata</summary><pre>${escapeHtml(JSON.stringify(surface.metadata ?? {}, null, 2))}</pre></details>
      <button type="button" data-action="launch-surface" data-surface-id="${escapeHtml(surface.id)}">Launch</button>
    </article>
  `;
}

function permissionBadge(requirement: SurfaceContributionRecord["surface"]["required_permissions"][number]) {
  const scope = requirement.scope ? ` · ${requirement.scope}` : "";
  return `<span class="permission-badge risk-${escapeHtml(requirement.risk)}" title="${escapeHtml(requirement.reason ?? "")}">${escapeHtml(requirement.permission + scope)}</span>`;
}

function placeholderCards() {
  return `
    <article class="experience-card muted">
      <div class="card-glow"></div>
      <p class="eyebrow">Empty catalog</p>
      <h3>Waiting for experience entries</h3>
      <p>Load packages that contribute <code>experience_entry</code> surfaces to populate Home.</p>
    </article>
  `;
}
