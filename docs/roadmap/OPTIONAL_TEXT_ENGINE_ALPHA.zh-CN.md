# 可选文本引擎 Alpha

> [English](./OPTIONAL_TEXT_ENGINE_ALPHA.md) · [中文](./OPTIONAL_TEXT_ENGINE_ALPHA.zh-CN.md)

这是一份临时执行计划，用于把当前 Text Surface Proof 推进成可选前端文本引擎轨道。Pretext 被视为可选 client-side layout engine，不是 kernel 功能，也不是官方能力包。

## 不变量

- 不新增 `kernel.text.*`、`kernel.model.*`、`kernel.agent.*` 或 `kernel.prompt.*` 方法。
- 不新增 `official/pretext-*` 包。
- Fallback text layout 永远可用。
- 如果使用 Pretext，也必须通过 Web client engine abstraction 和动态选择隔离。
- Assistant/Forge/Play 消费通用 stream/text surfaces，不引入 model/agent 语义。

## Phase T2 — 引擎抽象与 fallback registry ✅ 已完成

目标：

- 引入 `TextEngine` interface、engine registry、config 和 fallback engine implementation。
- 重构现有 text-layout adapter，保持当前 Assistant proof 行为不变。
- 增加 generic stream frame 到 text buffer 的 adapter helpers。

交付物：

- **`engine.ts`**：`TextEngine` 接口、`EngineConfig`/`TextEngineConfig`/`TextEngineName`/`TextEngineState`/`TextEngineDiagnostics` 类型。
- **`fallback-engine.ts`**：`FallbackTextEngine implements TextEngine`，封装原始 canvas adapter。保留向后兼容函数导出（`prepareText`、`layoutPreparedText`、`createStreamingBuffer` 等）。宽度缓存有上限（默认 4096 条，FIFO 淘汰）。
- **`registry.ts`**：`registerTextEngine`/`activateTextEngine`/`getActiveTextEngine`/`selectTextEngine`/`getTextEngineState`/`getTextEngineDiagnostics`/`unregisterTextEngine`。默认使用 fallback。支持 localStorage/URL 参数/环境变量偏好解析（T3 将连接 Pretext feature flags）。
- **`stream-adapter.ts`**：`feedStreamFrame(buffer, frame)` 通用适配器，支持 `start`/`chunk`/`progress`/`end`/`error`/`cancelled`/`timeout`。不引入 model/agent 语义。提供便捷帧构造函数。
- **`index.ts`**：更新导出 — 原有函数名不变；新增类型和函数一并导出。
- **Assistant Drawer**：在 Text Proof 元数据行显示活跃引擎名称、版本和状态徽章。
- **`clients/web/README.md`**、**`integrations/pretext/ui-map.yaml`**：已更新以记录 T2 新增内容。

验证：

- `tsc -p clients/web/tsconfig.json --noEmit` 通过。
- 现有 Rust/conformance 检查不受影响。
- 未修改 kernel/package/protocol。

## Phase T3 — 可选 Pretext engine 与 feature flags ✅ 已完成

目标：

- 增加可选 `PretextEngine`，通过 dynamic import / runtime engine selection 使用。
- 不安装 Pretext 时仓库仍可 build。
- 增加 URL/localStorage/build environment fallback 运行时控制。
- 更新 `integrations/pretext` ledger 和 client README。

交付物：

- **`pretext-shim.ts`**：本地类型定义，镜像 `@chenglou/pretext` API surface (v0.0.7)。定义 `PretextModuleShape` 接口用于安全的 dynamic import 类型转换。允许 TypeScript 在未安装包的情况下编译通过。
- **`pretext-bridge.ts`**：Ygg text-layout 类型与 Pretext shapes 之间的隔离映射。选项转换（`toPretextOptions`）、结果映射（`fromPretextLayoutResult`、`fromPretextLayoutLinesResult`、`fromPretextLineStats`、`fromPretextLineRange`）和不透明句柄桥接（`bridgePrepared`、`bridgePreparedWithSegments`、`unbridgePrepared`、`unbridgePreparedWithSegments`）。真实模块不可用时，类型骨架和适配器函数仍可编译。
- **`pretext-engine.ts`**：`PretextTextEngine implements TextEngine`，带异步 `initialize()`。使用 dynamic import 配合 unknown-safe 类型转换（`import("@chenglou/pretext")` → 转换为 `PretextModuleShape`）。模块不可用时，`initialize()` 抛出可诊断错误（含加载失败原因），供 registry 回退。导出 `isPretextAvailable()`、`getPretextLoadError()`、`resetPretextLoadState()`。
- **`config.ts`**：运行时引擎偏好解析。`TextEnginePreference` 类型：`"auto" | "fallback" | "pretext"`。按 URL 参数 `?text-engine=`、localStorage `ygg_text_engine`、`globalThis.__YGG_TEXT_ENGINE__` 解析。默认 `"auto"`（有 Pretext 就用，没有就 fallback）。导出 `TextEngineInitializationResult`，含 preferred/active engine、fallback reason、Pretext availability。
- **`registry.ts`（T3 新增）**：`initializeTextEnginePreference()` — 异步初始化：解析偏好，尝试加载并激活 Pretext，失败时优雅回退并记录原因。`getInitializationResult()` — 返回上次初始化结果供诊断。`isPretextEngineAvailable()` — 检查 Pretext 模块可用性。`getPretextAvailabilityError()` — 返回加载错误（如有）。原有同步 `getActiveTextEngine()` 不变。
- **Assistant Drawer（T3 新增）**：额外诊断徽章 — 引擎偏好（`pref auto`/`pref fallback`/`pref pretext`）、Pretext 可用性（`pretext available`/`pretext unavailable`）、回退原因（tooltip 显示完整原因）。
- **`index.ts`**：更新导出，包含所有 T3 类型和函数。
- **`clients/web/README.md`**：更新 T3 章节，文档化 PretextTextEngine、bridge、shim、config、async init、Assistant Drawer 新增。
- **`integrations/pretext/README.md`**：更新 T3 集成详情（shim、bridge、engine、config、registry、drawer）。
- **`integrations/pretext/upstream.lock.toml`**：更新 T3 备注（dynamic import、fallback 保证）。
- **`integrations/pretext/ui-map.yaml`**：更新至 T3-alpha-1，包含 bridge/shim/config/engine 条目和更新约束。

