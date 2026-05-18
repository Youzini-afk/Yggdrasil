import type { KernelEvent, PackageRecord, ProposalRecord, RegisteredCapability, SurfaceContributionRecord } from "../protocol/client";
import { extractEventPreview, extractProposalPreview, kindBadgeLabel } from "../text-layout/text-preview.js";
import { escapeHtml } from "../utils/html";

// --- Types ---

export interface AgentObservabilityModel {
  /** Packages whose id or metadata looks agent-like */
  agentPackages: PackageRecord[];
  /** Surfaces whose id or slot looks agent-like */
  agentSurfaces: SurfaceContributionRecord[];
  /** Events that look like run/tool/trace signals */
  runSignals: KernelEvent[];
  /** Events that look like tool bridge / capability invoke signals */
  toolSignals: KernelEvent[];
  /** Proposals that look agent-like (from agent packages or with trace/proposal payload) */
  proposalSignals: ProposalRecord[];
  /** Events that look like stream signals */
  streamSignals: KernelEvent[];
  /** Safety badges inferred from payloads */
  safetyBadges: SafetyBadge[];
}

export interface SafetyBadge {
  label: string;
  severity: "ok" | "warn" | "error" | "info";
  hint?: string;
}

// --- Detection heuristics (generic string-based, no hardcoded official package) ---

const AGENT_ID_HINTS = ["agent", "pi-agent", "tool-bridge", "trace", "run", "assistant"];
const TOOL_BRIDGE_KIND_HINTS = ["tool_bridge", "tool-bridge", "capability.invoke", "capability.stream"];
const TRACE_KIND_HINTS = ["trace", "tool", "run", "proposal"];
const STREAM_KINDS = [
  "kernel/stream.chunk",
  "kernel/stream.progress",
  "kernel/stream.error",
  "kernel/stream.cancelled",
  "kernel/stream.timeout",
  "kernel/stream.started",
  "kernel/stream.ended",
];

function looksAgentLike(id: string): boolean {
  const lower = id.toLowerCase();
  return AGENT_ID_HINTS.some((h) => lower.includes(h));
}

function isAgentLikePackage(pkg: PackageRecord): boolean {
  return looksAgentLike(pkg.id) || pkg.capability_count > 0;
}

function isAgentLikeSurface(surface: SurfaceContributionRecord): boolean {
  return looksAgentLike(surface.surface.id) || looksAgentLike(surface.surface.slot) || looksAgentLike(surface.surface.title);
}

function isAgentLikeCapability(cap: RegisteredCapability): boolean {
  return looksAgentLike(cap.capability_id);
}

function isToolBridgeEvent(event: KernelEvent): boolean {
  const kind = event.kind.toLowerCase();
  if (TOOL_BRIDGE_KIND_HINTS.some((h) => kind.includes(h))) return true;
  if (typeof event.payload === "object" && event.payload !== null) {
    const p = event.payload as Record<string, unknown>;
    // kernel.capability.invoke/stream method in payload
    const method = typeof p.method === "string" ? p.method.toLowerCase() : "";
    if (method.includes("capability.invoke") || method.includes("capability.stream")) return true;
    // tool_calls in payload
    if (Array.isArray(p.tool_calls)) return true;
    if (p.tool_bridge_plan !== undefined) return true;
  }
  return false;
}

function isTraceLikeEvent(event: KernelEvent): boolean {
  const kind = event.kind.toLowerCase();
  if (TRACE_KIND_HINTS.some((h) => kind.includes(h))) return true;
  if (typeof event.payload === "object" && event.payload !== null) {
    const p = event.payload as Record<string, unknown>;
    if (Array.isArray(p.trace_events)) return true;
    if (p.trace !== undefined) return true;
    if (p.proposal_draft !== undefined) return true;
    if (Array.isArray(p.stream_frames)) return true;
  }
  return false;
}

function isStreamEvent(event: KernelEvent): boolean {
  return STREAM_KINDS.includes(event.kind);
}

function isAgentLikeProposal(proposal: ProposalRecord, agentPackages: Set<string>): boolean {
  // A proposal is "agent-like" if it comes from an agent package (heuristic on id) or
  // its expected_effects contain agent/trace-like strings.
  if (agentPackages.has(proposal.id.split("/")[0] ?? "")) return true;
  const effects = JSON.stringify(proposal.expected_effects).toLowerCase();
  if (TRACE_KIND_HINTS.some((h) => effects.includes(h))) return true;
  return false;
}

// --- Safety badge extraction ---

