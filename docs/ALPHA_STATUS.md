# Alpha 状态

> [English](./ALPHA_STATUS.en.md) · [中文](./ALPHA_STATUS.md)

这是 Yggdrasil 当前状态的实时快照。每当一个里程碑关闭时更新。它不是愿景：下面每一行都有代码和 conformance 支撑（或被明确标注为 partial/deferred）。

长期架构和产品立场见 `docs/CHARTER.md`、`docs/architecture/VISION.md`、`docs/product/PLAY_CREATION_MODEL.md` 和 `docs/product/EXPERIENCE_LED_PLATFORM_BETA.md`。后续方向见 `docs/roadmap/NEXT_STEPS.md`。

## 概要

- **阶段：** Platform Foundation Alpha + Play/Forge Surface Contract Beta + Secure Execution Substrate Alpha + Optional Text Engine Alpha + Agent Infrastructure Alpha + Model Provider Integration Alpha + Live Model Calls Alpha + Creative Inference Capability Alpha + Agentic Forge Beta。
- **Conformance：** 180 个具名 CLI 用例，加上 crate 和 service 单元测试。
- **Charter 纪律：** 内核内容无关，官方包无特权，仅公开协议，包跨入口形式平等，trusted paths 阻止 raw secret，使用 secret_ref 引用，permission grants 可重新水化，网络权限强制执行并带 outbound audit/redaction，通用 streaming 与 cancellation lifecycle，SDK secure-execution helpers，networked/streaming 包模板，no-network readiness proof，**outbound executor boundary（deny-all 默认 + fake executor conformance）**。
- **代码健康：** CLI commands/templates/conformance、runtime domain behavior、protocol dispatch 与 runtime official in-process handlers 已按领域拆分，不再继续堆进巨型单文件。
- **当前主线：** Agentic Forge Beta 已完成；Yggdrasil 已具备 package-owned、branch-aware、tool-safe、inference-backed、deterministically testable 的 creative agent runtime scaffold。Agent 仍是普通包，不进入 kernel ontology；agent 在 scratch branch 探索并产生 candidate/proposal，而不是直接修改 target branch 或退化成 chat/coding-agent/API gateway。下一阶段方向已确定为 **Experience-Led Platform Beta**，但这是路线而非已实现能力：停止 foundation-first 扩张，用一个真实 AI-native playable experience 牵引 Experience Runtime Contract、State/Asset Pipeline、Memory/Knowledge Package、Experience Observability、Creator Loop 与 Sharing/Distribution 的后续底座建设。长期设计见 `docs/product/EXPERIENCE_LED_PLATFORM_BETA.md`。

## 已实现

### 内核

