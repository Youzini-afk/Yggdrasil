# Agentic Forge Beta Plan

> [English](./AGENTIC_FORGE_BETA.en.md) · [中文](./AGENTIC_FORGE_BETA.md)

Agentic Forge Beta is not another proof that Yggdrasil can host agent-like packages. It promotes agents from lab/proof code into a **Yggdrasil-native creative engine**: agents can observe branches/projections, maintain package-owned plan graphs, call scoped capability tools, explore in scratch branches, produce candidates, and ask users to promote them through inspectable proposals.

This plan draws from `/workspace/Yggdrasil/pi`, LangGraph, OpenAI Agents SDK, Temporal, AutoGen, CrewAI, Claude Code / Codex permission models, and OpenChamber workspace patterns. It does not copy their product ontologies. Mature frameworks contribute principles such as resumable runs, checkpoints, interrupts/resume, tool safety, human-in-the-loop, multi-agent orchestration, and workspace observability; in Yggdrasil those principles must land in the package layer, public protocol, and branch/proposal/asset/projection substrate.

## Core principles

1. **Agents are ordinary packages, not kernel primitives.** No new `kernel.agent.*`, `kernel.model.*`, `kernel.prompt.*`, `kernel.memory.*`, or `kernel.turn.*`.
2. **Agents produce candidates, not authoritative mutations.** Agents write scratch branches by default and cannot directly write target branches.
3. **Promote requires inspection.** A candidate enters the target branch only through proposal/inspection/approval/apply.
4. **Run state is package-owned.** Run lifecycle, plan graph, working state, task queue, and memory-like state are expressed as package-owned assets/events/projections.
5. **Tool safety is layered.** Capability declaration, scoped grant, approval, sandbox/resource boundary, and audit stay separate; no ambient privileged tool runner.
6. **Live inference is replaceable.** Cloud providers are practical delivery, but deterministic/fake/local providers must run the same class of run.
7. **Forge is a control room, not a chat shell.** The UI centers on run timelines, plan graphs, scratch diffs, candidate comparison, proposal queues, and tool/inference traces — not chat transcripts.
8. **Observability is part of the contract.** Runs, nodes, tools, inference, branches, candidates, proposals, approvals, retries, and cancellation must be traceable.

## Existing foundation

- `sdk/typescript/ygg-agent-adapter`: capability-to-tool adaptation, permission preview, proposal drafts, trace events, raw-secret blocking.
- `official/pi-agent-runtime-lab`: deterministic/no-network agent-like run/proposal/trace lab.
- `official/capability-tool-bridge-lab`: discover/preview/invoke/stream plans, explicit provider selection, ambiguous fail-closed.
- `examples/packages/thirdparty-agent-runtime` and `examples/compositions/agent-runtime-replacement`: third-party replacement proof.
- Forge/Assist Agent Observability: initial traces/tool diagnostics/readiness views.
- `sdk/typescript/inference-capability`, `official/inference-local-lab`, `official/inference-playtest-lab`: transport-neutral inference and inference→proposal proof.

## Phase 0 — Plan and boundary lock

**Goal:** Fix the Agentic Forge Beta boundary before implementation drifts into a LangChain clone, chat shell, coding-agent clone, or API gateway.

Deliverables:

- This plan document.
- `ALPHA_STATUS` / `NEXT_STEPS` point to Agentic Forge Beta in progress.
- Explicit absorption rules for mature frameworks: absorb run lifecycle, checkpoint/interrupt, tool permissioning, durable working state, and observability; do not absorb branded crew/swarm/chat/prompt/memory ontology.

Acceptance:

- Documentation links pass.
- No stale temporary-plan references.

## Phase A — Package-owned run lifecycle / working state / plan graph (complete)

**Goal:** Upgrade the agent lab from "one run emits trace/proposal" into a package-owned run contract.

Delivered:

- `official/agentic-forge-lab` exposing stable capabilities:
  - `describe_contract`
  - `start_run`
  - `inspect_run`
  - `cancel_run`
  - `summarize_run`
  - `export_plan_graph`
- Package-owned run lifecycle: `created`, `prepared`, `running`, `paused`, `waiting_for_approval`, `completed`, `failed`, `cancelled`, `archived`.
- Plan graph artifact: nodes/edges/status/revision/input_refs/output_refs/approval_policy/retry_policy/deterministic_mode.
- Working state artifact: run_id, owner_package, target_branch_ref, scratch_branch_ref, current_objective, local_context_refs, plan_graph_ref, candidate_refs, tool_observation_refs, inference_trace_refs, policy_state.
- TypeScript SDK helper: `sdk/typescript/agentic-forge`, for run events, plan graphs, working state, and candidate shapes.
- Conformance: 5 cases covering describe_contract, start_run plan graph/working state, inspect/cancel/summarize, raw-secret blocking, no kernel agent namespace.
- No `kernel.agent.*`, `kernel.model.*`, `kernel.prompt.*`, `kernel.memory.*`, or `kernel.turn.*` protocol methods added.

