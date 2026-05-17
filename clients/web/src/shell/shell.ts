export type RouteName = "play" | "forge";

export function renderShell(route: RouteName, body: string, assistant: string, error?: string) {
  const title = route === "play" ? "Home / Play" : "Forge";
  return `
    <div class="app-shell" data-route="${route}">
      <nav class="topbar" aria-label="Primary">
        <div>
          <p class="eyebrow">Yggdrasil</p>
          <strong>${title}</strong>
        </div>
        <div class="nav-actions">
          <button type="button" data-route="play" class="${route === "play" ? "active" : ""}">Home</button>
          <button type="button" data-route="forge" class="${route === "forge" ? "active" : ""}">Forge</button>
        </div>
      </nav>
      ${error ? `<div class="error-banner">${error}</div>` : ""}
      ${body}
      ${assistant}
    </div>
  `;
}
