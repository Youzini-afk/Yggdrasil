# Storage Backend Neutrality

> [English](./STORAGE_BACKEND_NEUTRALITY.en.md) · [中文](./STORAGE_BACKEND_NEUTRALITY.md)

Yggdrasil 目前有 SQLite-backed append-only event log，但平台契约不能是 SQLite-only。Storage Backend Neutrality Alpha 的结论是：SQLite、PostgreSQL、TDB、对象存储和向量/多模态检索都必须位于 backend/provider 层，而不是成为 kernel ontology 或公开协议里的数据库产品。

## 分层模型

1. **Kernel event spine**：内容无关的 append、replay、range、kind-prefix、subscription 与 rehydrate。当前实现有 in-memory 与 SQLite；未来 PostgreSQL 可以成为 host/runtime backend，但不改变协议。
2. **Package-scoped state store**：普通能力包可以预览 package-owned document/KV 状态契约，但不能获得 raw DB、SQL 或 backend credential。
3. **Blob / asset store**：大对象应通过 content address、hash、size、mime 和 provenance 进入平台；blob content 不应塞进 event payload。
4. **Projection / index materialization**：projection/index 是 event/assets 派生的 package-owned view。它可以计划 materialization、query preview 和 migration plan，但不暴露 table/SQL/query product。
5. **Retrieval / vector / multimodal providers**：TDB、pgvector、OpenSearch、Redis Vector、本地 embedding index 和远端检索服务都是 future provider slots。检索是 package/provider 层能力，不替代 event log、audit、proposal lifecycle 或 branch lineage。
6. **Forge observability**：Web shell 只通过 public protocol 调用 `official/storage-lab` 展示 contract summaries，不读 SQLite、PostgreSQL、TDB、文件系统或 runtime internals。

## 红线

- 不新增 `kernel.sqlite.*`、`kernel.postgres.*`、`kernel.tdb.*`、`kernel.vector.*`、`kernel.embedding.*`、`kernel.collection.*`、`kernel.sql.*`、`kernel.database.*`。
- 不把 `EventStore` 扩成万能 `DatabaseBackend`。
- 不向 package 暴露 SQL、DSN、connection string、table、transaction isolation、ANN index、vector dimension、backend topology 或 raw credential。
- SQLite 是 early/default/local backend，不是平台契约。
- PostgreSQL 是 future server/team backend，不是 package API。
- TDB 是 future multimodal retrieval provider slot，不是 kernel database。
- Retrieval/vector/multimodal search 不能替代 append-only events、audit、proposal lifecycle 或 branch/fork/replay。

## `official/storage-lab`

`official/storage-lab` 是普通 manifest-loaded package，用来证明 storage/data contract 可以通过能力包表达，而不是进入 kernel database namespace。

能力分组：

- storage contract：`describe_storage_contract`、`describe_backend_classes`
- package state：`plan_package_state_store`、`put_document_preview`、`get_document_preview`、`query_document_prefix_preview`、`delete_document_tombstone_preview`、`export_store_snapshot_preview`
- blob / asset：`describe_blob_store_contract`、`put_blob_preview`、`get_blob_metadata_preview`、`export_blob_manifest_preview`
- projection / index：`describe_projection_store_contract`、`plan_projection_materialization`、`query_projection_preview`、`migrate_projection_plan_preview`
- retrieval / multimodal：`describe_retrieval_provider_contract`、`draft_multimodal_index_plan`、`draft_vector_search_plan`、`explain_retrieval_backend_fit`

这些能力全部是 deterministic preview / plan-only：

- 不写真实 DB
- 不读写文件
- 不联网
- 不生成 embedding
- 不存 vector
- 不 materialize projection
- 不保存 blob content
- 不返回 raw backend secret

## Forge Storage Inspector

`clients/web/src/storage/storage-inspector.ts` 通过 public protocol 调用 `official/storage-lab`，在 Forge 中展示：

- event spine 与 backend class 摘要
- package-scoped state plan
- blob/asset content-addressed contract
- projection/index materialization contract
- retrieval/TDB future provider slot
- multimodal index plan preview

Assistant drawer 也提供轻量 storage lane。它只展示 contract/readiness，不执行数据库操作。

## 当前验证

Storage Backend Neutrality Alpha 结束时：

- `cargo test --workspace` 通过
- `cargo run -p ygg-cli -- conformance` 通过，310 个具名用例
- `cargo run -p ygg-cli -- conformance --tag storage` 通过
- `cargo run -p ygg-cli -- package check packages/official/storage-lab/manifest.yaml` 通过
- Web TypeScript 检查通过

## 下一步

未来真正接入 PostgreSQL、TDB 或其他 retrieval/vector 后端前，应先补：

- backend selection / host policy
- migration/export/import contract
- quota / retention / compaction policy
- content-addressed blob persistence
- projection rebuild scheduling
- retrieval provider permission/audit/redaction

这些都应继续走 package/provider + host policy，而不是把数据库产品语义放进 kernel。
