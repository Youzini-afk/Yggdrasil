// ========================================================================
// Experience Beta 3 — Forge Experience Observability UI
// ========================================================================
// Public-protocol-only panels for session health, package health, agent run
// health, proposal causal chain, failure breadcrumbs, cost/latency summary,
// asset provenance, and guardrail/audit summary.
//
// All data is heuristically extracted from public protocol types (events,
// proposals, packages, surfaces, assets). No kernel internals, no model
// calls, no SQLite, no runtime private modules. Chat-first patterns are
// deliberately excluded — this is a creation-workspace observability UI.
// ========================================================================

import type {
  AssetRecord,
  KernelEvent,
  PackageRecord,
  ProposalRecord,
  RegisteredCapability,
  SurfaceContributionRecord,
} from "../protocol/client";
import { escapeHtml, formatJson } from "../utils/html";

// ========================================================================
// Types
// ========================================================================

export interface ExperienceObservabilityModel {
  sessionHealth: SessionHealth;
  packages: PackageHealth[];
  agentRuns: AgentRunHealth[];
  causalChain: ProposalCausalChain[];
  failureBreadcrumbs: FailureBreadcrumb[];
  costLatency: CostLatencySummary;
  assetProvenance: AssetProvenanceSummary[];
  guardrailAudit: GuardrailAuditSummary;
}

export interface SessionHealth {
  sessionId: string;
  status: "active" | "forked" | "closed" | "unknown";
  eventCount: number;
  firstEventSequence: number;
  lastEventSequence: number;
  estimatedDurationMs: number;
  forkCount: number;
  label: string;
}

export interface PackageHealth {
  packageId: string;
  state: string;
  entryKind: string;
  capabilityCount: number;
  eventCount: number;
  surfaceCount: number;
  lastActiveSequence: number;
}

export interface AgentRunHealth {
  runId: string;
  label: string;
  status: string;
  packageId: string;
  nodeCount: number;
  edgeCount: number;
  estimatedDurationMs: number;
  startSequence: number;
  endSequence?: number;
  hasFailure: boolean;
  failureReason?: string;
}

export interface ProposalCausalChain {
  proposalId: string;
  status: string;
  targetSessionId?: string;
  parentProposalId?: string;
  childProposalIds: string[];
  operationCount: number;
  sequence: number;
  derivedFromEventSequence?: number;
}

export interface FailureBreadcrumb {
  sequence: number;
  kind: string;
  packageId: string;
  reason: string;
  severity: "error" | "warn" | "info";
  source: "event" | "proposal" | "inference";
  relatedRunId?: string;
  relatedProposalId?: string;
}

export interface CostLatencySummary {
  totalEvents: number;
  estimatedCost: string;
  estimatedLatencyMs: number;
  inferenceCount: number;
  toolCallCount: number;
  streamEventCount: number;
  proposalCount: number;
  costBreakdown: CostBreakdownItem[];
}

export interface CostBreakdownItem {
  category: string;
  count: number;
  estimatedCost: string;
}

export interface AssetProvenanceSummary {
  assetId: string;
  originPackageId: string;
  mime: string;
  sizeBytes: number;
  proposedBy?: string;
  approvedBy?: string;
  usedInRuns: string[];
  hash: string;
}

export interface GuardrailAuditSummary {
  totalGuardrailChecks: number;
  blockedCount: number;
  warningCount: number;
  passedCount: number;
  redactionCount: number;
  ambiguousProviderCount: number;
  rejectionCount: number;
  permissionDeniedCount: number;
  guardrailEntries: GuardrailEntry[];
}

export interface GuardrailEntry {
  sequence: number;
  kind: string;
  verdict: "pass" | "block" | "warn";
  detail: string;
}

// ========================================================================
// Heuristic detection helpers (public protocol strings only)
// ========================================================================

const FAILURE_KIND_HINTS = [
  "kernel/v1/stream.error", "stream.error", "stream.cancelled", "stream.timeout",
  "run.failed", "run.cancelled", "failed", "cancelled", "error",
  "rejected", "denied", "unsafe_blocked", "provider_missing",
];

const FAILURE_PAYLOAD_HINTS = [
  "error", "failure", "exception", "rejected", "denied",
  "unsafe_blocked", "provider_missing", "reason", "message",
  "traceback", "stack",
];

const COST_HINT_FIELDS = ["cost", "tokens", "latency", "duration_ms", "spent"];
const INFERENCE_KIND_HINTS = ["inference", "infer", "model_performed", "network_performed"];
const TOOL_CALL_KIND_HINTS = ["tool_bridge", "tool_call", "capability.invoke", "capability.stream"];

function payloadContains(event: KernelEvent, hints: string[]): string[] {
  if (typeof event.payload !== "object" || event.payload === null) return [];
  const str = JSON.stringify(event.payload).toLowerCase();
  return hints.filter((h) => str.includes(h.toLowerCase()));
}

function extractStringField(payload: Record<string, unknown>, keys: string[]): string | undefined {
  for (const key of keys) {
    const v = payload[key];
    if (typeof v === "string" && v.length > 0) return v;
  }
  return undefined;
}

function extractNumberField(payload: Record<string, unknown>, keys: string[]): number | undefined {
  for (const key of keys) {
    const v = payload[key];
    if (typeof v === "number") return v;
  }
  return undefined;
}

// ========================================================================
// Builder — produces ExperienceObservabilityModel from public protocol data
// ========================================================================

export function buildExperienceObservability(
  events: KernelEvent[],
  proposals: ProposalRecord[],
  packages: PackageRecord[],
  capabilities: RegisteredCapability[],
  allSurfaces: SurfaceContributionRecord[],
  assets: AssetRecord[],
  sessionId?: string,
): ExperienceObservabilityModel {
  // --- Session Health ---
  const sessionHealth = buildSessionHealth(events, sessionId);

  // --- Package Health ---
  const packageHealth = buildPackageHealth(packages, events, allSurfaces);

  // --- Agent Run Health ---
  const agentRuns = buildAgentRunHealth(events);

  // --- Proposal Causal Chain ---
  const causalChain = buildProposalCausalChain(proposals, events);

  // --- Failure Breadcrumbs ---
  const failureBreadcrumbs = buildFailureBreadcrumbs(events, proposals);

  // --- Cost / Latency Summary ---
  const costLatency = buildCostLatencySummary(events, proposals, capabilities);

  // --- Asset Provenance ---
  const assetProvenance = buildAssetProvenance(assets, proposals, events);

  // --- Guardrail / Audit Summary ---
  const guardrailAudit = buildGuardrailAuditSummary(events, proposals);

  return {
    sessionHealth,
    packages: packageHealth,
    agentRuns,
    causalChain,
    failureBreadcrumbs,
    costLatency,
    assetProvenance,
    guardrailAudit,
  };
}