Non-goals: real models, real long-running background agents, multi-agent orchestration.

## Phase B — Branch-aware scratch branch / candidate / compare / promote proof (complete)

**Goal:** Make the default agent workflow target branch → scratch branch → candidate → compare → proposal/promotion.

Delivered:

- `start_run` produces scratch branch policy metadata by default (intent, target_revision, promote_requires_proposal, stale_target_blocks_promote).
- Candidate artifact: candidate_id, run_id, target_branch_ref, scratch_branch_ref, changed_asset_refs, projection_refs, diff_summary, inspection_refs, confidence, uncertainty, provenance, status, target_revision.
- Candidate states: draft, ready, comparing, promoting, promoted, rejected, archived, failed.
- Capabilities:
  - `create_candidate` — deterministic candidate generation, never writes target branch.
  - `compare_candidate` — scratch vs target diff summary with stale detection (revision mismatch).
  - `draft_promote_proposal` — produces proposal_draft only, no direct mutation; stale target revision mismatch blocks promote with `stale_target_branch`.
  - `archive_candidate` — sets archived status, target branch unchanged.
  - `explain_branch_policy` — explains scratch/target/promote constraints.
- Raw-secret blocking applies to all new capabilities.
- TypeScript SDK extended with Candidate, CandidateComparison, PromoteProposalDraft, BranchPolicy types and createCandidate/compareCandidate/createPromoteProposalDraft/archiveCandidate/validateCandidate helpers.
- Conformance: 5 cases (create_candidate branch-aware, compare_candidate stale=false on matching revision, draft_promote_proposal no direct mutation, stale_promote_blocked on revision mismatch, archive_candidate target unchanged).
- No `kernel.agent.*`, `kernel.model.*`, `kernel.prompt.*`, `kernel.memory.*`, or `kernel.turn.*` protocol methods added.

Non-goals: complex merge engine, domain-specific diff ontology, automatic promote.

## Phase C — Inference-backed agent run with deterministic fallback (complete)

**Goal:** Connect live/fake inference to agent runs without making model output runtime authority.

Delivered:

- Plan node kinds: `observe`, `infer`, `tool_call`, `inspect`, `branch_op`, `compare`, `propose`, `wait`.
- `run_inference_node`: deterministic (default), recorded, cloud_adapter_plan, local_fake providers.
  - Deterministic/recorded/local_fake produce `candidate_seed` or `proposal_seed` only; never directly create candidates or promote.
  - `cloud_adapter_plan` returns `needs_host_policy` with no network performed.
  - Inference trace includes provider_kind, model_performed, network_performed, output_action, fingerprint.
  - Raw-secret blocking applies.
- `replay_inference_node`: fingerprint match → replay_ok, mismatch → replay_mismatch (flagged, never silently passed).
- `validate_inference_output`: allowlist (candidate_seed, proposal_seed, observation, needs_repair); reject privilege_escalation, auto_promote, secret_request, target_branch_write, unknown_action.
- `explain_inference_failure`: taxonomy (rate_limit, quota, timeout, auth, network_denied, invalid_output, malformed_output, replay_mismatch, policy_reject) with typed recovery hints.
- TypeScript SDK extended with ProviderKind, AllowedInferenceAction, ForbiddenInferenceAction, InferenceFailureKind, InferenceNodeResult, InferenceTrace, RunInferenceNodeResponse, ReplayInferenceNodeResponse, InferenceOutputValidation, InferenceFailureExplanation types and runInferenceNode, replayInferenceNode, validateInferenceOutput, explainInferenceFailure, computeDeterministicFingerprint helpers.
- Conformance: 5 cases (deterministic inference node candidate_seed, replay match/mismatch flagged, privilege escalation rejected, cloud_adapter_plan needs_host_policy no network, failure taxonomy recovery hints).
- No `kernel.agent.*`, `kernel.model.*`, `kernel.prompt.*`, `kernel.memory.*`, or `kernel.turn.*` protocol methods added.

Non-goals: always-on autonomous agents, provider router, cost optimizer, multi-model tournament.

## Phase D — Tool bridge v2: scoped toolchain observation / risk / replay (complete)

**Goal:** Let agents use tools without introducing confused deputy behavior or ambient authority.

Delivered:

- Extended `official/capability-tool-bridge-lab`:
  - `explain_tool_call`: scoped grant summary with branch-aware tool call context (requesting_package, run_id, plan_node_id, target_branch_scope, scratch_branch_scope, asset_scope, capability_grant, approval_policy, audit_context), no_execution=true, no_ambient_authority=true, requires_approval=true.
  - `record_tool_observation`: accepts untrusted tool output, marks untrusted=true, returns observation_ref/provenance; large output (>100KB) triggers asset_ref recommendation with truncation; raw-secret-like content blocked with redaction_state=unsafe_blocked.
  - `summarize_tool_risk`: risk categories (prompt_injection, secret_exfiltration, branch_write, outbound_expansion, nested_delegation, large_output) with typed mitigations; overall_risk level (critical/high/medium/low).
  - `replay_tool_plan`: deterministic fingerprint replay; match → replay_ok, mismatch → replay_mismatch (flagged, never silently passed).
  - `plan_toolchain`: multi-step plan-only; each step must have explicit provider_package_id; missing provider → blocked; nested delegation without explicit_delegation=true → blocked; target branch write without promote grant → blocked; provider not in candidates → blocked; valid steps → planned with no_execution=true, no_ambient_authority=true.
- Confused deputy protection: no provider or provider mismatch fails closed; target branch write without promote grant blocked; outbound host outside grant blocked.
- TypeScript SDK extended with ToolRiskCategory, ToolCallContext, ToolchainStep, ToolObservation, ToolRiskFinding types and createToolCallContext, computeToolPlanFingerprint, createToolchainStep, hasPromptInjectionPattern helpers.
- Conformance: 5 cases (explain_tool_call scoped/no ambient authority, record_observation untrusted/large output/redaction, tool_risk injection/exfiltration/outbound, replay_tool_plan mismatch flagged, plan_toolchain requires explicit provider/nested delegation blocked).
- No `kernel.agent.*`, `kernel.model.*`, `kernel.prompt.*`, `kernel.memory.*`, or `kernel.turn.*` protocol methods added.

Non-goals: all-powerful ToolExecutor, default shell/fs/git tools, automatic permission escalation.

## Phase E — Forge Agent Workspace / Observability UI shell (Completed)

**Goal:** Upgrade Forge from “can view agent proof” to the first agentic control room.

Delivered:

- New Agentic Forge sections in Forge:
  - Run timeline
  - Read-only plan graph view
  - Scratch branch diff / branch lineage panel
  - Candidate compare/promote panel
  - Tool/inference trace panel
  - Approval/reject/cancel/promote/fork affordances, expressed as public-protocol payloads and disabled-safe controls
- UI reads only public protocol / surfaces / events / proposals / assets / projections, never runtime internals.
- No chat-first UI — the Forge object model focuses on runs, plans, candidates, diffs, proposals, and traces.
- `clients/web/src/agent/observability.ts` adds `ForgeAgentWorkspaceModel` types plus `buildForgeAgentWorkspace`/`renderForgeAgentWorkspaceSections` functions.
- UI copy makes clear that agent output is candidate/proposal, not assistant message.
- `tsc -p clients/web/tsconfig.json --noEmit` passes.

## Phase F — Third-party replacement proof, hostile conformance, docs cleanup

**Goal:** Prove Agentic Forge has no first-party privilege and remove this temporary plan.

Planned deliverables:

- `examples/packages/thirdparty-agentic-forge` and composition replacement proof.
- Hostile conformance expansion:
  - no official priority
  - equivalent third-party run/candidate/proposal shape
  - target branch write denied
  - reject leaves target unchanged
  - stale promote blocked
  - prompt injection secret exfiltration blocked
  - confused deputy blocked
  - runaway loop budget/deadline stopped
  - cancellation state consistent
  - replay mismatch flagged
- Durable docs: `docs/guides/AGENTIC_FORGE_PACKAGE_AUTHORING.md`, `ALPHA_STATUS`, `NEXT_STEPS`, conformance matrix.
- Delete this temporary plan.

Final acceptance:

- `cargo test --workspace`
- `cargo run -p ygg-cli -- conformance`
- Relevant package check / composition check
- `tsc -p clients/web/tsconfig.json --noEmit`
- SDK self-test
- doc link check
- No new `kernel.agent.*` / `kernel.model.*` / `kernel.prompt.*` / `kernel.memory.*` protocol.

## Non-goals

- No LangChain clone, universal chain executor, or prompt template registry.
- No chat shell or assistant-message-as-core-object.
- No coding-agent clone; repository/fs/shell/patch is not the default worldview.
- No agent marketplace, agent SaaS, or always-on autonomous background agents.
- No provider zoo, model routing, or OpenAI-compatible agent endpoint.
- No agent/run/plan/task/memory/tool/model/prompt/chat/goal in the kernel.

## Success criteria

Agentic Forge Beta succeeds not because it supports more agent frameworks, but because:

1. Agent runs are package-owned, observable, cancellable, and archivable.
2. Agents explore in scratch branches by default and cannot directly write target branches.
3. Candidates can be compared, inspected, rejected, and promoted.
4. Promote goes through proposal/approval/apply and preserves lineage.
5. Tool calls have scoped grants, audit, risk summaries, and replay; no ambient authority.
6. Live inference and deterministic fallback run the same class of run.
7. Forge users see run/plan/candidate/diff/proposal/trace, not chat transcripts.
8. A third-party agentic forge package can replace the official one.
9. The kernel remains content-free.