- 内容无关的 session、只追加不透明事件、manifest 驱动的包、能力 fabric、hook fabric 切片、surface contributions、proposal lifecycle、asset/branch/projection 底座。
- SQLite 支撑的持久事件日志，每 session 单调递增序号，可重新水化的底座。
- JSON Schema 子集用于能力输入/输出和包声明的 event payload。
- Principal：`host_admin`、`host_dev`、`package`、`human`、`assistant`、`anonymous`。human 和 assistant principal 的作用域授权。
- 权限审计事件：`kernel/permission.granted`、`kernel/permission.revoked`、`kernel/permission.denied`。
- 包 lifecycle 事件：`kernel/package.loading|starting|ready|stopping|stopped|loaded|unloaded|degraded|log`。
- Proposal lifecycle 事件：`kernel/proposal.created|approved|rejected|applied|failed`。
- 持久权限授权：`kernel/permission.granted|revoked` 事件可在 SQLite-backed runtime 中重新水化，重启后授权仍可用于 human/assistant principal 的作用域检查。
- **Secret reference contract**：`SecretRef` 类型支持 `secret_ref:<vault>:<key>`、`secretRef:`、`secret-ref:` 和 `host:` reference patterns。包通过 `secret_ref` identifier 引用 secret；raw secrets 不得出现在 events、proposals、logs 或 audit records 中。
- **Host secret resolver**：`HostSecretResolver` trait 和 deny-all resolver 已存在，用于未来 host-level secret store。**`EnvSecretResolver`（Phase L1）**：host-owned 环境变量解析器，带显式 allowlist。支持 `secret_ref:env:NAME`、`secretRef:env:NAME`、`secret-ref:env:NAME`、`host:env:NAME`。默认 deny-all；env name 必须显式 allow。缺失/拒绝/格式错误返回 typed error，仅含 env name 不含 raw value。`Runtime::resolve_secret_ref` host 内部方法。Raw value 只短暂存在于 host 内部；`Debug`/`Serialize`/audit 不包含。`host:<key>`（不含 `env:` 前缀）不被视为 env 引用。
- **Raw-secret blocking**：Proposal operations/expected effects 与 asset metadata 会被保守扫描；明显 raw API keys、token/password fields 会被拒绝。Asset content 和普通 prose 字段不扫描，以避免误伤用户内容。
- **网络权限声明**：Manifest `permissions.network` 同时支持扁平 `hosts`（向后兼容）和结构化 `declarations`（含 `host`、`methods`、`purpose`）。Runtime 策略检查器根据声明的条目匹配出站请求。无网络声明的包被拒绝出站访问。官方包无绕过。
- **Outbound audit/redaction records**：`OutboundAuditRecord` 记录 principal、package_id、capability_id、destination_host、method、purpose、redaction_state、secret_refs_used、usage/cost 占位符、status/error。Raw body/header/prompt/response 不会被保存——仅记录 `secret_ref` 标识符和 `redaction_state` 枚举（`not_captured`、`redacted`、`policy_ref`、`unsafe_blocked`、`explicitly_approved`）。默认为 `redacted`。
- **网络策略检查器**：`check_network_policy` 纯函数和 `check_and_audit_outbound` runtime 方法。支持精确 host 匹配、通配符前缀（`*.example.com`）、method 白名单（空 = 任意）和扁平 `hosts` 向后兼容。被拒绝的请求产生 `kernel/outbound.denied` 审计事件；被允许的请求产生 `kernel/outbound.request` 事件。
  - **Outbound executor boundary（M3 + L2 + L3 + L4 + L5）**：内容无关的 `OutboundExecutor` trait，含 `OutboundExecutorRequest`（package_id、capability_id、destination_host、method、path、purpose、secret_refs、redaction_state、timeout_ms、metadata、body_shape、**secret_headers**、**resolved_secret_headers**、**static_headers**）和 `OutboundExecutorResponse`（status、status_code、headers_shape、body_shape、provider_request_id、usage、cost、redaction_state、network_performed、executor_kind）。`DenyAllOutboundExecutor` 无网络返回拒绝（默认，fail-closed）。`FakeOutboundExecutor` 按 host/method/path 提供确定性 fixture，带调用计数用于 conformance，无真实网络。**`LiveHttpOutboundExecutor`（L2）**：使用 reqwest + rustls 执行真实 HTTPS 请求。默认关闭（`RuntimeConfig::default()` 仍为 `DenyAll`，需显式 `OutboundExecutorConfig::LiveHttp`）。HTTPS-only（拒绝 http:// URL），redirect policy none（L2 不实现 redirect following），timeout/connect_timeout 可配置，只发送 content-type: application/json 和 x-ygg-outbound placeholder headers（不注入 secret）。**L4：`build_headers` 从 `resolved_secret_headers` 注入 Authorization 等 secret headers，raw value 只存在于 HTTP 请求中，不存入 audit/response/Debug**。响应只记录 redacted headers_shape（auth/cookie 等值为 `[redacted]`）、redacted body_shape（JSON 内 secret 字段替换为 `[redacted]`，非 JSON 只记录 kind/bytes_captured）、provider_request_id（仅 request-id 安全 headers）、redaction_state redacted、network_performed true、executor_kind Real。错误归一为 status="error"或"timeout"，不包含 raw body/secret。`allow_insecure_loopback_for_tests` 标志默认 false，仅允许 127.0.0.1/localhost 的 http:// URL，用于 conformance 测试。`LiveHttpOutboundExecutorConfig` 提供 timeout_ms、connect_timeout_ms、allow_redirects、max_response_preview_bytes、allow_insecure_loopback_for_tests 配置。`execute_outbound_with_policy` runtime 方法：先校验 policy/audit request 与 executor request 的 package/capability/host/method/secret_refs 一致，再检查策略；拒绝或不一致时不调用 executor，允许时调用 executor，raw body 不进审计，secret_refs 仅存引用。`RuntimeConfig` 上的 `OutboundExecutorConfig` 新增 `LiveHttp(LiveHttpOutboundExecutorConfig)` 变体，默认仍为 `DenyAll`。核心无 provider-specific 字段；用不透明 `metadata` 承载 executor 特定数据。它只保护 Ygg-provided outbound path，不声称 OS 级拦截任意 subprocess 网络调用。Redirect-target following 延后至 L4。**`kernel.outbound.execute`（L3 + L4）**：公开协议方法，允许 ordinary package 通过 host outbound executor 发起出站请求。Params 接受 capability_id、destination_host、method、path、secret_refs、metadata、body_shape。package_id 从 ProtocolContext principal 强制确定，调用者不能在 params 中 spoof 不同的 package_id（host_dev/host_admin principal 可在 params 中指定用于测试）。dispatch 调用 `execute_outbound_with_policy`，response 经过额外 raw-secret 防护 sweep（已知 secret 字段名值替换为 `[redacted]`）。不新增 `kernel.secret.resolve`（raw secret 不返回给包）。L3 不注入 secret headers（真实注入延后至 L4/L5）。
