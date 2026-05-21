# Performance and Code Health Guide

> [English](./PERFORMANCE_AND_CODE_HEALTH.en.md) · [中文](./PERFORMANCE_AND_CODE_HEALTH.md)

This is the durable guide after Performance & Code Health Beta. It replaces the temporary phase plan and records the measurement, feedback-loop, structure, event-store, and web-rendering discipline future Yggdrasil optimization should follow.

## Principles

1. **Measure before optimizing.** Use `ygg perf baseline`, conformance timing, Web TypeScript diagnostics, and focused tests before changing architecture.
2. **Optimization must not change the platform contract.** Official and third-party packages must keep sharing the same manifest, capability, permission, hook, schema, redaction, and audit path.
3. **UI stays on the public protocol.** The web shell must not read SQLite, runtime internals, or special-case official packages.
4. **Do not introduce content ontology in the name of performance.** Do not add `kernel.agent.*`, `kernel.model.*`, `kernel.memory.*`, `kernel.experience.*`, `kernel.sharing.*`, or similar product/content namespaces.
5. **Advanced optimization needs evidence.** Capability/surface caches, RawValue, registry helpers/codegen, per-domain crates, and similar changes require baseline or profiling evidence.

## Common commands

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

## Baseline scope

`ygg perf baseline` currently covers:

- Rust in-process capability invocation.
- Ordinary official package capability invocation.
- Subprocess echo invocation when Python is available.
- In-memory event store append/list/range.
- P3 scale scenarios: 1k / 10k / 100k events.
- Composition check.
- Profile YAML load.

The frontend side provides a pure TypeScript Forge render diagnostics helper at `clients/web/src/performance/render-diagnostics.ts`. It uses mock public-protocol events to record HTML bytes and elapsed_ms for 50/500 events. It does not connect to a host or read SQLite/runtime internals.

See [`BASELINE.en.md`](./BASELINE.en.md) for fields and limitations.

## Conformance feedback loop

Conformance now supports:

- `--list`: list case ids, tags, and descriptions.
- `--case <pattern>`: run cases matching a substring.
- `--tag <tag>`: run cases matching a tag.
- `--fail-fast`: stop after the first failure.
- `--slowest <N>`: print the slowest cases.
- Per-case duration in normal output.

New conformance cases must declare tags so the suite does not become an unfilterable serial script again. See [`CONFORMANCE_FEEDBACK.en.md`](./CONFORMANCE_FEEDBACK.en.md).

## Structure discipline

Performance & Code Health Beta completed these low-risk structural improvements:

- Protocol dispatch split into domain helpers while preserving `KernelMethod` as the source of truth.
- Official in-process dispatch moved from a linear chain to a provider-indexed table while preserving package-aware routing and avoiding official fast paths.
- Shared in-process safety helper for raw-secret and rejection logic.
- Composition/package diagnostics use sets/indexes to avoid obvious O(n²) scans.

Future structural splits should keep:

- Public protocol shapes unchanged.
- Replacement/no-official-priority conformance passing.
- No hard-to-review macros or generated artifacts as the sole truth.

## Event store / replay discipline

Performance & Code Health Beta completed:

- `EventStore::append_with_sequence` atomic append API.
- No duplicate sequence numbers under concurrent same-session append for SQLite and in-memory stores.
- `list_kind_prefix` / `list_session_kind_prefix` query pushdown.
- SQLite `kind` and `session+kind+sequence` indexes.
- Permission/outbound audit paths no longer routinely use `list_all()` plus full filtering.

Storage Backend Neutrality Alpha S1 added:

- `EventStore` trait documentation clarifies backend-neutral event spine contract positioning: `append_with_sequence` is the runtime-recommended append path; `append` + `next_sequence` is the low-level/test/admin path; ordering semantics are per-session `(session_id, sequence)`; kind-prefix queries are event-semantic queries, not SQL/index product APIs; no SQL/table/vector/DSN concepts in the contract.
- In-memory and SQLite conformance parity: 6 `storage_backend` tag conformance cases covering basic contract, kind-prefix equivalence, concurrent append no duplicates, subscription broadcast, and rehydrate event replay semantic identity.

Storage Backend Neutrality Alpha S2 added:

- `official/storage-lab` ordinary package provides package-scoped storage/data contract preview: 8 capabilities, 3 surfaces, 10 `storage_lab` tag conformance cases. Proves storage is an ordinary package-layer capability, not a kernel database/sql/vector API.
- Layered contract model: event spine backend / package state store / blob store future / projection index future / retrieval provider future.
- Backend class candidates contain capability flags only, no path/DSN/credentials.
- Document CRUD preview outputs write/read/query/delete/snapshot_performed=false with redacted content.

Storage Backend Neutrality Alpha S3 added:

