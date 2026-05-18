# Alpha 状态

> [English](./ALPHA_STATUS.en.md) · [中文](./ALPHA_STATUS.md)

这是 Yggdrasil 当前状态的实时快照。每当一个里程碑关闭时更新。它不是愿景：下面每一行都有代码和 conformance 支撑（或被明确标注为 partial/deferred）。

长期架构和产品立场见 `docs/CHARTER.md`、`docs/architecture/VISION.md` 和 `docs/product/PLAY_CREATION_MODEL.md`。后续方向见 `docs/roadmap/NEXT_STEPS.md`。

## 概要

- **阶段：** Platform Foundation Alpha + Play/Forge Surface Contract Beta + Secure Execution Substrate Phase S1/S2/S3/S4 + Text Surface Proof Phase T1/T2/T3/T4/T5。
- **Conformance：** 104 个具名 CLI 用例，加上 crate 和 service 单元测试。
- **Charter 纪律：** 内核内容无关，官方包无特权，仅公开协议，包跨入口形式平等，trusted paths 阻止 raw secret，使用 secret_ref 引用，permission grants 可重新水化，网络权限强制执行并带 outbound audit/redaction，通用 streaming 与 cancellation lifecycle，SDK secure-execution helpers，networked/streaming 包模板，no-network readiness proof。
- **代码健康：** CLI commands/templates/conformance、runtime domain behavior、protocol dispatch 与 runtime official in-process handlers 已按领域拆分，不再继续堆进巨型单文件。
- **下一阶段：** Agent Infrastructure Alpha（J6 完成）。

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
- **Host secret resolver placeholder**：`HostSecretResolver` trait 和 deny-all resolver 已存在，用于未来 host-level secret store。当前不做生产级 vault 或 provider-specific key handling。
- **Raw-secret blocking**：Proposal operations/expected effects 与 asset metadata 会被保守扫描；明显 raw API keys、token/password fields 会被拒绝。Asset content 和普通 prose 字段不扫描，以避免误伤用户内容。
- **网络权限声明**：Manifest `permissions.network` 同时支持扁平 `hosts`（向后兼容）和结构化 `declarations`（含 `host`、`methods`、`purpose`）。Runtime 策略检查器根据声明的条目匹配出站请求。无网络声明的包被拒绝出站访问。官方包无绕过。
- **Outbound audit/redaction records**：`OutboundAuditRecord` 记录 principal、package_id、capability_id、destination_host、method、purpose、redaction_state、secret_refs_used、usage/cost 占位符、status/error。Raw body/header/prompt/response 不会被保存——仅记录 `secret_ref` 标识符和 `redaction_state` 枚举（`not_captured`、`redacted`、`policy_ref`、`unsafe_blocked`、`explicitly_approved`）。默认为 `redacted`。
- **网络策略检查器**：`check_network_policy` 纯函数和 `check_and_audit_outbound` runtime 方法。支持精确 host 匹配、通配符前缀（`*.example.com`）、method 白名单（空 = 任意）和扁平 `hosts` 向后兼容。被拒绝的请求产生 `kernel/outbound.denied` 审计事件；被允许的请求产生 `kernel/outbound.request` 事件。
- **协议方法**：`kernel.outbound.audit` 列出给定包的出站审计事件。
- **Streaming invocation registry**：内存中的 `StreamRegistry` 追踪进行中的 streaming capability 调用，支持 start/append/end/cancel/timeout 生命周期。`StreamFrameEnvelope` 定义通用内容无关的 stream frame 类型（start/chunk/progress/end/error/cancelled/timeout），包含 invocation_id、stream_id、sequence、redaction_state 和 timestamp/metadata。不包含 model/prompt/agent 语义。
- **Streaming capability 生命周期**：`kernel.capability.stream` 启动 streaming invocation（验证 descriptor 中 `streaming=true`），`kernel.capability.cancel` 取消进行中的 invocation。Runtime 方法按序发出 kernel 事件：`kernel/stream.started`、`kernel/stream.chunk`、`kernel/stream.progress`、`kernel/stream.ended`、`kernel/stream.error`、`kernel/stream.cancelled`、`kernel/stream.timeout`。Cancel 和 timeout 阻断后续 chunk。非 streaming 能力（descriptor `streaming=false`）被拒绝。
- **Streaming invocation 记录**：`StreamInvocationRecord` 追踪 invocation_id、stream_id、capability_id、provider_package_id、session_id、状态（active/ended/error/cancelled/timeout）、frame_count、时间戳和 metadata。终态阻断后续 frame 追加。
- **Secure-execution TypeScript helpers**（`sdk/typescript/secure-execution/index.ts`）：`secretRef()`/`isValidSecretRef()`/`looksLikeRawSecret()`/`isSecretFieldName()` 用于 secret reference 构造和验证。`NetworkDeclaration` 类用于构建 manifest 兼容的网络权限条目，支持 host/method 匹配。`OutboundAuditHelper` 类用于构建审计安全的出站请求 payload，拒绝 raw secrets，仅包含 `secret_ref` 标识符。`StreamFrameClient` 类用于构建 faux stream frame envelope，支持完整生命周期（start/chunk/progress/end/error/cancel/timeout）。所有 helper 只包装公开协议和类型——无私有内部、无协议绕过。
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
- `official/model-routing-lab` —— no-inference consumer-slot binding、route planning、fallback planning 与 params normalization。
- `official/assistant-lab` —— assistant-action 能力，返回需要审批的 proposal。
- `official/pi-agent-runtime-lab` —— 参考代理运行时包，deterministic no-network run plan、trace summary、proposal draft 与 echo。
- `official/capability-tool-bridge-lab` —— 发现 capabilities、预览权限、显式 provider 选择、通过 kernel.capability.invoke/stream 的 invocation/streaming plan，不偏袒 official provider。
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

