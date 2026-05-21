# TriviumDB / TDB Integration Review

> [English](./TRIVIUMDB_REVIEW.en.md) · [中文](./TRIVIUMDB_REVIEW.md)

本 ledger 记录 `/workspace/Yggdrasil/TriviumDB` 当前源码对 Yggdrasil 的接入含义。

## 源码观察

- `Cargo.toml`：crate 名称 `triviumdb`，版本 `0.7.1`，`crate-type = ["cdylib", "rlib"]`，默认 features 为空，`python` / `nodejs` / `cli` 为可选 feature。
- `src/lib.rs`：公开 `Database`、`Config`、`SearchConfig`、`StorageMode`、`VectorType`、`Filter`、`SearchHit` 等类型。
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

## 为什么默认不直接链接真实 crate

当前 TDB 源码是 Yggdrasil 仓库外的 sibling checkout：`/workspace/Yggdrasil/TriviumDB`。如果 Yggdrasil 在 `Cargo.toml` 里提交 a sibling path dependency to the local TriviumDB checkout 依赖，普通 clone/CI 会缺失该路径而失败。

因此当前默认只实现：

- `official/tdb-retrieval-lab` deterministic fake/plan-only adapter；
- `describe_real_tdb_opt_in_seam` 真实接线说明；
- Forge UI readiness 展示；
- conformance 确认默认不链接、不打开、不建索引、不生成 embedding。

真实接线后续应在以下条件满足后启用：

1. TDB 以可解析方式存在（crates.io、git rev、submodule/vendor、或独立 subprocess adapter）；
2. host policy 显式启用；
3. backend path 使用 host ref，不进入 events/proposals/logs/public diagnostics；
4. resource limits 明确：dimension、max nodes、payload bytes、query top_k、expand_depth；
5. indexing/query execution 都有 approval/audit/redaction；
6. adapter 仍是普通 package/provider，无官方优先级。

## 推荐真实模式顺序

1. **Subprocess adapter package**：优先。隔离 native dependency、file lock、panic、repair/compaction 生命周期，适合跨平台二进制/Node/Python binding。
2. **Feature-gated inproc adapter**：仅当 TDB 已 vendored 或发布，且 host 明确接受 native in-process 风险时启用。