- **协议方法**：`kernel.outbound.audit` 列出给定包的出站审计事件。`kernel.outbound.execute` 允许 ordinary package 通过 host outbound executor 发起出站请求（L3）。
- **Streaming invocation registry**：内存中的 `StreamRegistry` 追踪进行中的 streaming capability 调用，支持 start/append/end/cancel/timeout 生命周期。`StreamFrameEnvelope` 定义通用内容无关的 stream frame 类型（start/chunk/progress/end/error/cancelled/timeout），包含 invocation_id、stream_id、sequence、redaction_state 和 timestamp/metadata。不包含 model/prompt/agent 语义。
- **Streaming capability 生命周期**：`kernel.capability.stream` 启动 streaming invocation（验证 descriptor 中 `streaming=true`），`kernel.capability.cancel` 取消进行中的 invocation。Runtime 方法按序发出 kernel 事件：`kernel/stream.started`、`kernel/stream.chunk`、`kernel/stream.progress`、`kernel/stream.ended`、`kernel/stream.error`、`kernel/stream.cancelled`、`kernel/stream.timeout`。Cancel 和 timeout 阻断后续 chunk。非 streaming 能力（descriptor `streaming=false`）被拒绝。
- **Streaming invocation 记录**：`StreamInvocationRecord` 追踪 invocation_id、stream_id、capability_id、provider_package_id、session_id、状态（active/ended/error/cancelled/timeout）、frame_count、时间戳和 metadata。终态阻断后续 frame 追加。
- **Secure-execution TypeScript helpers**（`sdk/typescript/secure-execution/index.ts`）：`secretRef()`/`isValidSecretRef()`/`looksLikeRawSecret()`/`isSecretFieldName()` 用于 secret reference 构造和验证。`NetworkDeclaration` 类用于构建 manifest 兼容的网络权限条目，支持 host/method 匹配。`OutboundAuditHelper` 类用于构建审计安全的出站请求 payload，拒绝 raw secrets，仅包含 `secret_ref` 标识符。`StreamFrameClient` 类用于构建 faux stream frame envelope，支持完整生命周期（start/chunk/progress/end/error/cancel/timeout）。所有 helper 只包装公开协议和类型——无私有内部、无协议绕过。
- **Inference capability TypeScript SDK**（`sdk/typescript/inference-capability/index.ts`）：Transport-neutral 推理能力契约 SDK。`InferenceRequest`/`InferenceResponse`/`InferenceStreamFrame`/`InferenceError` 类型，`InferenceOperationKind`/`TransportKind`/`ModalityKind`/`RuntimeKind` 枚举，`ProviderCapabilityManifest` provider 声明。`createInferenceRequest` 构造请求并拒绝 raw secrets。`classifyInferenceError` 映射 cloud 和 local/resource 错误。`InferenceStreamLifecycle` 管理 start→chunk→end/error/cancel/timeout 生命周期。`createProviderCapabilityManifest`/`validateProviderCapabilityManifest` 构建和验证 provider manifest。69 项纯 TS 自测通过。不包含 URL/header/status code/OpenAI messages 字段；cloud adapter 只是 `transport_kind="http"` 的一类 provider。
- **Agentic Forge TypeScript SDK**（`sdk/typescript/agentic-forge/index.ts`）：Package-owned agent run lifecycle、plan graph、working state、candidate、compare、promote、archive、inference node/replay/validation/failure 及 tool bridge v2 shape helper。`AgentRunLifecycleState` 枚举（9 个状态）。`CandidateState` 枚举（8 个状态）。`ProviderKind`/`AllowedInferenceAction`/`ForbiddenInferenceAction`/`InferenceFailureKind`/`ToolRiskCategory` 枚举。`PlanNode`/`PlanEdge`/`PlanGraph`/`WorkingState`/`RunEvent`/`CandidateShell`/`Candidate`/`CandidateComparison`/`PromoteProposalDraft`/`ObservabilitySummary`/`BranchPolicy`/`InferenceNodeResult`/`InferenceTrace`/`RunInferenceNodeResponse`/`ReplayInferenceNodeResponse`/`InferenceOutputValidation`/`InferenceFailureExplanation`/`ToolCallContext`/`ToolchainStep`/`ToolObservation`/`ToolRiskFinding` 类型。`createRunEvent`/`validatePlanGraph`/`createPlanGraph`/`createWorkingState`/`createCandidateShell`/`createCandidate`/`compareCandidate`/`createPromoteProposalDraft`/`archiveCandidate`/`validateCandidate` 构造器和验证器。`runInferenceNode`/`replayInferenceNode`/`validateInferenceOutput`/`explainInferenceFailure`/`computeDeterministicFingerprint` inference helper。`createToolCallContext`/`computeToolPlanFingerprint`/`createToolchainStep`/`hasPromptInjectionPattern` tool bridge v2 helper。`blockRawSecrets`/`looksLikeRawSecret`/`isSecretFieldName`/`hasKernelAgentNamespace` secret 安全与命名空间安全 helper。纯 TS 自测（154 项断言）。输出不含 `kernel.agent.*`/`kernel.model.*`/`kernel.prompt.*`/`kernel.memory.*`/`kernel.turn.*`。
- **包模板**：`--template networked` 生成带网络权限声明的 subprocess package（`host`、`methods`、`purpose`），包含带 `network` side effect 的 `fetch` capability 和 `echo` capability。演示 `secretRef`、`NetworkDeclaration` 和 `OutboundAuditHelper` 用法。`--template streaming` 生成带 streaming capability（`streaming: true`）的 subprocess package，演示 `StreamFrameClient` faux frame 生命周期。`--template agent-runtime` 生成 deterministic/no-network agent-like subprocess package，包含 streaming run、trace summary、proposal draft 与 echo capabilities，以及 assistant_action + forge_panel surfaces。使用 `StreamFrameClient`（secure-execution）与 `createTraceEvent`/`createProposalDraft`/`blockRawSecrets`（ygg-agent-adapter）。三个模板默认安全：无 raw secrets、无隐式 network 访问。
- **No-network readiness proof 示例**：`examples/packages/faux-model-readiness/` 证明 model-like 包的 substrate shape（网络声明、secret_ref 用法、discovery plans、faux streaming frames——不做真实 inference）。`examples/packages/faux-agent-readiness/` 证明 agent-like 包的 substrate shape（proposal/trace 模式、无网络权限、faux streaming trace——不做真实 agent loop 或 pi runtime coupling）。

