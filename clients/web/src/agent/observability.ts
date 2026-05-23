import type { AssetRecord, KernelEvent, PackageRecord, ProjectionRecord, ProposalRecord, RegisteredCapability, SurfaceContributionRecord } from "../protocol/client";
import { extractEventPreview, extractProposalPreview, kindBadgeLabel } from "../text-layout/text-preview.js";
import { escapeHtml, formatJson } from "../utils/html";

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
  "kernel/v1/stream.chunk",
  "kernel/v1/stream.progress",
  "kernel/v1/stream.error",
  "kernel/v1/stream.cancelled",
  "kernel/v1/stream.timeout",
  "kernel/v1/stream.started",
  "kernel/v1/stream.ended",
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
    // kernel.v1.capability.invoke/stream method in payload
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

// ========================================================================
// Phase E — Forge Agent Workspace / Observability UI Shell
// ========================================================================
// These view models and renderers build on public protocol data only:
// events, proposals, surfaces, capabilities, packages, assets, projections.
// No kernel internals, no model/network calls, no chat-first patterns.
// All agent-like shapes are heuristic-detected from public protocol fields.
// ========================================================================

// --- Forge Agent Workspace View Model ---

export interface ForgeAgentWorkspaceModel {
  /** Runs derived from events that carry run lifecycle payloads */
  runs: RunTimelineEntry[];
  /** Plan graph nodes derived from run-signal event payloads */
  planNodes: PlanGraphNode[];
  /** Branch-related events and projections */
  branchEntries: BranchLineageEntry[];
  /** Candidate-like proposals or events */
  candidates: CandidateCard[];
  /** Tool bridge events */
  toolTraces: ToolTraceEntry[];
  /** Inference events */
  inferenceTraces: InferenceTraceEntry[];
  /** Control action affordances derived from visible state */
  controlActions: ControlAction[];
}

export interface RunTimelineEntry {
  runId: string;
  label: string;
  status: string;
  packageId: string;
  eventSequence: number;
  nodeCount: number;
  edgeCount: number;
  workingStateFields: string[];
}

export interface PlanGraphNode {
  runId: string;
  nodeId: string;
  kind: string;
  label: string;
  status: string;
  inputRefs: string[];
  outputRefs: string[];
  approvalPolicy?: string;
}

export interface BranchLineageEntry {
  branchLabel: string;
  type: "scratch" | "target" | "fork" | "lineage";
  sourceEvent?: string;
  targetRevision?: string;
  intent?: string;
  promoteRequiresProposal?: boolean;
  staleDetected?: boolean;
}

export interface CandidateCard {
  candidateId: string;
  runId: string;
  status: string;
  targetBranch: string;
  scratchBranch: string;
  diffSummary?: string;
  confidence?: number;
  uncertainty?: number;
  provenance: string;
  hasProposal: boolean;
  proposalId?: string;
  proposalStatus?: string;
  /** Derived from proposal operations */
  changedAssetRefs: string[];
  inspectionRefs: string[];
}

export interface ToolTraceEntry {
  eventSequence: number;
  kind: string;
  packageId: string;
  summary: string;
  hasPlan: boolean;
  hasObservation: boolean;
  hasRisk: boolean;
  riskLevel?: string;
}

export interface InferenceTraceEntry {
  eventSequence: number;
  providerKind: string;
  modelPerformed: boolean;
  networkPerformed: boolean;
  outputAction: string;
  fingerprint?: string;
  hasReplay: boolean;
  replayMatched?: boolean;
}

export interface ControlAction {
  action: "approve" | "reject" | "cancel" | "promote" | "fork" | "archive";
  label: string;
  targetId: string;
  disabled: boolean;
  disabledReason: string;
  /** Public protocol payload preview (JSON snippet) */
  payloadPreview: string;
}

// --- Heuristic extractors (all public protocol, no runtime internals) ---

const RUN_LIFECYCLE_HINTS = [
  "run.created", "run.prepared", "run.running", "run.paused",
  "run.waiting_for_approval", "run.completed", "run.failed",
  "run.cancelled", "run.archived", "run_state", "working_state",
  "lifecycle_state",
];

const PLAN_NODE_HINTS = [
  "plan_node", "plan_graph", "node", "plan.node",
  "observe", "infer", "tool_call", "inspect", "branch_op",
  "compare", "propose", "wait",
];

const BRANCH_HINTS = [
  "scratch_branch", "target_branch", "branch_policy",
  "branch_lineage", "fork", "stale",
];

const CANDIDATE_HINTS = [
  "candidate", "candidate_id", "candidate_seed",
  "create_candidate", "compare_candidate",
];

const TOOL_TRACE_HINTS = [
  "tool_call", "tool_observation", "tool_risk",
  "tool_bridge", "toolchain", "tool_plan",
];

