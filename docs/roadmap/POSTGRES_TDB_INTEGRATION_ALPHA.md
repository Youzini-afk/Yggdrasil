# PostgreSQL + TDB Integration Alpha

> [English](./POSTGRES_TDB_INTEGRATION_ALPHA.en.md) · [中文](./POSTGRES_TDB_INTEGRATION_ALPHA.md)

这是临时执行计划。完成后删除，长期内容收敛到 `ALPHA_STATUS`、`NEXT_STEPS`、`STORAGE_BACKEND_NEUTRALITY`、conformance matrix 和相关包文档。

## 调研结论

本阶段基于三路调研：

- PostgreSQL/Rust：`sqlx` 更适合 Yggdrasil 的 async event-store backend，因为它提供 pool、migrations/test 支撑、rustls 路线和稳定 SQL 调用；`tokio-postgres` 更底层但需要额外 pool/migration；Diesel 更偏 ORM，不适合当前 event spine。
- PostgreSQL 语义：sequence/identity 可作为全局排序键但不适合 gapless 业务版本；per-session sequence 应通过 transaction + session-scoped lock / max(sequence)+1 / unique(session_id, sequence) 保证。默认测试不依赖真实 PostgreSQL，真实 backend 走 feature + env opt-in。
- TriviumDB/TDB：源码位于 `/workspace/Yggdrasil/TriviumDB`，形态是 Rust crate + cdylib/rlib、Python/Node bindings、CLI demo/repair 工具，没有现成 server/RPC/daemon。因此短期最佳是把它作为普通 retrieval/multimodal provider adapter 的实现依赖或 fake-compatible seam，而不是放进 kernel runtime。

## 边界决定

```text
PostgreSQL = host-owned EventStore backend，real opt-in
TDB = ordinary package/provider retrieval adapter，default fake/deterministic，real crate opt-in later
```

PostgreSQL 替代的是 SQLite durable event log，不给 package 暴露 SQL。TDB 增强的是 package/provider 层的检索与多模态索引能力，不成为 canonical asset/projection/backend truth。

## 红线

- 不新增 `kernel.postgres.*`、`kernel.sql.*`、`kernel.database.*`、`kernel.tdb.*`、`kernel.vector.*`、`kernel.embedding.*`。
- 不把 `EventStore` 改成万能 `DatabaseBackend`。
- package 不得访问 kernel PostgreSQL connection/pool/events table。
- TDB 不作为 kernel asset/projection store。
- pgvector/TDB/vector/embedding 语义不进入 kernel。
- DSN、连接串、DB user、TDB path/private topology/raw backend error 不写入 events/proposals/logs/public diagnostics。
- 默认 CI/conformance 不依赖真实 PostgreSQL/TDB；真实 backend smoke 必须显式 opt-in。

## Phase P0 — Plan, Research, Boundary Freeze

目标：固化 PostgreSQL/TDB 的接入边界，避免后续实现漂移。

交付：

- 本临时双语计划。
- README / ALPHA_STATUS / NEXT_STEPS 当前主线更新。
- 外部资料与 TDB 源码调研结论纳入计划。

验收：doc links、diff check、commit/push。

## Phase P1 — PostgreSQL EventStore Backend Proof ✅ 已完成

目标：实现 opt-in `PostgresEventStore`，只实现 `EventStore` event spine contract。

交付：

- ✅ `ygg-runtime` 增加 `postgres` feature 与 `PostgresEventStore`。
- ✅ 使用 `tokio-postgres` + `deadpool-postgres`（避免 `sqlx`+`rusqlite` `libsqlite3-sys` links 冲突），默认 feature 不启用。
- ✅ Schema 初始化：events table、unique(session_id, sequence)、session/sequence、kind、session+kind indexes。Payload/metadata 为 JSONB。
- ✅ `append_with_sequence` 在 transaction 中使用 `pg_advisory_xact_lock(hashtext(session_id))` + `max(sequence)+1` + insert，保证并发无重复 sequence。
- ✅ 实现 list_all/list_session/range/kind-prefix/session-kind-prefix/next_sequence/subscribe（本地 broadcast，暂不做 LISTEN/NOTIFY）。
- ✅ 新增 feature-gated / env-gated conformance helper：只有设置 `YGG_POSTGRES_TEST_DATABASE_URL` 且启用 feature 时才跑真实 PG；默认 CI 不受影响。
- ✅ Backend errors 对外 redacted，不把 DSN 写入任何 public output。

