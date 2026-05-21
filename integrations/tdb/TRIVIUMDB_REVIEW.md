# TriviumDB / TDB Integration Review

> [English](./TRIVIUMDB_REVIEW.en.md) · [中文](./TRIVIUMDB_REVIEW.md)

本 ledger 记录 `/workspace/Yggdrasil/TriviumDB` 当前源码对 Yggdrasil 的接入含义，并修正上一轮过度保守的判断：TriviumDB README 明确推荐 Rust 侧 `cargo add triviumdb`，所以 Yggdrasil 应当提供真实 Rust API adapter proof，而不只是 plan-only 入口。

## 源码观察

- `README.md` / `README_EN.md`：安装说明包含 `cargo add triviumdb`，Rust library 接入是官方路线。
- `Cargo.toml`：crate 名称 `triviumdb`，版本 `0.7.1`，`crate-type = ["cdylib", "rlib"]`，默认 features 为空，`python` / `nodejs` / `cli` 为可选 feature。
- `src/lib.rs`：crate root 公开 `Database`、`Result`、`TriviumError`、`Filter`、hook/node/vector 相关类型；`Config`、`SearchConfig`、`StorageMode` 需要从 `triviumdb::database` 导入。
- `src/database/config.rs`：`StorageMode::{Mmap, Rom}`，`Config { dim, sync_mode, storage_mode }`，`SearchConfig` 包含 `top_k`、`expand_depth`、hybrid text、DPP、PPR、payload filter 等检索参数。
- `src/database/mod.rs`：`Database<T>::open/open_with_config` 打开本地文件，创建目录、`.lock` 文件、WAL，支持 `insert`、`insert_with_id`、`link`、`begin_tx`、`search`、`search_advanced`、`search_hybrid`、`search_hybrid_with_context`。
- `src/database/transaction.rs`：事务使用 dry-run + WAL-first commit 语义；WAL replay 对 insert/link/delete/update 等操作做恢复。
- `triviumdb.d.ts`：Node binding 暴露 `TriviumDB` 类、vector、payload、filter、search config 与 search hits。

## 接入判断

TDB 适合成为 Yggdrasil 的 **retrieval/multimodal provider adapter**，而不是：

- kernel event store；
- canonical asset store；
- projection authority；
- package raw database；
- 全局 memory/chat/agent/world store。

理由：TDB 的优势是单机嵌入式多模态/向量/图/文档混合检索；Yggdrasil 的事件、proposal、permission、branch lineage 仍需要 event spine 作为 authority。

## 当前真实 Rust adapter proof

本轮新增真实 Rust 接入 proof：

```text
integrations/tdb/rust-adapter
integrations/tdb/rust-adapter-real-local
examples/packages/tdb-rust-adapter/manifest.yaml
```

默认 adapter：

- 是普通 JSON-RPC stdio subprocess package；
- 不依赖 `triviumdb`；
- 可通过 Ygg runtime 加载和调用；
- `run_real_tdb_smoke` 明确返回 `real_tdb_available=false`，不会伪装成功。

真实本地 proof：

```bash
cargo test --manifest-path integrations/tdb/rust-adapter-real-local/Cargo.toml --features real-tdb
```

该 proof 显式使用 sibling checkout `/workspace/Yggdrasil/TriviumDB`，真实调用：

```rust
Database::<f32>::open_with_config(...)
insert(...)
link(...)
search(...)
search_hybrid(...)
```

真实 proof 使用临时 redacted store，不输出 raw path，不联网，不进入主 workspace 默认构建。

## 为什么默认不直接链接真实 crate

不是因为 TDB 在仓库外就“不做”；相反，真实 Rust adapter proof 已经完成。默认不链接的原因只是工程边界：普通 clone / CI 不应被本地 sibling checkout 绑死。

因此采用双轨：

1. 默认 adapter shell：可在普通 Yggdrasil checkout 中编译、加载、conformance；
2. real-local adapter：在拥有 TriviumDB checkout 的开发环境中显式 opt-in 跑真实 Rust API proof。

后续如果 TDB 以 crates.io、固定 git rev、submodule 或 vendor 方式稳定可解析，可以把 real adapter 从 local proof 推进到更正式的 feature-gated package 构建。

## 推荐真实模式顺序

1. **Subprocess adapter package**：优先。隔离 native dependency、file lock、panic、repair/compaction 生命周期，适合跨平台二进制/Node/Python binding。
2. **Feature-gated inproc adapter**：仅当 TDB 已 vendored 或发布，且 host 明确接受 native in-process 风险时启用。
