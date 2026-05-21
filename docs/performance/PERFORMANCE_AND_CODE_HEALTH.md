# 性能与代码健康指南

> [English](./PERFORMANCE_AND_CODE_HEALTH.en.md) · [中文](./PERFORMANCE_AND_CODE_HEALTH.md)

本文档是 Performance & Code Health Beta 完成后的持久指南。它取代临时阶段计划，记录 Yggdrasil 后续优化应遵守的测量、反馈环、结构、事件存储与 Web 渲染纪律。

## 原则

1. **先测量，再优化。** 使用 `ygg perf baseline`、conformance timing、Web TypeScript diagnostics 和针对性单元测试证明热点；不要凭感觉替换架构。
2. **优化不得改变平台契约。** 官方包与第三方包必须继续走同一 manifest、capability、permission、hook、schema、redaction、audit 路径。
3. **UI 仍走公开协议。** Web shell 不得读取 SQLite、runtime internals 或 special-case official packages。
4. **不要用性能名义引入内容本体。** 不新增 `kernel.agent.*`、`kernel.model.*`、`kernel.memory.*`、`kernel.experience.*`、`kernel.sharing.*` 等内容/产品命名空间。
5. **高级优化必须有证据。** capability/surface cache、RawValue、registry helper/codegen、per-domain crates 等都必须由 baseline 或 profiling 证明必要。

## 常用命令

```bash
# workspace correctness
cargo test --workspace

# full charter/conformance gate with timings
cargo run -p ygg-cli -- conformance

# list/filter/fail-fast conformance during focused work
cargo run -p ygg-cli -- conformance --list
cargo run -p ygg-cli -- conformance --case sharing_lab
cargo run -p ygg-cli -- conformance --tag experience
cargo run -p ygg-cli -- conformance --fail-fast --slowest 5

# no-network deterministic performance baseline
cargo run -p ygg-cli -- perf baseline
cargo run -p ygg-cli -- perf baseline --format json

# web correctness
tsc -p clients/web/tsconfig.json --noEmit
```

## Baseline 范围

`ygg perf baseline` 当前覆盖：

- Rust in-process capability invoke。
- 官方包普通 capability invoke。
- subprocess echo invoke（Python 可用时）。
- in-memory event store append/list/range。
- P3 scale scenarios：1k / 10k / 100k events。
- composition check。
- profile YAML load。

前端侧由 `clients/web/src/performance/render-diagnostics.ts` 提供 pure TypeScript Forge render diagnostics helper，使用 mock public-protocol events 记录 50/500 events 的 HTML bytes 与 elapsed_ms。它不连接 host、不读取 SQLite/runtime internals。

详细字段见 [`BASELINE.md`](./BASELINE.md)。

## Conformance 反馈环

Conformance 现在支持：

- `--list`：列出 case id、tags、description。
- `--case <pattern>`：按 substring 运行部分 cases。
- `--tag <tag>`：按 tag 运行部分 cases。
- `--fail-fast`：首个失败后停止。
- `--slowest <N>`：输出最慢 cases。
- per-case duration：每个 case 都打印耗时。

新增 conformance case 时必须带 tags，避免反馈环重新变成不可筛选的巨型串行脚本。详见 [`CONFORMANCE_FEEDBACK.md`](./CONFORMANCE_FEEDBACK.md)。

## 结构优化纪律

Performance & Code Health Beta 已完成的低风险结构优化包括：

- protocol dispatch 按 domain helper 拆分，同时保持 `KernelMethod` 单一事实源。
- official in-process dispatch 从线性链改为 provider-indexed table，仍使用 package-aware routing，不给 official fast path。
- shared inproc safety helper 收敛 raw-secret / rejection 逻辑。
- composition/package diagnostics 使用 set/index 避免明显 O(n²) 扫描。

后续结构拆分应继续遵守：

- 公共协议 shape 不变。
- replacement/no-official-priority conformance 必须通过。
- 不为了少写 match 而引入难审计的宏或生成物作为唯一 truth。

## Event store / replay 纪律

Performance & Code Health Beta 已完成：

- `EventStore::append_with_sequence` 原子追加 API。
- SQLite / in-memory 同 session 并发 append sequence 不重复。
- `list_kind_prefix` / `list_session_kind_prefix` 查询下推。
- SQLite `kind` 与 `session+kind+sequence` 索引。
- permission/outbound audit 避免常规 `list_all()` 全量过滤。

Storage Backend Neutrality Alpha S1 新增：

- `EventStore` trait 文档明确 backend-neutral event spine contract 定位：`append_with_sequence` 是 runtime 推荐 append path；`append` + `next_sequence` 是 low-level/test/admin path；ordering semantics 是 per-session `(session_id, sequence)`；kind-prefix query 是 event semantic query 不是 SQL/index product；no SQL/table/vector/DSN concepts。
- In-memory 与 SQLite conformance parity：6 个 `storage_backend` tag conformance 用例覆盖基础契约、kind-prefix 等价性、并发 append 无重复、subscription 广播、rehydrate 事件重放语义一致。

Storage Backend Neutrality Alpha S2 新增：

