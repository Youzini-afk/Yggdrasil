import type { StreamingBufferState } from "../text-layout/types";
import type { TextEngineInitializationResult } from "../text-layout/config";

export type TextProofView = {
  text: string;
  state: StreamingBufferState;
  lineCount: number;
  height: number;
  chunkIndex: number;
  totalChunks: number;
  /** Active engine name (T2). */
  engineName?: string;
  /** Active engine version (T2). */
  engineVersion?: string;
  /** Active engine state (T2). */
  engineState?: string;
  /** User preference: auto/fallback/pretext (T3). */
  enginePreference?: string;
  /** Fallback reason if preferred engine not active (T3). */
  fallbackReason?: string;
  /** Whether Pretext module is available (T3). */
  pretextAvailable?: boolean;
};

export function renderAssistantDrawer(
  diagnostics: Record<string, unknown>,
  open = false,
  textProof?: TextProofView,
  agentReadinessHtml?: string,
  externalProjectHtml?: string,
) {
  const proof = textProof ?? {
    text: "",
    state: "idle" as StreamingBufferState,
    lineCount: 0,
    height: 0,
    chunkIndex: 0,
    totalChunks: 0,
    engineName: "fallback",
    engineVersion: "0.2.0",
    engineState: "active",
  };
  const engineBadge = proof.engineName
    ? `<span class="text-proof-badge engine-badge">engine ${escapeHtml(proof.engineName)} v${escapeHtml(proof.engineVersion ?? "?")} <span class="engine-state state-${proof.engineState}">${proof.engineState}</span></span>`
    : "";
  const preferenceBadge = proof.enginePreference
    ? `<span class="text-proof-badge pref-badge">pref ${escapeHtml(proof.enginePreference)}</span>`
    : "";
  const pretextBadge = proof.pretextAvailable !== undefined
    ? `<span class="text-proof-badge pretext-badge ${proof.pretextAvailable ? "pretext-available" : "pretext-unavailable"}">pretext ${proof.pretextAvailable ? "available" : "unavailable"}</span>`
    : "";
  const fallbackBadge = proof.fallbackReason
    ? `<span class="text-proof-badge fallback-reason-badge" title="${escapeHtml(proof.fallbackReason)}">fallback: ${escapeHtml(truncateReason(proof.fallbackReason))}</span>`
    : "";
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
        ${agentReadinessHtml ?? ""}
        ${externalProjectHtml ?? ""}
        <details><summary>Host diagnostics</summary><pre>${JSON.stringify(diagnostics, null, 2)}</pre></details>
        <details class="text-proof-details" open>
          <summary>Text Surface Proof (mock streaming)</summary>
          <div class="text-proof-panel">
            <div class="text-proof-meta">
              ${engineBadge}
              ${preferenceBadge}
              ${pretextBadge}
              ${fallbackBadge}
              <span class="text-proof-badge state-${proof.state}">${proof.state}</span>
              <span class="text-proof-badge">lines ${proof.lineCount}</span>
              <span class="text-proof-badge">height ${Math.round(proof.height)}px</span>
              <span class="text-proof-badge">chunks ${proof.chunkIndex}/${proof.totalChunks}</span>
            </div>
            <div class="text-proof-stage" aria-live="polite" aria-atomic="false">
              ${proof.text ? `<p class="text-proof-content">${escapeHtml(proof.text)}</p>` : `<p class="text-proof-placeholder">Tap replay to start mock stream…</p>`}
            </div>
            <div class="text-proof-controls">
              <button type="button" data-action="replay-stream-proof" ${proof.state === "streaming" ? "disabled" : ""}>Replay</button>
              <button type="button" data-action="reset-stream-proof" ${proof.state === "idle" ? "disabled" : ""}>Reset</button>
            </div>
          </div>
        </details>
      </div>
    </aside>
  `;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

/** Truncate a fallback reason for display in a badge. */
function truncateReason(reason: string, maxLen = 40): string {
  if (reason.length <= maxLen) return reason;
  return reason.slice(0, maxLen - 1) + "…";
}
