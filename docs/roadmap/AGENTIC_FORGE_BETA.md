# Agentic Forge Beta 计划

> [English](./AGENTIC_FORGE_BETA.en.md) · [中文](./AGENTIC_FORGE_BETA.md)

Agentic Forge Beta 的目标不是再证明 Yggdrasil 能托管 agent-like package，而是把 agent 从 lab/proof 推进为 **Yggdrasil-native creative engine**：agent 能观察 branch/projection，制定 package-owned plan graph，调用受限 capability tools，在 scratch branch 中探索，生成 candidate，并以可审查 proposal 的方式请求用户 promote。

本计划参考 `/workspace/Yggdrasil/pi`、LangGraph、OpenAI Agents SDK、Temporal、AutoGen、CrewAI、Claude Code / Codex permission model 与 OpenChamber workspace 模式；但它不复制任何框架的产品 ontology。成熟框架提供的是可恢复 run、checkpoint、interrupt/resume、tool safety、human-in-the-loop、multi-agent orchestration、workspace observability 等原则；在 Yggdrasil 中这些原则必须落在 package layer、public protocol、branch/proposal/asset/projection substrate 上。

## 核心原则

1. **Agent 是普通 package，不是 kernel primitive。** 不新增 `kernel.agent.*`、`kernel.model.*`、`kernel.prompt.*`、`kernel.memory.*`、`kernel.turn.*`。
2. **Agent produces candidates, not authoritative mutations.** Agent 默认写 scratch branch，不能直接写 target branch。
3. **Promote requires inspection.** Candidate 进入 target branch 必须通过 proposal/inspection/approval/apply。
4. **Run state is package-owned.** Run lifecycle、plan graph、working state、task queue、memory-like state 全部由 package-owned assets/events/projections 表达。
5. **Tool safety is layered.** Capability declaration、scoped grant、approval、sandbox/resource boundary、audit 分层，不做 ambient privileged tool runner。
6. **Live inference is replaceable.** Cloud provider 是现实交付路径，但 deterministic/fake/local provider 必须能跑同一类 run。
7. **Forge is a control room, not a chat shell.** UI 主对象是 run timeline、plan graph、scratch diff、candidate compare、proposal queue、tool/inference trace，不是聊天记录。
8. **Observability is part of the contract.** Run、node、tool、inference、branch、candidate、proposal、approval、retry、cancel 都必须可追踪。

## 已有基础

- `sdk/typescript/ygg-agent-adapter`：capability→tool、permission preview、proposal draft、trace event、raw-secret blocking。
- `official/pi-agent-runtime-lab`：deterministic/no-network agent-like run/proposal/trace lab。
- `official/capability-tool-bridge-lab`：discover/preview/invoke/stream plan，显式 provider，ambiguous fail-closed。
- `examples/packages/thirdparty-agent-runtime` 与 `examples/compositions/agent-runtime-replacement`：第三方替换证明。
- Forge/Assist Agent Observability：初步 traces/tool diagnostics/readiness 视图。
- `sdk/typescript/inference-capability`、`official/inference-local-lab`、`official/inference-playtest-lab`：transport-neutral inference 与 inference→proposal proof。

## Phase 0 — 计划与边界锁定

**目标：** 固化 Agentic Forge Beta 的边界，避免滑向 LangChain clone、chat shell、coding-agent clone 或 API gateway。

交付：

- 本计划文档。
- `ALPHA_STATUS` / `NEXT_STEPS` 指向 Agentic Forge Beta 执行中。
- 明确成熟 agent 框架的吸收方式：可吸收 run lifecycle、checkpoint/interrupt、tool permission、durable working state、observability；不吸收品牌化 crew/swarm/chat/prompt/memory ontology。

验收：

- 文档链接通过。
- 无临时计划引用错误。

## Phase A — Package-owned run lifecycle / working state / plan graph（已完成）

**目标：** 把当前 agent lab 从"一次 run 产 trace/proposal"提升为 package-owned run contract。

已交付：

- `official/agentic-forge-lab` 提供稳定 capabilities：
  - `describe_contract`
  - `start_run`
  - `inspect_run`
  - `cancel_run`
  - `summarize_run`
  - `export_plan_graph`
- Package-owned run lifecycle：`created`、`prepared`、`running`、`paused`、`waiting_for_approval`、`completed`、`failed`、`cancelled`、`archived`。
- Plan graph artifact：nodes/edges/status/revision/input_refs/output_refs/approval_policy/retry_policy/deterministic_mode。
- Working state artifact：run_id、owner_package、target_branch_ref、scratch_branch_ref、current_objective、local_context_refs、plan_graph_ref、candidate_refs、tool_observation_refs、inference_trace_refs、policy_state。
- TypeScript SDK helper：`sdk/typescript/agentic-forge`，用于构建 run event、plan graph、working state、candidate shape。
- Conformance：5 个用例覆盖 describe_contract、start_run plan graph/working state、inspect/cancel/summarize、raw-secret blocking、no kernel agent namespace。
- 未新增 `kernel.agent.*`、`kernel.model.*`、`kernel.prompt.*`、`kernel.memory.*` 或 `kernel.turn.*` 协议方法。

