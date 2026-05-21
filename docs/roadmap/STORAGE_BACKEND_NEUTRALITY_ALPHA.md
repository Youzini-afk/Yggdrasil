# Storage Backend Neutrality Alpha

> [English](./STORAGE_BACKEND_NEUTRALITY_ALPHA.en.md) · [中文](./STORAGE_BACKEND_NEUTRALITY_ALPHA.md)

这是临时执行计划。完成后删除，长期内容收敛到 `ALPHA_STATUS`、`NEXT_STEPS`、storage/data guide、conformance matrix 和相关包文档。

## 为什么现在做

Yggdrasil 当前已经有 SQLite-backed append-only event log、重水化的 assets/branches/projections/permissions，以及 `perf baseline` 里的 1k/10k/100k event store 场景。但这只是早期本地默认后端，不应变成平台契约。后续会需要 PostgreSQL 作为服务端/团队 host 后端，也会需要未来 TDB 多模态向量数据库或其他检索后端；这些必须作为 backend/provider 层能力，而不是进入 kernel ontology。

外部调研要点：

- SQLite WAL 是 append-first/checkpoint-later，有单 writer、同机 shared-memory、checkpoint、large transaction 和 network filesystem 限制；这说明 SQLite 行为不能成为平台语义。
- PostgreSQL transaction/WAL 适合未来 durable remote/team event-store backend，但事务、SQL、DSN、隔离级别不应泄漏到 kernel protocol。
- pgvector、OpenSearch、Redis Vector 等资料显示 vector/multimodal search 依赖显式 embedding/model/index/distance/top-k/filter 配置；它们应作为 retrieval/index capability packages 或 backend providers，不是 event spine contract。

## 分层模型

1. **Kernel Event Spine**：只拥有 append、replay、range、kind prefix、subscription、rehydrate 这些内容无关事件语义。
2. **Backend Implementations**：`InMemoryEventStore`、`SqliteEventStore`，未来可有 `PostgresEventStore`。它们是 host/runtime 配置，不改变 protocol。
3. **Package-scoped Data Capability**：普通包提供 document/KV/object/index preview 等能力，不能绕过 permission/audit/proposal。
4. **Blob / Asset Store**：未来可替代 event payload 中的大对象存储，先以 contract proof 和 content-addressed preview 为主。
5. **Retrieval / Vector / Multimodal Providers**：TDB、pgvector、Qdrant、OpenSearch、Redis Vector 等应作为 retrieval/vector/multimodal provider packages 或 backend adapters。
6. **UI / Observability**：Forge 只通过 public protocol 展示 storage contracts、backend capabilities、risk/diagnostics，不读 SQLite 或 backend internals。

## 红线

- 不新增 `kernel.sqlite.*`、`kernel.postgres.*`、`kernel.tdb.*`、`kernel.vector.*`、`kernel.embedding.*`、`kernel.collection.*`、`kernel.sql.*`、`kernel.database.*`。
- 不把 `EventStore` 扩成万能 `DatabaseBackend`，不暴露 SQL、table、transaction isolation、connection、DSN、ANN index、vector dimension 等产品语义。
- SQLite、PostgreSQL、TDB 都是 backend/provider，不是平台契约。
- 向量/多模态检索属于 package/provider 层，不进入 kernel。
- 包不能通过 storage backend 获得 raw DB 权限或绕过 events/assets/proposals/permissions。
- DSN、file paths、credentials、backend topology 不写入 events/proposals/logs/public diagnostics；只允许 coarse redacted diagnostics。
- 不实现真实 PostgreSQL/TDB/vector DB；本阶段是 neutrality 和 contract proof。

## Phase S0 — Research, ADR, Temporary Plan

目标：固化调研结论、阶段边界和红线。

交付：

- 本临时双语计划。
- README / ALPHA_STATUS / NEXT_STEPS 当前主线更新。
- 外部 evidence 路径记录：`/tmp/opencode/ygg-storage-neutrality-20260520/`（如工具不可保存完整网页，则以本计划和会话证据记录 URL）。

验收：doc links、diff check、commit/push。

## Phase S1 — EventStore Backend-Neutral Contract Hardening

目标：明确 `EventStore` 是 event spine contract，不是 database abstraction。

交付：