### 公开协议与传输

- 规范的请求/响应信封，附带 host 绑定的 principal 上下文。调用者不能自行断言 package 或 admin 身份。
- HTTP `POST /rpc` 和 host JSON-RPC stdio（`ygg host-stdio`）调用同一套 dispatcher。
- HTTP SSE 事件订阅，支持 `after_sequence` replay 和对 host-dev 调用者的实时追尾。
- Profile 驱动的 `ygg host serve` 自动加载包并暴露 `/rpc` 与 SSE。
- WebSocket 和 TCP 传输保留为未来工作；remote 和 WASM 入口保留为第一等 manifest 形式，执行延后。

### 包执行

- `rust_inproc` 包通过 host 提供的 package trait 和 catalog 执行。声明了 in-process provider 但 catalog 中缺失的 manifest 会被拒绝。
- `subprocess` 包通过 JSON-RPC over stdio 执行，支持 handshake、invoke、invoke 超时、degraded 状态、restart、kill-on-unload 和 stderr 日志捕获。
- `wasm` 和 `remote` 入口：manifest 支持已就绪，执行延后。
- 能力路由支持显式 provider 选择和简单精确匹配 / `^x.y` 版本约束。路由歧义时拒绝，除非调用者指定 `provider_package_id`。
- Hook fabric 切片：确定性排序、包拥有的 handler 能力、payload 元数据修改、veto、unload 清理，覆盖 `kernel/event.before_append|after_append` 和 `kernel/capability.before_invoke|after_invoke`。

### 底座

- Asset 注册表：不透明的 `id`/`mime`/`hash`/`size`/`origin_package_id`/`metadata`，可从 SQLite 重新水化。权限强制和内容寻址 blob 存储为下一步。
- Session fork/branch 血缘记录，可从事件日志重新水化。
- 通用 projection 注册表。Rebuild 以 `kind_prefix` 和 `writer_package_id` 过滤事件并写入 `kernel/projection.updated`。包拥有的 projection 执行为下一步。
- Surface contributions：带版本、slot、activation、所需权限、approval 策略、metadata 的类型化描述符。Slot：`experience_entry`、`home_card`、`play_renderer`、`forge_panel`、`asset_editor`、`assistant_action`。可通过 `kernel.surface.contribution.list` 和 `.describe` 发现。
- Proposal lifecycle：`kernel.proposal.create|get|list|approve|reject|apply`。Apply 当前执行通用 `asset.put` 和 `projection.rebuild` 操作。更广泛的事务和 revert/compensation 为下一步。

### 官方包

全部为普通包。无内核特权。位于 `packages/official/`，通过普通 manifest 加载：

