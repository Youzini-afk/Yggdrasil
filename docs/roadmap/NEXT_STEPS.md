# 后续步骤

> [English](./NEXT_STEPS.en.md) · [中文](./NEXT_STEPS.md)

平台基础已经就位。Yggdrasil 现在拥有内容无关的内核、基于 manifest 的包、真正的 `rust_inproc` 和 subprocess 执行、权限/principal 系统、hook fabric 切片、surface 贡献、proposal/approval lifecycle、asset/branch/projection 底座、安全执行原语、官方平台包、assistant 包、`official/playable-seed`、空白游创循环，以及走公开协议的 Home/Play、Forge、Assist 和受限文字界面 proof 的 Web shell。

Agent Infrastructure Alpha、Model Provider Integration Alpha、Live Model Calls Alpha、Creative Inference Capability Alpha、Agentic Forge Beta、Experience Beta 0、Experience Beta 1、Experience Beta 2、Experience Beta 3、Experience Beta 4 与 Experience Beta 5 已完成。Experience Beta 6（Sharing / Distribution Alpha）已完成。Yggdrasil 现在可以用普通能力包描述、验证、归一化并 fake/local 调用 OpenAI、Anthropic、Gemini、OpenAI-compatible、OpenRouter、DeepSeek、xAI、Fireworks 等 provider API 差异；也具备 host-owned `secret_ref:env:*`、public `kernel.outbound.execute`、LiveHttpOutboundExecutor、redacted audit、live loopback provider shapes、transport-neutral inference seam、inference→proposal proof、package-owned / branch-aware / tool-safe 的 Agentic Forge runtime scaffold、package-owned experience runtime contract、首个真实 playable vertical slice（含 board/module/constraint/marker state），稳定 content-addressed asset helper 与 provenance graph 和 state snapshot convention，包拥有的体验可观测性实验室和 Forge 体验观测面板，包拥有的长期记忆与知识实验室（proposal-gated update / correction / forget-redaction / branch-aware view——全部确定性、无网络、无推理），创作者循环（template-to-playable 路径、面向创作者的诊断），以及包拥有的分享与分发实验室（export/import composition bundle、branch/session bundle manifest、package-set lockfile、compatibility/migration report、AI disclosure metadata bundle、read-only shared session manifest、async fork sharing plan——全部确定性、无网络、无 marketplace、无 billing）。默认 conformance 不依赖公网；手动 live smoke 必须显式 opt-in。临时阶段计划已删除，结果收敛到 durable docs。

## 当前位置