- 更新/扩展 runtime storage contract 文档与代码注释，明确 `append_with_sequence`、range replay、kind prefix query、subscription、rehydrate 的 backend-neutral semantics。
- 增加 backend-neutral event-store conformance helper，同一组行为覆盖 in-memory 与 SQLite：append/list/range、concurrent append no duplicate、kind prefix equivalence、subscription、rehydrate parity。
- 避免 runtime 新增 SQLite-specific 依赖。

验收：workspace tests、storage tag conformance、baseline event-store scenarios。

## Phase S2 — Package-Scoped Data Contract + `storage-lab`

目标：提供普通 package-facing storage/data contract proof，而不是 kernel database API。

交付：

- 新增普通官方包 `official/storage-lab`（`rust_inproc`）。
- 能力：`describe_storage_contract`、`describe_backend_classes`、`plan_package_state_store`、`put_document_preview`、`get_document_preview`、`query_document_prefix_preview`、`delete_document_tombstone_preview`、`export_store_snapshot_preview`。
- 输出是 deterministic preview / package-owned data model；不写真实 DB、不执行 SQL、不读写文件、不联网。
- Profile autoload、surfaces、conformance。

验收：package check、storage-lab conformance、无 SQL/kernel database namespace。

## Phase S3 — Blob / Asset Store Contract Proof

目标：为大对象和 asset content-addressed backend 预留，不把 blob content 塞进 event payload。

交付：

- 扩展 `storage-lab`：`describe_blob_store_contract`、`put_blob_preview`、`get_blob_metadata_preview`、`export_blob_manifest_preview`。
- 明确 backend candidates：local content-addressed、filesystem、object store future；只输出 hash/size/mime/provenance，不保存真实 blob。
- raw secret / unsafe path 阻断。

验收：conformance 覆盖 content-address determinism、no raw secret、no filesystem write。

## Phase S4 — Projection / Index Materialization Contract Proof

目标：定义 package-owned projection/index store 的最小 contract，避免直接把 projection 变成 DB table。

交付：

- 扩展 `storage-lab`：`describe_projection_store_contract`、`plan_projection_materialization`、`query_projection_preview`、`migrate_projection_plan_preview`。
- 支持 SQLite/Postgres future materialization 作为 backend candidates，但只输出 plan。
- 与现有 `projection-lab` 文档/Forge inspector 对齐。

验收：conformance 覆盖 no DB table leakage、plan-only、backend-neutral output。

## Phase S5 — Retrieval / Vector / Multimodal Provider Contract

目标：给 TDB/pgvector/OpenSearch/Redis Vector 等未来检索后端留槽，但不实现。

交付：

- 扩展 `storage-lab` 或新增 ordinary retrieval descriptor 能力：`describe_retrieval_provider_contract`、`draft_multimodal_index_plan`、`draft_vector_search_plan`、`explain_retrieval_backend_fit`。
- 后端候选包含 `tdb_future`、`pgvector_future`、`local_embedding_index_future`、`remote_vector_provider_future`。
- 输出只含 redacted plan、asset refs、modality flags、index capability flags；不生成 embedding、不存 vector、不联网。

验收：conformance 证明 no kernel vector namespace、no embedding generation、no backend credentials、TDB 只是 future provider slot。

## Phase S6 — Forge Storage Inspector + Durable Docs Cleanup

目标：Web 能用 public protocol 展示 storage contracts；删除临时计划，收敛 durable docs。

交付：

- Forge 增加 Storage/Data panel，展示 event spine、package state store、blob store、projection/index、retrieval provider 的 contract summaries。
- Assistant drawer 可显示 storage guide hint（轻量）。
- 新增 `docs/guides/STORAGE_BACKEND_NEUTRALITY.md` 与 `.en.md`。
- 删除本临时计划；更新 README、ALPHA_STATUS、NEXT_STEPS、CONFORMANCE_MATRIX、performance docs。

最终验证：

- `cargo test --workspace`
- `cargo run -p ygg-cli -- conformance`
- `cargo run -p ygg-cli -- conformance --tag storage`
- `cargo run -p ygg-cli -- package check packages/official/storage-lab/manifest.yaml`
- `cargo run -p ygg-cli -- perf baseline --iterations 1 --format json`
- `tsc -p clients/web/tsconfig.json --noEmit`
- markdown local links
- `git diff --check`
- temporary plan residue check
