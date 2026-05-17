export function renderAssistantDrawer(diagnostics: Record<string, unknown>, open = false) {
  return `
    <aside class="assistant-drawer ${open ? "open" : ""}" aria-label="Assist drawer">
      <button type="button" class="assist-toggle" data-action="toggle-assist">Assist</button>
      <div class="assist-panel">
        <p class="eyebrow">Assist</p>
        <h2>Play-Creation Bridge</h2>
        <p>Lightweight edits in Play, deeper protocol-guided creation in Forge. No privileged assistant path.</p>
        <textarea placeholder="Ask a package-backed assistant to branch, explain, or inspect..."></textarea>
        <div class="quick-actions">
          <button type="button">Fork idea</button>
          <button type="button">Explain events</button>
          <button type="button">Suggest capability</button>
        </div>
        <pre>${JSON.stringify(diagnostics, null, 2)}</pre>
      </div>
    </aside>
  `;
}
