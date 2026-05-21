# PostgreSQL + TDB Integration Alpha

> [English](./POSTGRES_TDB_INTEGRATION_ALPHA.en.md) · [中文](./POSTGRES_TDB_INTEGRATION_ALPHA.md)

This is a temporary execution plan. Delete it after completion and fold durable content into `ALPHA_STATUS`, `NEXT_STEPS`, `STORAGE_BACKEND_NEUTRALITY`, the conformance matrix, and package docs.

## Research conclusion

This phase is based on three research tracks:

- PostgreSQL/Rust: `sqlx` fits Yggdrasil's async event-store backend best because it provides pooling, migrations/test support, a rustls path, and stable SQL calls. `tokio-postgres` is lower-level but needs separate pooling/migration. Diesel is ORM-oriented and not a good default for the event spine.
- PostgreSQL semantics: sequences/identity are useful as global ordering keys but not for gapless business versions. Per-session sequence should be guaranteed with transaction + session-scoped lock / max(sequence)+1 / unique(session_id, sequence). Default tests must not depend on real PostgreSQL; real backend smoke is feature + env opt-in.
- TriviumDB/TDB: source lives at `/workspace/Yggdrasil/TriviumDB`. It is a Rust crate + cdylib/rlib, Python/Node bindings, and CLI demo/repair tools. It does not currently expose a server/RPC/daemon. Therefore the short-term fit is an ordinary retrieval/multimodal provider adapter or fake-compatible seam, not kernel runtime integration.

## Boundary decision

```text
PostgreSQL = host-owned EventStore backend, real opt-in
TDB = ordinary package/provider retrieval adapter, default fake/deterministic, real crate opt-in later
```

PostgreSQL replaces the SQLite durable event log without exposing SQL to packages. TDB augments package/provider-layer retrieval and multimodal indexing without becoming canonical asset/projection/backend truth.

## Red lines

- Do not add `kernel.postgres.*`, `kernel.sql.*`, `kernel.database.*`, `kernel.tdb.*`, `kernel.vector.*`, or `kernel.embedding.*`.
- Do not turn `EventStore` into a generic `DatabaseBackend`.
- Packages must not access the kernel PostgreSQL connection/pool/events table.
- TDB must not become a kernel asset/projection store.
- pgvector/TDB/vector/embedding semantics must not enter the kernel.
- DSNs, connection strings, DB users, TDB paths/private topology, and raw backend errors must not be written to events, proposals, logs, or public diagnostics.
- Default CI/conformance must not depend on real PostgreSQL/TDB; real backend smoke must be explicit opt-in.

## Phase P0 — Plan, Research, Boundary Freeze

Goal: freeze PostgreSQL/TDB integration boundaries before implementation drift.

Deliverables:

- This temporary bilingual plan.
- README / ALPHA_STATUS / NEXT_STEPS current-line updates.
- External docs and TDB source research folded into the plan.

Acceptance: doc links, diff check, commit/push.

## Phase P1 — PostgreSQL EventStore Backend Proof ✅ COMPLETE

Goal: implement opt-in `PostgresEventStore` for the `EventStore` event spine contract only.

Deliverables:

- ✅ Add a `postgres` feature and `PostgresEventStore` to `ygg-runtime`.
- ✅ Use `tokio-postgres` + `deadpool-postgres` (avoids `sqlx`+`rusqlite` `libsqlite3-sys` links conflict), disabled by default.
- ✅ Schema initialization: events table, unique(session_id, sequence), session/sequence, kind, session+kind indexes. Payload/metadata as JSONB.
- ✅ `append_with_sequence` allocates per-session sequence and inserts inside a transaction with `pg_advisory_xact_lock(hashtext(session_id))`, preserving concurrent no-duplicate sequence behavior.
- ✅ Implement list_all/list_session/range/kind-prefix/session-kind-prefix/next_sequence/subscribe (local broadcast, no LISTEN/NOTIFY yet).
- ✅ Feature-gated / env-gated conformance helper: only run real PG when `YGG_POSTGRES_TEST_DATABASE_URL` is set and the feature is enabled; default CI is unaffected.
- ✅ Redact backend errors and never write DSNs to public output.

Acceptance: workspace tests, default conformance, `cargo check -p ygg-runtime --features postgres`, and opt-in storage conformance when PG is available — all passed.

## Phase P2 — Host/Profile Backend Selection

Goal: let host choose memory/sqlite/postgres backend while keeping backend configuration host-only.

Deliverables:

- Add redacted event-store backend config shape to host profiles.
- CLI/host startup supports postgres backend as feature-gated / env-ref opt-in.
- Diagnostics expose only backend kind/status/redaction, never DSNs or private topology.
- Default behavior remains unchanged.

Acceptance: default host path unchanged; postgres feature compiles; public protocol contains no DSN.

## Phase T1 — TDB Retrieval Adapter Contract/Fake Provider

Goal: prove the TDB path as an ordinary package/provider, not a kernel feature.

Deliverables:

- Add an ordinary package (suggested `official/tdb-retrieval-lab`) as deterministic fake retrieval provider proof.
- Capabilities: describe_tdb_boundary, plan_index_asset_refs, fake_index_asset_refs, fake_search_refs, explain_retrieval_trace, summarize_provider_health.
- Outputs refs/trace/provider_health; no raw vectors/embeddings; no canonical asset/projection mutation.
- Conformance proves no kernel namespace, no raw secrets, deterministic fake index/search, refs only.

Acceptance: package check, conformance, Forge Storage Inspector can show TDB provider readiness.

## Phase T2 — TDB Real-Crate Opt-in Seam

Goal: reserve the real TDB crate integration seam without making default CI or core runtime depend on it.

Deliverables:

- Document real mode prerequisites after reading `/workspace/Yggdrasil/TriviumDB` crate APIs.
- If compatible, add feature-gated path dependency / adapter stub; default remains fake mode.
- If the real crate should not be pulled into the current workspace yet, keep an external adapter guide instead of forcing coupling.
- Update UI/docs/conformance matrix.

Acceptance: default build/test does not require TDB; real mode is opt-in only; no kernel/vector namespace leakage.

## Phase C — Durable Cleanup and Final Validation

Goal: delete temporary plans, converge durable docs, and complete final validation.

Deliverables:

- Delete this plan.
- Update `STORAGE_BACKEND_NEUTRALITY`, ALPHA_STATUS, NEXT_STEPS, README, performance docs, and the conformance matrix.
- Final report with commit sequence, validation results, and next recommendations.

Final validation:

- `cargo test --workspace`
- `cargo run -p ygg-cli -- conformance`
- `cargo run -p ygg-cli -- conformance --tag storage`
- `cargo run -p ygg-cli -- package check packages/official/storage-lab/manifest.yaml`
- new TDB adapter package check
- `cargo check -p ygg-runtime --features postgres`
- `tsc -p clients/web/tsconfig.json --noEmit`
- markdown local links
- `git diff --check`
- temporary plan residue check