function buildSessionHealth(events: KernelEvent[], sessionId?: string): SessionHealth {
  const id = sessionId ?? "not-opened";
  const eventCount = events.length;
  const firstSeq = events.length > 0 ? events[0]!.sequence : 0;
  const lastSeq = events.length > 0 ? events[events.length - 1]!.sequence : 0;
  const firstTime = events.length > 0 ? new Date(events[0]!.created_at).getTime() : 0;
  const lastTime = events.length > 0 ? new Date(events[events.length - 1]!.created_at).getTime() : 0;
  const durationMs = firstTime > 0 && lastTime > 0 ? lastTime - firstTime : 0;

  // Count fork-related events
  const forkCount = events.filter((e) => e.kind.toLowerCase().includes("fork")).length;

  // Determine status
  let status: SessionHealth["status"] = "unknown";
  if (eventCount === 0) {
    status = "unknown";
  } else {
    const hasClose = events.some((e) => e.kind.toLowerCase().includes("close") || e.kind.toLowerCase().includes("end_session"));
    const hasFork = forkCount > 0;
    if (hasClose) status = "closed";
    else if (hasFork) status = "forked";
    else status = "active";
  }

  return {
    sessionId: id,
    status,
    eventCount,
    firstEventSequence: firstSeq,
    lastEventSequence: lastSeq,
    estimatedDurationMs: durationMs,
    forkCount,
    label: sessionId ? `Session ${id.slice(0, 8)}` : "No session opened",
  };
}

function buildPackageHealth(
  packages: PackageRecord[],
  events: KernelEvent[],
  allSurfaces: SurfaceContributionRecord[],
): PackageHealth[] {
  const eventCountByPackage = new Map<string, number>();
  const lastSeqByPackage = new Map<string, number>();

  for (const event of events) {
    const pid = event.writer_package_id;
    eventCountByPackage.set(pid, (eventCountByPackage.get(pid) ?? 0) + 1);
    lastSeqByPackage.set(pid, Math.max(lastSeqByPackage.get(pid) ?? 0, event.sequence));
  }

  const surfaceCountByPackage = new Map<string, number>();
  for (const s of allSurfaces) {
    surfaceCountByPackage.set(s.package_id, (surfaceCountByPackage.get(s.package_id) ?? 0) + 1);
  }

  return packages.map((pkg) => ({
    packageId: pkg.id,
    state: pkg.state,
    entryKind: pkg.entry_kind,
    capabilityCount: pkg.capability_count,
    eventCount: eventCountByPackage.get(pkg.id) ?? 0,
    surfaceCount: surfaceCountByPackage.get(pkg.id) ?? 0,
    lastActiveSequence: lastSeqByPackage.get(pkg.id) ?? 0,
  }));
}

function buildAgentRunHealth(events: KernelEvent[]): AgentRunHealth[] {
  const runMap = new Map<string, {
    runId: string;
    label: string;
    status: string;
    packageId: string;
    nodeCount: number;
    edgeCount: number;
    startSequence: number;
    endSequence?: number;
    hasFailure: boolean;
    failureReason?: string;
    timestamps: number[];
  }>();

  for (const event of events) {
    const p = typeof event.payload === "object" && event.payload !== null
      ? (event.payload as Record<string, unknown>)
      : {};

    const runId = extractStringField(p, ["run_id", "runId"]);
    if (!runId) continue;

    if (!runMap.has(runId)) {
      runMap.set(runId, {
        runId,
        label: extractStringField(p, ["label", "current_objective"]) ?? `Run ${runId.slice(0, 8)}`,
        status: extractStringField(p, ["lifecycle_state", "status"]) ?? "unknown",
        packageId: event.writer_package_id,
        nodeCount: extractNumberField(p, ["node_count", "nodeCount"]) ?? 0,
        edgeCount: extractNumberField(p, ["edge_count", "edgeCount"]) ?? 0,
        startSequence: event.sequence,
        endSequence: undefined,
        hasFailure: false,
        failureReason: undefined,
        timestamps: [new Date(event.created_at).getTime()],
      });
    }

    const entry = runMap.get(runId)!;

    // Update status from later events
    const newStatus = extractStringField(p, ["lifecycle_state", "status"]);
    if (newStatus) entry.status = newStatus;

    entry.packageId = event.writer_package_id;
    entry.nodeCount = Math.max(entry.nodeCount, extractNumberField(p, ["node_count", "nodeCount"]) ?? 0);
    entry.edgeCount = Math.max(entry.edgeCount, extractNumberField(p, ["edge_count", "edgeCount"]) ?? 0);

    if (["run.completed", "run.failed", "run.cancelled", "completed", "failed", "cancelled"].some((s) => event.kind.toLowerCase().includes(s))) {
      entry.endSequence = event.sequence;
    }

    const ts = new Date(event.created_at).getTime();
    if (!isNaN(ts)) entry.timestamps.push(ts);

    // Failure detection
    const isFailure = event.kind.toLowerCase().includes("failed") || event.kind.toLowerCase().includes("cancelled");
    if (isFailure || FAILURE_PAYLOAD_HINTS.some((h) => JSON.stringify(event.payload).toLowerCase().includes(h))) {
      entry.hasFailure = true;
      entry.failureReason = extractStringField(p, ["reason", "error", "message", "failure_reason", "failureReason"]);
    }
  }

  const runs: AgentRunHealth[] = [];
  for (const [, entry] of runMap) {
    const timestamps = entry.timestamps.filter((t) => !isNaN(t));
    const duration = timestamps.length >= 2
      ? Math.max(...timestamps) - Math.min(...timestamps)
      : 0;
    runs.push({
      runId: entry.runId,
      label: entry.label,
      status: entry.status,
      packageId: entry.packageId,
      nodeCount: entry.nodeCount,
      edgeCount: entry.edgeCount,
      estimatedDurationMs: duration,
      startSequence: entry.startSequence,
      endSequence: entry.endSequence,
      hasFailure: entry.hasFailure,
      failureReason: entry.failureReason,
    });
  }

  return runs;
}