- Platform Foundation Alpha：已完成。
- Play/Forge Surface Contract Beta：已完成。
- First Real Capability Package Track：seed 已完成（`composition-lab`、`asset-lab`、`projection-lab`、`playable-seed`；55 个 conformance 用例）。
- Platform Host Alpha：已实现切片完成；剩余项目（streaming 分发、hook 超时审计、持久 provider 策略、更广的传输层一致性、更丰富的 SDK 打包）在下方 Phase I 中追踪。
- Code Health Split Alpha：已完成；CLI commands/templates/conformance、runtime domain behavior、protocol dispatch（按领域委托的 helper）、runtime official in-process handlers（provider-package indexed dispatch、共享 safety 模块）已按领域拆分。
- Authoring & Composition Beta+：已完成；生成 package templates、fixture/reload tooling、composition v2 diagnostics、Forge authoring panels 与第三方 playable replacement proof 已就位。
- Secure Execution Substrate：Alpha 切片已完成。持久 grants、`secret_ref`、host resolver placeholder、raw-secret blocking、网络权限声明、outbound audit/redaction、通用 streaming/cancel 生命周期、secure-execution TypeScript helpers、networked/streaming templates，以及 no-network model/agent readiness examples 已就位。
- Text Surface Proof：Phase T1/T2/T3/T4/T5 已完成。`integrations/pretext` 记录 Pretext 参考边界，Assistant Drawer 中已有基于 `clients/web/src/text-layout` 的受限 mock streaming text proof，且没有 kernel/protocol/package 变更。`sdk/typescript/text-surface` 提供纯 TypeScript 前端 SDK 供第三方 UI 使用。字体加载、缓存诊断和自测模块已就位。
- Agent Infrastructure Alpha：已完成；`integrations/pi` ledger、`sdk/typescript/ygg-agent-adapter`、`--template agent-runtime`、`official/pi-agent-runtime-lab`、`official/capability-tool-bridge-lab`、Forge/Assist Agent Observability、`thirdparty/agent-runtime` replacement proof 和 [`docs/guides/AGENT_PACKAGE_AUTHORING.md`](../guides/AGENT_PACKAGE_AUTHORING.md) 已就位。
- Model Provider Integration Alpha：已完成；`integrations/model-providers` research ledger、`sdk/typescript/model-provider-adapter`、`official/model-provider-lab`、provider profile examples 和 [`docs/guides/MODEL_PROVIDER_INTEGRATION.md`](../guides/MODEL_PROVIDER_INTEGRATION.md) 已就位。
- Live Model Calls Alpha：已完成；成果已收敛进 [`docs/guides/MODEL_PROVIDER_INTEGRATION.md`](../guides/MODEL_PROVIDER_INTEGRATION.md)、[`docs/ALPHA_STATUS.md`](../ALPHA_STATUS.md) 和 conformance matrix。
- Creative Inference Capability Alpha：已完成；`sdk/typescript/inference-capability` transport-neutral envelope/stream/error/manifest helpers、[`docs/guides/INFERENCE_CAPABILITY_AUTHORING.md`](../guides/INFERENCE_CAPABILITY_AUTHORING.md)、`official/inference-local-lab` deterministic non-HTTP fake inference provider proof、`official/model-provider-lab` cloud API adapter 降级定位、`official/inference-playtest-lab` Ygg-native inference proposal vertical slice 均已就位。Conformance 包含 155 个具名用例。
- Agentic Forge Beta Phase A：已完成；`official/agentic-forge-lab` 提供 describe_contract/start_run/inspect_run/cancel_run/summarize_run/export_plan_graph 能力，`sdk/typescript/agentic-forge` TS SDK，5 个 conformance 用例。Conformance 包含 160 个具名用例。
- Agentic Forge Beta Phase B：已完成；扩展 `official/agentic-forge-lab` 增加 create_candidate/compare_candidate/draft_promote_proposal/archive_candidate/explain_branch_policy 能力；branch-aware scratch branch intent/metadata；candidate artifacts 含 stale 检测；proposal draft 不直接修改 target branch；stale target revision 不匹配时阻止 promote；5 个 conformance 用例。Conformance 包含 165 个具名用例。
- Agentic Forge Beta Phase C：已完成；扩展 `official/agentic-forge-lab` 增加 run_inference_node/replay_inference_node/validate_inference_output/explain_inference_failure 能力；8 个显式 plan node kind；inference provider（deterministic/recorded/cloud_adapter_plan/local_fake）；cloud_adapter_plan 返回 needs_host_policy 且不执行网络；replay 指纹不匹配时标记而非静默通过；inference output action allowlist 与 forbidden actions；9 项 failure taxonomy 含 typed recovery hint；5 个 conformance 用例。Conformance 包含 170 个具名用例。
- Agentic Forge Beta Phase D：已完成；扩展 `official/capability-tool-bridge-lab` 增加 explain_tool_call（scoped grant summary，branch-aware tool call context，no_execution，no_ambient_authority）/record_tool_observation（untrusted=true，大输出 asset_ref，raw-secret 阻断）/summarize_tool_risk（prompt_injection/secret_exfiltration/branch_write/outbound_expansion/nested_delegation/large_output 含 typed mitigations）/replay_tool_plan（指纹匹配/不匹配）/plan_toolchain（多步 plan-only，显式 provider 必需，嵌套 delegation 无 explicit_delegation 时阻止，target branch 写入无 promote grant 时阻止）；5 个 conformance 用例。Conformance 包含 175 个具名用例。
- Agentic Forge Beta Phase E：已完成；Forge 中新增 Agentic Forge 六个 workspace panels（Run timeline / Plan graph / Branch lineage / Candidate compare / Tool & inference trace / Controls），所有数据来自 public protocol，不做 chat-first UI。`clients/web/src/agent/observability.ts` 新增 `ForgeAgentWorkspaceModel` 及 build/render 函数。`tsc -p clients/web/tsconfig.json --noEmit` 通过。
- Agentic Forge Beta Phase F：已完成；第三方替换证明（`thirdparty/agentic-forge` manifest + 替换 composition，无 official 优先）、hostile conformance（prompt injection + secret exfiltration 跨包阻断，privilege escalation 拒绝）、budget/deadline 契约（describe_contract 中 run_constraints，cancellation 状态一致）、跨包 replay 不匹配标记；5 个 conformance 用例。持久指南：[`docs/guides/AGENTIC_FORGE_PACKAGE_AUTHORING.md`](../guides/AGENTIC_FORGE_PACKAGE_AUTHORING.md)。Conformance 包含 180 个具名用例；状态已收敛到 ALPHA_STATUS/NEXT_STEPS/guide/conformance matrix。
- Experience Beta 0 — Thin Experience Runtime Contract：已完成；`official/experience-runtime-lab` 提供 describe_contract/create_checkpoint/inspect_checkpoint/draft_recovery/bind_agent_run 能力与 4 个 surface（experience_entry、play_renderer、forge_panel、assistant_action）；`sdk/typescript/experience-runtime` TS SDK（85 项自测断言）；`--template experience-runtime` 生成 deterministic/no-network subprocess；Forge profile 自动加载；7 个 conformance 用例。持久指南：[`docs/guides/EXPERIENCE_RUNTIME_AUTHORING.md`](../guides/EXPERIENCE_RUNTIME_AUTHORING.md)。Conformance 包含 187 个具名用例。
- Experience Beta 1 — First Real Playable Vertical Slice：已完成；`official/playable-creation-board` 提供 describe_contract/launch/project_state/render_payload/record_player_action/request_change/create_checkpoint/inspect_checkpoint/draft_recovery/bind_agent_run/explain_provenance 共 11 项能力与 4 个 surface（experience_entry、play_renderer、forge_panel、assistant_action）；package-owned board/module/constraint/marker state；player action 产生 state_delta_asset_ref/projection_ref/sequence/provenance；request_change 输出 structured agent objective / allowed_change_kinds / risk/budget / bindable refs（不是聊天消息）；bind_agent_run 产出 scoped agentic-forge binding；explain_provenance 输出 player_action_event→state_delta_asset→checkpoint→agent_run→candidate→proposal→projection_rebuild 因果链；checkpoint/recovery 对齐 experience-runtime-lab 形状；raw-secret blocking；第三方 agentic-forge 替换 composition 证明无 official priority；CLI demo `playable-board-demo`；Forge profile 自动加载；10 个 conformance 用例。Conformance 包含 197 个具名用例。
- Experience-Led Platform Beta：当前方向；长期设计见 [`docs/product/EXPERIENCE_LED_PLATFORM_BETA.md`](../product/EXPERIENCE_LED_PLATFORM_BETA.md)。核心判断：基础设施已经足以停止 foundation-first，下一阶段应由真实 playable experience 牵引 Experience Runtime Contract、State/Asset Pipeline、Memory/Knowledge Package、Experience Observability、Creator Loop 与 Sharing/Distribution。Experience Beta 0–6 已完成。Performance & Code Health Beta 已完成，持久指南见 [`docs/performance/PERFORMANCE_AND_CODE_HEALTH.md`](../performance/PERFORMANCE_AND_CODE_HEALTH.md)。External Project Operating Plane Alpha Phase E1–E3 已完成，临时计划见 [`docs/roadmap/EXTERNAL_PROJECT_OPERATING_PLANE_ALPHA.md`](./EXTERNAL_PROJECT_OPERATING_PLANE_ALPHA.md)。

