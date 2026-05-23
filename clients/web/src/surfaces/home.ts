import { type ProjectRecord } from "../protocol/client";
import { escapeHtml } from "../utils/html";

export interface HomeViewModel {
  projects: ProjectRecord[];
  loading: boolean;
  error?: string;
}

export function renderHomeSurface(model: HomeViewModel): string {
  if (model.loading) {
    return `<div class="home-surface"><div class="home-loading">Loading projects...</div></div>`;
  }

  if (model.error) {
    return `<div class="home-surface"><div class="home-error">Error: ${escapeHtml(model.error)}</div></div>`;
  }

  const projectCards = model.projects.map(renderProjectCard).join("");

  return `
    <div class="home-surface">
      <header class="home-header">
        <h1>Projects</h1>
        <button class="install-button" data-action="install">+ Install project</button>
      </header>
      <div class="home-project-grid">
        ${projectCards}
        ${renderInstallCard()}
      </div>
    </div>
  `;
}

function renderProjectCard(project: ProjectRecord): string {
  const statePill = renderStatePill(project.state);
  const playButton = renderPlayButton(project);
  const icon = project.icon
    ? `<img class="project-icon" src="${escapeHtml(project.icon)}" alt="" />`
    : `<div class="project-icon-default">${escapeHtml(project.title.slice(0, 1).toUpperCase())}</div>`;

  return `
    <div class="home-project-card" data-project-id="${escapeHtml(project.id)}">
      ${icon}
      <div class="project-info">
        <h3 class="project-title">${escapeHtml(project.title)}</h3>
        <p class="project-description">${escapeHtml(project.description || "")}</p>
        <div class="project-meta">
          ${statePill}
          <span class="project-type">${escapeHtml(project.type)}</span>
        </div>
      </div>
      <div class="project-actions">
        ${playButton}
      </div>
    </div>
  `;
}

function renderStatePill(state: string): string {
  const stateClass = `state-${state}`;
  return `<span class="state-pill ${stateClass}">${escapeHtml(state)}</span>`;
}

function renderPlayButton(project: ProjectRecord): string {
  if (project.state === "running") {
    return `<button class="play-button play-button-running" data-action="stop" data-project-id="${escapeHtml(project.id)}">Stop</button>`;
  }
  if (project.state === "stopped" || project.state === "installed" || project.state === "failed") {
    return `<button class="play-button play-button-start" data-action="play" data-project-id="${escapeHtml(project.id)}">▶ Play</button>`;
  }
  if (project.state === "starting" || project.state === "stopping") {
    return `<button class="play-button play-button-disabled" disabled>${escapeHtml(project.state)}...</button>`;
  }
  return `<button class="play-button play-button-disabled" disabled>${escapeHtml(project.state)}</button>`;
}

function renderInstallCard(): string {
  return `
    <div class="home-project-card project-card-install" data-action="install">
      <div class="install-card-content">
        <div class="install-icon">+</div>
        <h3>Install a project</h3>
        <p>Install from a GitHub URL or local path</p>
      </div>
    </div>
  `;
}