- `official/package-lab` —— 包创作辅助，以普通能力和 surface 暴露。
- `official/schema-tools` —— schema 验证辅助。
- `official/event-tools` —— 事件过滤与检查辅助。
- `official/composition-lab` —— composition 验证、launch-plan、permission-preview、surface-graph 与 compat-report 辅助，支持 v2 descriptor 诊断（capabilities、permissions、replacements、compatibility notes）。
- `official/asset-lab` —— 通用 asset preview、diff、export 与 import-plan 辅助。
- `official/projection-lab` —— projection describe、diff、rebuild-plan 与 source-event 辅助。
- `official/persona-lab` —— persona profile import、normalization、rendering 与 compatibility diagnostics。
- `official/knowledge-lab` —— structured knowledge collection normalization、matching、injection planning 与 diagnostics。
- `official/context-lab` —— bounded context block assembly、layer inspection、budget planning 与 template rendering。
- `official/text-transform-lab` —— deterministic text transform import、validation、preview、pipeline explanation 与 diagnostics。
- `official/model-connector-lab` —— no-network provider family metadata、profile validation、secret masking、discovery plans 与 compatibility reports。
- `official/model-provider-lab` —— cloud API adapter lab，不是 Yggdrasil 模型抽象、不是 API gateway、无 kernel privilege。提供 no-network 八家 cloud provider 的 adapter-local request builders/profile validation（拒绝 raw secret）、fake/local invoke（覆盖全部八家：OpenAI chat/responses、Anthropic messages、Gemini generateContent、OpenAI-compatible chat、OpenRouter chat/responses、DeepSeek chat、xAI chat/responses、Fireworks chat/responses；outbound_request_shape 可审计）、stream normalization（delta SSE、semantic SSE、typed chunk stream → StreamFrameEnvelope frames：start/chunk/progress/end/error/cancelled/timeout；覆盖全部八家；terminal_frame_consistent 校验；provider event 输入归一化）、error explanation、echo。`normalize_request` 是 package-local helper，不是平台 canonical inference request。
- `official/model-routing-lab` —— no-inference consumer-slot binding、route planning、fallback planning 与 params normalization。
- `official/assistant-lab` —— assistant-action 能力，返回需要审批的 proposal。
- `official/pi-agent-runtime-lab` —— 参考代理运行时包，deterministic no-network run plan、trace summary、proposal draft 与 echo。
- `official/capability-tool-bridge-lab` —— 发现 capabilities、预览权限、显式 provider 选择、通过 kernel.capability.invoke/stream 的 invocation/streaming plan，不偏袒 official provider。Phase D（Agentic Forge Beta）新增：explain_tool_call（scoped grant summary 含 branch-aware tool call context，no_execution，no_ambient_authority）、record_tool_observation（untrusted=true，大输出 asset_ref 推荐，raw-secret 阻断）、summarize_tool_risk（prompt_injection/secret_exfiltration/branch_write/outbound_expansion/nested_delegation/large_output 含 typed mitigations）、replay_tool_plan（确定性指纹匹配/不匹配，绝不静默通过）、plan_toolchain（多步 plan-only，显式 provider 必需，嵌套 delegation 无 explicit_delegation 时阻止，target branch 写入无 promote grant 时阻止，provider 不匹配 fail closed）。
- `thirdparty/agentic-forge` —— 第三方 agentic forge 替换证明。证明 official/agentic-forge-lab 可被替换：package-owned run lifecycle、plan graph、scratch branch candidates、promote proposals、inference fallback 和 tool bridge scoped grants——全部确定性，无网络，无 kernel 特权。参见 `examples/compositions/agentic-forge-replacement/`。
- `official/inference-local-lab` —— deterministic non-HTTP fake local inference provider proof。证明 inference capability seam 不依赖 cloud API、HTTP、Bearer token、JSON provider schema。提供 describe_capabilities（transport_kinds in_memory/local_process、network_required=false、secrets_required=false）、invoke（拒绝 http transport、HTTP-shaped/messages-shaped 字段、raw secret；返回 deterministic output、network_performed=false、transport_performed=in_memory_fake）、stream（deterministic start/chunk/progress/end frames、无 URL/header/status/provider_schema）、explain_error（覆盖 local/resource 错误类）。5 个 conformance 用例。
- `official/inference-playtest-lab` —— Ygg-native inference proposal vertical slice。证明推理不是"prompt -> text response"，而是参与 Yggdrasil session/branch/proposal/inspection/fork 创作运行时。提供 draft_proposal（产 proposal_draft，含 requires_user_approval=true、asset.put、source_inference provenance、拒绝 raw secret）、inspect_proposal（返回 risk/operations/permissions/provenance）、branch_plan（返回建议 fork metadata，不直接 fork）、explain_flow（返回 session→inference→proposal→inspect→approve/reject→apply→fork 说明）。5 个 conformance 用例。
- `official/agentic-forge-lab` —— Agentic Forge Beta Phase A+B+C：package-owned agent run lifecycle、working state、plan graph 契约，branch-aware scratch branch / candidate / compare / promote 证明，以及 inference-backed agent run with deterministic fallback。Phase A 提供 describe_contract（列出所有能力、lifecycle 状态、plan graph 字段、working state 字段）、start_run（产生含 plan graph 和 working state 的确定性 run，阻断 raw-secret-like 输入并返回 redaction_state=unsafe_blocked）、inspect_run（返回 run 检查结果，含 working state 和 lifecycle）、cancel_run（将 run 转换为 cancelled 状态并生成 trace event）、summarize_run（返回含 event/node/candidate 计数的 observability summary）、export_plan_graph（返回含 nodes/edges/status/revision/approval_policy/retry_policy/deterministic_mode 的 plan graph artifact）。Phase B 新增 create_candidate（确定性生成 candidate，不写 target）、compare_candidate（scratch vs target diff summary 含 stale 检测）、draft_promote_proposal（仅生成 proposal draft，不直接修改 target；stale target revision 不匹配时阻止 promote）、archive_candidate（设置 archived，target 不变）、explain_branch_policy（说明 scratch/target/promote 约束）。Phase C 新增 run_inference_node（deterministic/recorded/cloud_adapter_plan/local_fake provider；仅产生 candidate_seed/proposal_seed；cloud_adapter_plan 返回 needs_host_policy 且不执行网络）、replay_inference_node（指纹匹配 ok / 不匹配标记，绝不静默通过）、validate_inference_output（allowlist: candidate_seed/proposal_seed/observation/needs_repair；拒绝 privilege_escalation/auto_promote/secret_request/target_branch_write/unknown_action）、explain_inference_failure（9 项 taxonomy 含 typed recovery hint）。无 inference、无 network、无 kernel.agent/model/prompt/memory/turn 命名空间。15 个 conformance 用例。
- `official/blank-experience` —— 最小体验，被 `ygg play-create-demo` 用来跑通游创循环。
- `official/playable-seed` —— 带有 entry/play/Forge/assistant surfaces 的 reference playable package。

Forge profile（`profiles/forge-alpha.yaml`）自动加载这些包以及示例 fixture 包。

### Web shell（`clients/web`）

