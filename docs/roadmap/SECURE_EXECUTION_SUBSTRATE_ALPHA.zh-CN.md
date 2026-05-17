# 安全执行底座 Alpha

> [English](./SECURE_EXECUTION_SUBSTRATE_ALPHA.md) · [中文](./SECURE_EXECUTION_SUBSTRATE_ALPHA.zh-CN.md)

这是一份临时执行计划，合并两条线：

1. **安全执行底座** —— 在真实模型推理、pi agent 能力包、Tavern bridge、remote packages 或其他联网/流式/高副作用 package 之前必须补齐的通用安全与运行时契约。
2. **Text Surface Proof** —— 受 Pretext 启发的小型前端文字输出 proof，用于未来 agent/model 流式文字体验。它是 UI 基础设施，不是内核功能。

这份计划刻意先做底座。它不会把 model、prompt、agent、chat、memory、Tavern 或 director 概念加入内核。

## 不变量

- Kernel 保持 content-free。
- Secret references、network permissions、audit envelopes、redaction states、streams 和 cancellation 都是通用执行契约。
- Provider/model/agent/Tavern 语义仍由 package 拥有。
- 官方包没有特殊权限或路由优先级。
- UI proof 只使用 public protocol / client-side infrastructure。

## Phase S1 — 持久权限与 secret references

目标：

- 通过 event log 持久化 scoped permission grants，让 host 重启后可以 rehydrate grants。**已完成。**
- 增加通用 `secret_ref` 契约与 host resolver placeholder。**已完成。**
- 为 durable grants 和 trusted paths 中的 raw-secret blocking 增加 hostile conformance。**已完成。**

非目标：

- 不做生产级 secret vault。**仍为非目标。**
- 不做 provider-specific key handling。**仍为非目标。**
- 不做真实 network/model calls。**仍为非目标。**

## Phase S2 — 网络权限、outbound audit 与 redaction skeleton ✅

目标：

- 扩展 manifest permission metadata，加入 network declarations。**已完成。**
- 增加通用 outbound audit/redaction records 与 helpers。**已完成。**
- 通过 package capabilities 或 host helpers 增加 no-network/allowlisted-network conformance fixtures。**已完成。**

非目标：

- 不声称实现完整 OS-level subprocess sandbox。**满足。**
- 不做 provider-specific audit schema。**满足。**

## Phase S3 — 通用 streaming 与 cancellation lifecycle ✅

目标：

- 定义通用 capability output stream frames。**已完成。**
- 增加 cancellation/timeout lifecycle records。**已完成。**
- 增加 normal end、error、cancel、timeout 的 fixture/conformance 覆盖。**已完成。**

非目标：

- 不做 model streaming API。**满足。**
- 不做 agent turn API。**满足。**

## Phase S4 — SDK/templates 与 no-network readiness proof ✅

目标：

- 增加 TypeScript package-authoring helpers/templates，用于 secret refs、network permission metadata、audit/redaction 与 streaming fixtures。**已完成。**
- 增加 no-network faux model/agent readiness examples，证明 substrate shape，而不做真实 inference 或 pi runtime coupling。**已完成。**

交付物：

