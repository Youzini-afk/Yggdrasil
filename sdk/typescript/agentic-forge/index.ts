/**
 * Yggdrasil Agentic Forge SDK — Package-owned run lifecycle, plan graph,
 * working state, and candidate shapes.
 *
 * This module defines the **agentic forge contract** at the package/SDK layer.
 * It does NOT enter the kernel, does NOT add Rust protocol methods, and does
 * NOT add `kernel.agent.*`, `kernel.model.*`, `kernel.prompt.*`,
 * `kernel.memory.*`, or `kernel.turn.*`.
 *
 * ## Design principles
 *
 * - **Package-owned**: Run lifecycle, plan graph, working state are ordinary
 *   package artifacts, not kernel primitives.
 * - **Deterministic**: No network, no real model inference, no random.
 * - **Secret-safe**: Uses `secret_ref` identifiers; rejects raw secrets.
 * - **No kernel agent namespace**: Output never contains `kernel.agent.*`,
 *   `kernel.model.*`, `kernel.prompt.*`, `kernel.memory.*`, `kernel.turn.*`.
 *
 * ## API surface
 *
 * Types:
 * - `AgentRunLifecycleState` — lifecycle state enum
 * - `PlanNode` / `PlanEdge` — plan graph elements
 * - `PlanGraph` — full plan graph artifact
 * - `WorkingState` — run working state artifact
 * - `RunEvent` — trace event
 * - `CandidateShell` — candidate shape (Phase A shell)
 * - `ObservabilitySummary` — run observability summary
 *
 * Helpers:
 * - `createRunEvent()` — build a valid run event
 * - `validatePlanGraph()` — validate a plan graph
 * - `createPlanGraph()` — build a minimal plan graph
 * - `createWorkingState()` — build a working state
 * - `createCandidateShell()` — build a candidate shell (Phase A)
 * - `blockRawSecrets()` — check for raw-secret-like content
 * - `runAgenticForgeSelfTest()` — pure-TS self-test
 */

// ---------------------------------------------------------------------------
// Lifecycle states
// ---------------------------------------------------------------------------

/** Package-owned agent run lifecycle states. */
export type AgentRunLifecycleState =
  | "created"
  | "prepared"
  | "running"
  | "paused"
  | "waiting_for_approval"
  | "completed"
  | "failed"
  | "cancelled"
  | "archived";

const VALID_LIFECYCLE_STATES: AgentRunLifecycleState[] = [
  "created", "prepared", "running", "paused", "waiting_for_approval",
  "completed", "failed", "cancelled", "archived",
];

export function isValidLifecycleState(s: string): s is AgentRunLifecycleState {
  return (VALID_LIFECYCLE_STATES as string[]).includes(s);
}

// ---------------------------------------------------------------------------
// Plan graph types
// ---------------------------------------------------------------------------

export type PlanNodeKind = "observe" | "infer" | "tool_call" | "inspect" | "branch_op" | "compare" | "propose" | "wait";

export interface PlanNode {
  node_id: string;
  kind: PlanNodeKind;
  label: string;
  status: "pending" | "running" | "completed" | "failed" | "skipped";
  input_refs?: string[];
  output_refs?: string[];
}

export interface PlanEdge {
  from_node_id: string;
  to_node_id: string;
  kind: "sequential" | "conditional" | "parallel";
}

export interface RetryPolicy {
  max_retries: number;
  backoff: "none" | "linear" | "exponential";
}

export interface PlanGraph {
  nodes: PlanNode[];
  edges: PlanEdge[];
  status: "prepared" | "running" | "completed" | "failed";
  revision: number;
  input_refs: string[];
  output_refs: string[];
  approval_policy: string;
  retry_policy: RetryPolicy;
  deterministic_mode: boolean;
}

// ---------------------------------------------------------------------------
// Working state
// ---------------------------------------------------------------------------

export interface PolicyState {
  approval_policy: string;
  retry_budget_remaining: number;
  deterministic_mode: boolean;
}

