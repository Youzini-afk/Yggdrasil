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

## Phase S4 — SDK/templates 与 no-network readiness proof

目标：

- 增加 TypeScript package-authoring helpers/templates，用于 secret refs、network permission metadata、audit/redaction 与 streaming fixtures。
- 增加 no-network faux model/agent readiness examples，证明 substrate shape，而不做真实 inference 或 pi runtime coupling。

非目标：

- 暂不真实接入 `pi-agent-core`。
- 不做真实 model inference。

## Phase T1 — Pretext-inspired text surface proof

目标：

- 增加 `integrations/pretext` ledger，记录 Pretext 适合/不适合的场景。
- 在 Assistant drawer 或独立 web module 中增加轻量 client-side text layout / progressive streaming proof。
- 保持现有 Play/Forge dashboard 稳定；不重写整个 web shell。

非目标：

- Pretext 不进入 kernel/package/protocol。
- 不承诺完整 Markdown engine。
- Proof 通过前不承诺正式依赖。

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