function buildProposalCausalChain(
  proposals: ProposalRecord[],
  events: KernelEvent[],
): ProposalCausalChain[] {
  // Build a "causal chain" from proposals. Link proposals by shared
  // target_session_id or by heuristic parent/child relationships
  // discovered in event payloads.
  const chainMap = new Map<string, {
    proposalId: string;
    status: string;
    targetSessionId?: string;
    operationCount: number;
    sequence: number;
    derivedFromEventSequence?: number;
    children: Set<string>;
  }>();

  // Walk proposals in reverse to build parent links
  // A proposal whose target_session_id matches another proposal's session
  // is a "child". We also look at event payloads for parent_proposal references.
  const proposalsBySession = new Map<string, ProposalRecord[]>();
  for (const p of proposals) {
    if (p.target_session_id) {
      const list = proposalsBySession.get(p.target_session_id) ?? [];
      list.push(p);
      proposalsBySession.set(p.target_session_id, list);
    }
  }

  // Also scan events for proposal parent/child references
  const parentChildFromEvents = new Map<string, string[]>();
  for (const event of events) {
    const p = typeof event.payload === "object" && event.payload !== null
      ? (event.payload as Record<string, unknown>)
      : {};
    const proposalId = extractStringField(p, ["proposal_id", "proposalId"]);
    const parentId = extractStringField(p, ["parent_proposal_id", "parentProposalId", "derived_from"]);
    if (proposalId && parentId) {
      const children = parentChildFromEvents.get(parentId) ?? [];
      if (!children.includes(proposalId)) children.push(proposalId);
      parentChildFromEvents.set(parentId, children);
    }
  }

  for (const proposal of proposals) {
    chainMap.set(proposal.id, {
      proposalId: proposal.id,
      status: proposal.status,
      targetSessionId: proposal.target_session_id,
      operationCount: proposal.operations.length,
      sequence: 0, // will calculate below
      derivedFromEventSequence: undefined,
      children: new Set(parentChildFromEvents.get(proposal.id) ?? []),
    });
  }

  // Populate children from session grouping: earlier proposals are "parents" of later ones
  for (const [, sessionProposals] of proposalsBySession) {
    sessionProposals.sort((a, b) => {
      const idxA = events.findIndex((e) => JSON.stringify(e.payload).includes(a.id));
      const idxB = events.findIndex((e) => JSON.stringify(e.payload).includes(b.id));
      return idxA - idxB;
    });
    for (let i = 1; i < sessionProposals.length; i++) {
      const parent = sessionProposals[i - 1]!;
      const child = sessionProposals[i]!;
      const entry = chainMap.get(parent.id);
      if (entry) {
        entry.children.add(child.id);
      }
    }
  }

  // Assign sequences based on event ordering
  for (const proposal of proposals) {
    const entry = chainMap.get(proposal.id);
    if (!entry) continue;
    // Find the event sequence that most likely corresponds to this proposal
    for (const event of events) {
      const p = typeof event.payload === "object" && event.payload !== null
        ? (event.payload as Record<string, unknown>)
        : {};
      const refProposalId = extractStringField(p, ["proposal_id", "proposalId"]);
      if (refProposalId === proposal.id) {
        entry.sequence = event.sequence;
        entry.derivedFromEventSequence = extractNumberField(p, ["from_sequence", "derived_from_sequence", "forked_from_sequence"]);
        break;
      }
    }
  }

  return Array.from(chainMap.values()).map((entry) => ({
    proposalId: entry.proposalId,
    status: entry.status,
    targetSessionId: entry.targetSessionId,
    parentProposalId: undefined, // computed below
    childProposalIds: Array.from(entry.children),
    operationCount: entry.operationCount,
    sequence: entry.sequence,
    derivedFromEventSequence: entry.derivedFromEventSequence,
  }));
}

function buildFailureBreadcrumbs(
  events: KernelEvent[],
  proposals: ProposalRecord[],
): FailureBreadcrumb[] {
  const breadcrumbs: FailureBreadcrumb[] = [];

  for (const event of events) {
    const kind = event.kind.toLowerCase();
    const p = typeof event.payload === "object" && event.payload !== null
      ? (event.payload as Record<string, unknown>)
      : {};

    const isFailure = FAILURE_KIND_HINTS.some((h) => kind.includes(h.toLowerCase()));
    const hasFailurePayload = FAILURE_PAYLOAD_HINTS.some((h) => JSON.stringify(event.payload).toLowerCase().includes(h));

    if (!isFailure && !hasFailurePayload) continue;

    const reason = extractStringField(p, ["error", "reason", "message", "failure_reason", "failureReason", "summary"])
      ?? `kind: ${event.kind}`;

    let severity: FailureBreadcrumb["severity"] = "info";
    if (kind.includes("error") || kind.includes("failed") || kind.includes("rejected") || kind.includes("denied")) {
      severity = "error";
    } else if (kind.includes("warn") || kind.includes("ambiguous") || kind.includes("missing")) {
      severity = "warn";
    }

    const runId = extractStringField(p, ["run_id", "runId"]);
    const proposalId = extractStringField(p, ["proposal_id", "proposalId"]);

    breadcrumbs.push({
      sequence: event.sequence,
      kind: event.kind,
      packageId: event.writer_package_id,
      reason,
      severity,
      source: event.kind.toLowerCase().includes("inference") ? "inference" : "event",
      relatedRunId: runId,
      relatedProposalId: proposalId,
    });
  }

  // Also scan proposals for failures
  for (const proposal of proposals) {
    const status = proposal.status.toLowerCase();
    if (status === "rejected" || status === "failed") {
      breadcrumbs.push({
        sequence: -1,
        kind: `proposal.${proposal.status}`,
        packageId: "",
        reason: `Proposal ${proposal.id.slice(0, 12)} was ${proposal.status}`,
        severity: status === "rejected" ? "warn" : "error",
        source: "proposal",
        relatedProposalId: proposal.id,
      });
    }
  }

  // Sort by sequence (negative-sequence proposals go last)
  breadcrumbs.sort((a, b) => a.sequence - b.sequence);

  return breadcrumbs;
}