- `sdk/typescript/secure-execution/index.ts`：Secret reference 构造/验证（`secretRef`、`isValidSecretRef`、`looksLikeRawSecret`、`isSecretFieldName`），network declaration helper（`NetworkDeclaration` 类，支持 manifest entry 和 host/method 匹配），outbound audit/redaction helper（`OutboundAuditHelper`，构建审计安全请求 payload，拒绝 raw secrets），以及 stream frame client（`StreamFrameClient`，完整 start/chunk/progress/end/error/cancel/timeout 生命周期）。所有 helper 只包装公开协议和类型——无私有内部、无协议绕过。
- `--template networked`：生成带网络权限声明的 subprocess package（`host`、`methods`、`purpose`），包含带 `network` side effect 的 `fetch` capability 和 `echo` capability。TypeScript 模板导入 secure-execution helpers，演示 `secretRef`、`NetworkDeclaration` 和 `OutboundAuditHelper` 用法。Manifest 包含 `permissions.network.declarations`。无 raw secrets、无隐式 network 访问。
- `--template streaming`：生成带 streaming capability（`streaming: true`）的 subprocess package。TypeScript 模板导入 `StreamFrameClient`，演示 faux streaming frame 生命周期（start、chunk、end）。不做真实 model inference。
- `examples/packages/faux-model-readiness/`：面向 model-like capability packages 的 no-network readiness proof。声明网络权限，提供 `discover` 和 `stream-faux` capabilities，使用 `secret_ref` 引用凭证，返回 discovery plans（非真实 API 响应），产生 faux streaming frames。不做真实 inference 或 network 调用。
- `examples/packages/faux-agent-readiness/`：面向 agent-like capability packages 的 no-network readiness proof。提供 `propose` 和 `stream-trace` capabilities，仅产出 proposals/traces/plans（无真实 agent loop），强调公开 protocol/capability/proposal 模式，无需网络权限。不连接 pi runtime 或 model inference。
- Conformance：5 个新用例，覆盖生成的 networked 模板、streaming 模板、faux-model-readiness manifest 结构、faux-agent-readiness manifest 结构。全部验证无 raw secrets、正确的网络声明、streaming capabilities 与 substrate shape。

非目标：

- 暂不真实接入 `pi-agent-core`。**满足。**
- 不做真实 model inference。**满足。**

## Phase T1 — Pretext-inspired text surface proof ✅

目标：

- 增加 `integrations/pretext` ledger，记录 Pretext 适合/不适合的场景。**已完成。**
- 在 Assistant drawer 或独立 web module 中增加轻量 client-side text layout / progressive streaming proof。**已完成。**
- 保持现有 Play/Forge dashboard 稳定；不重写整个 web shell。**已完成。**

交付：

- `integrations/pretext/README.md`、`upstream.lock.toml`、`ui-map.yaml`：记录 Pretext 参考仓库路径、MIT license、核心 API 映射（`prepare`/`layout`/`prepareWithSegments`/`layoutWithLines`/`measureLineStats`/`walkLineRanges`）、适用场景（streaming agent/model text、long text measurement、resize stability）、不适用场景（Markdown engine/kernel/package/protocol）以及 font/Canvas/system-ui 风险。
- `clients/web/src/text-layout/`：轻量 fallback adapter skeleton，TS 类型安全，API 形状对齐 Pretext。包含 `prepareText`、`layoutPreparedText`、`prepareTextWithSegments`、`layoutPreparedTextWithLines`、`measureLineStats`、`walkLineRanges`、`createStreamingBuffer` 和 `clearAdapterCache`。不安装 `@chenglou/pretext` 也能运行，并保留未来 Pretext swap-in 点。
- Assistant Drawer mock streaming text proof：使用 deterministic mock chunks，不接真实 agent/model，不出站联网。展示渐进文本、实时行数/高度估算、类似 `redaction_state` 的 stream 生命周期徽章（`idle`/`streaming`/`ended`/`reset`）以及 reset/replay 控件。不改变现有 drawer 行为。
- CSS 字体变量 `--font-text-surface` 与 `--font-text-surface-mono`，避免 macOS 上 `system-ui` 测量不一致风险。

非目标：

- Pretext 不进入 kernel/package/protocol。**已满足。**
- 不承诺完整 Markdown engine。**已满足。**
- Proof 通过前不承诺正式依赖。**已满足。**

## Final phase — durable docs and cleanup

目标：

- 更新 durable docs/status/conformance matrix。
- milestone 完成后删除这份临时计划文档。
- 跑完整验证。

必跑检查：

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- play-create-demo
tsc -p clients/web/tsconfig.json --noEmit
```

同时跑代表性 package/composition checks 和 doc-link validation。
