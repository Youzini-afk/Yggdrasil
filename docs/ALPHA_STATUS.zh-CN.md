# Alpha 状态

> [English](./ALPHA_STATUS.md) · [中文](./ALPHA_STATUS.zh-CN.md)

这是 Yggdrasil 当前状态的实时快照。每当一个里程碑关闭时更新。它不是愿景：下面每一行都有代码和 conformance 支撑（或被明确标注为 partial/deferred）。

长期架构和产品立场见 `docs/CHARTER.md`、`docs/architecture/VISION.md` 和 `docs/product/PLAY_CREATION_MODEL.md`。后续方向见 `docs/roadmap/NEXT_STEPS.md`。

## 概要

- **阶段：** Platform Foundation Alpha + Play/Forge Surface Contract Beta。
- **Conformance：** 68 个具名 CLI 用例，加上 crate 和 service 单元测试。
- **Charter 纪律：** 内核内容无关，官方包无特权，仅公开协议，包跨入口形式平等。
- **代码健康：** CLI commands/templates/conformance、runtime domain behavior、protocol dispatch 与 runtime official in-process handlers 已按领域拆分，不再继续堆进巨型单文件。
- **下一阶段：** Authoring & Composition Beta+（见 `docs/roadmap/NEXT_STEPS.md`）。

## 已实现

### 内核

- 内容无关的 session、只追加不透明事件、manifest 驱动的包、能力 fabric、hook fabric 切片、surface contributions、proposal lifecycle、asset/branch/projection 底座。
- SQLite 支撑的持久事件日志，每 session 单调递增序号，可重新水化的底座。
- JSON Schema 子集用于能力输入/输出和包声明的 event payload。
- Principal：`host_admin`、`host_dev`、`package`、`human`、`assistant`、`anonymous`。human 和 assistant principal 的作用域授权。
- 权限审计事件：`kernel/permission.granted`、`kernel/permission.revoked`、`kernel/permission.denied`。
- 包 lifecycle 事件：`kernel/package.loading|starting|ready|stopping|stopped|loaded|unloaded|degraded|log`。
- Proposal lifecycle 事件：`kernel/proposal.created|approved|rejected|applied|failed`。

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
- `official/composition-lab` —— composition 验证、launch-plan、permission-preview 与 surface-graph 辅助。
- `official/asset-lab` —— 通用 asset preview、diff、export 与 import-plan 辅助。
- `official/projection-lab` —— projection describe、diff、rebuild-plan 与 source-event 辅助。
- `official/persona-lab` —— persona profile import、normalization、rendering 与 compatibility diagnostics。
- `official/knowledge-lab` —— structured knowledge collection normalization、matching、injection planning 与 diagnostics。
- `official/context-lab` —— bounded context block assembly、layer inspection、budget planning 与 template rendering。
- `official/text-transform-lab` —— deterministic text transform import、validation、preview、pipeline explanation 与 diagnostics。
- `official/model-connector-lab` —— no-network provider family metadata、profile validation、secret masking、discovery plans 与 compatibility reports。
- `official/model-routing-lab` —— no-inference consumer-slot binding、route planning、fallback planning 与 params normalization。
- `official/assistant-lab` —— assistant-action 能力，返回需要审批的 proposal。
- `official/blank-experience` —— 最小体验，被 `ygg play-create-demo` 用来跑通游创循环。
- `official/playable-seed` —— 带有 entry/play/Forge/assistant surfaces 的 reference playable package。

Forge profile（`profiles/forge-alpha.yaml`）自动加载这些包以及示例 fixture 包。

### Web shell（`clients/web`）

- 骨架化的 Home/Play、Forge 和 Assist surface，走公开协议。
- Home 发现 `experience_entry` surface，通过包声明的 launch 能力启动 session，支持 session fork。
- Forge 检查事件、能力、asset、projection、proposal 和 Forge-panel surface contributions，提供 proposal 的 approve/apply 控制。
- 没有官方包硬编码。Shell 和其他客户端一样是公开协议客户端。

### 创作

- `ygg init-package` 生成 Python 或 TypeScript subprocess 包骨架。TypeScript 变体使用 `sdk/typescript/subprocess` 下的 SDK runtime。
- `--template basic|experience|play-renderer|forge-panel|assistant-action|asset-editor|full-surface` 控制生成的 surface 描述符。未指定 `--template` 时，`--language *-experience` 自动检测为 legacy 4-surface 体验模式以兼容旧行为；否则默认 basic。
- `--language typescript-experience`（未指定 `--template`）仍生成原始 4-surface 体验描述符以兼容旧行为。
- `ygg init-composition` 和 `ygg composition check` 提供本地 composition descriptor 流程。
- `ygg package check` 和 `ygg package conformance` 在本地验证生成的包。
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

- `cargo run -p ygg-cli -- conformance` 运行 68 个具名 CLI 用例，覆盖：session、事件、包、能力、hook、schema、principal、权限、subprocess 执行、host 传输、surface、proposal、官方包、composition-lab、asset-lab、projection-lab、persona-lab、knowledge-lab、context-lab、text-transform-lab、model-connector-lab、model-routing-lab、in-process package fallback hardening、playable-seed、空白游创循环、asset/branch/projection 底座、生成包创作（basic、experience、assistant-action、asset-editor、full-surface 模板）和 composition descriptor。
- 加上 `cargo test --workspace` 下的 crate 和 service 单元测试。
- `tsc -p clients/web/tsconfig.json --noEmit` 检查 web shell。

## 部分实现

- 能力调用 lifecycle 事件（`kernel/capability.invoked|completed|failed`）已在契约中预留；尚未发出。
- Streaming 协议分发和 package-principal 的 `event.subscribe` 权限。
- Hook handler 超时/错误审计，面向包拥有的 handler。
- 持久化的能力 provider 选择策略（超越单次调用显式选择）。
- 持久化的权限授权重新水化和更丰富的资源策略覆盖（网络/文件系统/包/projection 强制矩阵）。
- 内容寻址的 asset blob 存储和 package-principal asset 权限检查。
- 包拥有的 projection 执行。
- 更丰富的崩溃监控和健康检查（超出当前 lifecycle 事件）。
- 更广泛的传输一致性覆盖（超出当前核心协议 dispatcher 和 service 测试）。
- 更丰富的 TypeScript SDK 打包（超出当前薄 subprocess 辅助层）。
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