非目标：真实模型、真实长期后台 agent、多 agent 编排。

## Phase B — Branch-aware scratch branch / candidate / compare / promote proof（已完成）

**目标：** 让 agent 的默认工作模式变成 target branch → scratch branch → candidate → compare → proposal/promotion。

已交付：

- `start_run` 默认生成 scratch branch policy metadata（intent、target_revision、promote_requires_proposal、stale_target_blocks_promote）。
- Candidate artifact：candidate_id、run_id、target_branch_ref、scratch_branch_ref、changed_asset_refs、projection_refs、diff_summary、inspection_refs、confidence、uncertainty、provenance、status、target_revision。
- Candidate states：draft、ready、comparing、promoting、promoted、rejected、archived、failed。
- Capabilities：
  - `create_candidate` — 确定性 candidate 生成，不写 target branch。
  - `compare_candidate` — scratch vs target diff summary，含 stale 检测（revision 不匹配）。
  - `draft_promote_proposal` — 仅生成 proposal_draft，不直接修改 target；stale target revision 不匹配时返回 `stale_target_branch` 阻断 promote。
  - `archive_candidate` — 设置 archived 状态，target branch 不变。
  - `explain_branch_policy` — 说明 scratch/target/promote 约束。
- Raw-secret blocking 适用于所有新增 capabilities。
- TypeScript SDK 扩展 Candidate、CandidateComparison、PromoteProposalDraft、BranchPolicy 类型和 createCandidate/compareCandidate/createPromoteProposalDraft/archiveCandidate/validateCandidate helper。
- Conformance：5 个用例（create_candidate branch-aware、compare_candidate stale=false for matching revision、draft_promote_proposal no direct mutation、stale_promote_blocked on revision mismatch、archive_candidate target unchanged）。
- 未新增 `kernel.agent.*`、`kernel.model.*`、`kernel.prompt.*`、`kernel.memory.*` 或 `kernel.turn.*` 协议方法。

非目标：复杂 merge engine、domain-specific diff ontology、自动 promote。

## Phase C — Inference-backed agent run with deterministic fallback（已完成）

**目标：** 把 live/fake inference 接进 agent run，但模型输出不能成为 runtime authority。

已交付：

- Plan node kinds：`observe`、`infer`、`tool_call`、`inspect`、`branch_op`、`compare`、`propose`、`wait`。
- `run_inference_node`：deterministic（默认）、recorded、cloud_adapter_plan、local_fake provider。
  - Deterministic/recorded/local_fake 仅产生 `candidate_seed` 或 `proposal_seed`；不直接 create candidate 或 promote。
  - `cloud_adapter_plan` 返回 `needs_host_policy`，不执行网络。
  - Inference trace 包含 provider_kind、model_performed、network_performed、output_action、fingerprint。
  - Raw-secret blocking 适用。
- `replay_inference_node`：指纹匹配 → replay_ok，不匹配 → replay_mismatch（标记，绝不静默通过）。
- `validate_inference_output`：allowlist（candidate_seed、proposal_seed、observation、needs_repair）；拒绝 privilege_escalation、auto_promote、secret_request、target_branch_write、unknown_action。
- `explain_inference_failure`：taxonomy（rate_limit、quota、timeout、auth、network_denied、invalid_output、malformed_output、replay_mismatch、policy_reject）含 typed recovery hint。
- TypeScript SDK 扩展 ProviderKind、AllowedInferenceAction、ForbiddenInferenceAction、InferenceFailureKind、InferenceNodeResult、InferenceTrace、RunInferenceNodeResponse、ReplayInferenceNodeResponse、InferenceOutputValidation、InferenceFailureExplanation 类型和 runInferenceNode、replayInferenceNode、validateInferenceOutput、explainInferenceFailure、computeDeterministicFingerprint helper。
- Conformance：5 个用例（deterministic inference node candidate_seed、replay match/mismatch flagged、privilege escalation rejected、cloud_adapter_plan needs_host_policy no network、failure taxonomy recovery hints）。
- 未新增 `kernel.agent.*`、`kernel.model.*`、`kernel.prompt.*`、`kernel.memory.*` 或 `kernel.turn.*` 协议方法。

非目标：自动长期 autonomous agent、provider router、cost optimizer、多模型 tournament。

## Phase D — Tool bridge v2: scoped toolchain observation / risk / replay（已完成）

**目标：** 让 agent 能使用 tools，但不引入 confused deputy 或 ambient authority。

已交付：