function buildCostLatencySummary(
  events: KernelEvent[],
  proposals: ProposalRecord[],
  capabilities: RegisteredCapability[],
): CostLatencySummary {
  const inferenceCount = events.filter((e) =>
    INFERENCE_KIND_HINTS.some((h) => e.kind.toLowerCase().includes(h))
  ).length;

  const toolCallCount = events.filter((e) =>
    TOOL_CALL_KIND_HINTS.some((h) => e.kind.toLowerCase().includes(h))
  ).length;

  const streamEventCount = events.filter((e) => e.kind.startsWith("kernel/v1/stream.")).length;

  // Accumulate cost hints from event payloads
  let totalCostUnits = 0;
  let costFieldsFound = 0;
  for (const event of events) {
    const p = typeof event.payload === "object" && event.payload !== null
      ? (event.payload as Record<string, unknown>)
      : {};
    for (const hint of COST_HINT_FIELDS) {
      const val = p[hint];
      if (typeof val === "number") {
        totalCostUnits += val;
        costFieldsFound++;
      } else if (typeof val === "string") {
        const parsed = parseFloat(val);
        if (!isNaN(parsed)) {
          totalCostUnits += parsed;
          costFieldsFound++;
        }
      }
    }
  }

  // Estimate latency: use timestamp spread
  const timestamps = events
    .map((e) => new Date(e.created_at).getTime())
    .filter((t) => !isNaN(t));
  const estimatedLatencyMs = timestamps.length >= 2
    ? Math.max(...timestamps) - Math.min(...timestamps)
    : 0;

  // Build cost breakdown by category
  const breakdown: CostBreakdownItem[] = [
    { category: "inference", count: inferenceCount, estimatedCost: `${inferenceCount} unit${inferenceCount === 1 ? "" : "s"}` },
    { category: "tool_call", count: toolCallCount, estimatedCost: `${toolCallCount} unit${toolCallCount === 1 ? "" : "s"}` },
    { category: "stream", count: streamEventCount, estimatedCost: `${streamEventCount} unit${streamEventCount === 1 ? "" : "s"}` },
    { category: "proposal", count: proposals.length, estimatedCost: `${proposals.length} unit${proposals.length === 1 ? "" : "s"}` },
  ];

  // Symbolic cost — models are never called; this is purely observational
  const estimatedCost = costFieldsFound > 0
    ? `~${totalCostUnits.toFixed(1)} estimated units`
    : "no cost data (mock/protocol-only)";

  return {
    totalEvents: events.length,
    estimatedCost,
    estimatedLatencyMs,
    inferenceCount,
    toolCallCount,
    streamEventCount,
    proposalCount: proposals.length,
    costBreakdown: breakdown,
  };
}

function buildAssetProvenance(
  assets: AssetRecord[],
  proposals: ProposalRecord[],
  events: KernelEvent[],
): AssetProvenanceSummary[] {
  // Map proposal operations to asset references
  const proposedAssets = new Map<string, string>(); // assetId -> proposalId
  const approvedAssets = new Map<string, string>(); // assetId -> proposalId

  for (const proposal of proposals) {
    for (const op of proposal.operations) {
      const opStr = JSON.stringify(op).toLowerCase();
      // Heuristic: operations mentioning assets
      for (const asset of assets) {
        if (opStr.includes(asset.id.toLowerCase())) {
          if (proposal.status === "created" || proposal.status === "approved") {
            proposedAssets.set(asset.id, proposal.id);
          }
          if (proposal.status === "approved" || proposal.status === "applied") {
            approvedAssets.set(asset.id, proposal.id);
          }
        }
      }
    }
  }

  // Find which runs referenced each asset via event payloads
  const assetRunRefs = new Map<string, Set<string>>();
  for (const event of events) {
    const p = typeof event.payload === "object" && event.payload !== null
      ? (event.payload as Record<string, unknown>)
      : {};
    const runId = extractStringField(p, ["run_id", "runId"]);
    if (!runId) continue;
    const eventStr = JSON.stringify(event.payload).toLowerCase();
    for (const asset of assets) {
      if (eventStr.includes(asset.id.toLowerCase())) {
        const refs = assetRunRefs.get(asset.id) ?? new Set();
        refs.add(runId);
        assetRunRefs.set(asset.id, refs);
      }
    }
  }

  return assets.map((asset) => ({
    assetId: asset.id,
    originPackageId: asset.origin_package_id,
    mime: asset.mime,
    sizeBytes: asset.size_bytes,
    proposedBy: proposedAssets.get(asset.id),
    approvedBy: approvedAssets.get(asset.id),
    usedInRuns: Array.from(assetRunRefs.get(asset.id) ?? []),
    hash: asset.hash,
  }));
}

function buildGuardrailAuditSummary(
  events: KernelEvent[],
  proposals: ProposalRecord[],
): GuardrailAuditSummary {
  const entries: GuardrailEntry[] = [];
  let blockedCount = 0;
  let warningCount = 0;
  let passedCount = 0;
  let redactionCount = 0;
  let ambiguousProviderCount = 0;
  let rejectionCount = 0;
  let permissionDeniedCount = 0;

  for (const event of events) {
    const kind = event.kind.toLowerCase();
    const payload = typeof event.payload === "object" && event.payload !== null
      ? (event.payload as Record<string, unknown>)
      : {};
    const payloadStr = JSON.stringify(event.payload).toLowerCase();

    // Blocked
    if (kind.includes("denied") || payloadStr.includes("denied")) {
      blockedCount++;
      permissionDeniedCount++;
      entries.push({
        sequence: event.sequence,
        kind: event.kind,
        verdict: "block",
        detail: extractStringField(payload, ["error", "reason", "message", "denied_reason", "deniedReason"])
          ?? "Permission denied",
      });
    }

    // Rejected
    if (kind.includes("rejected") || payloadStr.includes("rejected")) {
      rejectionCount++;
      entries.push({
        sequence: event.sequence,
        kind: event.kind,
        verdict: "block",
        detail: extractStringField(payload, ["error", "reason", "message"])
          ?? "Rejected by guardrail",
      });
    }

    // Redaction
    if (payloadStr.includes("redact") || payloadStr.includes("unsafe_blocked") || payloadStr.includes("secret_ref")) {
      redactionCount++;
      entries.push({
        sequence: event.sequence,
        kind: event.kind,
        verdict: "warn",
        detail: "Redaction or secret reference detected",
      });
    }

    // Ambiguous provider
    if (kind.includes("ambiguous") || payloadStr.includes("ambiguous")) {
      ambiguousProviderCount++;
      entries.push({
        sequence: event.sequence,
        kind: event.kind,
        verdict: "warn",
        detail: extractStringField(payload, ["error", "reason", "message"])
          ?? "Ambiguous provider selection",
      });
    }

    // Missing provider
    if (payloadStr.includes("provider_missing") || payloadStr.includes("missing provider")) {
      warningCount++;
      entries.push({
        sequence: event.sequence,
        kind: event.kind,
        verdict: "warn",
        detail: "Missing provider referenced in tool bridge plan",
      });
    }
  }

  // Count proposals as guardrail events
  for (const proposal of proposals) {
    if (proposal.status === "rejected") {
      rejectionCount++;
      entries.push({
        sequence: -1,
        kind: `proposal.${proposal.status}`,
        verdict: "block",
        detail: `Proposal ${proposal.id.slice(0, 12)} was rejected`,
      });
    }
  }

  const totalGuardrailChecks = entries.length;
  passedCount = totalGuardrailChecks - blockedCount - warningCount;

  return {
    totalGuardrailChecks,
    blockedCount,
    warningCount,
    passedCount,
    redactionCount,
    ambiguousProviderCount,
    rejectionCount,
    permissionDeniedCount,
    guardrailEntries: entries.sort((a, b) => a.sequence - b.sequence),
  };
}