export interface WorkingState {
  run_id: string;
  owner_package: string;
  target_branch_ref: string;
  scratch_branch_ref: string;
  current_objective: string;
  local_context_refs: string[];
  plan_graph_ref: string;
  candidate_refs: string[];
  tool_observation_refs: string[];
  inference_trace_refs: string[];
  policy_state: PolicyState;
}

// ---------------------------------------------------------------------------
// Run event
// ---------------------------------------------------------------------------

export type RunEventType =
  | "run_created"
  | "run_prepared"
  | "run_running"
  | "run_paused"
  | "run_waiting_for_approval"
  | "run_completed"
  | "run_failed"
  | "run_cancelled"
  | "run_archived"
  | "node_started"
  | "node_completed"
  | "node_failed";

export interface RunEvent {
  event_type: RunEventType;
  run_id: string;
  timestamp: number;
  payload: Record<string, unknown>;
}

// ---------------------------------------------------------------------------
// Candidate shell (Phase A placeholder)
// ---------------------------------------------------------------------------

export interface CandidateShell {
  candidate_id: string;
  run_id: string;
  target_branch_ref: string;
  scratch_branch_ref: string;
  provenance: {
    package_id: string;
    capability_id: string;
  };
}

// ---------------------------------------------------------------------------
// Observability summary
// ---------------------------------------------------------------------------

export interface ObservabilitySummary {
  run_id: string;
  lifecycle_state: AgentRunLifecycleState;
  trace_event_count: number;
  plan_node_count: number;
  candidate_count: number;
  tool_observation_count: number;
  inference_trace_count: number;
  inference_performed: boolean;
  network_performed: boolean;
  summary: string;
}

// ---------------------------------------------------------------------------
// Raw-secret detection (conservative)
// ---------------------------------------------------------------------------

const SECRET_FIELD_NAMES = [
  "api_key", "secret", "token", "password", "private_key",
  "access_token", "refresh_token", "auth_token",
];

const SECRET_VALUE_PREFIXES = ["sk-", "Bearer ", "bearer "];

function isSecretRefValue(val: string): boolean {
  return val.startsWith("secret_ref:") ||
         val.startsWith("secretRef:") ||
         val.startsWith("secret-ref:") ||
         val.startsWith("host:");
}

export function looksLikeRawSecret(val: string): boolean {
  for (const prefix of SECRET_VALUE_PREFIXES) {
    if (val.startsWith(prefix)) return true;
  }
  if (val.length >= 32) {
    const hasUpper = /[A-Z]/.test(val);
    const hasLower = /[a-z]/.test(val);
    const hasDigit = /[0-9]/.test(val);
    if (hasUpper && hasLower && hasDigit && val.length >= 40) return true;
  }
  return false;
}

export function isSecretFieldName(name: string): boolean {
  const lower = name.toLowerCase();
  return SECRET_FIELD_NAMES.some(sn => lower === sn || lower.includes(sn));
}

/** Recursively check a value for raw-secret-like content. Returns true if blocked. */
export function blockRawSecrets(value: unknown): boolean {
  if (value === null || value === undefined) return false;
  if (typeof value === "string") return looksLikeRawSecret(value);
  if (typeof value !== "object") return false;

  if (Array.isArray(value)) {
    return value.some(item => blockRawSecrets(item));
  }

  const obj = value as Record<string, unknown>;
  for (const [key, val] of Object.entries(obj)) {
    if (isSecretFieldName(key) && typeof val === "string") {
      if (!isSecretRefValue(val) && val !== "") {
        return true;
      }
    }
    if (typeof val === "string" && looksLikeRawSecret(val)) return true;
    if (blockRawSecrets(val)) return true;
  }
  return false;
}

/** Check that a JSON-serializable output contains no kernel.agent/model/prompt/memory/turn namespace. */
export function hasKernelAgentNamespace(value: unknown): boolean {
  const str = JSON.stringify(value);
  return str.includes("kernel.agent") ||
         str.includes("kernel.model") ||
         str.includes("kernel.prompt") ||
         str.includes("kernel.memory") ||
         str.includes("kernel.turn");
}