详见 `docs/ALPHA_STATUS.md` 获取详细快照。

## Phase F — Foundation Alpha 收敛（已完成）

目标：停止扩大表面积。打磨粗糙边缘，锁定契约，让现有基础便于 demo、文档和扩展。

- 跨 `README.md`、`README.en.md` 和文档树刷新文档。
- 添加 `docs/product/PLAY_CREATION_MODEL.md` 以固定游创产品立场。
- 添加 `docs/ALPHA_STATUS.md` 作为已完成、partial 和 deferred 内容的活快照。
- 在代价较低处解决 Platform Host Alpha 的剩余 partial 项目。
- @oracle-led 审查轮次，检查内容形态泄漏、官方特权泄漏和 YAGNI 清理。
- 一条规范的端到端 demo 路径，有文档记录并通过 conformance 验证。

当新贡献者可以 clone 仓库、读一份 README、运行一条 host serve 命令、到达空白游创循环且没有意外时，此阶段完成。

## Phase G — Playable Experience Alpha seed（已完成）

目标：通过构建可启动、可检查、可 fork、可由 assistant 辅助的 reference packages 来证明底座，全部作为普通包实现。

这是平台第一次产出游创创作者可以坐下来体验超过一个 demo 的东西。它不是 SillyTavern，不是纯对话运行时，不是 director —— 它是最小的、诚实演练每个底座原语的体验。

这个 seed 刻意不是 canonical game runtime。`official/playable-seed` 证明 package 路径；`official/composition-lab`、`official/asset-lab` 和 `official/projection-lab` 证明周边创作与检查循环。

带入此阶段的约束：

- 内核变更是最后手段。如果体验需要新原语，先重新设计体验。
- 实现该体验的官方包必须保持可被第三方包替换。
- Assistant 必须通过 `kernel.proposal.*` 提出变更，而非通过特权路径。
- Forge 必须能够仅使用公开协议检查、fork 和编辑体验。
- Conformance 随包一起增长：至少一个 hostile 用例证明第三方体验包可以到达相同的 surface。

## Phase H — Authoring & Composition Beta+（已完成）

目标：将当前的创作切片（`init-package`、`init-composition`、`composition check`、生成的 experience 模板）转化为此仓库外的人可以用来发布包的真实创作循环。

- 各 surface slot 的模板变体（`basic`、`experience`、`play-renderer`、`forge-panel`、`assistant-action`、`asset-editor`、`full-surface`）。
- 本地 fixture 与 reload tooling：`package check`、`package run-fixture`、`package reload` 与 generated package conformance。
- Composition descriptor v2 diagnostics，覆盖 optional packages、required capabilities、permission expectations、replacement candidates 与 compatibility notes。
- Forge authoring surface 改进：package/capability inventory、按 slot 分组的 surface descriptor inventory、composition diagnostics 与 manifest/template CLI guidance。
- 第三方 replacement proof：`examples/packages/thirdparty-playable-seed` 与 `examples/compositions/playable-seed-replacement` 证明官方包可替换，且没有 official priority。
- `docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md` durable walkthrough 更新。

