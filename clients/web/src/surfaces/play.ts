import type { PackageRecord } from "../protocol/client";
import { escapeHtml } from "../utils/html";

export function renderPlaySurface(packages: PackageRecord[]) {
  const cards = packages.length ? packages.map((pkg) => experienceCard(pkg)).join("") : placeholderCards();
  return `
    <section class="surface surface-play" aria-labelledby="play-title">
      <div class="hero-panel">
        <p class="eyebrow">Play</p>
        <h1 id="play-title">Choose an Experience</h1>
        <p>Launcher-first shell for playable experiences. Cards stay package-backed and content-neutral.</p>
      </div>
      <section class="rail" aria-label="Continue experiences">
        <div class="rail-header">
          <h2>Continue</h2>
          <span>${packages.length} loaded package${packages.length === 1 ? "" : "s"}</span>
        </div>
        <div class="experience-grid">${cards}</div>
      </section>
    </section>
  `;
}

function experienceCard(pkg: PackageRecord) {
  return `
    <article class="experience-card">
      <div class="card-glow"></div>
      <p class="eyebrow">Capability Package</p>
      <h3>${escapeHtml(pkg.id)}</h3>
      <p>${escapeHtml(pkg.entry_kind)} · ${escapeHtml(pkg.state)} · ${pkg.capability_count} capabilities</p>
      <button type="button" data-route="forge">Inspect in Forge</button>
    </article>
  `;
}

function placeholderCards() {
  return ["Blank Experience", "Branch Lab", "Package Playground"]
    .map((title) => `
      <article class="experience-card muted">
        <div class="card-glow"></div>
        <p class="eyebrow">Placeholder</p>
        <h3>${title}</h3>
        <p>Waiting for package-contributed experience providers.</p>
      </article>
    `)
    .join("");
}