function extractSafetyBadges(events: KernelEvent[]): SafetyBadge[] {
  const badges: SafetyBadge[] = [];
  let hasAmbiguous = false;
  let hasRejected = false;
  let hasProviderMissing = false;
  let hasPermissionDenied = false;
  let hasRedaction = false;

  for (const event of events) {
    const payloadStr = JSON.stringify(event.payload).toLowerCase();
    const kind = event.kind.toLowerCase();

    if (kind.includes("ambiguous") || payloadStr.includes("ambiguous")) hasAmbiguous = true;
    if (kind.includes("rejected") || payloadStr.includes("rejected")) hasRejected = true;
    if (kind.includes("denied") || payloadStr.includes("denied")) hasPermissionDenied = true;
    if (payloadStr.includes("provider_missing") || payloadStr.includes("missing provider")) hasProviderMissing = true;
    if (payloadStr.includes("redact") || payloadStr.includes("unsafe_blocked") || payloadStr.includes("secret_ref")) {
      hasRedaction = true;
    }
  }

  if (hasAmbiguous) badges.push({ label: "ambiguous provider", severity: "warn", hint: "At least one tool/capability call had ambiguous provider selection." });
  if (hasRejected) badges.push({ label: "rejected", severity: "error", hint: "At least one call or proposal was rejected." });
  if (hasProviderMissing) badges.push({ label: "missing provider", severity: "warn", hint: "A tool bridge plan referenced a missing provider." });
  if (hasPermissionDenied) badges.push({ label: "permission denied", severity: "error", hint: "A capability or event was blocked by permission policy." });
  if (hasRedaction) badges.push({ label: "audit/redaction", severity: "info", hint: "Secret references or redaction states detected in payloads." });

  return badges;
}

// --- Main builder ---

export function buildAgentObservability(
  packages: PackageRecord[],
  allSurfaces: SurfaceContributionRecord[],
  events: KernelEvent[],
  proposals: ProposalRecord[],
  capabilities: RegisteredCapability[],
): AgentObservabilityModel {
  const agentPackages = packages.filter(isAgentLikePackage);
  const agentSurfaces = allSurfaces.filter(isAgentLikeSurface);
  const agentPackageIds = new Set(agentPackages.map((p) => p.id));

  const runSignals: KernelEvent[] = [];
  const toolSignals: KernelEvent[] = [];
  const streamSignals: KernelEvent[] = [];

  for (const event of events) {
    if (isStreamEvent(event)) {
      streamSignals.push(event);
    }
    if (isToolBridgeEvent(event)) {
      toolSignals.push(event);
    }
    if (isTraceLikeEvent(event) && !isToolBridgeEvent(event)) {
      // Trace-like but not already counted as tool bridge
      runSignals.push(event);
    }
  }

  const proposalSignals = proposals.filter((p) => isAgentLikeProposal(p, agentPackageIds));
  const safetyBadges = extractSafetyBadges(events);

  return {
    agentPackages,
    agentSurfaces,
    runSignals,
    toolSignals,
    proposalSignals,
    streamSignals,
    safetyBadges,
  };
}

// --- Rendering helpers ---

export function renderAgentObservabilitySection(model: AgentObservabilityModel, events: KernelEvent[], proposals: ProposalRecord[]) {
  const totalSignals = model.runSignals.length + model.toolSignals.length + model.streamSignals.length + model.proposalSignals.length;
  return `
    <div class="forge-section agent-observability-section">
      <div class="section-header">
        <h2>Agent Observability</h2>
        <span class="section-meta">${model.agentPackages.length} pkg · ${model.agentSurfaces.length} surf · ${totalSignals} signal${totalSignals === 1 ? "" : "s"}</span>
      </div>

      <div class="diagnostics-grid">
        <div class="metric-card agent-metric"><strong>${model.agentPackages.length}</strong><span>Agent Packages</span></div>
        <div class="metric-card agent-metric"><strong>${model.agentSurfaces.length}</strong><span>Agent Surfaces</span></div>
        <div class="metric-card agent-metric"><strong>${model.runSignals.length}</strong><span>Run Signals</span></div>
        <div class="metric-card agent-metric"><strong>${model.toolSignals.length}</strong><span>Tool Signals</span></div>
        <div class="metric-card agent-metric"><strong>${model.proposalSignals.length}</strong><span>Proposal Signals</span></div>
        <div class="metric-card agent-metric"><strong>${model.streamSignals.length}</strong><span>Stream Signals</span></div>
      </div>

      ${model.safetyBadges.length ? `
        <div class="safety-badge-row">
          ${model.safetyBadges.map((b) => `<span class="safety-badge severity-${b.severity}" title="${escapeHtml(b.hint ?? "")}">${escapeHtml(b.label)}</span>`).join("")}
        </div>
      ` : ""}

      ${model.runSignals.length || model.toolSignals.length || model.streamSignals.length ? `
        <div class="agent-timeline">
          <h3 class="slot-title">Trace Timeline</h3>
          <div class="timeline-list">
            ${[...model.runSignals, ...model.toolSignals, ...model.streamSignals].slice(-20).map((e) => renderTimelineEvent(e)).join("")}
          </div>
        </div>
      ` : ""}

      ${model.proposalSignals.length ? `
        <div class="agent-proposal-list">
          <h3 class="slot-title">Proposal Explanations</h3>
          ${model.proposalSignals.map((p) => renderAgentProposal(p)).join("")}
        </div>
      ` : ""}
    </div>
  `;
}

