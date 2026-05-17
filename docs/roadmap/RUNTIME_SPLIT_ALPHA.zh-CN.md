# Runtime Split Alpha

> [English](./RUNTIME_SPLIT_ALPHA.md) · [中文](./RUNTIME_SPLIT_ALPHA.zh-CN.md)

Runtime Split Alpha 是代码健康与契约 hardening 路线。它防止 `crates/ygg-runtime/src/runtime.rs` 和官方 in-process package fallback 变成长期架构陷阱。

这不是功能扩张。它保持公开 `Runtime<S>` API 和当前 package/protocol 模型，同时有意修正两个不安全模式：protocol registry/dispatch 漂移，以及 suffix-only in-process fallback routing。

## 目标

- 保持 `Runtime<S>` 公开方法稳定。
- 按 kernel domain 拆分 runtime 行为，而不是按临时 helper bucket 拆分。
- 让 protocol methods 成为 registry 与 dispatch 的单一事实源。
- 确保 implemented protocol methods 不会静默缺少 dispatch 覆盖。
- 确保 in-process official handlers 按 provider package 和声明的 capability 路由，而不是只按 suffix。
- 保持 kernel content-free，官方包无特权。

## 非目标

- 不重做 asset store。
- 不重做 projection engine。
- 不新增 gameplay/content/model 语义。
- 不新增 direct service routes。
- 不引入 trait-heavy service layer 或第二套 runtime 实现。
- 不做 package dependency resolver、WASM execution、remote execution 或 marketplace。

## Phase A — Protocol single source of truth

创建 `KernelMethod` 单一事实源，拥有 method id、status、streaming flag 和 parsing。Protocol registry metadata 和 dispatch matching 都从它派生。

验收：

- `kernel.session.close` 有 dispatch 覆盖或修正 status；优先 dispatch，因为 runtime 已支持 close。
- `Runtime::call_protocol` dispatch 的 methods 都出现在 registry 中。
- implemented/partial method coverage 有测试。
- 公开 method names 不变。

## Phase B — 按 kernel domain 拆分 `runtime.rs`

保留 `runtime.rs` 作为稳定 module root，把 impl blocks/types 移入 domain modules：

- `session.rs`
- `events.rs`
- `packages.rs`
- `capabilities.rs`
- `hooks.rs`
- `permissions.rs`
- `assets.rs`
- `branches.rs`
- `projections.rs`
- `proposals.rs`
- `protocol_dispatch.rs`

验收：

- 现有 `Runtime<S>` 公开方法对调用者仍可编译。
- 移动后的公开 request/record types 仍从 `ygg_runtime::runtime` re-export。
- `runtime.rs` 成为 table of contents，而不是 coordinator blob。
- Runtime unit tests 通过。

## Phase C — Harden in-process official fallback routing

把 `inproc/common.rs` 中 suffix-only fallback 行为改成 package-aware routing。

验收：

- Handler 根据 provider package id 加声明的 local capability name 选择。
- unknown 或 unimplemented registered in-process capabilities 必须 loud fail，而不是返回 generic success。
- Conformance 证明无关包不会因为 suffix 获得 official fallback 行为。
- 现有 official labs 仍通过。

## Phase D — 文档和最终验证

更新状态与 roadmap 文档，然后运行完整验证。

必跑检查：

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- play-create-demo
tsc -p clients/web/tsconfig.json --noEmit
```

还要对代表性 official labs 跑 package checks，并跑 doc-link check。

## 不变量

- Runtime 保持 content-free platform kernel。
- 官方包不获得 prefix-based privilege。
- Protocol methods 有唯一 registry status 和唯一 dispatch decision。
- 除非明确审查，否则不应跨 `.await` 持有 `RwLock` guard。
- Error text 不应意外改变，因为 protocol error classification 当前依赖 message contents。