## Phase I — 安全执行与 host hardening（后台）

作为后台工作推进，不是主角：

- 超出 network declarations 的更丰富资源策略覆盖，尤其是 filesystem 和 package-principal asset/projection 权限。
- 内容寻址 asset blob。
- 包拥有的 projection 执行。
- Package-principal subscribe 权限和更广的 stream transport parity。
- Hook handler 超时/错误审计。
- 持久 capability provider 选择策略。
- Conformance 中更广的传输层一致性覆盖。
- WASM 和 remote 包 entry 执行。

这些项目解除特定用例的阻塞。它们不阻塞 Agent Infrastructure Alpha，但所有 agent/model 包都必须使用现有 public protocol、permission、audit、redaction、streaming 和 proposal 路径。

## Phase J — Agent Infrastructure Alpha（已完成）

目标：让 Yggdrasil 能托管、约束、观察和替换 agent-like packages，同时保持 agent 语义在内核之外。

已交付：

- `docs/architecture/PI_INTEGRATION.md` 与 `integrations/pi` ledger 固定 pi 吸收边界。
- `sdk/typescript/ygg-agent-adapter` 把 Yggdrasil capabilities 通过公开协议映射为 pi-style tools；不访问私有 runtime。
- `--template agent-runtime` 生成 deterministic/no-network agent-like 包，发出 package-owned traces 和 approval-gated proposals。
- `official/pi-agent-runtime-lab` 是普通参考包；无特殊路由、无隐藏权限、无真实模型调用。
- `official/capability-tool-bridge-lab` 发现 capabilities、预览权限、强制显式 provider 选择，并只生成 `kernel.capability.invoke` / `kernel.capability.stream` plan，避免 confused deputy。
- Forge/Assist 通过 package-owned events、proposals、surfaces 和 public protocol 展示 agent traces、tool diagnostics 与 readiness badges。
- `examples/packages/thirdparty-agent-runtime` 与 `examples/compositions/agent-runtime-replacement` 证明官方 agent 包没有特权。
- `docs/guides/AGENT_PACKAGE_AUTHORING.md` 作为 durable 创作指南。

Phase J 非目标：

- 不做真实 model inference，除非专门 package 使用安全执行底座和显式 host policy。
- 不新增 kernel `agent`、`prompt`、`memory`、`turn` 或 `model` 方法。
- 不整体嵌入 `pi-coding-agent` 的产品假设。

## Phase K — Model Provider Integration Alpha（已完成）

目标：直接开始真实模型 provider 接入，但保持 Yggdrasil 方式：普通能力包、`secret_ref`、network allowlist、redacted audit、stream/cancel、fake/local conformance、manual live opt-in、无官方特权、无 kernel model ontology。

已交付：provider API 调研 ledger（M0）、`sdk/typescript/model-provider-adapter`（M1）、`official/model-provider-lab` no-network normalization（M2）、host outbound executor boundary（M3）、OpenAI/Anthropic/Gemini invoke adapters（M4）、OpenAI-compatible/OpenRouter/DeepSeek/xAI/Fireworks presets（M5）、streaming normalization（M6）、provider profile examples、durable guide 和 114 个 conformance 用例。

非目标：用户余额、计费、渠道后台、admin UI、托管平台代理 key、`kernel.model.*`、`kernel.prompt.*`、`kernel.chat.*`、`kernel.embedding.*`。

## Phase L — Live Model Calls Alpha（已完成）

目标：把 fake/local provider path 推进到真实 live calls，但仍通过普通能力包、host-owned secrets、public outbound boundary、redacted audit 和 opt-in live conformance 工作。

已交付：L0 live-call contract、L1 `EnvSecretResolver`、L2 `LiveHttpOutboundExecutor`（`reqwest + rustls`，默认关闭）、L3 public `kernel.outbound.execute`、L4 DeepSeek canary / secret header injection / loopback live HTTP、L5 OpenAI / Anthropic / Gemini live adapter shapes、L6 OpenRouter / DeepSeek / xAI / Fireworks quirks 与 sanitized fixtures、L7 durable docs cleanup。当前 conformance 145 个具名用例。

非目标：中转站、用户金额/计费系统、渠道后台、平台代理 key、默认联网 CI、provider 直接读 env、provider 直接 HTTP 绕过 host、`kernel.model.*`。

## Phase M — Creative Inference Capability Alpha（已完成）

目标：Yggdrasil 近期产品路径保持 cloud API first，但平台抽象不能 cloud API shaped。Cloud API adapter 只是普通包，不是 Ygg 的模型抽象。下一阶段要证明 transport-neutral inference capability seam、非 HTTP fake provider，以及 inference 参与 proposal/inspection/branch/fork 的创作运行时闭环。