- `cargo run -p ygg-cli -- conformance` 运行 104 个具名 CLI 用例，覆盖：session、事件、包、能力、hook、schema、principal、权限、subprocess 执行、host 传输、surface、proposal、官方包、composition-lab（含 v2 诊断与 compat-report）、asset-lab、projection-lab、persona-lab、knowledge-lab、context-lab、text-transform-lab、model-connector-lab、model-routing-lab、**pi-agent-runtime-lab（no-inference/no-network、approval-gated proposal、surfaces 可发现、provider_package_id 匹配）**、**capability-tool-bridge-lab（ambiguous provider 标记 rejected、explicit third-party provider 可用、official 不优先、missing provider rejected、denied preview 报告 missing permission、raw secret unsafe_blocked、surfaces 可发现）**、in-process package fallback hardening、playable-seed、空白游创循环、asset/branch/projection 底座、生成包创作（basic、experience、assistant-action、asset-editor、full-surface、**networked**、**streaming**、**agent-runtime** 模板）、composition descriptor（v1 与 v2）、package check 诊断、package reload 冒烟测试、第三方 playable-seed 替换证明（surface 可发现性、能力调用、歧义路由无官方优先、composition check）、**第三方 agent-runtime 替换证明（assistant_action/forge_panel/home_card surfaces 可发现、no-inference/no-network、approval-gated proposal、provenance 匹配、composition check 通过 official 仅 replacement_candidate）**、**permission grant 通过 SQLite 重新水化**、**secret_ref validation**、**proposals 和 asset metadata 中的 raw-secret blocking**、**official-package no-secret-bypass**、**无网络权限的包被拒绝出站并产生 outbound.denied 审计**、**allowlisted host+method 允许并记录 redacted audit**、**host/method 不匹配拒绝**、**official-package 无 network bypass**、**审计记录不包含 raw secret/body，只包含 secret_ref 和 redaction_state**，**网络策略检查器纯函数测试**，**streaming/cancellation 生命周期（normal end、cancel、timeout、error、non-streaming 拒绝、无 model/agent 方法、protocol dispatch）**，**生成的 networked 模板 conformance（网络声明、无 raw secrets）**，**生成的 streaming 模板 conformance（streaming capability）**，**faux-model-readiness manifest 结构（网络声明、secret_ref、streaming、无 raw secrets）**，以及 **faux-agent-readiness manifest 结构（无网络权限、streaming、proposal/trace 模式、无 raw secrets）**，以及 **生成的 agent-runtime 模板 conformance（4 capabilities、streaming run、assistant_action + forge_panel surfaces、no-network、无 raw secrets、无 kernel.agent/model/prompt/memory/turn 文本）**。
- 加上 `cargo test --workspace` 下的 crate 和 service 单元测试。
- `tsc -p clients/web/tsconfig.json --noEmit` 检查 web shell。

## 部分实现

- 能力调用 lifecycle 事件（`kernel/capability.invoked|completed|failed`）已在契约中预留；尚未发出。
- Streaming 协议分发自 partial（stream start/cancel 生命周期可用；真实网络 streaming 延后）。
- Package-principal 的 `event.subscribe` 权限。
- Hook handler 超时/错误审计，面向包拥有的 handler。
- 持久化的能力 provider 选择策略（超越单次调用显式选择）。
- 更丰富的资源策略覆盖（filesystem 强制矩阵）—— Phase S4+ 目标。
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
- pi 集成（见 `docs/architecture/PI_INTEGRATION.md`）。
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
- `docs/roadmap/NEXT_STEPS.md` —— 当前与下一阶段。