function renderTimelineEvent(event: KernelEvent) {
  const preview = extractEventPreview(event.kind, event.payload);
  const previewHtml = preview.hasPreview
    ? `<details class="text-preview-details"><summary>Text preview</summary><div class="text-preview-panel"><div class="text-preview-meta"><span class="text-proof-badge">${escapeHtml(kindBadgeLabel(preview.kind))}</span><span class="text-proof-badge">~${preview.lineEstimate} line${preview.lineEstimate === 1 ? "" : "s"}</span><span class="text-proof-badge">~${preview.heightEstimate}px</span><span class="text-proof-badge">engine:${escapeHtml(preview.engineName)}</span></div><pre class="text-preview-stage">${escapeHtml(preview.text)}</pre></div></details>`
    : "";

  const isTool = isToolBridgeEvent(event);
  const isStream = isStreamEvent(event);
  const badgeClass = isStream ? "stream-badge" : isTool ? "tool-badge" : "trace-badge";
  const badgeLabel = isStream ? "stream" : isTool ? "tool" : "trace";

  return `
    <article class="timeline-row">
      <div class="timeline-meta">
        <span class="timeline-badge ${badgeClass}">${badgeLabel}</span>
        <span class="timeline-seq">#${event.sequence}</span>
        <span class="timeline-kind">${escapeHtml(event.kind)}</span>
        <span class="timeline-writer">${escapeHtml(event.writer_package_id)}</span>
      </div>
      <code class="timeline-payload">${escapeHtml(JSON.stringify(event.payload, null, 2).slice(0, 360))}${JSON.stringify(event.payload).length > 360 ? "…" : ""}</code>
      ${previewHtml}
    </article>
  `;
}

function renderAgentProposal(proposal: ProposalRecord) {
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
      <span>${proposal.operations.length} op${proposal.operations.length === 1 ? "" : "s"}${proposal.target_session_id ? ` · ${escapeHtml(proposal.target_session_id.slice(0, 8))}` : ""}</span>
      <details class="surface-metadata"><summary>Inspect proposal</summary><code>${escapeHtml(JSON.stringify(proposal, null, 2))}</code></details>
      ${previewHtml}
    </article>
  `;
}

export function renderAgentReadinessPanel(agentSurfaces: SurfaceContributionRecord[], agentCapabilities: RegisteredCapability[]) {
  const surfaceCount = agentSurfaces.length;
  const capCount = agentCapabilities.length;
  const hasAgentLike = surfaceCount > 0 || capCount > 0;

  return `
    <div class="agent-readiness-panel">
      <div class="agent-readiness-header">
        <span class="agent-readiness-badge ${hasAgentLike ? "ready" : "empty"}">${hasAgentLike ? "●" : "○"}</span>
        <span class="agent-readiness-title">Agent Readiness</span>
      </div>
      <div class="agent-readiness-body">
        <span class="text-proof-badge">surfaces ${surfaceCount}</span>
        <span class="text-proof-badge">capabilities ${capCount}</span>
      </div>
      <p class="agent-readiness-note">
        ${hasAgentLike
          ? "Agent-like surfaces/capabilities detected. No real model or network calls — all agent behavior is proposal-gated and plan-only."
          : "No agent-like surfaces or capabilities detected. Load an agent-runtime or tool-bridge package to enable observability."
        }
      </p>
      <div class="agent-readiness-actions">
        <button type="button" disabled title="Template only — no real agent loop">Start agent</button>
        <button type="button" disabled title="Template only — no real model call">Run tool plan</button>
      </div>
    </div>
  `;
}

export function filterAgentLikeCapabilities(capabilities: RegisteredCapability[]): RegisteredCapability[] {
  return capabilities.filter(isAgentLikeCapability);
}