- 骨架化的 Home/Play、Forge 和 Assist surface，走公开协议。
- Home 发现 `experience_entry` surface，通过包声明的 launch 能力启动 session，支持 session fork。
- Forge 检查事件、能力、asset、projection、proposal 和 Forge-panel surface contributions，提供 proposal 的 approve/apply 控制。
- 没有官方包硬编码。Shell 和其他客户端一样是公开协议客户端。
- **Text Surface Proof（Phase T1）**：Assistant Drawer 中加入受限 mock streaming text proof，使用 `clients/web/src/text-layout/`。它展示渐进 mock chunks、行数/高度估算、stream 生命周期徽章和 reset/replay 控件。不调用真实 agent/model，不出网，不改变 kernel/package/protocol surface。
- **Optional Text Engine（Phase T2）**：`TextEngine` 接口、engine registry、带限宽缓存（4096 条）的 fallback engine、通用 stream-frame-to-buffer adapter。未修改 kernel/protocol。
- **Optional Pretext Engine（Phase T3）**：`PretextTextEngine` 通过 dynamic import 加载，运行时 feature flags（`auto`/`fallback`/`pretext`），优雅降级。仓库无需安装 `@chenglou/pretext` 即可 build。Assistant Drawer 显示引擎偏好、Pretext 可用性和 fallback 原因。
- **Forge Text Preview（Phase T4）**：文本预览 helper，从 event payload、stream frame 和 proposal 对象中提取安全纯文本。Forge Events 和 Proposals 中新增可选 `<details>`，含预览文本、行数/高度估算和引擎徽章。不替换 JSON inspector。
- **SDK 抽取与硬化（Phase T5）**：`sdk/typescript/text-surface` — 纯 TypeScript 前端 SDK，提供 `createTextSurfaceBuffer`、`applyStreamFrame`、`extractTextChunk`、`createScrollAnchor`（不依赖 `clients/web`）。字体加载 helper（`ensureTextSurfaceFontLoaded`、`describeFontLoadState`）。缓存诊断（`getCacheDiagnostics` 含 `totalEntries`/`fontCount`/`maxEntries`/`estimatedBytes`）。自测模块（`runTextLayoutSelfTest`），用纯 TS 断言覆盖 fallback engine、registry、stream adapter 和 text preview。
- **Agent Observability（Phase J5）**：`clients/web/src/agent/observability.ts` — 纯 UI helper，用通用字符串启发式从 events、proposals、surfaces、capabilities 中提取 agent-like 观测数据（不 hardcode official 包，不做真实 model/network 调用）。Forge surface 新增 "Agent Observability" section：cards/summary、trace timeline、tool bridge diagnostics badges、proposal explanation（复用 T4 text preview）。Assistant Drawer 新增轻量 "Agent Readiness" panel：显示当前发现的 agent-like surfaces/capabilities count，强调 no real model / no network / proposal-gated / plan-only；按钮 disabled，不真正启动 agent。
- **Forge Agent Workspace / Observability UI Shell（Phase E）**：`clients/web/src/agent/observability.ts` 扩展 `ForgeAgentWorkspaceModel`、`buildForgeAgentWorkspace`、`renderForgeAgentWorkspaceSections`。Forge surface 中新增六个 Agentic Forge workspace panels：Run timeline（run lifecycle events）、Plan graph read-only（plan node events）、Branch diff & lineage（scratch/target branch events）、Candidate compare & promote（candidate-like proposals）、Tool & inference trace（tool bridge & inference events）、Controls（approval/reject/cancel/promote/fork/archive affordances 含 public protocol payload previews）。所有数据仅来自 public protocol events、proposals、surfaces、capabilities、packages、assets、projections，无 runtime internals、无 chat-first UI。所有 panels 包含 protocol shape docs 和第三方可替换文案。`tsc -p clients/web/tsconfig.json --noEmit` 通过。

### 创作

- `ygg init-package` 生成 Python 或 TypeScript subprocess 包骨架。TypeScript 变体使用 `sdk/typescript/subprocess` 下的 SDK runtime。
- `--template basic|experience|play-renderer|forge-panel|assistant-action|asset-editor|full-surface|networked|streaming|agent-runtime` 控制生成的 surface 描述符。未指定 `--template` 时，`--language *-experience` 自动检测为 legacy 4-surface 体验模式以兼容旧行为；否则默认 basic。`networked` 模板增加网络权限声明，演示 `secretRef`/`NetworkDeclaration`/`OutboundAuditHelper` 用法。`streaming` 模板增加 streaming capability，演示 `StreamFrameClient` faux frame 生命周期。`agent-runtime` 模板生成 agent-like 包，包含 streaming run/trace/proposal/echo capabilities 与 assistant_action/forge_panel surfaces，使用 `ygg-agent-adapter` SDK。
- `--language typescript-experience`（未指定 `--template`）仍生成原始 4-surface 体验描述符以兼容旧行为。
- `ygg init-composition` 和 `ygg composition check` 提供本地 composition descriptor 流程，支持 v2 字段（title、description、optional packages、required capabilities、default activation、permission expectations、replacement candidates、compatibility notes）。`composition check` 输出结构化诊断：已加载的 required/optional 包、surfaces 按 slot 归类、capabilities、entry activation、缺失的 required surfaces/capabilities（失败）、以及 optional 包缺失警告。
- `ygg package check` 和 `ygg package conformance` 在本地验证生成的包。`ygg package check` 输出结构化诊断信息：entry kind、trust level、capability 数量、surfaces 按 slot 归类、permissions 摘要、sandbox policy 摘要，以及对无 capability 或无 surface 的包发出警告。
- `ygg package reload <manifest>` 将包加载到内存 runtime，重启（仅 subprocess），输出重启前后状态和日志数量，然后卸载。使用现有 Runtime::restart_package 路径；不新增协议方法。
- `ygg package run-fixture` 使用确定性 canned 输入调用所有声明的非 streaming 能力，并输出结构化 JSON 摘要。
- `ygg play-create-demo` 通过普通公开协议调用端到端地编排空白游创循环。