- C0：API-first but not API-shaped ADR 与临时计划（已完成）。
- C1：transport-neutral inference capability contract（已完成；`sdk/typescript/inference-capability` + `docs/guides/INFERENCE_CAPABILITY_AUTHORING.md`）。
- C2：non-HTTP fake local provider proof（已完成；`official/inference-local-lab` + 5 个 conformance 用例）。
- C3：cloud adapter package reposition（已完成；`official/model-provider-lab` 是 cloud adapter，不是平台抽象）。
- C4：Ygg-native inference proposal vertical slice（已完成；`official/inference-playtest-lab` + 5 个 conformance 用例）。
- C5：durable docs cleanup（已完成；临时计划删除，持久内容收敛到 guide/status/next steps）。

非目标：本地大模型平台、权重/GPU/调度系统、继续扩 provider zoo、统一 chat schema、API gateway、`kernel.model.*`。

## Phase N — Agentic Forge Beta（已完成）

目标：把 Agent Infrastructure Alpha 从安全托管 proof 推进为 Yggdrasil-native creative agent runtime。Agentic Forge 的 agent 是普通 package 拥有的 creative process：它维护 run lifecycle、working state、plan graph 和 candidates；默认在 scratch branch 中探索；通过 candidate compare / proposal / inspection / approval / promote 与目标 branch 交互；tool 调用使用 scoped grants 和 audit；live inference 与 deterministic fallback 可替换；Forge UI 展示 run timeline、plan graph、scratch diff、candidate compare、tool/inference trace，而不是聊天记录。所有阶段（A–F）已完成。持久指南见 `docs/guides/AGENTIC_FORGE_PACKAGE_AUTHORING.md`。

已完成阶段：

- Phase A：package-owned run lifecycle / working state / plan graph。
- Phase B：branch-aware scratch branch / candidate / compare / promote proof。
- Phase C：inference-backed agent run with deterministic fallback。
- Phase D：tool bridge v2 scoped toolchain observation / risk / replay。
- Phase E：Forge Agent Workspace / Observability UI shell。
- Phase F：third-party replacement proof、hostile conformance、budget/deadline 契约、durable docs cleanup。

非目标：LangChain clone、chat shell、coding-agent clone、agent marketplace、always-on autonomous background agents、provider zoo、OpenAI-compatible agent endpoint、`kernel.agent.*` / `kernel.model.*` / `kernel.prompt.*` / `kernel.memory.*`。

## Experience Beta 0 — Thin Experience Runtime Contract（已完成）

目标：定义普通 package-owned experience 如何连续运行、暂停、恢复、checkpoint、fork，并被 Agentic Forge 修改。

已交付：
- `official/experience-runtime-lab` — experience 描述符、state projection、checkpoint、recovery 与 Play/Forge/Assist surface 绑定，全部作为普通能力。
- `sdk/typescript/experience-runtime` — 纯 TypeScript SDK，85 项自测断言。无依赖，无私有运行时。
- `--template experience-runtime` — 生成 deterministic/no-network subprocess，包含 contract/checkpoint/recovery 能力和 4 个 experience surface。
- Forge profile 自动加载 `official/experience-runtime-lab`。
- 7 个 conformance 用例，覆盖：describe_contract shape、checkpoint/recovery shape、no kernel experience namespace、template generation、bind_agent_run shape。
- 持久指南：[`docs/guides/EXPERIENCE_RUNTIME_AUTHORING.md`](../guides/EXPERIENCE_RUNTIME_AUTHORING.md)。

非目标：`kernel.experience.*`、`kernel.world.*`、`kernel.turn.*`。

## Experience Beta 1 — First Real Playable Vertical Slice（已完成）

目标：尽早做一个可以连续玩 20–30 分钟的 AI-native experience。它不是聊天壳、不是 Tavern clone、不是只有 prompt/response 的 demo。

已交付：

- `official/playable-creation-board` — package-owned playable creation board，包含 board/module/constraint/marker state，11 项能力、4 个 surface。
- record_player_action 产生 state_delta_asset_ref / projection_ref / sequence / provenance。
- request_change 输出 structured agent objective / allowed_change_kinds / risk / budget / bindable refs，不是聊天消息。
- bind_agent_run 输出与 agentic-forge 的 scoped binding。
- explain_provenance 输出因果链。
- checkpoint / recovery 对齐 experience-runtime-lab 形状。
- Raw-secret blocking。
- 第三方 agentic-forge 替换 composition 证明无 official priority。
- CLI demo `ygg playable-board-demo`。
- Forge profile 自动加载。
- 10 个 conformance 用例。

非目标：`kernel.experience.*`、`kernel.world.*`、聊天壳、assistant messages/conversation/prompt transcript。

## Experience Beta 2 — State + Asset Pipeline Alpha（已完成）

目标：让体验状态和生成资产真正可追踪、可比较、可恢复。

已交付：

