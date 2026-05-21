# Real TDB Rust Adapter Alpha（临时计划）

> [English](./REAL_TDB_RUST_ADAPTER_ALPHA.en.md) · [中文](./REAL_TDB_RUST_ADAPTER_ALPHA.md)

本计划修正上一轮过度保守的 TDB 接入：TriviumDB README 明确推荐 Rust 侧 `cargo add triviumdb`，因此本轮目标不是继续停在 plan-only，而是做出 **真实 Rust API adapter proof**。

## 目标

- 真实调用 TriviumDB Rust API：`Database::open/open_with_config`、`insert`、`link`、`search/search_hybrid`。
- 仍保持 TDB 作为普通 retrieval / multimodal provider adapter，不进入 kernel，不成为 `EventStore`。
- 默认 Yggdrasil build / CI 不被本地 TDB checkout 绑死。
- 本地 opt-in proof 可以使用 `/workspace/Yggdrasil/TriviumDB` path dependency 真实运行。

## 红线

- 不新增 `kernel.tdb.*`、`kernel.vector.*`、`kernel.embedding.*`、`kernel.database.*`、`kernel.sql.*`。
- 不把 TDB 作为 event authority、permission audit、proposal lifecycle、branch lineage 或 canonical asset store。
- 不让 Web 直接读 TDB、本地文件或 runtime internals。
- 不让 package 输入 raw path / backend topology；真实 adapter 使用 bounded temp store 或 host-ref shape。
- 默认 `cargo test --workspace` 不要求 TriviumDB checkout。

## Phase R0 — Plan / API correction

- 补读 TriviumDB README / API，承认 Rust crate 接入是官方路线。
- 写入本计划并切换状态入口。
- 继续保留上轮 `official/tdb-retrieval-lab` plan/contract 能力，但把它升级为真实 adapter 路线的前置 contract。

## Phase R1 — Independent adapter crate shell ✅

- ✅ 新增 `integrations/tdb/rust-adapter/`，不加入主 workspace members。
- ✅ 默认构建提供 safe stub / JSON-RPC stdio package handler，不依赖 TriviumDB。
- ✅ 新增显式 subprocess package manifest `examples/packages/tdb-rust-adapter/manifest.yaml`（不 autoload）。
- ✅ 默认 adapter shell 能 `cargo check` / `cargo test`，并可由 package manifest 描述为 ordinary subprocess package。

## Phase R2 — Real TriviumDB API proof

- 给 adapter crate 增加 `real-tdb` feature 与本地 path dependency 配置。
- 使用 TriviumDB `Database<f32>` 真实执行：open temp `.tdb`、insert 两个节点、link、search、search_hybrid。
- 将真实 proof 限定在 adapter crate 的 opt-in test / smoke 命令，不进入默认主 workspace。
- 验证：`cargo test --manifest-path integrations/tdb/rust-adapter/Cargo.real-tdb.local.toml --features real-tdb`。

## Phase R3 — Package capability + conformance boundary

- Adapter stdio 新增能力：`describe_real_tdb_adapter`、`run_real_tdb_smoke`。
- 默认未启用 real feature 时返回 `real_tdb_available=false`，不是伪装成功。
- 本地 real feature 时返回真实 open/insert/link/search 结果 summary，仍不输出 raw path。
- 新增 CLI conformance：默认 safe shell、package check、disabled real proof；真实 opt-in conformance 仅在环境允许时运行。

## Phase R4 — Docs / UI / cleanup

- Forge Storage Inspector 展示“real Rust adapter available/disabled/opt-in proof”状态。
- 更新 `docs/guides/POSTGRES_TDB_INTEGRATION*.md` 与 `integrations/tdb/TRIVIUMDB_REVIEW*.md`。
- 删除本临时计划，合并到持久指南。
- 最终全量验证、commit、push。