- 扩展 `official/capability-tool-bridge-lab`：
  - `explain_tool_call`：scoped grant summary 含 branch-aware tool call context（requesting_package、run_id、plan_node_id、target_branch_scope、scratch_branch_scope、asset_scope、capability_grant、approval_policy、audit_context），no_execution=true、no_ambient_authority=true、requires_approval=true。
  - `record_tool_observation`：接受 untrusted tool output，标记 untrusted=true，返回 observation_ref/provenance；大输出（>100KB）触发 asset_ref 推荐含 truncation；raw-secret-like 内容阻断返回 redaction_state=unsafe_blocked。
  - `summarize_tool_risk`：risk categories（prompt_injection、secret_exfiltration、branch_write、outbound_expansion、nested_delegation、large_output）含 typed mitigations；overall_risk level（critical/high/medium/low）。
  - `replay_tool_plan`：确定性指纹重放；匹配 → replay_ok，不匹配 → replay_mismatch（标记，绝不静默通过）。
  - `plan_toolchain`：多步 plan-only；每步必须有 explicit provider_package_id；缺 provider → blocked；nested delegation 无 explicit_delegation=true → blocked；target branch 写入无 promote grant → blocked；provider 不在 candidates 中 → blocked；合法步骤 → planned 含 no_execution=true、no_ambient_authority=true。
- Confused deputy 保护：无 provider 或 provider 不匹配 fail closed；target branch 写入无 promote grant 阻断；outbound host 不在 grant scope 内阻断。
- TypeScript SDK 扩展 ToolRiskCategory、ToolCallContext、ToolchainStep、ToolObservation、ToolRiskFinding 类型和 createToolCallContext、computeToolPlanFingerprint、createToolchainStep、hasPromptInjectionPattern helper。
- Conformance：5 个用例（explain_tool_call scoped/no ambient authority、record_observation untrusted/large output/redaction、tool_risk injection/exfiltration/outbound、replay_tool_plan mismatch flagged、plan_toolchain requires explicit provider/nested delegation blocked）。
- 未新增 `kernel.agent.*`、`kernel.model.*`、`kernel.prompt.*`、`kernel.memory.*` 或 `kernel.turn.*` 协议方法。

非目标：全能 ToolExecutor、shell/fs/git 默认工具、自动权限提升。

## Phase E — Forge Agent Workspace / Observability UI shell（已完成）

**目标：** 把 Forge 从"能看 agent proof"升级为 agentic control room 的第一版。

已交付：

- Forge 中新增 Agentic Forge sections：
  - Run timeline
  - Plan graph read-only view
  - Scratch branch diff / branch lineage panel
  - Candidate compare/promote panel
  - Tool/inference trace panel
  - Approval/reject/cancel/promote/fork affordances（以 public protocol payload/disabled-safe controls 表达）
- 所有数据只来自 public protocol / surfaces / events / proposals / assets / projections，不读 runtime internals。
- 不做 chat-first UI。
- `clients/web/src/agent/observability.ts` 新增 `ForgeAgentWorkspaceModel` 类型及 `buildForgeAgentWorkspace`/`renderForgeAgentWorkspaceSections` 函数。
- UI 文案明确 agent 输出是 candidate/proposal，不是 assistant message。
- `tsc -p clients/web/tsconfig.json --noEmit` 通过。

## Phase F — Third-party replacement proof, hostile conformance, docs cleanup

**目标：** 证明 Agentic Forge 不是官方包特权，并清理临时计划。

计划交付：

- `examples/packages/thirdparty-agentic-forge` 与 composition replacement proof。
- Hostile conformance 扩展：
  - official 无优先级
  - third-party run/candidate/proposal shape 等价
  - target branch write denied
  - reject target unchanged
  - stale promote blocked
  - prompt injection secret exfiltration blocked
  - confused deputy blocked
  - runaway loop budget/deadline stopped
  - cancellation state consistent
  - replay mismatch flagged
- Durable docs：`docs/guides/AGENTIC_FORGE_PACKAGE_AUTHORING.md`、`ALPHA_STATUS`、`NEXT_STEPS`、conformance matrix。
- 删除本临时计划。

最终验收：

- `cargo test --workspace`
- `cargo run -p ygg-cli -- conformance`
- 相关 package check / composition check
- `tsc -p clients/web/tsconfig.json --noEmit`
- SDK self-test
- doc link check
- 无 `kernel.agent.*` / `kernel.model.*` / `kernel.prompt.*` / `kernel.memory.*` 新增协议。

## 非目标

- 不做 LangChain clone、universal chain executor、prompt template registry。
- 不做 chat shell、assistant message 作为核心对象。
- 不做 coding-agent clone，不以 repo/fs/shell/patch 为默认世界观。
- 不做 agent marketplace、agent SaaS、always-on autonomous background agents。
- 不做 provider zoo、模型路由、OpenAI-compatible agent endpoint。
- 不把 agent/run/plan/task/memory/tool/model/prompt/chat/goal 放入 kernel。

## 成功标准

Agentic Forge Beta 成功不是因为“支持更多 agent 框架”，而是因为：

1. Agent run 是 package-owned，可观察、可取消、可归档。
2. Agent 默认在 scratch branch 探索，不能直接写 target branch。
3. Candidate 可 compare、inspect、reject、promote。
4. Promote 经过 proposal/approval/apply，并保留 lineage。
5. Tool 调用有 scoped grant、audit、risk、replay，不存在 ambient authority。
6. Live inference 与 deterministic fallback 能跑同一类 run。
7. Forge 用户看到的是 run/plan/candidate/diff/proposal/trace，不是聊天记录。
8. 第三方 agentic forge package 能替代官方包。
9. Kernel 仍然 content-free。