- `official/storage-lab` adds 4 blob/asset store contract proof capabilities: describe_blob_store_contract, put_blob_preview, get_blob_metadata_preview, export_blob_manifest_preview. 12 capabilities, 16 `storage_lab` tag conformance cases.
- Blob contract outputs content-addressed type, backend candidates (local_content_addressed_future / filesystem_backend_future / object_store_future), red lines (no blob content in events / no raw secrets / no filesystem path leak / content address required).
- put_blob_preview outputs content_address (sha256: normalized if content_hash provided, deterministic hash otherwise), blob_stored=false, filesystem_performed=false, network_performed=false, event_payload_contains_blob=false. Blocks raw secret, unsafe id, oversized inline sample (>4096 chars).
- No real blob store implementation, no filesystem reads/writes, no network, no blob content in event payloads.

Storage Backend Neutrality Alpha S4 added:

- `official/storage-lab` adds 4 projection/index materialization contract proof capabilities: describe_projection_store_contract, plan_projection_materialization, query_projection_preview, migrate_projection_plan_preview. 16 capabilities, 22 `storage_lab` tag conformance cases.
- Projection contract outputs backend candidates (event_derived_projection / package_owned_index / sqlite_materialized_view_future / postgres_materialized_view_future), red lines (no_table_exposure / no_sql_exposure / no_backend_credentials / no_query_product_leakage / projection_derives_from_events_assets_only).
- plan_projection_materialization outputs materialized=false, write_performed=false, backend_selected=false, plan_only=true. Blocks raw secret, validates projection_id/package_id safe-id.
- query_projection_preview outputs query_executed=false, rows_returned=false, preview_shape. No SQL/table/collection/vector terms.
- migrate_projection_plan_preview outputs migration_applied=false, data_rewritten=false, requires_rebuild=true.
- No real projection storage, no DB table/index creation, no SQL/query execution, no data rewrite.

Storage Backend Neutrality Alpha S5 added:

- `official/storage-lab` adds 4 retrieval/vector/multimodal provider contract proof capabilities: describe_retrieval_provider_contract, draft_multimodal_index_plan, draft_vector_search_plan, explain_retrieval_backend_fit. 20 capabilities, 29 `storage_lab` tag conformance cases.
- Retrieval contract outputs backend candidates (tdb_future / pgvector_future / local_embedding_index_future / remote_vector_provider_future / opensearch_vector_future / redis_vector_future), red lines (no_embedding_generation / no_vector_storage / no_network / no_credentials / no_kernel_vector_namespace / no_raw_vectors_in_output / no_distance_metric_leakage).
- draft_multimodal_index_plan outputs embedding_generated=false, index_created=false, vectors_stored=false, network_performed=false, plan_only=true. Blocks raw secret, validates package_id/index_id safe-id, modalities allow text/image/audio/video/structured only, asset_refs capped at 64.
- draft_vector_search_plan outputs search_executed=false, embedding_generated=false, vectors_loaded=false, plan_only=true. No actual search results.
- explain_retrieval_backend_fit outputs fit matrix without DSN/credentials/path. TDB is only a future multimodal provider slot.
- No real vector DB/TDB/embedding implementation, no raw vector/embedding/credentials/DSN output, no new kernel vector/database/sql namespace.

Future event-store optimization priority:

1. Prove a concrete scale bottleneck with baseline data.
2. Prefer query/index/transaction improvements over event payload contract changes.
3. Keep event payloads opaque; do not put content semantics into kernel query layers.
4. Do not bypass redaction, schema validation, hooks, or audit.

## Web render discipline

Performance & Code Health Beta completed:

- 16ms render scheduler to avoid repeated full renders during SSE/action bursts.
- Bounded JSON previews limiting depth, array items, object keys, and string length.
- Display caps for Forge events/proposals/assets/projections/surfaces.
- Event/proposal/surface/projection payloads render as preview details by default.
- Pure TypeScript Forge render diagnostics helper.

Future web optimization priority:

1. Prove the slow path with render diagnostics or a browser profiler.
2. Prefer partitioning view-models / renderers / detectors before changing UI frameworks.
3. Collapse large payloads by default; expand on demand.
4. Batch/debounce SSE bursts.
5. Forbid runtime-internal or SQLite reads from the web client.

## When to consider advanced optimization

Consider cache/codegen/RawValue-like optimizations only when:

- Baseline or profiler data proves the path is a bottleneck.
- The optimization does not change the public protocol or package equality.
- Redaction/schema/hook/audit behavior remains explicit and reviewable.
- Conformance or unit tests cover invalidation, mismatch, and hostile paths.

There is currently no evidence requiring heavy codegen, RawValue rewrites, arenas, or official-package fast paths; keep them deferred.