验证：

- `tsc -p clients/web/tsconfig.json --noEmit` 通过（未安装 `@chenglou/pretext`）。
- Pretext 不可用时 fallback 正常工作。
- Assistant proof 显示 engine selection diagnostics（偏好、可用性、回退原因徽章）。
- 未修改 kernel/package/protocol。

## Phase T4 — Forge/Assistant stream 文本集成 ✅ 已完成

目标：

- 把 text buffer adapter 连接到 generic stream frame shape。
- 在 Forge 增加受限文本预览，用于 stream/proposal/tool/audit-like long text，不替换 JSON inspector。
- Play 保持不变，只记录未来 optional hint 设计。

交付物：

- **`text-preview.ts`**：文本预览 helper，从任意 event payload、stream frame 和 proposal-like 对象中提取安全纯文本预览。支持 `kernel/stream.chunk`、`kernel/stream.progress`、`kernel/stream.error`、`kernel/stream.cancelled`、`kernel/stream.timeout` 事件 payload；通用字段（`text`、`message`、`summary`、`reason`、`content`）；proposal `expected_effects`/`operations` 中的长 string 字段。不引入 model/agent 语义。使用活跃 text engine（或 fallback）进行 line/height 估算。导出 `extractEventPreview`、`extractProposalPreview`、`kindBadgeLabel`，以及 `TextPreviewKind` 和 `TextPreviewResult` 类型。
- **`forge.ts`（T4 新增）**：`renderEvent` 在现有 JSON `<code>` 下方显示可选的 `<details class="text-preview-details">`，当检测到 stream payload 或长文本字段时展开。预览显示转义纯文本、line/height 估算、engine name 和 kind badge。`renderProposal` 新增类似的 `<details>` 用于 proposal 文本预览（effects/operations），保留原有"Inspect proposal" JSON 详情。
- **`styles.css`（T4 新增）**：`.text-preview-details`、`.text-preview-panel`、`.text-preview-meta`、`.text-preview-stage` CSS 类。紧凑、不侵入的样式，与现有 Forge event row 一致。
- **`index.ts`**：更新导出，包含 T4 的 `TextPreviewKind`、`TextPreviewResult`、`extractEventPreview`、`extractProposalPreview`、`kindBadgeLabel`。
- **`clients/web/README.md`**：更新 T4 章节，文档化 text-preview helper、Forge event/proposal 预览和 CSS 新增。
- **`integrations/pretext/ui-map.yaml`**：更新至 T4-alpha-1，包含 text-preview 条目和更新约束。

验证：

- `tsc -p clients/web/tsconfig.json --noEmit` 通过。
- UI 行为仍只走 public protocol。
- 未修改 kernel/package/protocol。
- Play 不变。

## Phase T5 — SDK 抽取、测试与硬化

目标：

- 在 `sdk/typescript/text-surface` 抽取可复用 text-surface helpers。
- 增加 fallback engine、registry、stream adapter 和 engine selection 的轻量单元测试。
- 增加 cache limits 和 font-loading helpers。
- 文档记录第三方 client 用法。

验证：

- TypeScript tests 通过。
- 现有 Rust/conformance/play demo 通过。

## Final phase — durable docs and cleanup

目标：

- 更新 durable docs/status/roadmap。
- 完成后删除这份临时计划。
- 跑完整验证。

必跑检查：

```bash
tsc -p clients/web/tsconfig.json --noEmit
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- play-create-demo
```