验收：workspace tests、default conformance、`cargo check -p ygg-runtime --features postgres`，如环境有 PG 则 opt-in storage conformance — 全部通过。

## Phase P2 — Host/Profile Backend Selection ✅

目标：让 host 可以选择 memory/sqlite/postgres backend，但 backend 配置仍属于 host-only。

交付：

- ✅ Host profile 增加 `event_store` backend config shape：memory/sqlite/postgres。
- ✅ CLI/host 启动支持 SQLite path 和 PostgreSQL env-ref opt-in；PostgreSQL 仍需 `--features postgres`。
- ✅ Host stdout diagnostics 只显示 backend kind 与 `config redacted`，不显示 DSN 或 private topology。
- ✅ 新增 `profiles/forge-postgres.example.yaml`，只引用 env var 名，不包含连接串。
- ✅ 默认行为保持 memory backend，不影响现有 host/profile。

验收：host 默认路径不变；postgres feature 编译；public protocol 不含 DSN。

## Phase T1 — TDB Retrieval Adapter Contract/Fake Provider

目标：以普通 package/provider 证明 TDB 接入路径，而不是把 TDB 放进 kernel。

交付：

- 新增 ordinary package（建议 `official/tdb-retrieval-lab` 或同等名称），作为 deterministic fake retrieval provider proof。
- 能力：describe_tdb_boundary、plan_index_asset_refs、fake_index_asset_refs、fake_search_refs、explain_retrieval_trace、summarize_provider_health。
- 输出 refs/trace/provider_health，不输出 raw vectors/embeddings，不修改 canonical assets/projections。
- Conformance 证明 no kernel namespace、no raw secrets、deterministic fake index/search、returns refs only。

验收：package check、conformance、Forge Storage Inspector 能显示 TDB provider readiness。

## Phase T2 — TDB Real-Crate Opt-in Seam

目标：预留真实 TDB crate 接入 seam，但不让默认 CI 或 core runtime 依赖它。

交付：

- 研究 `/workspace/Yggdrasil/TriviumDB` crate API 后，在 adapter 文档/manifest 中记录 real mode prerequisites。
- 如兼容性允许，增加 feature-gated path dependency / adapter stub；默认 fake mode。
- 若 real crate 不适合直接纳入当前 workspace，则保留 external adapter guide，不强行耦合。
- 更新 UI/docs/conformance matrix。

验收：默认 build/test 不需要 TDB；real mode 仅 opt-in；无 kernel/vector namespace 泄漏。

## Phase C — Durable Cleanup and Final Validation

目标：删除临时计划，收敛 durable docs，完成最终验证。

交付：

- 删除本计划。
- 更新 `STORAGE_BACKEND_NEUTRALITY`、ALPHA_STATUS、NEXT_STEPS、README、performance docs、conformance matrix。
- 最终报告 commit 序列、验证结果和后续建议。

最终验证：

- `cargo test --workspace`
- `cargo run -p ygg-cli -- conformance`
- `cargo run -p ygg-cli -- conformance --tag storage`
- `cargo run -p ygg-cli -- package check packages/official/storage-lab/manifest.yaml`
- 新 TDB adapter package check
- `cargo check -p ygg-runtime --features postgres`
- `tsc -p clients/web/tsconfig.json --noEmit`
- markdown local links
- `git diff --check`
- temporary plan residue check
