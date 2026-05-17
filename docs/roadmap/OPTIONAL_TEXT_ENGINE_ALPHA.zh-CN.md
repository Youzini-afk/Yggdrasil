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

## Phase T3 — 可选 Pretext engine 与 feature flags

目标：

- 增加可选 `PretextEngine`，通过 dynamic import / runtime engine selection 使用。
- 不安装 Pretext 时仓库仍可 build。
- 增加 URL/localStorage/build environment fallback 运行时控制。
- 更新 `integrations/pretext` ledger 和 client README。

验证：

- Pretext 不可用时 fallback 正常工作。
- Assistant proof 显示 engine selection diagnostics。

## Phase T4 — Forge/Assistant stream text integration

目标：

- 把 text buffer adapter 连接到 generic stream frame shape。
- 在 Forge 增加受限文本预览，用于 stream/proposal/tool/audit-like long text，不替换 JSON inspector。
- Play 保持不变，只记录未来 optional hint 设计。

验证：

- Web TypeScript 通过。
- UI 行为仍只走 public protocol。

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