const INFERENCE_HINTS = [
  "inference", "infer", "provider_kind",
  "model_performed", "network_performed", "replay",
  "fingerprint", "inference_node",
];

const CONTROL_ACTIONS = [
  "approve", "reject", "cancel_run", "promote",
  "fork", "archive_candidate",
] as const;

function payloadHints(payload: unknown, hints: string[]): string[] {
  if (typeof payload !== "object" || payload === null) return [];
  const str = JSON.stringify(payload).toLowerCase();
  return hints.filter((h) => str.includes(h.toLowerCase()));
}

function extractStrings(payload: Record<string, unknown>, keys: string[]): string[] {
  const results: string[] = [];
  for (const key of keys) {
    const v = payload[key];
    if (typeof v === "string") results.push(v);
    if (Array.isArray(v)) {
      for (const item of v) {
        if (typeof item === "string") results.push(item);
      }
    }
  }
  return results;
}

function extractSafeString(payload: Record<string, unknown>, key: string): string | undefined {
  const v = payload[key];
  return typeof v === "string" && v.length > 0 ? v : undefined;
}

function extractSafeNumber(payload: Record<string, unknown>, key: string): number | undefined {
  const v = payload[key];
  return typeof v === "number" ? v : undefined;
}

/** Build the Forge Agent Workspace view model from public protocol data. */
export function buildForgeAgentWorkspace(
  events: KernelEvent[],
  proposals: ProposalRecord[],
  capabilities: RegisteredCapability[],
  packages: PackageRecord[],
  assets: AssetRecord[],
  projections: ProjectionRecord[],
): ForgeAgentWorkspaceModel {
  const runs: RunTimelineEntry[] = [];
  const planNodes: PlanGraphNode[] = [];
  const branchEntries: BranchLineageEntry[] = [];
  const candidates: CandidateCard[] = [];
  const toolTraces: ToolTraceEntry[] = [];
  const inferenceTraces: InferenceTraceEntry[] = [];
  const controlActions: ControlAction[] = [];

  // Map run IDs from events that carry run lifecycle fields
  const seenRunIds = new Set<string>();

  for (const event of events) {
    const p = typeof event.payload === "object" && event.payload !== null
      ? (event.payload as Record<string, unknown>)
      : {};

    // Check for run lifecycle signals
    const runHints = payloadHints(event.payload, RUN_LIFECYCLE_HINTS);
    if (runHints.length > 0) {
      const runId = extractSafeString(p, "run_id") ?? `run-from-event-${event.sequence}`;
      if (!seenRunIds.has(runId)) {
        seenRunIds.add(runId);
        runs.push({
          runId,
          label: extractSafeString(p, "label") ?? extractSafeString(p, "current_objective") ?? `Event #${event.sequence}`,
          status: extractSafeString(p, "lifecycle_state") ?? extractSafeString(p, "status") ?? "unknown",
          packageId: event.writer_package_id,
          eventSequence: event.sequence,
          nodeCount: extractSafeNumber(p, "node_count") ?? 0,
          edgeCount: extractSafeNumber(p, "edge_count") ?? 0,
          workingStateFields: extractStrings(p, [
            "working_state", "current_objective", "plan_graph_ref",
            "candidate_refs", "tool_observation_refs", "inference_trace_refs",
          ]),
        });
      }
    }

    // Plan graph nodes
    const nodeHints = payloadHints(event.payload, PLAN_NODE_HINTS);
    if (nodeHints.length > 0) {
      const runId = extractSafeString(p, "run_id") ?? "unknown";
      const nodeId = extractSafeString(p, "node_id") ?? `node-${event.sequence}`;
      planNodes.push({
        runId,
        nodeId,
        kind: extractSafeString(p, "kind") ?? extractSafeString(p, "node_kind") ?? "unknown",
        label: extractSafeString(p, "label") ?? extractSafeString(p, "objective") ?? nodeId,
        status: extractSafeString(p, "status") ?? extractSafeString(p, "node_status") ?? "unknown",
        inputRefs: extractStrings(p, ["input_refs", "inputs"]),
        outputRefs: extractStrings(p, ["output_refs", "outputs"]),
        approvalPolicy: extractSafeString(p, "approval_policy"),
      });
    }

    // Branch lineage signals
    const branchHints = payloadHints(event.payload, BRANCH_HINTS);
    if (branchHints.length > 0) {
      branchEntries.push({
        branchLabel: extractSafeString(p, "branch") ?? extractSafeString(p, "branch_ref") ?? `branch-event-${event.sequence}`,
        type: extractSafeString(p, "branch_type") === "scratch" ? "scratch"
          : extractSafeString(p, "branch_type") === "target" ? "target"
          : extractSafeString(p, "event_kind")?.includes("fork") ? "fork"
          : "lineage",
        sourceEvent: extractSafeString(p, "source_event") ?? extractSafeString(p, "forked_from"),
        targetRevision: extractSafeString(p, "target_revision") ?? extractSafeString(p, "revision"),
        intent: extractSafeString(p, "intent") ?? extractSafeString(p, "branch_intent"),
        promoteRequiresProposal: extractSafeString(p, "promote_requires_proposal") === "true",
        staleDetected: extractSafeString(p, "stale") === "true" || extractSafeString(p, "stale_target") === "true",
      });
    }

    // Tool traces
    const toolHints = payloadHints(event.payload, TOOL_TRACE_HINTS);
    if (toolHints.length > 0) {
      const riskLevels = extractStrings(p, ["overall_risk", "risk_level"]);
      toolTraces.push({
        eventSequence: event.sequence,
        kind: event.kind,
        packageId: event.writer_package_id,
        summary: toolHints.slice(0, 3).join(", "),
        hasPlan: extractSafeString(p, "tool_plan") !== undefined || extractSafeString(p, "plan_toolchain") !== undefined,
        hasObservation: extractSafeString(p, "observation") !== undefined || extractSafeString(p, "tool_observation") !== undefined,
        hasRisk: extractSafeString(p, "risk") !== undefined || extractSafeString(p, "tool_risk") !== undefined,
        riskLevel: riskLevels[0],
      });
    }

    // Inference traces
    const infHints = payloadHints(event.payload, INFERENCE_HINTS);
    if (infHints.length > 0) {
      inferenceTraces.push({
        eventSequence: event.sequence,
        providerKind: extractSafeString(p, "provider_kind") ?? extractSafeString(p, "provider") ?? "unknown",
        modelPerformed: p["model_performed"] === true,
        networkPerformed: p["network_performed"] === true,
        outputAction: extractSafeString(p, "output_action") ?? extractSafeString(p, "action") ?? "unknown",
        fingerprint: extractSafeString(p, "fingerprint") ?? extractSafeString(p, "plan_fingerprint"),
        hasReplay: extractSafeString(p, "replay_status") !== undefined,
        replayMatched: extractSafeString(p, "replay_status") === "replay_ok",
      });
    }
  }

  // Candidate cards from proposals
  for (const proposal of proposals) {
    const proposalStr = JSON.stringify(proposal).toLowerCase();
    const isCandidate = CANDIDATE_HINTS.some((h) => proposalStr.includes(h.toLowerCase()));
    if (isCandidate || proposal.operations.length > 0) {
      const p = typeof proposal.expected_effects === "object" && proposal.expected_effects !== null
        ? (proposal.expected_effects as Record<string, unknown>)
        : {};
      candidates.push({
        candidateId: proposal.id,
        runId: extractSafeString(p, "run_id") ?? "unknown",
        status: proposal.status,
        targetBranch: extractSafeString(p, "target_branch") ?? extractSafeString(p, "target_branch_ref") ?? "unknown",
        scratchBranch: extractSafeString(p, "scratch_branch") ?? extractSafeString(p, "scratch_branch_ref") ?? "unknown",
        diffSummary: extractSafeString(p, "diff_summary") ?? extractSafeString(p, "summary"),
        confidence: extractSafeNumber(p, "confidence"),
        uncertainty: extractSafeNumber(p, "uncertainty"),
        provenance: extractSafeString(p, "provenance") ?? proposal.id,
        hasProposal: proposal.status !== "created",
        proposalId: proposal.id,
        proposalStatus: proposal.status,
        changedAssetRefs: extractStrings(p, ["changed_asset_refs", "asset_refs"]),
        inspectionRefs: extractStrings(p, ["inspection_refs", "inspection"]),
      });
    }
  }

  // Control actions derived from current visible state
  // Approve: for candidates with status "draft" or proposals with status "created"
  for (const proposal of proposals) {
    if (proposal.status === "created") {
      controlActions.push({
        action: "approve",
        label: `Approve ${proposal.id.slice(0, 12)}`,
        targetId: proposal.id,
        disabled: false,
        disabledReason: "",
        payloadPreview: JSON.stringify({ proposal_id: proposal.id, reason: "forge-workspace" }, null, 2),
      });
    }
    if (proposal.status === "created" || proposal.status === "approved") {
      controlActions.push({
        action: "reject",
        label: `Reject ${proposal.id.slice(0, 12)}`,
        targetId: proposal.id,
        disabled: false,
        disabledReason: "",
        payloadPreview: JSON.stringify({ proposal_id: proposal.id, reason: "rejected-from-workspace" }, null, 2),
      });
    }
  }
  // Cancel: for active runs
  for (const run of runs) {
    if (["prepared", "running", "paused", "waiting_for_approval"].includes(run.status)) {
      controlActions.push({
        action: "cancel",
        label: `Cancel ${run.runId.slice(0, 12)}`,
        targetId: run.runId,
        disabled: false,
        disabledReason: "",
        payloadPreview: JSON.stringify({ run_id: run.runId, reason: "user-cancelled" }, null, 2),
      });
    }
  }
  // Promote: for ready candidates
  for (const c of candidates) {
    if (c.status === "ready" || c.status === "draft") {
      controlActions.push({
        action: "promote",
        label: `Promote ${c.candidateId.slice(0, 12)}`,
        targetId: c.candidateId,
        disabled: false,
        disabledReason: "",
        payloadPreview: JSON.stringify({ candidate_id: c.candidateId, target_branch: c.targetBranch }, null, 2),
      });
    }
  }
  // Fork
  if (events.length > 0) {
    controlActions.push({
      action: "fork",
      label: "Fork session",
      targetId: "",
      disabled: false,
      disabledReason: "",
      payloadPreview: JSON.stringify({ forked_from_sequence: events[events.length - 1].sequence }, null, 2),
    });
  }
  // Archive: for candidates that are ready or promoting
  for (const c of candidates) {
    if (c.status === "ready" || c.status === "promoting") {
      controlActions.push({
        action: "archive",
        label: `Archive ${c.candidateId.slice(0, 12)}`,
        targetId: c.candidateId,
        disabled: false,
        disabledReason: "",
        payloadPreview: JSON.stringify({ candidate_id: c.candidateId }, null, 2),
      });
    }
  }
  // Add disabled-safe controls as affordance stubs
  controlActions.push({
    action: "approve",
    label: "Approve (disabled-safe)",
    targetId: "__stub__",
    disabled: true,
    disabledReason: "No actionable approval target — needs a run waiting_for_approval or proposal in created status",
    payloadPreview: JSON.stringify({ proposal_id: "<proposal-id>", reason: "forge-workspace" }, null, 2),
  });
  controlActions.push({
    action: "cancel",
    label: "Cancel run (disabled-safe)",
    targetId: "__stub__",
    disabled: true,
    disabledReason: "No active run to cancel — needs a run in prepared/running/paused/waiting_for_approval status",
    payloadPreview: JSON.stringify({ run_id: "<run-id>", reason: "user-cancelled" }, null, 2),
  });
  controlActions.push({
    action: "promote",
    label: "Promote candidate (disabled-safe)",
    targetId: "__stub__",
    disabled: true,
    disabledReason: "No ready candidate to promote — needs a candidate in ready or draft status",
    payloadPreview: JSON.stringify({ candidate_id: "<candidate-id>", target_branch: "<target-branch>" }, null, 2),
  });

  return {
    runs,
    planNodes,
    branchEntries,
    candidates,
    toolTraces,
    inferenceTraces,
    controlActions,
  };
}

