# 后续步骤

> [English](./NEXT_STEPS.en.md) · [中文](./NEXT_STEPS.md)

平台基础已经就位。Yggdrasil 现在拥有内容无关的内核、基于 manifest 的包、真正的 `rust_inproc` 和 subprocess 执行、权限/principal 系统、hook fabric 切片、surface 贡献、proposal/approval lifecycle、asset/branch/projection 底座、安全执行原语、官方平台包、assistant 包、`official/playable-seed`、空白游创循环，以及走公开协议的 Home/Play、Forge、Assist 和受限文字界面 proof 的 Web shell。

Agent Infrastructure Alpha、Model Provider Integration Alpha、Live Model Calls Alpha 与 Creative Inference Capability Alpha 已完成。Yggdrasil 现在可以用普通能力包描述、验证、归一化并 fake/local 调用 OpenAI、Anthropic、Gemini、OpenAI-compatible、OpenRouter、DeepSeek、xAI、Fireworks 等 provider API 差异；也具备 host-owned `secret_ref:env:*`、public `kernel.outbound.execute`、LiveHttpOutboundExecutor、redacted audit、live loopback provider shapes、transport-neutral inference seam 与 inference→proposal proof。默认 conformance 不依赖公网；手动 live smoke 必须显式 opt-in。当前主线是 **Agentic Forge Beta**：把 agent 从 lab/proof 推进为 package-owned、branch-aware、tool-safe、inference-backed、deterministically testable 的 creative agent runtime。

## 当前位置

- Platform Foundation Alpha：已完成。
- Play/Forge Surface Contract Beta：已完成。
- First Real Capability Package Track：seed 已完成（`composition-lab`、`asset-lab`、`projection-lab`、`playable-seed`；55 个 conformance 用例）。
- Platform Host Alpha：已实现切片完成；剩余项目（streaming 分发、hook 超时审计、持久 provider 策略、更广的传输层一致性、更丰富的 SDK 打包）在下方 Phase I 中追踪。
- Code Health Split Alpha：已完成；CLI commands/templates/conformance、runtime domain behavior、protocol dispatch 与 runtime official in-process handlers 已按领域拆分。
- Authoring & Composition Beta+：已完成；生成 package templates、fixture/reload tooling、composition v2 diagnostics、Forge authoring panels 与第三方 playable replacement proof 已就位。
- Secure Execution Substrate：Alpha 切片已完成。持久 grants、`secret_ref`、host resolver placeholder、raw-secret blocking、网络权限声明、outbound audit/redaction、通用 streaming/cancel 生命周期、secure-execution TypeScript helpers、networked/streaming templates，以及 no-network model/agent readiness examples 已就位。
- Text Surface Proof：Phase T1/T2/T3/T4/T5 已完成。`integrations/pretext` 记录 Pretext 参考边界，Assistant Drawer 中已有基于 `clients/web/src/text-layout` 的受限 mock streaming text proof，且没有 kernel/protocol/package 变更。`sdk/typescript/text-surface` 提供纯 TypeScript 前端 SDK 供第三方 UI 使用。字体加载、缓存诊断和自测模块已就位。
- Agent Infrastructure Alpha：已完成；`integrations/pi` ledger、`sdk/typescript/ygg-agent-adapter`、`--template agent-runtime`、`official/pi-agent-runtime-lab`、`official/capability-tool-bridge-lab`、Forge/Assist Agent Observability、`thirdparty/agent-runtime` replacement proof 和 [`docs/guides/AGENT_PACKAGE_AUTHORING.md`](../guides/AGENT_PACKAGE_AUTHORING.md) 已就位。
- Model Provider Integration Alpha：已完成；`integrations/model-providers` research ledger、`sdk/typescript/model-provider-adapter`、`official/model-provider-lab`、provider profile examples 和 [`docs/guides/MODEL_PROVIDER_INTEGRATION.md`](../guides/MODEL_PROVIDER_INTEGRATION.md) 已就位。
- Live Model Calls Alpha：已完成；成果已收敛进 [`docs/guides/MODEL_PROVIDER_INTEGRATION.md`](../guides/MODEL_PROVIDER_INTEGRATION.md)、[`docs/ALPHA_STATUS.md`](../ALPHA_STATUS.md) 和 conformance matrix。
- Creative Inference Capability Alpha：已完成；`sdk/typescript/inference-capability` transport-neutral envelope/stream/error/manifest helpers、[`docs/guides/INFERENCE_CAPABILITY_AUTHORING.md`](../guides/INFERENCE_CAPABILITY_AUTHORING.md)、`official/inference-local-lab` deterministic non-HTTP fake inference provider proof、`official/model-provider-lab` cloud API adapter 降级定位、`official/inference-playtest-lab` Ygg-native inference proposal vertical slice 均已就位。Conformance 包含 155 个具名用例。
- Agentic Forge Beta Phase A：已完成；`official/agentic-forge-lab` 提供 describe_contract/start_run/inspect_run/cancel_run/summarize_run/export_plan_graph 能力，`sdk/typescript/agentic-forge` TS SDK，5 个 conformance 用例。Conformance 包含 160 个具名用例。
- Agentic Forge Beta Phase B：已完成；扩展 `official/agentic-forge-lab` 增加 create_candidate/compare_candidate/draft_promote_proposal/archive_candidate/explain_branch_policy 能力；branch-aware scratch branch intent/metadata；candidate artifacts 含 stale 检测；proposal draft 不直接修改 target branch；stale target revision 不匹配时阻止 promote；5 个 conformance 用例。Conformance 包含 165 个具名用例。
- Agentic Forge Beta Phase C：已完成；扩展 `official/agentic-forge-lab` 增加 run_inference_node/replay_inference_node/validate_inference_output/explain_inference_failure 能力；8 个显式 plan node kind；inference provider（deterministic/recorded/cloud_adapter_plan/local_fake）；cloud_adapter_plan 返回 needs_host_policy 且不执行网络；replay 指纹不匹配时标记而非静默通过；inference output action allowlist 与 forbidden actions；9 项 failure taxonomy 含 typed recovery hint；5 个 conformance 用例。Conformance 包含 170 个具名用例。下一阶段为 Phase D。