- `official/storage-lab` 普通包提供 package-scoped storage/data 契约预览：8 项能力、3 个 surface、10 个 `storage_lab` tag conformance 用例。证明 storage 是普通 package 层能力，而非 kernel database/sql/vector API。
- 合约分层模型：event spine backend / package state store / blob store future / projection index future / retrieval provider future。
- Backend class 候选只含 capability flags，不含 path/DSN/credentials。
- Document CRUD preview 输出 write/read/query/delete/snapshot_performed=false，redacted content。

Storage Backend Neutrality Alpha S3 新增：

- `official/storage-lab` 新增 4 项 blob/asset store 契约证明能力：describe_blob_store_contract、put_blob_preview、get_blob_metadata_preview、export_blob_manifest_preview。12 项能力、16 个 `storage_lab` tag conformance 用例。
- Blob 契约输出 content-addressed 类型、backend 候选（local_content_addressed_future / filesystem_backend_future / object_store_future）、red lines（no blob content in events / no raw secrets / no filesystem path leak / content address required）。
- put_blob_preview 输出 content_address（content_hash 提供则 sha256: 规范化，否则 deterministic hash），blob_stored=false，filesystem_performed=false，network_performed=false，event_payload_contains_blob=false。阻断 raw secret、unsafe id、过大 inline sample（>4096 chars）。
- 不实现真实 blob store、不读写文件、不联网、不把 blob content 放入 event payload。

Storage Backend Neutrality Alpha S4 新增：

- `official/storage-lab` 新增 4 项 projection/index 物化契约证明能力：describe_projection_store_contract、plan_projection_materialization、query_projection_preview、migrate_projection_plan_preview。16 项能力、22 个 `storage_lab` tag conformance 用例。
- Projection 契约输出 backend candidates（event_derived_projection / package_owned_index / sqlite_materialized_view_future / postgres_materialized_view_future），red lines（no_table_exposure / no_sql_exposure / no_backend_credentials / no_query_product_leakage / projection_derives_from_events_assets_only）。
- plan_projection_materialization 输出 materialized=false、write_performed=false、backend_selected=false、plan_only=true。阻断 raw secret，校验 projection_id/package_id safe-id。
- query_projection_preview 输出 query_executed=false、rows_returned=false、preview_shape。不含 SQL/table/collection/vector 术语。
- migrate_projection_plan_preview 输出 migration_applied=false、data_rewritten=false、requires_rebuild=true。
- 不实现真实 projection storage，不建 DB table/index，不执行 SQL/query，不重写数据。

Storage Backend Neutrality Alpha S5 新增：

- `official/storage-lab` 新增 4 项 retrieval/vector/multimodal provider 契约证明能力：describe_retrieval_provider_contract、draft_multimodal_index_plan、draft_vector_search_plan、explain_retrieval_backend_fit。20 项能力、29 个 `storage_lab` tag conformance 用例。
- Retrieval 契约输出 backend candidates（tdb_future / pgvector_future / local_embedding_index_future / remote_vector_provider_future / opensearch_vector_future / redis_vector_future），red lines（no_embedding_generation / no_vector_storage / no_network / no_credentials / no_kernel_vector_namespace / no_raw_vectors_in_output / no_distance_metric_leakage）。
- draft_multimodal_index_plan 输出 embedding_generated=false、index_created=false、vectors_stored=false、network_performed=false、plan_only=true。阻断 raw secret，校验 package_id/index_id safe-id，modalities 只允许 text/image/audio/video/structured，asset_refs 上限 64。
- draft_vector_search_plan 输出 search_executed=false、embedding_generated=false、vectors_loaded=false、plan_only=true。无实际搜索结果。
- explain_retrieval_backend_fit 输出 fit matrix，不含 DSN/credentials/path。TDB 只作为 future multimodal provider slot。
- 不实现真实 vector DB/TDB/embedding，不输出 raw vector/embedding/credentials/DSN，不新增 kernel vector/database/sql namespace。

后续 event store 优化优先级：

1. 用 baseline 证明具体规模下的瓶颈。
2. 优先 query/index/transaction 改善，而不是改变 event payload contract。
3. 保持 event payload opaque；不要把内容语义塞进 kernel 查询层。
4. 不绕过 redaction、schema、hook、audit。

## Web render 纪律

Performance & Code Health Beta 已完成：

- 16ms render scheduler，避免 SSE/action burst 连续触发整页 render。
- bounded JSON preview，限制 depth、array items、object keys、string length。
- Forge events/proposals/assets/projections/surfaces 显示 cap。
- event/proposal/surface/projection payload 默认 preview details。
- pure TS Forge render diagnostics helper。

后续 Web 优化优先级：

1. 先用 render diagnostics 或浏览器 profiler 证明慢点。
2. 优先分区 view-model / renderer / detector，而不是替换 UI 框架。
3. 大 payload 默认折叠；需要时再展开。
4. SSE burst 必须 batching/debounce。
5. 禁止 Web 读取 runtime internals 或 SQLite。

## 何时考虑高级优化

只有满足以下条件时才考虑 cache/codegen/RawValue 等高级优化：

- 有 baseline 或 profiler 数据证明某路径是瓶颈。
- 优化不会改变 public protocol 或 package equality。
- redaction/schema/hook/audit 仍显式可审计。
- 有 conformance 或 unit test 覆盖 invalidation / mismatch / hostile path。

当前没有证据要求引入 heavy codegen、RawValue rewrite、arena 或官方包 fast path；这些保持延后。