### 代码组织

- `crates/ygg-cli/src/main.rs` 是薄入口。CLI 类型位于 `cli.rs`；commands 位于 `commands/`；包生成模板位于 `templates/`；conformance 用例按领域位于 `conformance/` 模块。
- `crates/ygg-runtime/src/runtime/` 按 session、events、packages、capabilities、hooks、permissions、assets、branches、projections、proposals 和 protocol dispatch 模块承载 runtime domain behavior；`runtime/mod.rs` 保持公开 `Runtime<S>` API，并 re-export 移动后的公开 request/record types。
- Protocol method metadata 与 dispatch 共享 `KernelMethod` 单一事实源，并有 registry/dispatch 一致性单元覆盖。
- `crates/ygg-runtime/src/inproc.rs` 保留 in-process package API，并把 official lab 行为委托给 `crates/ygg-runtime/src/inproc/` 下的聚焦模块。
- `crates/ygg-runtime/src/inproc/common.rs` 按 provider package 和 local capability name 路由共享 official in-process handlers，而不是 suffix-only fallback。
- 这次拆分不改变行为，目的是让后续 package、conformance 和 handler 增长保持可审查。

### Conformance

- `cargo run -p ygg-cli -- conformance` 运行 180 个具名 CLI 用例，覆盖：session、事件、包、能力、hook、schema、principal、权限、subprocess 执行、host 传输、surface、proposal、官方包、composition-lab、asset-lab、projection-lab、persona-lab、knowledge-lab、context-lab、text-transform-lab、model-connector-lab、model-provider-lab、model-routing-lab、pi-agent-runtime-lab、capability-tool-bridge-lab、inference-local-lab、**inference-playtest-lab（draft/inspect/reject-apply-denied/apply+branch/no-chat-kernel-terms proof）**、**agentic-forge-lab（Phase A: describe_contract/start_run plan graph+working state/inspect+cancel+summarize/raw-secret blocked/no-kernel-agent-namespace + Phase B: create_candidate/compare_candidate+stale_detection/draft_promote_proposal_no_mutation/stale_promote_blocked/archive_candidate_target_unchanged + Phase C: inference_node_deterministic_candidate_seed/replay_match_mismatch_flagged/inference_output_privilege_escalation_rejected/cloud_adapter_needs_host_policy_no_network/inference_failure_taxonomy_recovery_hints + Phase D: explain_tool_call_scoped_no_ambient_authority/record_observation_untrusted_large_output_redaction/tool_risk_injection_exfiltration_outbound/replay_tool_plan_mismatch_flagged/plan_toolchain_requires_explicit_provider_nested_delegation_blocked + Phase F: thirdparty_replacement_shape_no_official_priority/no_official_priority_ordinary_package/hostile_injection_secret_blocked_cross_package/budget_deadline_contract_cancellation_consistent/cross_package_replay_mismatch_flagged）**、in-process fallback hardening、playable-seed、游创循环、生成包创作、composition descriptor、package check、reload 冒烟测试、第三方替换证明、permission grant rehydrate、secret_ref validation、raw-secret blocking、official no-secret-bypass、**env secret resolver（allowed/denied/missing-no-leak；deny-all 默认；allowlist-only；no raw value leak）**、网络权限审计、策略纯函数测试、**outbound executor boundary（被拒绝请求不调用 executor、policy/executor mismatch fail-closed、allowlisted fake executor 返回 network_performed:false、raw body 不进审计、secret_refs 仅存引用、host mismatch redirect denied）**、**model provider invoke adapters（OpenAI chat/responses、Anthropic messages、Gemini generateContent fake/local invoke、raw credential rejected、unsupported family diagnostic、outbound_request_shape 可审计）**、**model provider outbound shape fake executor（三 provider host/method/path/secret_ref shape 通过 outbound boundary、call_count=3、executor_kind Fake）**、**model provider stream normalization（八家 delta SSE/semantic SSE/typed chunk stream 归一为 start/chunk/progress/end frames、terminal_frame_consistent、provider event 输入归一化、raw secret 不 echo）**、streaming/cancellation 生命周期、模板 conformance、no-network readiness proof、**inference-local-lab（non-HTTP describe/invoke/reject/stream/error proof）**、**live HTTP outbound executor（默认 DenyAll 仍生效、非 HTTPS URL fail-closed 无网络、response shape 不含 raw body/header/secret）**、**kernel.outbound.execute public protocol（package principal 通过 context 确定 package_id 不能 spoof、FakeOutboundExecutor + allowed network declaration 成功且 audit 产生、spoofed package_id 被拒绝、无 network permission denied 且 executor 不调用、secret_refs 仅引用 no raw secret in response）**、**L4：outbound secret_headers 解析验证（secret_headers params 格式正确解析、raw secret 不出现在 response）、local loopback HTTP server secret injection conformance（Authorization header 真实到达 server、raw secret 不出现在 protocol response/audit/log）、DeepSeek SSE stream normalize canary（delta_sse start→chunk→end lifecycle、terminal_frame_consistent、no raw secrets）、opt-in live DeepSeek conformance（默认跳过、YGG_LIVE_MODEL_TESTS=1 + DEEPSEEK_API_KEY 时才尝试真实调用）、canary DeepSeek profile shape（endpoint/dialect/stream_family 正确、secret_ref placeholder 不含 raw key）**、**L5：OpenAI/Anthropic/Gemini live adapter conformance（OpenAI chat loopback 验证 Authorization bearer、OpenAI responses loopback、Anthropic messages loopback 验证 x-api-key secret + anthropic-version static header、Gemini generateContent loopback 验证 x-goog-api-key、missing secret fails closed、provider normalize_request alignment、no raw secret leak across all providers、static_headers safe allowlist、static_headers 阻止 secret-bearing 名）**。
- 加上 `cargo test --workspace` 下的 crate 和 service 单元测试。
- `tsc -p clients/web/tsconfig.json --noEmit` 检查 web shell。