- 稳定 content-addressed asset helper，使用 FNV-1a 64-bit hash（`fnv1a64:` 前缀，跨运行确定性，替代不稳定的 DefaultHasher）。
- 标准 Beta 2 元数据约定：`content_address`、`provenance`、`disclosure`、`source_refs`、`derived_refs`、`branch_ref`、`state_snapshot_ref`、`projection_ref`、`proposal_ref`、`inference_ref`、`large_output_policy`。
- `official/asset-lab` 扩展 `content_address` 能力（稳定内容地址 + 元数据约定）和 `provenance_graph` 能力（asset provenance graph 形状，含 source/derived/disclosure 元数据）。
- `official/projection-lab` 扩展 `state_snapshot` 能力（state snapshot asset 约定和 branch-aware diff preview 形状）。
- `official/playable-creation-board` 扩展 `preview_state_diff`（branch-aware state diff preview，含 before/after content addresses）和 `describe_asset_provenance`（asset provenance graph，含 source/derived/disclosure 元数据），共 13 个能力。
- Asset provenance graph：source refs、derived refs、package/provider/inference refs、AI-generated/live-generated/disclosure 元数据。
- State snapshot asset 约定：checkpoint/recovery/replay hints。
- State/asset diff preview：branch-aware、projection-backed、package-owned。
- 大输出处理：tool/model 大输出通过 asset refs（已有 capability-tool-bridge-lab 推荐已强化）。
- Package-scoped asset permission proof：origin_package_id 强制，跨包 spoof fail-closed。
- Beta 2 能力阻止 raw secret。
- 新增 9 个 conformance 用例（总计 206 个）。

非目标：完整媒体编辑器、统一 media schema、内核世界状态模型——均已遵守。

## Experience Beta 3 — Experience Observability（已完成）

目标：让用户和创作者看懂一次 experience 中发生了什么、为什么失败、成本/延迟在哪里。

已交付：

- `official/experience-observability-lab`：包拥有的体验可观测性——session health、package health、agent run health、proposal causal chain、failure breadcrumbs、cost/latency summary、guardrail/audit summary。8 项能力、3 个 surface（forge_panel、assistant_action、home_card）。Deterministic、no-network、no inference。全部从协议可见引用派生，不读 SQLite 或 runtime internals。
- Runtime inproc handler：deterministic/no-network/no inference，输出 public protocol shapes（session_health、package_health、agent_run_health、proposal_causal_chain、failure_breadcrumbs、cost_latency_summary、guardrail_audit_summary）。不得输出 chat/message/prompt/world/scene/turn/memory 等形态。
- 与 playable-creation-board 关联：新增 `summarize_experience_health` 能力，输出含 observability 交叉引用。
- Conformance：10 个具名用例（contract/session_health/package_health/agent_run_health/proposal_causality/cost_latency/failure_breadcrumbs/guardrail_audit/no_forbidden_namespace/no_raw_secrets）。
- Profile autoload：forge-alpha.yaml 自动加载新包。
- Web Forge Experience Observability panels：Experience Health、Causal Chain、Failure Breadcrumbs、Cost/Latency、Asset Provenance、Guardrail/Audit Summary。只使用 public protocol types，不读 SQLite 或 runtime internals。
- 不新增 kernel.observability.* 或 kernel.experience.*；不读取 SQLite/runtime internals；不做实时监控后台或特权 Studio。

非目标：完整 APM、SaaS monitoring backend、特权 Studio。

## Experience Beta 4 — Memory / Knowledge Package Alpha（已完成）

目标：普通包形式的长期记忆与知识，不进入 kernel。

已交付：

- `official/memory-lab` — 包拥有的长期记忆与知识实验室，提供 describe_memory_contract / record_memory / retrieve_memory / trace_retrieval / draft_memory_update / apply_memory_correction / draft_forget_redaction / branch_memory_view / explain_memory_provenance 共 9 项能力与 3 个 surface（forge_panel、assistant_action、home_card）。Deterministic、no-network、no inference。Raw-secret blocking。Proposal-gated update（draft_memory_update 只产 proposal/update draft，不直接改持久状态）。Forget/redaction 输出 redaction plan，不直接删除。Branch-aware view。Provenance chain 含 content_address。无 kernel.memory.* 命名空间。
- `official/playable-creation-board` 新增 `memory_refs` / `knowledge_refs` / `retrieve_context_plan` 可选交叉引用。Board 不依赖 memory-lab 才能运行。
- 第三方替换证明：`thirdparty/memory-lab` manifest + `examples/compositions/memory-lab-replacement/` composition 证明无 official 优先。
- Conformance：10 个具名用例，覆盖 contract、record/retrieve/trace、proposal-gated update、correction、forget/redaction、branch-aware view、no forbidden namespace、no raw secrets。
- 持久指南：[`docs/guides/MEMORY_PACKAGE_AUTHORING.md`](../guides/MEMORY_PACKAGE_AUTHORING.md)。

非目标：`kernel.memory.*`、官方唯一 RAG、聊天记忆系统。

## Experience Beta 5 — Creator Loop Beta

目标：一个新创作者不读源码，只靠 docs、template、Forge，一天内做出可玩的 package。

已交付（核心 / 非 Web）：

