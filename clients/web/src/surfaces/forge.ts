import type { AssetRecord, KernelEvent, PackageRecord, ProjectionRecord, ProposalRecord, RegisteredCapability, SurfaceContributionRecord } from "../protocol/client";
import { escapeHtml, formatJson } from "../utils/html";
import { extractEventPreview, extractProposalPreview, kindBadgeLabel } from "../text-layout/text-preview.js";

export function renderForgeSurface(input: {
  capabilities: RegisteredCapability[];
  events: KernelEvent[];
  assets: AssetRecord[];
  projections: ProjectionRecord[];
  proposals: ProposalRecord[];
  forgeSurfaces: SurfaceContributionRecord[];
  packages: PackageRecord[];
  allSurfaces: SurfaceContributionRecord[];
  sessionId?: string;
}) {
  const { capabilities, events, assets, projections, proposals, forgeSurfaces, packages, allSurfaces, sessionId } = input;
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

        <div class="forge-section">
          <div class="section-header">
            <h2>Package Inventory</h2>
            <span class="section-meta">${packages.length} package${packages.length === 1 ? "" : "s"} · ${capabilities.length} capability${capabilities.length === 1 ? "" : "s"}</span>
          </div>
          ${packages.length ? renderPackageInventory(packages, capabilities, allSurfaces) : "<p class='empty'>No packages loaded.</p>"}
        </div>

        <div class="forge-section">
          <div class="section-header">
            <h2>Surface Inventory</h2>
            <span class="section-meta">${allSurfaces.length} surface${allSurfaces.length === 1 ? "" : "s"}</span>
          </div>
          ${allSurfaces.length ? renderSurfaceInventory(allSurfaces) : "<p class='empty'>No surface contributions.</p>"}
        </div>

        <div class="forge-section">
          <div class="section-header">
            <h2>Composition Diagnostics</h2>
          </div>
          ${renderAuthoringDiagnostics(packages, capabilities, allSurfaces, assets, projections)}
        </div>

        <div class="forge-section">
          <div class="section-header">
            <h2>Authoring Commands</h2>
          </div>
          ${renderManifestGuidance()}
        </div>

        <div class="forge-section">
          <h2>Package Forge Panels</h2>
          ${forgeSurfaces.length ? forgeSurfaces.map(renderSurface).join("") : "<p class='empty'>No package-contributed Forge panels.</p>"}
        </div>

        <div class="forge-section">
          <h2>Proposals</h2>
          ${proposals.length ? proposals.map(renderProposal).join("") : "<p class='empty'>No proposals yet.</p>"}
        </div>

        <div class="forge-section event-tail-section">
          <h2>Events</h2>
          <div class="event-tail">
            ${events.length ? events.map(renderEvent).join("") : "<p class='empty'>Open a session to inspect events.</p>"}
          </div>
        </div>
      </div>

      <aside class="workspace-column secondary">
        <div class="section-header">
          <h2>Capabilities</h2>
          <span class="section-meta">${capabilities.length}</span>
        </div>
        <div class="capability-list">
          ${capabilities.length ? renderCapabilitiesGrouped(capabilities) : "<p class='empty'>No capabilities discovered.</p>"}
        </div>

        <div class="section-header">
          <h2>Assets</h2>
          <span class="section-meta">${assets.length}</span>
        </div>
        <div class="capability-list">${assets.length ? assets.map(renderAsset).join("") : "<p class='empty'>No assets.</p>"}</div>

        <div class="section-header">
          <h2>Projections</h2>
          <span class="section-meta">${projections.length}</span>
        </div>
        <div class="capability-list">${projections.length ? projections.map(renderProjection).join("") : "<p class='empty'>No projections.</p>"}</div>
      </aside>
    </section>
  `;
}

function groupBy<T>(arr: T[], keyFn: (item: T) => string): Record<string, T[]> {
  const result: Record<string, T[]> = {};
  for (const item of arr) {
    const key = keyFn(item);
    if (!result[key]) result[key] = [];
    result[key].push(item);
  }
  return result;
}

function renderPackageInventory(packages: PackageRecord[], capabilities: RegisteredCapability[], allSurfaces: SurfaceContributionRecord[]) {
  const capsByProvider = groupBy(capabilities, (c) => c.provider_package_id);
  const surfacesByPackage = groupBy(allSurfaces, (s) => s.package_id);
  return `
    <div class="inspector-grid">
      ${packages.map((pkg) => {
        const pkgCaps = capsByProvider[pkg.id] ?? [];
        const pkgSurfaces = surfacesByPackage[pkg.id] ?? [];
        return `
          <article class="package-card">
            <div class="package-header">
              <strong>${escapeHtml(pkg.id)}</strong>
              <span class="surface-chip">${escapeHtml(pkg.state)}</span>
              <span class="surface-chip">${escapeHtml(pkg.entry_kind)}</span>
            </div>
            <div class="package-meta">
              <span>v${escapeHtml(pkg.version)}</span>
              <span>${pkg.capability_count} cap${pkg.capability_count === 1 ? "" : "s"}</span>
              <span>${pkg.hook_count} hook${pkg.hook_count === 1 ? "" : "s"}</span>
              <span>${pkgSurfaces.length} surface${pkgSurfaces.length === 1 ? "" : "s"}</span>
            </div>
            ${pkgCaps.length ? `<div class="package-caps">${pkgCaps.map((c) => `<span class="surface-chip">${escapeHtml(c.capability_id)}${c.streaming ? " ●" : ""}</span>`).join("")}</div>` : ""}
            ${pkgSurfaces.length ? `<div class="package-slots">${pkgSurfaces.map((s) => `<span class="surface-chip">${escapeHtml(s.surface.slot)}</span>`).join("")}</div>` : ""}
          </article>
        `;
      }).join("")}
    </div>
  `;
}

function renderSurfaceInventory(allSurfaces: SurfaceContributionRecord[]) {
  const bySlot = groupBy(allSurfaces, (s) => s.surface.slot);
  const slots = Object.keys(bySlot).sort();
  return `
    <div class="surface-slot-list">
      ${slots.map((slot) => {
        const surfaces = bySlot[slot];
        return `
          <div class="slot-group">
            <h3 class="slot-title">${escapeHtml(slot)} <span class="section-meta">${surfaces.length}</span></h3>
            <div class="slot-list">
              ${surfaces.map((s) => `
                <article class="capability-row">
                  <strong>${escapeHtml(s.surface.title)}</strong>
                  <span>${escapeHtml(s.package_id)} · ${escapeHtml(s.surface.id)} · v${escapeHtml(s.surface.version)}</span>
                </article>
              `).join("")}
            </div>
          </div>
        `;
      }).join("")}
    </div>
  `;
}

function renderAuthoringDiagnostics(
  packages: PackageRecord[],
  capabilities: RegisteredCapability[],
  allSurfaces: SurfaceContributionRecord[],
  assets: AssetRecord[],
  projections: ProjectionRecord[]
) {
  const experienceEntries = allSurfaces.filter((s) => s.surface.slot === "experience_entry");
  const packagesWithoutCaps = packages.filter((p) => p.capability_count === 0);
  const packagesWithoutSurfaces = packages.filter((p) => allSurfaces.filter((s) => s.package_id === p.id).length === 0);
  return `
    <div class="diagnostics-grid">
      <div class="metric-card"><strong>${packages.length}</strong><span>Packages</span></div>
      <div class="metric-card"><strong>${capabilities.length}</strong><span>Capabilities</span></div>
      <div class="metric-card"><strong>${allSurfaces.length}</strong><span>Surfaces</span></div>
      <div class="metric-card"><strong>${assets.length}</strong><span>Assets</span></div>
      <div class="metric-card"><strong>${projections.length}</strong><span>Projections</span></div>
      <div class="metric-card"><strong>${experienceEntries.length}</strong><span>Experiences</span></div>
    </div>
    <div class="diagnostics-checklist">
      ${packagesWithoutCaps.length ? `<p class="diagnostic-warn">⚠ ${packagesWithoutCaps.length} package${packagesWithoutCaps.length === 1 ? "" : "s"} with no capabilities.</p>` : `<p class="diagnostic-ok">✓ All packages declare capabilities.</p>`}
      ${packagesWithoutSurfaces.length ? `<p class="diagnostic-warn">⚠ ${packagesWithoutSurfaces.length} package${packagesWithoutSurfaces.length === 1 ? "" : "s"} with no surfaces.</p>` : `<p class="diagnostic-ok">✓ All packages declare surfaces.</p>`}
      ${experienceEntries.length ? `<p class="diagnostic-ok">✓ ${experienceEntries.length} experience entry surface${experienceEntries.length === 1 ? "" : "s"} available to launch.</p>` : `<p class="diagnostic-warn">⚠ No experience_entry surfaces found. Add a package with an experience surface to enable Play.</p>`}
    </div>
  `;
}

function renderManifestGuidance() {
  return `
    <div class="guidance-grid">
      <div class="guidance-card">
        <h4>Init package templates</h4>
        <div class="command-block">
          <code>cargo run -p ygg-cli -- init-package ./my-package --id example/my-package --entry subprocess --language python</code>
          <code>cargo run -p ygg-cli -- init-package ./my-package --id example/my-package --entry subprocess --language typescript</code>
          <code>cargo run -p ygg-cli -- init-package ./my-package --id example/my-package --entry subprocess --language typescript-experience</code>
        </div>
        <p class="guidance-note">Templates: <code>basic</code>, <code>experience</code>, <code>play-renderer</code>, <code>forge-panel</code>, <code>assistant-action</code>, <code>asset-editor</code>, <code>full-surface</code></p>
      </div>
      <div class="guidance-card">
        <h4>Package check &amp; reload</h4>
        <div class="command-block">
          <code>cargo run -p ygg-cli -- package check ./my-package/manifest.yaml</code>
          <code>cargo run -p ygg-cli -- package conformance ./my-package/manifest.yaml</code>
          <code>cargo run -p ygg-cli -- package run-fixture ./my-package/manifest.yaml</code>
          <code>cargo run -p ygg-cli -- package reload ./my-package/manifest.yaml</code>
        </div>
      </div>
      <div class="guidance-card">
        <h4>Composition</h4>
        <div class="command-block">
          <code>cargo run -p ygg-cli -- init-composition ./my-composition --id example/my-composition</code>
          <code>cargo run -p ygg-cli -- composition check ./my-composition/composition.yaml</code>
        </div>
      </div>
    </div>
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
  const preview = extractProposalPreview({
    expected_effects: proposal.expected_effects,
    operations: proposal.operations,
  });
  const previewHtml = preview.hasPreview
    ? `<details class="text-preview-details"><summary>Proposal text preview</summary><div class="text-preview-panel"><div class="text-preview-meta"><span class="text-proof-badge">${escapeHtml(kindBadgeLabel(preview.kind))}</span><span class="text-proof-badge">~${preview.lineEstimate} line${preview.lineEstimate === 1 ? "" : "s"}</span><span class="text-proof-badge">~${preview.heightEstimate}px</span><span class="text-proof-badge">engine:${escapeHtml(preview.engineName)}</span></div><pre class="text-preview-stage">${escapeHtml(preview.text)}</pre></div></details>`
    : "";
  return `
    <article class="event-row">
      <strong>${escapeHtml(proposal.id)} · ${escapeHtml(proposal.status)}</strong>
      <span>${operationCount} operation${operationCount === 1 ? "" : "s"}${proposal.target_session_id ? ` · ${escapeHtml(proposal.target_session_id.slice(0, 8))}` : ""}</span>
      <div class="proposal-timeline"><span class="${proposal.status === "created" ? "active" : ""}">created</span><span class="${proposal.status === "approved" ? "active" : ""}">approved</span><span class="${proposal.status === "applied" ? "active" : ""}">applied</span></div>
      <details class="surface-metadata"><summary>Inspect proposal</summary><code>${formatJson(proposal)}</code></details>
      ${previewHtml}
      <div class="proposal-actions">
        ${proposal.status === "created" ? `<button type="button" class="button-warn" data-action="approve-proposal" data-proposal-id="${escapeHtml(proposal.id)}">Approve</button>` : ""}
        ${proposal.status === "approved" ? `<button type="button" class="button-success" data-action="apply-proposal" data-proposal-id="${escapeHtml(proposal.id)}">Apply</button>` : ""}
      </div>
    </article>
  `;
}