## 部分实现

- 能力调用 lifecycle 事件（`kernel/capability.invoked|completed|failed`）已在契约中预留；尚未发出。
- Streaming 协议分发自 partial（stream start/cancel 生命周期可用；真实网络 streaming 延后）。
- Package-principal 的 `event.subscribe` 权限。
- Hook handler 超时/错误审计，面向包拥有的 handler。
- 持久化的能力 provider 选择策略（超越单次调用显式选择）。
- 更丰富的资源策略覆盖（filesystem 强制矩阵）—— Secure Execution 后续 hardening 目标。
- 内容寻址的 asset blob 存储和 package-principal asset 权限检查。
- 包拥有的 projection 执行。
- 更丰富的崩溃监控和健康检查（超出当前 lifecycle 事件）。
- 更广泛的传输一致性覆盖（超出当前核心协议 dispatcher 和 service 测试）。
- 更丰富的 TypeScript SDK 打包（超出当前薄 subprocess 辅助层和 secure-execution helpers）。
- 完整的 `kernel.session.get|list`、`kernel.package.describe`、`kernel.capability.describe`、`kernel.extension_point.describe`、`kernel.host.principal`、`kernel.host.ping` 路由暴露。

## 延后事项

这些是内核的非目标，预期以普通包或未来工作的形式交付：

- 对话 runtime、提示词、模型、采样、消息/回合语义。
- 记忆模型、检索、摘要、agent loop、director。
- 世界、场景、角色、规则、骰子、背包语义。
- SillyTavern 资源和行为兼容（见 `docs/tavern/TAVERN_COMPAT.md`）。
- 生产级长期自治 agent、多 agent 协作、生产级记忆系统与更完整 live-ops（Agentic Forge、provider adapter、live calls 与 inference capability 底座已完成；见 `docs/guides/AGENTIC_FORGE_PACKAGE_AUTHORING.md`、`docs/guides/INFERENCE_CAPABILITY_AUTHORING.md` 与 `docs/guides/MODEL_PROVIDER_INTEGRATION.md`）。
- 外部游戏引擎桥接（UE5、Godot、Unity、web 客户端）。
- 市场、包签名、依赖解析器。
- 最终 UI 视觉设计、完整 Studio、ComfyUI 风格节点编辑器。
- WASM 和 remote 包执行。

## 如何验证此快照

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- play-create-demo
tsc -p clients/web/tsconfig.json --noEmit
```

如果以上任何一步失败，以这份文档为准的是代码；请更新此文档。

## 延伸阅读

- `docs/CHARTER.md` —— 不变的根本原则。
- `docs/architecture/VISION.md` —— 平台为何而存在。
- `docs/architecture/ARCHITECTURE.md` —— kernel + packages 两层架构。
- `docs/architecture/PLATFORM_KERNEL.md` —— 内核做什么、不做什么。
- `docs/architecture/CAPABILITY_PACKAGE.md` —— 能力包契约。
- `docs/architecture/EVENT_MODEL.md` —— 不透明事件日志。
- `docs/architecture/EXTENSION_POINTS.md` —— hook 契约。
- `docs/architecture/RUNTIME_LIFECYCLE.md` —— 内核侧生命周期。
- `docs/protocol/PROTOCOL_V0.md` —— 公开协议。
- `docs/spec/KERNEL_V0_ALPHA_CONTRACT.md` —— 可执行的 alpha 契约矩阵。
- `docs/spec/CONFORMANCE_MATRIX.md` —— hostile conformance 路线图。
- `docs/product/PLAY_CREATION_MODEL.md` —— 游创一体的产品立场。
- `docs/product/EXPERIENCE_LED_PLATFORM_BETA.md` —— Agentic Forge 之后的体验牵引平台路线。
- `docs/guides/AGENT_PACKAGE_AUTHORING.md` —— agent-like 能力包创作指南。
- `docs/guides/MODEL_PROVIDER_INTEGRATION.md` —— 多 provider cloud API 接入指南。
- `docs/guides/INFERENCE_CAPABILITY_AUTHORING.md` —— transport-neutral 推理能力包创作指南。
- `docs/roadmap/NEXT_STEPS.md` —— 当前与下一阶段。