- `--template playable-board` 和 `--template playable-experience` 模板：deterministic/no-network playable package 骨架，最接近 `official/playable-creation-board` 形态。
- 面向创作者的 `package check` 诊断：experience surface coverage、缺失 checkpoint/recovery capability 警告、危险 permissions 警告、non-deterministic path hint。
- 面向创作者的 `package run-fixture` 诊断：capability 失败时提供针对性修复提示。
- 面向创作者的 `package reload` 诊断：状态不可用或 degraded 时发出警告。
- Experience package set 的 `composition check` 诊断：experience surface coverage、replacement candidates、checkpoint/recovery coverage、memory/observability hints。
- Walkthrough §8：template-to-playable 路径，记录在 `docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md` / `.en.md`。
- 9 个新 conformance 用例（共 235 个）。

剩余（UI）：Forge authoring workflow panels。

非目标：marketplace、creator monetization。

## Experience Beta 6 — Sharing / Distribution Alpha（已完成）

目标：先支持可分享、可复现、可导入，再考虑市场。

已交付：

- `official/sharing-lab` — 包拥有的分享与分发实验室，提供 describe_sharing_contract / export_composition_bundle / import_composition_bundle / create_branch_session_bundle / create_package_set_lockfile / compatibility_report / ai_disclosure_bundle / read_only_share_manifest / async_fork_share_plan 共 9 项能力与 3 个 surface（forge_panel、assistant_action、home_card）。Deterministic、no-network、no marketplace、no billing、no signing network。Raw-secret blocking + marketplace/billing/signing field blocking。无 kernel.sharing/marketplace/billing/distribution 命名空间。
- 示例 artifacts：`examples/bundles/playable-creation-board-composition-bundle/`（bundle.json、branch-session-bundle.json、read-only-share-manifest.json、async-fork-share-plan.json）。
- 持久指南：[`docs/guides/SHARING_DISTRIBUTION.md`](../guides/SHARING_DISTRIBUTION.md)。
- 临时阶段计划已删除，结果收敛到 ALPHA_STATUS/NEXT_STEPS/guide/conformance matrix。
- Conformance：10 个具名用例，覆盖 contract shape、export/import bundle、lockfile、compatibility report、AI disclosure、read-only share、async fork、no marketplace/no raw secrets。

非目标：marketplace、package signing network、dependency resolver economy、hosted billing。

## Performance & Code Health Beta（已完成）

目标：在进入第一个平台产品前建立 baseline、缩短 conformance 反馈环、优化 SQLite event replay、收敛 Web 全量渲染，并控制 runtime/CLI/Web 文件增长。

已交付：

- **P0 — Baseline & Measurement**：`ygg perf baseline` CLI，覆盖 inproc invoke、official capability invoke、subprocess echo、event store append/list/range、1k/10k/100k event scale、composition check、profile load；JSON stdout 可脚本解析，`--iterations 0` fail-closed。参考 [`docs/performance/BASELINE.md`](../performance/BASELINE.md)。
- **P1 — Conformance Feedback Loop**：`--list`、`--case <pattern>`、`--tag <tag>`、`--fail-fast`、`--slowest <N>`，per-case duration 与 slowest report；245 cases 保持默认全量运行。参考 [`docs/performance/CONFORMANCE_FEEDBACK.md`](../performance/CONFORMANCE_FEEDBACK.md)。
- **P2 — Low-risk Structural Split**：protocol dispatch domain helpers、provider-indexed inproc dispatch、共享 safety helper、composition/package diagnostics set/index；保持 public protocol shape、package-aware routing 和 no official priority。
- **P3 — Event Store & Replay Optimization**：`EventStore::append_with_sequence` 原子追加，SQLite/in-memory 并发同 session sequence 不重复；`list_kind_prefix` / `list_session_kind_prefix` 查询 pushdown；SQLite `kind` 与 `session+kind+sequence` 索引；permission/outbound audit 避免常规 `list_all()` 全量过滤。
- **P4 — Web Render & UI Organization**：16ms render scheduler、bounded JSON preview、Forge events/proposals/assets/projections/surfaces 显示 cap、payload preview details、pure TS Forge render diagnostics helper。
- **P5 — Durable cleanup**：删除临时计划，新增 [`docs/performance/PERFORMANCE_AND_CODE_HEALTH.md`](../performance/PERFORMANCE_AND_CODE_HEALTH.md)，并把 README、ALPHA_STATUS、NEXT_STEPS、CONFORMANCE_MATRIX 收敛到持久指南。

红线：不做官方包 fast path；不绕过 permission/hook/schema/redaction/audit；Web 不读 SQLite/runtime internals；不新增 kernel content/product namespace；不做无证据的 macro/codegen/RawValue/arena 重写。


## External Project Operating Plane Alpha（E1–E3 已完成；E4–E6 执行中）

目标：让 Yggdrasil 能围绕未适配的 git/npm/local/archive 项目提供静态 intake、workspace plan、风险摘要、受控 workspace、项目聚合 UI 和 adapter/wrapper 生成，而不是要求所有项目先成为 Ygg package。

阶段：