// --- Rendering helpers ---

/** Render the full Agentic Forge Workspace sections. */
export function renderForgeAgentWorkspaceSections(model: ForgeAgentWorkspaceModel): string {
  const totalRuns = model.runs.length;
  const totalNodes = model.planNodes.length;
  const totalBranches = model.branchEntries.length;
  const totalCandidates = model.candidates.length;
  const totalTools = model.toolTraces.length;
  const totalInferences = model.inferenceTraces.length;

  return `
    <div class="forge-section forge-workspace-section">
      <div class="section-header">
        <h2>Agentic Forge Workspace</h2>
        <span class="section-meta" title="Third-party agentic-forge-lab packages can contribute runs, plan graphs, candidates, and traces via public protocol events. No kernel agent namespace.">
          ${totalRuns} run${totalRuns === 1 ? "" : "s"} · ${totalNodes} node${totalNodes === 1 ? "" : "s"} · ${totalCandidates} candidate${totalCandidates === 1 ? "" : "s"} · ${totalBranches} branch${totalBranches === 1 ? "" : ""}
        </span>
      </div>
      <p class="workspace-note">
        Agentic Forge operates on runs, plans, candidates, diffs, proposals, and traces — not chat transcripts.
        All data is derived from public protocol events, proposals, surfaces, and capabilities.
        Agent-like packages (official or third-party) drive these panels through package-owned events and proposals.
      </p>

      <div class="forge-workspace-grid">
        ${renderRunTimelineSection(model.runs)}
        ${renderPlanGraphSection(model.planNodes)}
        ${renderBranchLineageSection(model.branchEntries)}
        ${renderCandidateCompareSection(model.candidates)}
        ${renderToolTraceSection(model.toolTraces)}
        ${renderInferenceTraceSection(model.inferenceTraces)}
      </div>

      ${renderControlActionsSection(model.controlActions)}
    </div>
  `;
}