// ========================================================================
// Rendering — HTML string panels for the Forge surface
// ========================================================================

function fmtDuration(ms: number): string {
  if (ms <= 0) return "—";
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  const mins = Math.floor(ms / 60000);
  const secs = Math.floor((ms % 60000) / 1000);
  return `${mins}m ${secs}s`;
}

function fmtBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function renderExperienceObservabilitySection(model: ExperienceObservabilityModel): string {
  const hasRuns = model.agentRuns.length > 0;
  const hasFailures = model.failureBreadcrumbs.length > 0;
  const hasGuardrails = model.guardrailAudit.totalGuardrailChecks > 0;

  return `
    <div class="forge-section experience-observability-section">
      <div class="section-header">
        <h2>Experience Observability <span class="phase-badge">Beta 3</span></h2>
        <span class="section-meta">public-protocol · no runtime internals</span>
      </div>

      <p class="workspace-note">
        Experience Observability panels show session health, package health, agent run health,
        proposal causal chains, failure breadcrumbs, cost/latency summaries, asset provenance,
        and guardrail/audit summaries — all derived from public protocol data.
        No kernel internals, no model/network calls, no SQLite access. Chat transcripts
        are deliberately excluded; this is a creation-workspace observability UI.
      </p>

      <div class="exp-obs-grid">
        ${renderSessionHealthPanel(model.sessionHealth)}
        ${renderPackageHealthPanel(model.packages)}
        ${hasRuns ? renderAgentRunHealthPanel(model.agentRuns) : renderEmptyPanel("Agent Run Health", "No agent runs detected. Load a package that emits run-lifecycle events.")}
        ${renderProposalCausalChainPanel(model.causalChain)}
        ${hasFailures ? renderFailureBreadcrumbsPanel(model.failureBreadcrumbs) : renderEmptyPanel("Failure Breadcrumbs", "No failures detected. All signals are healthy.")}
        ${renderCostLatencyPanel(model.costLatency)}
        ${renderAssetProvenancePanel(model.assetProvenance)}
        ${renderGuardrailAuditPanel(model.guardrailAudit)}
      </div>
    </div>
  `;
}

function renderEmptyPanel(title: string, message: string): string {
  return `
    <details class="exp-obs-panel" open>
      <summary class="exp-obs-panel-header">
        <span class="panel-icon">◈</span>
        <span class="panel-title">${escapeHtml(title)}</span>
      </summary>
      <div class="exp-obs-panel-body">
        <p class="empty">${escapeHtml(message)}</p>
      </div>
    </details>
  `;
}

// --- Session Health Panel ---

