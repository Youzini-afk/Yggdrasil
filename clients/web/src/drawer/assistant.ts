export function renderAssistantDrawer(diagnostics: Record<string, unknown>, open = false) {
  return `
    <aside class="assistant-drawer ${open ? "open" : ""}" aria-label="Assist drawer">
      <button type="button" class="assist-toggle" data-action="toggle-assist">Assistant</button>
      <div class="assist-panel">
        <p class="eyebrow">Assistant</p>
        <h2>Play-Creation Bridge</h2>
        <p>Lightweight edits in Play, deeper protocol-guided creation in Forge. No privileged assistant path.</p>
        <textarea placeholder="Ask a package-backed assistant to branch, explain, or inspect..."></textarea>
        <button type="button" class="button-success">Draft proposal</button>
        <div class="quick-actions">
          <button type="button" title="Template only">Fork idea</button>
          <button type="button" title="Template only">Explain events</button>
          <button type="button" title="Template only">Suggest capability</button>
        </div>
        <details><summary>Host diagnostics</summary><pre>${JSON.stringify(diagnostics, null, 2)}</pre></details>
      </div>
    </aside>
  `;
}