- **E0 — Plan, Research, ADR**：写入双语临时计划，保存外部证据，切换当前主线。
- **E1 — Project Intake Lab**（已完成）：`official/project-intake-lab`，7 项能力（describe_intake_contract / inspect_external_project_ref / detect_project_stack_from_metadata / draft_workspace_plan / draft_security_risk_summary / list_candidate_entrypoints / draft_adapter_plan）、3 个 surface（forge_panel / assistant_action / home_card）、source classification（git/npm/local/archive/unknown）、stack detection（node/rust/python/static/unknown）、npm lifecycle risk detection（preinstall/install/postinstall/prepare/prepublish 标注 executes_code/requires_approval）、unsafe local path rejection、plan-only workspace/adapter plans、raw-secret blocking、no execution、no network、no filesystem、无 project/workspace/git/npm/deploy/ide kernel protocol namespace。8 个 conformance cases。
- **E2 — Workspace Action Policy Boundary**（已完成）：`official/workspace-lab`，5 项能力（describe_workspace_contract / draft_workspace_creation / explain_required_permissions / request_workspace_action / summarize_workspace_audit）、3 个 surface（forge_panel / assistant_action / home_card）、10 项 action taxonomy（clone_project / read_metadata / install_dependencies / run_command / run_tests / stop_process / read_logs / discover_entrypoints / write_patch / deploy_plan），含 risk_level / requires_approval / executes_code / network_required / filesystem_write_required 标注，deny-by-default fake executor（executor_invoked=false、execution_performed=false、proposal_required=true），approval_token 不生效，policy/action mismatch fail-closed，未知 action fail-closed，raw-secret 阻断，audit redaction（不含 raw env/logs/commands/secrets），no execution、no network、no filesystem、no shell、无 project/workspace/git/npm/deploy/ide kernel 协议 namespace。7 个 conformance cases。
- **E3 — Managed Workspace Deterministic Proof**（已完成）：`official/workspace-lab` 扩展 7 项 deterministic fixture managed workspace 能力（create_fixture_workspace / inspect_workspace / read_workspace_metadata / plan_workspace_run / record_fixture_process_result / discover_workspace_entrypoints / draft_workspace_patch）。Deterministic fixture workspace descriptor 含 managed_workspace_kind="fixture"、execution_performed=false、workspace_created_in_host=false、真实创建需要 approval/policy/executor。无文件系统、无进程、无网络、无 shell。Patch 仅生成 proposal，含 unsafe path 阻断和 raw secret 阻断。Entrypoint discovery 来自 stack_hint/metadata/scripts。7 个新增 conformance 用例（总计 267）。
- **E4 — Web Project Aggregation UI**：Home/Forge 显示 external projects、workspaces、risk、entrypoints、logs、adapter candidates，public protocol-only。
- **E5 — Adapter / Wrapper Generation Proof**：`official/adapter-lab` 从 fixture workspace 生成 ordinary subprocess adapter package。
- **E6 — Durable cleanup**：删除临时计划，收敛到外部项目操作平面指南、ALPHA_STATUS、NEXT_STEPS、CONFORMANCE_MATRIX。

红线：external project 不是 package；managed workspace 不是 kernel object；adapter/wrapper 才是 package；不新增 `kernel.project.*` / `kernel.workspace.*` / `kernel.git.*` / `kernel.npm.*` / `kernel.deploy.*`；危险动作必须 policy/proposal/audit gated。

## 内核范围内的无限期延后

这些仍是内核的非目标。它们可能以未来包的形式存在。

- SillyTavern 兼容 —— 见 `docs/tavern/TAVERN_COMPAT.md`。
- pi 产品嵌入 —— 见 `docs/architecture/PI_INTEGRATION.md`。Agent 基础设施只能作为普通 package/SDK 工作推进。
- 外部游戏引擎桥接（UE5/Godot/Unity，web 客户端）。
- 特权内置 Studio、绕过公开协议的 UI、或由 kernel 拥有的官方检查器。公开协议客户端和普通 package-contributed surfaces 可以继续演进。
- 内核中的记忆模型、世界模拟、director、提示词渲染和模型 provider 抽象。Agent loops、production-grade live model calls 和 model providers 只能作为普通包存在。
- 市场、包签名、依赖解析器（本地分享 proof 已完成；见 [`docs/guides/SHARING_DISTRIBUTION.md`](../guides/SHARING_DISTRIBUTION.md)）。

## 如何阅读这份列表

Phase F、Phase G 的 seed 形态、Creative Capability Kit Alpha、Model Connectivity Kit Alpha、Code Health Split Alpha、Runtime Split Alpha、Authoring & Composition Beta+、Secure Execution Substrate Alpha、Optional Text Engine Alpha、Agent Infrastructure Alpha、Model Provider Integration Alpha、Live Model Calls Alpha、Creative Inference Capability Alpha、Agentic Forge Beta、Experience Beta 0、Experience Beta 1、Experience Beta 2、Experience Beta 3、Experience Beta 4、Experience Beta 5、Experience Beta 6 和 Performance & Code Health Beta 已完成。所有后续阶段都以 charter 纪律评分：无内容形态泄漏到内核，无官方特权通过任何路径泄漏，所有 package/UI 行为都使用公开协议边界，并且新增 substrate 必须服务真实 playable experience 的压力。