详见 `docs/ALPHA_STATUS.md` 获取详细快照。

## Phase F — Foundation Alpha 收敛（已完成）

目标：停止扩大表面积。打磨粗糙边缘，锁定契约，让现有基础便于 demo、文档和扩展。

- 跨 `README.md`、`README.md` 和文档树刷新文档。
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

## Phase N — Agentic Forge Beta（执行中）

目标：把 Agent Infrastructure Alpha 从安全托管 proof 推进为 Yggdrasil-native creative agent runtime。Agentic Forge 的 agent 是普通 package 拥有的 creative process：它维护 run lifecycle、working state、plan graph 和 candidates；默认在 scratch branch 中探索；通过 candidate compare / proposal / inspection / approval / promote 与目标 branch 交互；tool 调用使用 scoped grants 和 audit；live inference 与 deterministic fallback 可替换；Forge UI 展示 run timeline、plan graph、scratch diff、candidate compare、tool/inference trace，而不是聊天记录。

阶段见 [`AGENTIC_FORGE_BETA.md`](AGENTIC_FORGE_BETA.md)：

- Phase 0：计划与边界锁定（执行中）。
- Phase A：package-owned run lifecycle / working state / plan graph。
- Phase B：branch-aware scratch branch / candidate / compare / promote proof。
- Phase C：inference-backed agent run with deterministic fallback。
- Phase D：tool bridge v2 scoped toolchain observation / risk / replay。
- Phase E：Forge Agent Workspace / Observability UI shell。
- Phase F：third-party replacement proof、hostile conformance、durable docs cleanup。

非目标：LangChain clone、chat shell、coding-agent clone、agent marketplace、always-on autonomous background agents、provider zoo、OpenAI-compatible agent endpoint、`kernel.agent.*` / `kernel.model.*` / `kernel.prompt.*` / `kernel.memory.*`。

## 内核范围内的无限期延后

这些仍是内核的非目标。它们可能以未来包的形式存在。

- SillyTavern 兼容 —— 见 `docs/tavern/TAVERN_COMPAT.md`。
- pi 产品嵌入 —— 见 `docs/architecture/PI_INTEGRATION.md`。Agent 基础设施只能作为普通 package/SDK 工作推进。
- 外部游戏引擎桥接（UE5/Godot/Unity，web 客户端）。
- 任何超出公开协议 Web shell 骨架的 UI shell、检查器或 studio。
- 内核中的记忆模型、世界模拟、director、提示词渲染和模型 provider 抽象。Agent loops、production-grade live model calls 和 model providers 只能作为普通包存在。
- 市场、包签名、依赖解析器。

## 如何阅读这份列表

Phase F、Phase G 的 seed 形态、Creative Capability Kit Alpha、Model Connectivity Kit Alpha、Code Health Split Alpha、Runtime Split Alpha、Authoring & Composition Beta+、Secure Execution Substrate Alpha、Optional Text Engine Alpha、Agent Infrastructure Alpha、Model Provider Integration Alpha、Live Model Calls Alpha 和 Creative Inference Capability Alpha 已完成。Agentic Forge Beta 正在执行。所有后续阶段都以 charter 纪律评分：无内容形态泄漏到内核，无官方特权通过任何路径泄漏，所有 package/UI 行为都使用公开协议边界。