function renderSessionHealthPanel(session: SessionHealth): string {
  const statusClass = session.status === "active" ? "status-ok"
    : session.status === "forked" ? "status-warn"
    : session.status === "closed" ? "status-info"
    : "status-error";
  const statusDotClass = session.status === "active" ? "run-status-dot status-ok"
    : session.status === "forked" ? "run-status-dot status-warn"
    : session.status === "closed" ? "run-status-dot status-info"
    : "run-status-dot status-error";

  return `
    <details class="exp-obs-panel exp-obs-panel-wide" open>
      <summary class="exp-obs-panel-header">
        <span class="panel-icon">◆</span>
        <span class="panel-title">Session Health</span>
        <span class="section-meta">${escapeHtml(session.label)} · ${session.eventCount} event${session.eventCount === 1 ? "" : "s"}</span>
      </summary>
      <div class="exp-obs-panel-body">
        <div class="exp-obs-metrics">
          <div class="exp-obs-metric">
            <span class="${statusDotClass}"></span>
            <strong>${escapeHtml(session.status)}</strong>
            <span class="run-meta-item">session status</span>
          </div>
          <div class="exp-obs-metric">
            <strong>${session.eventCount}</strong>
            <span class="run-meta-item">events</span>
          </div>
          <div class="exp-obs-metric">
            <strong>#${session.firstEventSequence} – #${session.lastEventSequence}</strong>
            <span class="run-meta-item">event range</span>
          </div>
          <div class="exp-obs-metric">
            <strong>${fmtDuration(session.estimatedDurationMs)}</strong>
            <span class="run-meta-item">duration</span>
          </div>
          <div class="exp-obs-metric">
            <strong>${session.forkCount}</strong>
            <span class="run-meta-item">fork${session.forkCount === 1 ? "" : "s"}</span>
          </div>
        </div>
        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (session health)</summary>
          <pre class="protocol-preview-code">${formatJson({
            session_id: "<session-id>",
            metrics: {
              event_count: "...",
              event_range: { first_sequence: 0, last_sequence: 0 },
              fork_count: 0,
              duration_ms: 0,
            },
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

// --- Package Health Panel ---

function renderPackageHealthPanel(packages: PackageHealth[]): string {
  const totalEvents = packages.reduce((sum, p) => sum + p.eventCount, 0);
  return `
    <details class="exp-obs-panel" open>
      <summary class="exp-obs-panel-header">
        <span class="panel-icon">◆</span>
        <span class="panel-title">Package Health</span>
        <span class="section-meta">${packages.length} pkg · ${totalEvents} event${totalEvents === 1 ? "" : "s"}</span>
      </summary>
      <div class="exp-obs-panel-body">
        ${packages.length === 0 ? `<p class="empty">No packages loaded.</p>` : `
          <div class="exp-obs-list">
            ${packages.map(renderPackageHealthEntry).join("")}
          </div>
        `}
        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (package health)</summary>
          <pre class="protocol-preview-code">${formatJson({
            package_id: "<package-id>",
            capability_count: 0,
            event_count: 0,
            surface_count: 0,
            state: "loaded|activated|error",
            last_active_sequence: 0,
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

function renderPackageHealthEntry(pkg: PackageHealth): string {
  const stateClass = pkg.state === "activated" ? "status-ok"
    : pkg.state === "error" ? "status-error"
    : "status-info";
  return `
    <article class="exp-obs-entry">
      <div class="exp-obs-entry-header">
        <span class="run-status-dot ${stateClass}"></span>
        <strong>${escapeHtml(pkg.packageId)}</strong>
        <span class="surface-chip">${escapeHtml(pkg.state)}</span>
        <span class="surface-chip">${escapeHtml(pkg.entryKind)}</span>
      </div>
      <div class="exp-obs-entry-meta">
        <span class="run-meta-item">${pkg.capabilityCount} cap${pkg.capabilityCount === 1 ? "" : "s"}</span>
        <span class="run-meta-item">${pkg.eventCount} event${pkg.eventCount === 1 ? "" : "s"}</span>
        <span class="run-meta-item">${pkg.surfaceCount} surface${pkg.surfaceCount === 1 ? "" : "s"}</span>
        ${pkg.lastActiveSequence > 0 ? `<span class="run-meta-item">last: #${pkg.lastActiveSequence}</span>` : ""}
      </div>
    </article>
  `;
}

// --- Agent Run Health Panel ---

function renderAgentRunHealthPanel(runs: AgentRunHealth[]): string {
  const healthyRuns = runs.filter((r) => !r.hasFailure).length;
  const failedRuns = runs.filter((r) => r.hasFailure).length;
  return `
    <details class="exp-obs-panel" open>
      <summary class="exp-obs-panel-header">
        <span class="panel-icon">◆</span>
        <span class="panel-title">Agent Run Health</span>
        <span class="section-meta">${runs.length} run${runs.length === 1 ? "" : "s"} · ${healthyRuns} healthy · ${failedRuns} failed</span>
      </summary>
      <div class="exp-obs-panel-body">
        ${runs.length === 0 ? `<p class="empty">No agent run events detected.</p>` : `
          <div class="exp-obs-list">
            ${runs.map(renderAgentRunEntry).join("")}
          </div>
        `}
        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (agent run health)</summary>
          <pre class="protocol-preview-code">${formatJson({
            run_id: "<package-owned-run-id>",
            lifecycle_state: "running|completed|failed|cancelled",
            node_count: 0,
            edge_count: 0,
            duration_ms: 0,
            has_failure: false,
            failure_reason: null,
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

function renderAgentRunEntry(run: AgentRunHealth): string {
  const statusClass = run.status.includes("fail") || run.status.includes("cancell") ? "status-error"
    : run.status.includes("complete") || run.status.includes("ready") ? "status-ok"
    : run.status.includes("wait") || run.status.includes("pause") || run.status.includes("pending") ? "status-warn"
    : "status-info";
  return `
    <article class="exp-obs-entry ${run.hasFailure ? "exp-obs-entry-error" : ""}">
      <div class="exp-obs-entry-header">
        <span class="run-status-dot ${statusClass}"></span>
        <strong>${escapeHtml(run.label)}</strong>
        <span class="surface-chip ${statusClass}">${escapeHtml(run.status)}</span>
        <span class="run-meta-item">${escapeHtml(run.packageId)}</span>
      </div>
      <div class="exp-obs-entry-meta">
        <span class="run-meta-item">Nodes: ${run.nodeCount} · Edges: ${run.edgeCount}</span>
        <span class="run-meta-item">Duration: ${fmtDuration(run.estimatedDurationMs)}</span>
        <span class="run-meta-item">Seq: #${run.startSequence}${run.endSequence ? ` – #${run.endSequence}` : " →"}</span>
      </div>
      ${run.hasFailure ? `
        <div class="exp-obs-failure-reason">
          <span class="safety-badge severity-error">failed</span>
          <code>${escapeHtml(run.failureReason ?? "Unknown failure")}</code>
        </div>
      ` : ""}
    </article>
  `;
}

// --- Proposal Causal Chain Panel ---

function renderProposalCausalChainPanel(chains: ProposalCausalChain[]): string {
  const totalChildren = chains.reduce((sum, c) => sum + c.childProposalIds.length, 0);
  return `
    <details class="exp-obs-panel exp-obs-panel-wide" open>
      <summary class="exp-obs-panel-header">
        <span class="panel-icon">◆</span>
        <span class="panel-title">Proposal Causal Chain</span>
        <span class="section-meta">${chains.length} proposal${chains.length === 1 ? "" : "s"} · ${totalChildren} child link${totalChildren === 1 ? "" : "s"}</span>
      </summary>
      <div class="exp-obs-panel-body">
        ${chains.length === 0 ? `<p class="empty">No proposals detected. Proposals are the causal backbone of experience sessions — they record what changed and why.</p>` : `
          <div class="exp-obs-causal-chain">
            ${chains.map(renderProposalChainEntry).join("")}
          </div>
        `}
        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (proposal causal chain)</summary>
          <pre class="protocol-preview-code">${formatJson({
            proposal_id: "<proposal-id>",
            status: "created|approved|applied|rejected|failed",
            target_session_id: "<session-id>",
            parent_proposal_id: "<proposal-id-or-null>",
            child_proposal_ids: ["<proposal-id>"],
            operation_count: 0,
            sequence: 0,
            derived_from_event_sequence: 0,
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

function renderProposalChainEntry(chain: ProposalCausalChain): string {
  const statusClass = chain.status === "applied" ? "status-ok"
    : chain.status === "approved" ? "status-info"
    : chain.status === "rejected" || chain.status === "failed" ? "status-error"
    : "status-warn";
  const hasChildren = chain.childProposalIds.length > 0;
  const hasParent = !!chain.parentProposalId;
  return `
    <article class="exp-obs-chain-entry">
      <div class="exp-obs-chain-header">
        <span class="run-status-dot ${statusClass}"></span>
        <strong>${escapeHtml(chain.proposalId.slice(0, 16))}</strong>
        <span class="surface-chip ${statusClass}">${escapeHtml(chain.status)}</span>
        <span class="run-meta-item">${chain.operationCount} op${chain.operationCount === 1 ? "" : "s"}</span>
        ${chain.sequence > 0 ? `<span class="run-meta-item">#${chain.sequence}</span>` : ""}
      </div>
      <div class="exp-obs-chain-links">
        ${hasParent ? `
          <span class="exp-obs-chain-link">
            <span class="trace-chip active">↑ parent: ${escapeHtml(chain.parentProposalId!.slice(0, 12))}</span>
          </span>
        ` : `<span class="exp-obs-chain-link"><span class="trace-chip">↑ root proposal</span></span>`}
        ${hasChildren ? `
          <span class="exp-obs-chain-link">
            <span class="trace-chip active">↓ ${chain.childProposalIds.length} child${chain.childProposalIds.length === 1 ? "" : "ren"}: ${chain.childProposalIds.map((id) => escapeHtml(id.slice(0, 8))).join(", ")}</span>
          </span>
        ` : `<span class="exp-obs-chain-link"><span class="trace-chip">↓ no children</span></span>`}
        ${chain.derivedFromEventSequence ? `<span class="run-meta-item">derived from #${chain.derivedFromEventSequence}</span>` : ""}
        ${chain.targetSessionId ? `<span class="run-meta-item">session: ${escapeHtml(chain.targetSessionId.slice(0, 8))}</span>` : ""}
      </div>
    </article>
  `;
}

// --- Failure Breadcrumbs Panel ---

function renderFailureBreadcrumbsPanel(breadcrumbs: FailureBreadcrumb[]): string {
  const errors = breadcrumbs.filter((b) => b.severity === "error").length;
  const warnings = breadcrumbs.filter((b) => b.severity === "warn").length;
  return `
    <details class="exp-obs-panel" open>
      <summary class="exp-obs-panel-header">
        <span class="panel-icon">◆</span>
        <span class="panel-title">Failure Breadcrumbs</span>
        <span class="section-meta">${breadcrumbs.length} total · ${errors} error${errors === 1 ? "" : "s"} · ${warnings} warning${warnings === 1 ? "" : "s"}</span>
      </summary>
      <div class="exp-obs-panel-body">
        ${breadcrumbs.length === 0 ? `<p class="empty">No failure breadcrumbs detected.</p>` : `
          <div class="exp-obs-breadcrumb-list">
            ${breadcrumbs.map(renderBreadcrumb).join("")}
          </div>
        `}
        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (failure breadcrumb)</summary>
          <pre class="protocol-preview-code">${formatJson({
            sequence: 0,
            kind: "<event-kind>",
            package_id: "<package-id>",
            reason: "error message or reason",
            severity: "error|warn|info",
            source: "event|proposal|inference",
            related_run_id: "<run-id-or-null>",
            related_proposal_id: "<proposal-id-or-null>",
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

function renderBreadcrumb(b: FailureBreadcrumb): string {
  const severityClass = b.severity === "error" ? "severity-error"
    : b.severity === "warn" ? "severity-warn"
    : "severity-info";
  return `
    <article class="exp-obs-breadcrumb-entry severity-${b.severity}">
      <div class="exp-obs-breadcrumb-header">
        <span class="safety-badge ${severityClass}">${escapeHtml(b.severity)}</span>
        <span class="timeline-badge trace-badge">${escapeHtml(b.source)}</span>
        <span class="run-meta-item">#${b.sequence > 0 ? b.sequence : "—"}</span>
        <strong>${escapeHtml(b.kind)}</strong>
        <span class="run-meta-item">${escapeHtml(b.packageId)}</span>
      </div>
      <div class="exp-obs-breadcrumb-reason">
        <code>${escapeHtml(b.reason)}</code>
      </div>
      <div class="exp-obs-breadcrumb-meta">
        ${b.relatedRunId ? `<span class="trace-chip active">run: ${escapeHtml(b.relatedRunId.slice(0, 12))}</span>` : ""}
        ${b.relatedProposalId ? `<span class="trace-chip active">proposal: ${escapeHtml(b.relatedProposalId.slice(0, 12))}</span>` : ""}
      </div>
    </article>
  `;
}

// --- Cost / Latency Summary Panel ---

function renderCostLatencyPanel(cost: CostLatencySummary): string {
  return `
    <details class="exp-obs-panel" open>
      <summary class="exp-obs-panel-header">
        <span class="panel-icon">◆</span>
        <span class="panel-title">Cost / Latency</span>
        <span class="section-meta">${cost.totalEvents} events · ${fmtDuration(cost.estimatedLatencyMs)}</span>
      </summary>
      <div class="exp-obs-panel-body">
        <div class="exp-obs-metrics">
          <div class="exp-obs-metric">
            <strong>${cost.totalEvents}</strong>
            <span class="run-meta-item">total events</span>
          </div>
          <div class="exp-obs-metric">
            <strong>${cost.inferenceCount}</strong>
            <span class="run-meta-item">inference${cost.inferenceCount === 1 ? "" : "s"}</span>
          </div>
          <div class="exp-obs-metric">
            <strong>${cost.toolCallCount}</strong>
            <span class="run-meta-item">tool call${cost.toolCallCount === 1 ? "" : "s"}</span>
          </div>
          <div class="exp-obs-metric">
            <strong>${cost.streamEventCount}</strong>
            <span class="run-meta-item">stream events</span>
          </div>
          <div class="exp-obs-metric">
            <strong>${cost.proposalCount}</strong>
            <span class="run-meta-item">proposal${cost.proposalCount === 1 ? "" : "s"}</span>
          </div>
          <div class="exp-obs-metric">
            <strong>${fmtDuration(cost.estimatedLatencyMs)}</strong>
            <span class="run-meta-item">estimated latency</span>
          </div>
        </div>

        <div class="exp-obs-cost-note">
          <span class="safety-badge severity-info">cost estimate</span>
          <code>${escapeHtml(cost.estimatedCost)}</code>
        </div>

        ${cost.costBreakdown.length > 0 ? `
          <div class="exp-obs-breakdown">
            <h3 class="slot-title">Cost Breakdown by Category</h3>
            <div class="exp-obs-breakdown-list">
              ${cost.costBreakdown.map((item) => `
                <div class="exp-obs-breakdown-item">
                  <span class="surface-chip">${escapeHtml(item.category)}</span>
                  <span class="run-meta-item">${item.count} call${item.count === 1 ? "" : "s"}</span>
                  <span class="run-meta-item">${escapeHtml(item.estimatedCost)}</span>
                </div>
              `).join("")}
            </div>
          </div>
        ` : ""}

        <p class="workspace-note" style="margin-top: 0.5rem;">
          Cost/latency data is estimated from public protocol events only. No real model calls
          are made — all inference is proposal-gated and plan-only. The "cost" is a symbolic
          count of observable units, not actual spend.
        </p>

        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (cost/latency)</summary>
          <pre class="protocol-preview-code">${formatJson({
            total_events: 0,
            events_by_category: {
              inference: 0,
              tool_call: 0,
              stream: 0,
              proposal: 0,
            },
            estimated_latency_ms: 0,
            estimated_cost_units: "...",
            cost_breakdown: [{ category: "inference", count: 0, estimated_cost: "..." }],
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

// --- Asset Provenance Panel ---

function renderAssetProvenancePanel(assets: AssetProvenanceSummary[]): string {
  const totalSize = assets.reduce((sum, a) => sum + a.sizeBytes, 0);
  const proposedCount = assets.filter((a) => a.proposedBy).length;
  const approvedCount = assets.filter((a) => a.approvedBy).length;
  return `
    <details class="exp-obs-panel" open>
      <summary class="exp-obs-panel-header">
        <span class="panel-icon">◆</span>
        <span class="panel-title">Asset Provenance</span>
        <span class="section-meta">${assets.length} asset${assets.length === 1 ? "" : "s"} · ${fmtBytes(totalSize)} total · ${proposedCount} proposed · ${approvedCount} approved</span>
      </summary>
      <div class="exp-obs-panel-body">
        ${assets.length === 0 ? `<p class="empty">No assets detected. Package-declared assets appear here with their origin, proposal history, and run usage.</p>` : `
          <div class="exp-obs-list">
            ${assets.map(renderAssetProvenanceEntry).join("")}
          </div>
        `}
        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (asset provenance)</summary>
          <pre class="protocol-preview-code">${formatJson({
            asset_id: "<asset-id>",
            origin_package_id: "<package-id>",
            mime: "application/json|text/plain|image/png",
            size_bytes: 0,
            proposed_by: "<proposal-id-or-null>",
            approved_by: "<proposal-id-or-null>",
            used_in_runs: ["<run-id>"],
            hash: "<asset-hash>",
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

function renderAssetProvenanceEntry(asset: AssetProvenanceSummary): string {
  const hasProvenance = !!asset.proposedBy || !!asset.approvedBy || asset.usedInRuns.length > 0;
  return `
    <article class="exp-obs-entry">
      <div class="exp-obs-entry-header">
        <span class="surface-chip">${escapeHtml(asset.mime)}</span>
        <strong>${escapeHtml(asset.assetId.slice(0, 20))}</strong>
        <span class="run-meta-item">${escapeHtml(asset.originPackageId)}</span>
        <span class="run-meta-item">${fmtBytes(asset.sizeBytes)}</span>
      </div>
      <div class="exp-obs-entry-meta">
        <span class="trace-chip">hash: ${escapeHtml(asset.hash.slice(0, 12))}…</span>
        ${asset.proposedBy ? `<span class="trace-chip active">proposed: ${escapeHtml(asset.proposedBy.slice(0, 12))}</span>` : `<span class="trace-chip">not proposed</span>`}
        ${asset.approvedBy ? `<span class="trace-chip active">approved: ${escapeHtml(asset.approvedBy.slice(0, 12))}</span>` : ""}
        ${hasProvenance ? "" : `<span class="safety-badge severity-info">no proposal trail</span>`}
      </div>
      ${asset.usedInRuns.length > 0 ? `
        <div class="exp-obs-asset-runs">
          <span class="run-meta-label">Used in runs:</span>
          ${asset.usedInRuns.map((r) => `<span class="surface-chip">${escapeHtml(r.slice(0, 12))}</span>`).join("")}
        </div>
      ` : ""}
    </article>
  `;
}

// --- Guardrail / Audit Summary Panel ---

function renderGuardrailAuditPanel(guardrail: GuardrailAuditSummary): string {
  return `
    <details class="exp-obs-panel" open>
      <summary class="exp-obs-panel-header">
        <span class="panel-icon">◆</span>
        <span class="panel-title">Guardrail / Audit Summary</span>
        <span class="section-meta">${guardrail.totalGuardrailChecks} check${guardrail.totalGuardrailChecks === 1 ? "" : "s"} · ${guardrail.blockedCount} blocked · ${guardrail.warningCount} warnings</span>
      </summary>
      <div class="exp-obs-panel-body">
        <div class="exp-obs-metrics">
          <div class="exp-obs-metric">
            <strong>${guardrail.passedCount}</strong>
            <span class="run-meta-item">passed</span>
          </div>
          <div class="exp-obs-metric">
            <strong>${guardrail.blockedCount}</strong>
            <span class="run-meta-item">blocked</span>
          </div>
          <div class="exp-obs-metric">
            <strong>${guardrail.warningCount}</strong>
            <span class="run-meta-item">warnings</span>
          </div>
          <div class="exp-obs-metric">
            <strong>${guardrail.redactionCount}</strong>
            <span class="run-meta-item">redaction${guardrail.redactionCount === 1 ? "" : "s"}</span>
          </div>
          <div class="exp-obs-metric">
            <strong>${guardrail.ambiguousProviderCount}</strong>
            <span class="run-meta-item">ambiguous provider${guardrail.ambiguousProviderCount === 1 ? "" : "s"}</span>
          </div>
          <div class="exp-obs-metric">
            <strong>${guardrail.rejectionCount}</strong>
            <span class="run-meta-item">rejection${guardrail.rejectionCount === 1 ? "" : "s"}</span>
          </div>
          <div class="exp-obs-metric">
            <strong>${guardrail.permissionDeniedCount}</strong>
            <span class="run-meta-item">permission denied</span>
          </div>
        </div>

        ${guardrail.guardrailEntries.length > 0 ? `
          <div class="exp-obs-guardrail-entries">
            <h3 class="slot-title">Guardrail Log</h3>
            <div class="exp-obs-list">
              ${guardrail.guardrailEntries.map(renderGuardrailEntry).join("")}
            </div>
          </div>
        ` : `
          <p class="empty">No guardrail events detected. Guardrail/audit entries appear when permission policies, redactions, or ambiguous providers are triggered.</p>
        `}

        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (guardrail/audit)</summary>
          <pre class="protocol-preview-code">${formatJson({
            guardrail_event: {
              kind: "kernel/v1/event.append (permission_denied|rejected|redacted)",
              payload: {
                denied_reason: "...",
                redaction_state: "safe|unsafe_blocked",
                risk: { categories: ["prompt_injection"], overall_risk: "critical" },
              },
            },
            summary: {
              total_checks: 0,
              blocked: 0,
              warnings: 0,
              passed: 0,
              redactions: 0,
              ambiguous_providers: 0,
              rejections: 0,
              permission_denied: 0,
            },
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

function renderGuardrailEntry(entry: GuardrailEntry): string {
  const verdictClass = entry.verdict === "block" ? "severity-error"
    : entry.verdict === "warn" ? "severity-warn"
    : "severity-ok";
  const verdictIcon = entry.verdict === "block" ? "⊘"
    : entry.verdict === "warn" ? "⚠"
    : "✓";
  return `
    <article class="exp-obs-entry verdict-${entry.verdict}">
      <div class="exp-obs-entry-header">
        <span class="safety-badge ${verdictClass}">${verdictIcon} ${escapeHtml(entry.verdict)}</span>
        <span class="run-meta-item">#${entry.sequence > 0 ? entry.sequence : "—"}</span>
        <strong>${escapeHtml(entry.kind)}</strong>
      </div>
      <div class="exp-obs-breadcrumb-reason">
        <code>${escapeHtml(entry.detail)}</code>
      </div>
    </article>
  `;
}
