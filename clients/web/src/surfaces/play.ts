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
  const policy = surface.approval_policy ?? "none";
  const permissionCount = surface.required_permissions.length;
  const official = entry.package_id.startsWith("official/");
  const hasConfig = surface.activation.input_schema ? "Has config" : "No config";
  return `
    <article class="experience-card">
      <div class="card-glow"></div>
      <div class="card-header-row">
        <p class="eyebrow">${packageIcon(entry.package_id)} ${escapeHtml(entry.package_id)} · ${escapeHtml(entry.entry_kind)}</p>
        <span class="approval-policy policy-${escapeHtml(policy)}">${escapeHtml(policy)}</span>
      </div>
      <h3>${escapeHtml(surface.title)} <small>${escapeHtml(surface.version)}</small></h3>
      <p>${escapeHtml(surface.description ?? "No package description supplied.")}</p>
      <div class="surface-chip-row">
        ${official ? `<span class="surface-chip official">official</span>` : `<span class="surface-chip">third-party</span>`}
        <span class="surface-chip">${escapeHtml(entry.package_state)}</span>
        <span class="surface-chip">${escapeHtml(hasConfig)}</span>
        <span class="surface-chip">${permissionCount} permission${permissionCount === 1 ? "" : "s"}</span>
      </div>
      ${permissionCount ? `<div class="permission-strip">${surface.required_permissions.slice(0, 3).map(permissionBadge).join("")}${permissionCount > 3 ? `<span class="permission-badge">+${permissionCount - 3}</span>` : ""}</div>` : ""}
      <details class="surface-metadata"><summary>Inspect surface descriptor</summary><pre>${escapeHtml(JSON.stringify(surface, null, 2))}</pre></details>
      <button type="button" data-action="launch-surface" data-surface-id="${escapeHtml(surface.id)}">Start · ${escapeHtml(entry.package_id)}</button>
    </article>
  `;
}

function packageIcon(packageId: string) {
  if (packageId.includes("playable")) return "🧬";
  if (packageId.includes("asset")) return "🎨";
  if (packageId.includes("projection")) return "📈";
  if (packageId.includes("composition")) return "🧩";
  if (packageId.startsWith("official/")) return "✦";
  return "□";
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