function renderRunTimelineSection(runs: RunTimelineEntry[]): string {
  return `
    <details class="workspace-panel" open>
      <summary class="workspace-panel-header">
        <span class="panel-icon">◈</span>
        <span class="panel-title">Run Timeline</span>
        <span class="section-meta">${runs.length} run${runs.length === 1 ? "" : "s"}</span>
      </summary>
      <div class="workspace-panel-body">
        ${runs.length === 0 ? `<p class="empty">No run lifecycle events detected. Load a package that emits run-lifecycle events (e.g., an agentic-forge-lab package).</p>` : `
          <div class="run-timeline">
            ${runs.map(renderRunEntry).join("")}
          </div>
        `}
        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (run lifecycle event)</summary>
          <pre class="protocol-preview-code">${formatJson({
            kind: "kernel/v1/event.append or package-owned run.*",
            payload: {
              run_id: "<package-owned-run-id>",
              lifecycle_state: "created | prepared | running | paused | waiting_for_approval | completed | failed | cancelled | archived",
              label: "optional human label",
              current_objective: "optional objective string",
              node_count: 0,
              edge_count: 0,
              working_state: { /* package-owned fields */ },
            },
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

function renderRunEntry(run: RunTimelineEntry): string {
  const badgeClass = statusBadgeClass(run.status);
  return `
    <article class="run-entry">
      <div class="run-entry-header">
        <span class="run-status-dot ${badgeClass}"></span>
        <strong class="run-label">${escapeHtml(run.label)}</strong>
        <span class="surface-chip ${badgeClass}">${escapeHtml(run.status)}</span>
      </div>
      <div class="run-entry-meta">
        <span class="run-meta-item">ID: ${escapeHtml(run.runId.slice(0, 16))}</span>
        <span class="run-meta-item">Package: ${escapeHtml(run.packageId)}</span>
        <span class="run-meta-item">Nodes: ${run.nodeCount} · Edges: ${run.edgeCount}</span>
        <span class="run-meta-item">Event: #${run.eventSequence}</span>
      </div>
      ${run.workingStateFields.length > 0 ? `
        <div class="run-working-state">
          <span class="run-meta-label">Working state fields:</span>
          ${run.workingStateFields.map((f) => `<span class="surface-chip">${escapeHtml(f)}</span>`).join("")}
        </div>
      ` : ""}
    </article>
  `;
}

function statusBadgeClass(status: string): string {
  const s = status.toLowerCase();
  if (["completed", "ready", "active", "ok", "running"].some((v) => s.includes(v))) return "status-ok";
  if (["failed", "error", "cancelled", "rejected", "archived"].some((v) => s.includes(v))) return "status-error";
  if (["paused", "waiting", "pending", "draft"].some((v) => s.includes(v))) return "status-warn";
  return "status-info";
}

function renderPlanGraphSection(nodes: PlanGraphNode[]): string {
  return `
    <details class="workspace-panel" open>
      <summary class="workspace-panel-header">
        <span class="panel-icon">◈</span>
        <span class="panel-title">Plan Graph <span class="panel-subtitle">Read-only</span></span>
        <span class="section-meta">${nodes.length} node${nodes.length === 1 ? "" : "s"}</span>
      </summary>
      <div class="workspace-panel-body">
        ${nodes.length === 0 ? `<p class="empty">No plan graph events detected. Load a package that emits plan-graph events (e.g., an agentic-forge-lab package with plan_node payloads).</p>` : `
          <div class="plan-node-list">
            ${nodes.map(renderPlanNode).join("")}
          </div>
        `}
        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (plan graph artifact)</summary>
          <pre class="protocol-preview-code">${formatJson({
            run_id: "<package-owned-run-id>",
            plan_graph: {
              nodes: [{ id: "node-1", kind: "observe|infer|tool_call|inspect|branch_op|compare|propose|wait", label: "...", status: "...", input_refs: [], output_refs: [], approval_policy: "none|user_approval|fork_then_approve" }],
              edges: [{ from: "node-1", to: "node-2" }],
              revision: 1,
              deterministic_mode: true,
            },
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

function renderPlanNode(node: PlanGraphNode): string {
  return `
    <article class="plan-node-entry">
      <div class="plan-node-header">
        <span class="plan-node-kind-badge">${escapeHtml(node.kind)}</span>
        <strong>${escapeHtml(node.label)}</strong>
        <span class="surface-chip">${escapeHtml(node.status)}</span>
      </div>
      <div class="plan-node-meta">
        <span class="run-meta-item">ID: ${escapeHtml(node.nodeId.slice(0, 20))}</span>
        <span class="run-meta-item">Run: ${escapeHtml(node.runId.slice(0, 12))}</span>
        ${node.approvalPolicy ? `<span class="approval-policy policy-${escapeHtml(node.approvalPolicy)}">${escapeHtml(node.approvalPolicy)}</span>` : ""}
      </div>
      ${node.inputRefs.length > 0 ? `<div class="plan-node-refs"><span class="run-meta-label">Inputs:</span> ${node.inputRefs.map((r) => `<span class="surface-chip">${escapeHtml(r)}</span>`).join("")}</div>` : ""}
      ${node.outputRefs.length > 0 ? `<div class="plan-node-refs"><span class="run-meta-label">Outputs:</span> ${node.outputRefs.map((r) => `<span class="surface-chip">${escapeHtml(r)}</span>`).join("")}</div>` : ""}
    </article>
  `;
}

function renderBranchLineageSection(entries: BranchLineageEntry[]): string {
  return `
    <details class="workspace-panel" open>
      <summary class="workspace-panel-header">
        <span class="panel-icon">◈</span>
        <span class="panel-title">Branch Diff / Lineage</span>
        <span class="section-meta">${entries.length} entry${entries.length === 1 ? "" : "ies"}</span>
      </summary>
      <div class="workspace-panel-body">
        ${entries.length === 0 ? `<p class="empty">No branch lineage events detected. Load a package that emits branch-policy or scratch-branch events.</p>` : `
          <div class="branch-lineage-list">
            ${entries.map(renderBranchEntry).join("")}
          </div>
        `}
        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (branch policy / scratch branch)</summary>
          <pre class="protocol-preview-code">${formatJson({
            run_id: "<package-owned-run-id>",
            scratch_branch: {
              intent: "...",
              target_revision: "<revision>",
              promote_requires_proposal: true,
              stale_target_blocks_promote: true,
            },
            branch_type: "scratch | target",
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

function renderBranchEntry(entry: BranchLineageEntry): string {
  const typeIcon = entry.type === "scratch" ? "↳" : entry.type === "target" ? "◉" : entry.type === "fork" ? "↘" : "─";
  return `
    <article class="branch-entry">
      <div class="branch-entry-header">
        <span class="branch-type-icon">${typeIcon}</span>
        <strong>${escapeHtml(entry.branchLabel)}</strong>
        <span class="surface-chip">${escapeHtml(entry.type)}</span>
        ${entry.staleDetected ? `<span class="safety-badge severity-warn">stale</span>` : ""}
        ${entry.promoteRequiresProposal ? `<span class="safety-badge severity-info">requires proposal</span>` : ""}
      </div>
      <div class="branch-entry-meta">
        ${entry.targetRevision ? `<span class="run-meta-item">Revision: ${escapeHtml(entry.targetRevision)}</span>` : ""}
        ${entry.intent ? `<span class="run-meta-item">Intent: ${escapeHtml(entry.intent)}</span>` : ""}
        ${entry.sourceEvent ? `<span class="run-meta-item">Source: ${escapeHtml(entry.sourceEvent)}</span>` : ""}
      </div>
    </article>
  `;
}

function renderCandidateCompareSection(candidates: CandidateCard[]): string {
  return `
    <details class="workspace-panel" open>
      <summary class="workspace-panel-header">
        <span class="panel-icon">◈</span>
        <span class="panel-title">Candidate Compare / Promote</span>
        <span class="section-meta">${candidates.length} candidate${candidates.length === 1 ? "" : "s"}</span>
      </summary>
      <div class="workspace-panel-body">
        ${candidates.length === 0 ? `<p class="empty">No candidate-like proposals or events detected. Load a package that emits candidate artifacts (e.g., an agentic-forge-lab package with create_candidate capability).</p>` : `
          <div class="candidate-list">
            ${candidates.map(renderCandidateCard).join("")}
          </div>
        `}
        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (candidate artifact)</summary>
          <pre class="protocol-preview-code">${formatJson({
            candidate_id: "<package-owned-candidate-id>",
            run_id: "<package-owned-run-id>",
            target_branch_ref: "<branch>",
            scratch_branch_ref: "<branch>",
            diff_summary: "optional diff summary",
            changed_asset_refs: ["asset-1"],
            projection_refs: ["projection-1"],
            inspection_refs: ["inspection-1"],
            confidence: 0.0,
            uncertainty: 0.0,
            provenance: "...",
            status: "draft|ready|comparing|promoting|promoted|rejected|archived|failed",
            target_revision: "<revision>",
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

function renderCandidateCard(c: CandidateCard): string {
  const statusClass = statusBadgeClass(c.status);
  return `
    <article class="candidate-entry">
      <div class="candidate-entry-header">
        <span class="run-status-dot ${statusClass}"></span>
        <strong>${escapeHtml(c.candidateId.slice(0, 20))}</strong>
        <span class="surface-chip ${statusClass}">${escapeHtml(c.status)}</span>
        ${c.hasProposal ? `<span class="safety-badge severity-info">proposal: ${escapeHtml(c.proposalStatus ?? "")}</span>` : ""}
      </div>
      <div class="candidate-entry-meta">
        <span class="run-meta-item">Run: ${escapeHtml(c.runId.slice(0, 12))}</span>
        <span class="run-meta-item">Target: ${escapeHtml(c.targetBranch)}</span>
        <span class="run-meta-item">Scratch: ${escapeHtml(c.scratchBranch)}</span>
        <span class="run-meta-item">Provenance: ${escapeHtml(c.provenance.slice(0, 24))}</span>
      </div>
      ${c.diffSummary ? `<div class="candidate-diff"><span class="run-meta-label">Diff summary:</span><code>${escapeHtml(c.diffSummary.slice(0, 200))}</code></div>` : ""}
      ${c.confidence !== undefined ? `<div class="candidate-stats"><span class="run-meta-item">Confidence: ${c.confidence}</span><span class="run-meta-item">Uncertainty: ${c.uncertainty ?? "?"}</span></div>` : ""}
      ${c.changedAssetRefs.length > 0 ? `<div class="candidate-refs"><span class="run-meta-label">Changed assets:</span> ${c.changedAssetRefs.map((r) => `<span class="surface-chip">${escapeHtml(r)}</span>`).join("")}</div>` : ""}
      ${c.inspectionRefs.length > 0 ? `<div class="candidate-refs"><span class="run-meta-label">Inspections:</span> ${c.inspectionRefs.map((r) => `<span class="surface-chip">${escapeHtml(r)}</span>`).join("")}</div>` : ""}
    </article>
  `;
}

function renderToolTraceSection(traces: ToolTraceEntry[]): string {
  return `
    <details class="workspace-panel" open>
      <summary class="workspace-panel-header">
        <span class="panel-icon">◈</span>
        <span class="panel-title">Tool / Inference Trace</span>
        <span class="section-meta">${traces.length} tool trace${traces.length === 1 ? "" : "s"}</span>
      </summary>
      <div class="workspace-panel-body">
        ${traces.length === 0 ? `<p class="empty">No tool trace events detected. Load a package that emits tool-bridge or tool-observation events (e.g., capability-tool-bridge-lab).</p>` : `
          <div class="tool-trace-list">
            ${traces.map(renderToolTrace).join("")}
          </div>
        `}
        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (tool trace)</summary>
          <pre class="protocol-preview-code">${formatJson({
            event_kind: "tool_bridge | tool_call | tool_observation | tool_risk | plan_toolchain",
            payload: {
              tool_call_context: { requesting_package: "...", run_id: "...", no_execution: true, no_ambient_authority: true, requires_approval: true },
              observation: { untrusted: true, redaction_state: "safe|unsafe_blocked" },
              risk: { categories: ["prompt_injection", "secret_exfiltration"], overall_risk: "critical|high|medium|low" },
            },
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

function renderToolTrace(t: ToolTraceEntry): string {
  return `
    <article class="trace-entry">
      <div class="trace-entry-header">
        <span class="timeline-badge tool-badge">tool</span>
        <strong>${escapeHtml(t.kind)}</strong>
        <span class="run-meta-item">#${t.eventSequence} · ${escapeHtml(t.packageId)}</span>
      </div>
      <div class="trace-entry-meta">
        <span class="trace-chip ${t.hasPlan ? "active" : ""}">${t.hasPlan ? "●" : "○"} plan</span>
        <span class="trace-chip ${t.hasObservation ? "active" : ""}">${t.hasObservation ? "●" : "○"} observation</span>
        <span class="trace-chip ${t.hasRisk ? "active" : ""}">${t.hasRisk ? "●" : "○"} risk</span>
        ${t.riskLevel ? `<span class="safety-badge severity-${t.riskLevel === "critical" || t.riskLevel === "high" ? "error" : t.riskLevel === "medium" ? "warn" : "info"}">${escapeHtml(t.riskLevel)}</span>` : ""}
      </div>
      <div class="trace-entry-summary">${escapeHtml(t.summary)}</div>
    </article>
  `;
}

function renderInferenceTraceSection(traces: InferenceTraceEntry[]): string {
  return `
    <details class="workspace-panel" open>
      <summary class="workspace-panel-header">
        <span class="panel-icon">◈</span>
        <span class="panel-title">Inference Traces</span>
        <span class="section-meta">${traces.length} trace${traces.length === 1 ? "" : "s"}</span>
      </summary>
      <div class="workspace-panel-body">
        ${traces.length === 0 ? `<p class="empty">No inference trace events detected. Load a package that emits inference events (e.g., an agentic-forge-lab package with run_inference_node).</p>` : `
          <div class="inference-trace-list">
            ${traces.map(renderInferenceTrace).join("")}
          </div>
        `}
        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (inference trace)</summary>
          <pre class="protocol-preview-code">${formatJson({
            event_kind: "inference | inference_node | replay_inference_node",
            payload: {
              provider_kind: "deterministic|recorded|cloud_adapter_plan|local_fake",
              model_performed: false,
              network_performed: false,
              output_action: "candidate_seed|proposal_seed|observation|needs_repair",
              fingerprint: "<deterministic-fingerprint>",
              replay_status: "replay_ok|replay_mismatch",
            },
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

function renderInferenceTrace(t: InferenceTraceEntry): string {
  return `
    <article class="trace-entry">
      <div class="trace-entry-header">
        <span class="timeline-badge trace-badge">inference</span>
        <strong>${escapeHtml(t.providerKind)}</strong>
        <span class="run-meta-item">#${t.eventSequence}</span>
      </div>
      <div class="trace-entry-meta">
        <span class="trace-chip ${t.modelPerformed ? "active" : ""}">${t.modelPerformed ? "●" : "○"} model</span>
        <span class="trace-chip ${t.networkPerformed ? "active" : ""}">${t.networkPerformed ? "●" : "○"} network</span>
        <span class="trace-chip ${t.hasReplay ? "active" : ""}">${t.hasReplay ? "●" : "○"} replay</span>
        ${t.replayMatched !== undefined ? `<span class="safety-badge ${t.replayMatched ? "severity-ok" : "severity-warn"}">${t.replayMatched ? "match" : "mismatch"}</span>` : ""}
      </div>
      <div class="trace-entry-summary">
        <span class="run-meta-item">Output action: ${escapeHtml(t.outputAction)}</span>
        ${t.fingerprint ? `<span class="run-meta-item">Fingerprint: ${escapeHtml(t.fingerprint.slice(0, 20))}…</span>` : ""}
      </div>
    </article>
  `;
}

function renderControlActionsSection(actions: ControlAction[]): string {
  // Group by action type for display
  const live = actions.filter((a) => !a.disabled);
  const disabled = actions.filter((a) => a.disabled);

  return `
    <details class="workspace-panel workspace-controls-panel" open>
      <summary class="workspace-panel-header">
        <span class="panel-icon">◈</span>
        <span class="panel-title">Controls</span>
        <span class="section-meta">${live.length} live · ${disabled.length} disabled-safe</span>
      </summary>
      <div class="workspace-panel-body">
        <p class="workspace-note">
          These controls show public-protocol payload previews. Actions are proposal-gated and plan-only — no real model, no network, no ambient authority.
          Agentic Forge packages (official or third-party) interpret these payloads through their own capabilities.
        </p>
        ${live.length > 0 ? `
          <div class="control-action-list">
            <h3 class="slot-title">Live actions</h3>
            ${live.map(renderControlAction).join("")}
          </div>
        ` : ""}
        ${disabled.length > 0 ? `
          <div class="control-action-list">
            <h3 class="slot-title">Disabled-safe affordances (no current target)</h3>
            ${disabled.map(renderControlAction).join("")}
          </div>
        ` : ""}
      </div>
    </details>
  `;
}

function renderControlAction(action: ControlAction): string {
  const actionIcon = action.action === "approve" ? "✓" : action.action === "reject" ? "✗" : action.action === "cancel" ? "⊘" : action.action === "promote" ? "↑" : action.action === "fork" ? "↘" : "☰";
  return `
    <article class="control-action-entry ${action.disabled ? "disabled-safe" : ""}">
      <div class="control-action-header">
        <span class="control-action-icon">${actionIcon}</span>
        <strong>${escapeHtml(action.action)}</strong>
        <span class="run-meta-item">${escapeHtml(action.label)}</span>
        ${action.disabled ? `<span class="safety-badge severity-info">disabled-safe</span>` : `<span class="safety-badge severity-ok">ready</span>`}
      </div>
      ${action.disabledReason ? `<p class="control-action-reason">${escapeHtml(action.disabledReason)}</p>` : ""}
      <details class="protocol-preview-details">
        <summary class="protocol-preview-summary">Public protocol payload preview</summary>
        <pre class="protocol-preview-code">${escapeHtml(action.payloadPreview)}</pre>
      </details>
      ${action.disabled
        ? `<button type="button" class="button-disabled-safe" disabled title="${escapeHtml(action.disabledReason)}">${escapeHtml(action.action)} (disabled-safe)</button>`
        : `<button type="button" class="button-control ${action.action === "approve" ? "button-success" : action.action === "reject" ? "button-warn" : ""}" data-action="forge-control" data-control-action="${escapeHtml(action.action)}" data-target-id="${escapeHtml(action.targetId)}">${escapeHtml(action.action)}</button>`
      }
    </article>
  `;
}