// ---------------------------------------------------------------------------
// Helper constructors
// ---------------------------------------------------------------------------

/** Build a valid run event. */
export function createRunEvent(
  eventType: RunEventType,
  runId: string,
  timestamp: number,
  payload: Record<string, unknown>,
): RunEvent {
  return { event_type: eventType, run_id: runId, timestamp, payload };
}

/** Validate a plan graph structure. Returns diagnostics. */
export function validatePlanGraph(pg: Partial<PlanGraph>): string[] {
  const diagnostics: string[] = [];
  if (!pg.nodes || !Array.isArray(pg.nodes)) {
    diagnostics.push("plan_graph.nodes must be an array");
  } else {
    const nodeIds = new Set(pg.nodes!.map(n => n.node_id));
    for (const edge of pg.edges ?? []) {
      if (!nodeIds.has(edge.from_node_id)) {
        diagnostics.push(`edge references unknown from_node_id: ${edge.from_node_id}`);
      }
      if (!nodeIds.has(edge.to_node_id)) {
        diagnostics.push(`edge references unknown to_node_id: ${edge.to_node_id}`);
      }
    }
  }
  if (pg.approval_policy && pg.approval_policy !== "fork_then_approve" && pg.approval_policy !== "approve_then_fork" && pg.approval_policy !== "none") {
    diagnostics.push(`unknown approval_policy: ${pg.approval_policy}`);
  }
  return diagnostics;
}

/** Build a minimal plan graph. */
export function createPlanGraph(runId: string, objective: string): PlanGraph {
  return {
    nodes: [
      { node_id: `${runId}_node_observe`, kind: "observe", label: "Observe context", status: "pending" },
      { node_id: `${runId}_node_plan`, kind: "tool_call", label: objective, status: "pending" },
      { node_id: `${runId}_node_propose`, kind: "propose", label: "Produce candidate", status: "pending" },
    ],
    edges: [
      { from_node_id: `${runId}_node_observe`, to_node_id: `${runId}_node_plan`, kind: "sequential" },
      { from_node_id: `${runId}_node_plan`, to_node_id: `${runId}_node_propose`, kind: "sequential" },
    ],
    status: "prepared",
    revision: 1,
    input_refs: [],
    output_refs: [],
    approval_policy: "fork_then_approve",
    retry_policy: { max_retries: 0, backoff: "none" },
    deterministic_mode: true,
  };
}

/** Build a working state. */
export function createWorkingState(
  runId: string,
  ownerPackage: string,
  options: Partial<WorkingState> = {},
): WorkingState {
  return {
    run_id: runId,
    owner_package: ownerPackage,
    target_branch_ref: options.target_branch_ref ?? "branch:target:default",
    scratch_branch_ref: options.scratch_branch_ref ?? "branch:scratch:default",
    current_objective: options.current_objective ?? "deterministic agentic forge run",
    local_context_refs: options.local_context_refs ?? [],
    plan_graph_ref: options.plan_graph_ref ?? `plan_graph:${runId}`,
    candidate_refs: options.candidate_refs ?? [],
    tool_observation_refs: options.tool_observation_refs ?? [],
    inference_trace_refs: options.inference_trace_refs ?? [],
    policy_state: options.policy_state ?? {
      approval_policy: "fork_then_approve",
      retry_budget_remaining: 0,
      deterministic_mode: true,
    },
  };
}

/** Build a candidate shell (Phase A placeholder). */
export function createCandidateShell(
  candidateId: string,
  runId: string,
  targetBranchRef: string,
  scratchBranchRef: string,
  packageId: string,
  capabilityId: string,
): CandidateShell {
  return {
    candidate_id: candidateId,
    run_id: runId,
    target_branch_ref: targetBranchRef,
    scratch_branch_ref: scratchBranchRef,
    provenance: { package_id: packageId, capability_id: capabilityId },
  };
}

// ---------------------------------------------------------------------------
// Self-test
// ---------------------------------------------------------------------------

export interface SelfTestResult {
  passed: number;
  failed: number;
  failures: string[];
}

