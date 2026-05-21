# Storage Backend Neutrality Alpha

> [English](./STORAGE_BACKEND_NEUTRALITY_ALPHA.en.md) · [中文](./STORAGE_BACKEND_NEUTRALITY_ALPHA.md)

This is a temporary execution plan. Delete it when complete and converge durable content into `ALPHA_STATUS`, `NEXT_STEPS`, the storage/data guide, the conformance matrix, and package docs.

## Why now

Yggdrasil already has a SQLite-backed append-only event log, rehydratable assets/branches/projections/permissions, and `perf baseline` event-store scenarios at 1k/10k/100k events. That is an early local default backend, not the platform contract. Future hosts will need PostgreSQL for server/team deployments and, later, TDB or other multimodal/vector retrieval backends. Those must remain backend/provider layers, not kernel ontology.

External research takeaways:

- SQLite WAL is append-first/checkpoint-later, with one writer, same-host shared memory, checkpointing, large-transaction caveats, and network-filesystem limits. SQLite behavior must not become platform semantics.
- PostgreSQL transactions/WAL are suitable for a future durable remote/team event-store backend, but transactions, SQL, DSNs, and isolation levels must not leak into kernel protocol.
- pgvector, OpenSearch, Redis Vector, and similar systems show vector/multimodal search depends on explicit embedding/model/index/distance/top-k/filter configuration. These are retrieval/index capability-package or backend-provider concerns, not event-spine contract.

## Layer model

1. **Kernel Event Spine**: append, replay, range, kind prefix, subscription, and rehydrate semantics only.
2. **Backend Implementations**: `InMemoryEventStore`, `SqliteEventStore`, future `PostgresEventStore`; host/runtime config, no protocol change.
3. **Package-scoped Data Capability**: ordinary packages expose document/KV/object/index previews without bypassing permission/audit/proposal.
4. **Blob / Asset Store**: future large-object storage outside event payloads; alpha contract proof and content-address previews only.
5. **Retrieval / Vector / Multimodal Providers**: TDB, pgvector, Qdrant, OpenSearch, Redis Vector, etc. as retrieval/vector/multimodal provider packages or backend adapters.
6. **UI / Observability**: Forge shows storage contracts, backend capability flags, risk/diagnostics through public protocol only.

## Red lines

- Do not add `kernel.sqlite.*`, `kernel.postgres.*`, `kernel.tdb.*`, `kernel.vector.*`, `kernel.embedding.*`, `kernel.collection.*`, `kernel.sql.*`, or `kernel.database.*`.
- Do not turn `EventStore` into a universal `DatabaseBackend`; no SQL, table, transaction isolation, connection, DSN, ANN index, vector dimension, or vendor concepts in the kernel contract.
- SQLite, PostgreSQL, and TDB are backend/providers, not platform contracts.
- Vector/multimodal retrieval belongs to package/provider layers, not the kernel.
- Packages must not gain raw DB access or bypass events/assets/proposals/permissions through storage backends.
- DSNs, paths, credentials, and backend topology must not be written to events/proposals/logs/public diagnostics; only coarse redacted diagnostics are allowed.
- No real PostgreSQL/TDB/vector database implementation in this alpha; this is neutrality and contract proof.

## Phase S0 — Research, ADR, Temporary Plan

Goal: lock research conclusions, phase boundaries, and red lines.

Deliverables:

- This temporary bilingual plan.
- README / ALPHA_STATUS / NEXT_STEPS current-headline updates.
- External evidence path: `/tmp/opencode/ygg-storage-neutrality-20260520/` when full-page saves are available; otherwise URLs are recorded in this plan/session evidence.

Acceptance: doc links, diff check, commit/push.

## Phase S1 — EventStore Backend-Neutral Contract Hardening ✅

Goal: make explicit that `EventStore` is the event-spine contract, not a database abstraction.

Deliverables:

- ✅ Update/extend runtime storage contract docs and code comments for backend-neutral semantics of `append_with_sequence`, range replay, kind prefix query, subscription, and rehydrate.
- ✅ Add backend-neutral event-store conformance helpers covering in-memory and SQLite: append/list/range, concurrent append no duplicate, kind prefix equivalence, subscription, and rehydrate parity.
- ✅ Avoid new SQLite-specific runtime dependencies.

Acceptance: workspace tests, storage-tag conformance, baseline event-store scenarios.

## Phase S2 — Package-Scoped Data Contract + `storage-lab`

Goal: provide ordinary package-facing storage/data contract proof, not kernel database APIs.

Deliverables:

- Add ordinary official package `official/storage-lab` (`rust_inproc`).
- Capabilities: `describe_storage_contract`, `describe_backend_classes`, `plan_package_state_store`, `put_document_preview`, `get_document_preview`, `query_document_prefix_preview`, `delete_document_tombstone_preview`, `export_store_snapshot_preview`.
- Outputs are deterministic previews / package-owned data models; no real DB writes, no SQL, no filesystem, no network.
- Profile autoload, surfaces, conformance.

Acceptance: package check, storage-lab conformance, no SQL/kernel database namespace.

## Phase S3 — Blob / Asset Store Contract Proof

Goal: reserve large-object and asset content-addressed backends without stuffing blob content into event payloads.

Deliverables:

- Extend `storage-lab`: `describe_blob_store_contract`, `put_blob_preview`, `get_blob_metadata_preview`, `export_blob_manifest_preview`.
- Backend candidates: local content-addressed, filesystem, future object store; output hash/size/mime/provenance only, no real blob storage.
- Raw-secret and unsafe-path blocking.

Acceptance: conformance covers content-address determinism, no raw secret, no filesystem write.

## Phase S4 — Projection / Index Materialization Contract Proof

Goal: define minimal package-owned projection/index store contracts without turning projections into DB tables.

Deliverables:

- Extend `storage-lab`: `describe_projection_store_contract`, `plan_projection_materialization`, `query_projection_preview`, `migrate_projection_plan_preview`.
- SQLite/Postgres future materialization are backend candidates only; output plans only.
- Align with existing `projection-lab` docs/Forge inspector.

Acceptance: conformance covers no DB table leakage, plan-only, backend-neutral output.

## Phase S5 — Retrieval / Vector / Multimodal Provider Contract

Goal: reserve future TDB/pgvector/OpenSearch/Redis Vector backend slots without implementing them.

Deliverables:

- Extend `storage-lab` or add ordinary retrieval descriptor capabilities: `describe_retrieval_provider_contract`, `draft_multimodal_index_plan`, `draft_vector_search_plan`, `explain_retrieval_backend_fit`.
- Backend candidates include `tdb_future`, `pgvector_future`, `local_embedding_index_future`, and `remote_vector_provider_future`.
- Output redacted plans, asset refs, modality flags, and index capability flags only; no embedding generation, vector storage, or network.

Acceptance: conformance proves no kernel vector namespace, no embedding generation, no backend credentials, and TDB is only a future provider slot.

## Phase S6 — Forge Storage Inspector + Durable Docs Cleanup

Goal: Web exposes storage contracts through public protocol; delete temporary plan and converge durable docs.

Deliverables:

- Forge Storage/Data panel showing contract summaries for event spine, package state store, blob store, projection/index, and retrieval providers.
- Assistant drawer may show a lightweight storage guide hint.
- Add `docs/guides/STORAGE_BACKEND_NEUTRALITY.md` and `.en.md`.
- Delete this temporary plan; update README, ALPHA_STATUS, NEXT_STEPS, CONFORMANCE_MATRIX, and performance docs.

Final validation:

- `cargo test --workspace`
- `cargo run -p ygg-cli -- conformance`
- `cargo run -p ygg-cli -- conformance --tag storage`
- `cargo run -p ygg-cli -- package check packages/official/storage-lab/manifest.yaml`
- `cargo run -p ygg-cli -- perf baseline --iterations 1 --format json`
- `tsc -p clients/web/tsconfig.json --noEmit`
- markdown local links
- `git diff --check`
- temporary plan residue check