function renderCapabilitiesGrouped(capabilities: RegisteredCapability[]) {
  const byProvider = groupBy(capabilities, (c) => c.provider_package_id);
  const providers = Object.keys(byProvider).sort();
  return providers
    .map((provider) => {
      const caps = byProvider[provider];
      return `
        <div class="provider-group">
          <h3 class="provider-title">${escapeHtml(provider)} <span class="section-meta">${caps.length}</span></h3>
          <div class="slot-list">
            ${caps.map(renderCapability).join("")}
          </div>
        </div>
      `;
    })
    .join("");
}

function renderCapability(capability: RegisteredCapability) {
  return `
    <article class="capability-row">
      <strong>${escapeHtml(capability.capability_id)}</strong>
      <span>${escapeHtml(capability.version)}${capability.streaming ? " · ● Live" : ""}</span>
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
  const preview = extractEventPreview(event.kind, event.payload);
  const previewHtml = preview.hasPreview
    ? `<details class="text-preview-details"><summary>Text preview</summary><div class="text-preview-panel"><div class="text-preview-meta"><span class="text-proof-badge">${escapeHtml(kindBadgeLabel(preview.kind))}</span><span class="text-proof-badge">~${preview.lineEstimate} line${preview.lineEstimate === 1 ? "" : "s"}</span><span class="text-proof-badge">~${preview.heightEstimate}px</span><span class="text-proof-badge">engine:${escapeHtml(preview.engineName)}</span></div><pre class="text-preview-stage">${escapeHtml(preview.text)}</pre></div></details>`
    : "";
  return `
    <article class="event-row">
      <span>#${event.sequence}</span>
      <strong>${escapeHtml(event.kind)}</strong>
      <code>${formatJson(event.payload)}</code>
      ${previewHtml}
    </article>
  `;
}