export function runAgenticForgeSelfTest(): SelfTestResult {
  const failures: string[] = [];
  let passed = 0;

  function assert(condition: boolean, label: string) {
    if (condition) { passed++; } else { failures.push(label); }
  }

  // Lifecycle states
  assert(isValidLifecycleState("created"), "created is valid state");
  assert(isValidLifecycleState("archived"), "archived is valid state");
  assert(!isValidLifecycleState("unknown"), "unknown is not valid state");
  assert(VALID_LIFECYCLE_STATES.length === 9, "9 lifecycle states");

  // Run event
  const evt = createRunEvent("run_created", "run_test", 0, { step: "test" });
  assert(evt.event_type === "run_created", "run event type");
  assert(evt.run_id === "run_test", "run event run_id");

  // Plan graph creation
  const pg = createPlanGraph("run_test", "test objective");
  assert(pg.nodes.length === 3, "plan graph has 3 nodes");
  assert(pg.edges.length === 2, "plan graph has 2 edges");
  assert(pg.deterministic_mode === true, "deterministic mode");
  assert(pg.approval_policy === "fork_then_approve", "approval policy");

  // Plan graph validation
  const validDiags = validatePlanGraph(pg);
  assert(validDiags.length === 0, "valid plan graph has no diagnostics");

  const badPg = { nodes: [{ node_id: "n1", kind: "observe" as const, label: "x", status: "pending" as const }], edges: [{ from_node_id: "n1", to_node_id: "n_missing", kind: "sequential" as const }] };
  const badDiags = validatePlanGraph(badPg);
  assert(badDiags.length > 0, "plan graph with bad edge has diagnostics");

  // Working state
  const ws = createWorkingState("run_test", "official/agentic-forge-lab");
  assert(ws.run_id === "run_test", "working state run_id");
  assert(ws.owner_package === "official/agentic-forge-lab", "working state owner");
  assert(ws.policy_state.deterministic_mode === true, "working state deterministic mode");

  // Candidate shell
  const cs = createCandidateShell("c1", "run_test", "branch:target:main", "branch:scratch:s1", "official/agentic-forge-lab", "official/agentic-forge-lab/start_run");
  assert(cs.candidate_id === "c1", "candidate shell id");
  assert(cs.provenance.package_id === "official/agentic-forge-lab", "candidate provenance");

  // Raw secret blocking
  assert(looksLikeRawSecret("RawSecretExample1234567890abcdefABCDEF123456"), "raw-looking secret detected");
  assert(looksLikeRawSecret("Bearer abc"), "Bearer prefix detected");
  assert(!looksLikeRawSecret("hello world"), "normal string ok");
  assert(!looksLikeRawSecret("secret_ref:env:KEY"), "secret_ref not flagged");

  assert(blockRawSecrets({ api_key: "RawSecretExample1234567890abcdefABCDEF123456" }), "raw secret in api_key blocked");
  assert(blockRawSecrets({ token: "Bearer xyz" }), "raw secret in token blocked");
  assert(!blockRawSecrets({ api_key: "secret_ref:env:MY_KEY" }), "secret_ref in api_key allowed");
  assert(!blockRawSecrets({ api_key: "secret-ref:env:MY_KEY" }), "secret-ref in api_key allowed");
  assert(!blockRawSecrets({ api_key: "host:env:MY_KEY" }), "host secret ref in api_key allowed");
  assert(!blockRawSecrets({ objective: "safe text" }), "normal objective allowed");

  // No kernel agent namespace
  assert(!hasKernelAgentNamespace({ kind: "agentic_forge_run_started", run_id: "r1" }), "clean output has no kernel agent namespace");
  assert(hasKernelAgentNamespace({ method: "kernel.agent.run" }), "kernel.agent detected");
  assert(hasKernelAgentNamespace({ method: "kernel.model.infer" }), "kernel.model detected");

  // isSecretFieldName
  assert(isSecretFieldName("api_key"), "api_key is secret field");
  assert(isSecretFieldName("token"), "token is secret field");
  assert(!isSecretFieldName("objective"), "objective is not secret field");

  return { passed, failed: failures.length, failures };
}
