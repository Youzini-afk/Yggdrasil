# PostgreSQL + TDB Integration

> [English](./POSTGRES_TDB_INTEGRATION.en.md) · [中文](./POSTGRES_TDB_INTEGRATION.md)

This guide records the final state of PostgreSQL + TDB Integration Alpha and Real TDB Rust Adapter Alpha: PostgreSQL is an optional host-owned `EventStore` backend; TDB/TriviumDB is an ordinary retrieval/multimodal provider adapter path with a real Rust API adapter proof, not a kernel database.

## PostgreSQL event store

`ygg-runtime` provides a feature-gated `PostgresEventStore`:

- feature: `postgres`
- driver: `tokio-postgres` + `deadpool-postgres`
- schema: `events` table, `unique(session_id, sequence)`, session/range/kind/session+kind indexes, payload/metadata as JSONB
- per-session sequence: session-scoped advisory lock + `max(sequence)+1` + unique constraint inside a transaction
- subscribe: host-local broadcast for now, not PostgreSQL LISTEN/NOTIFY
- default: disabled; ordinary builds, CI, and conformance are unaffected

Real PostgreSQL conformance only runs when explicitly requested:

```bash
cargo run -p ygg-cli --features postgres -- conformance --tag postgres
```

with the host-owned test environment variable set:

```text
YGG_POSTGRES_TEST_DATABASE_URL
```

The value must never be written to package manifests, events, proposals, logs, or public diagnostics.

## Host profile backend selection

Host profiles can select event-store backends:

```yaml
event_store:
  kind: memory
```

```yaml
event_store:
  kind: sqlite
  path: ./forge-alpha.sqlite
```

```yaml
event_store:
  kind: postgres
  env: YGG_POSTGRES_DATABASE_URL
```

Example:

```text
profiles/forge-postgres.example.yaml
```

The profile references only an env var name; the real connection details belong only to the host runtime. Host stdout diagnostics show backend kind plus redacted status only.

## TDB / TriviumDB route

The TDB source review lives at:

```text
integrations/tdb/TRIVIUMDB_REVIEW.en.md
```

Conclusion: TriviumDB/TDB fits as a retrieval / multimodal provider adapter, not as:

- kernel event store;
- canonical asset store;
- projection authority;
- raw package database;
- global memory/chat/agent/world store.

TDB is valuable as a local embedded vector/graph/document/multimodal hybrid retrieval engine; Yggdrasil events, permissions, proposals, branch lineage, and audit still need the event spine as the authoritative substrate.

## `official/tdb-retrieval-lab`

New ordinary official package:

```text
packages/official/tdb-retrieval-lab
```

Capabilities:

```text
describe_tdb_retrieval_contract
draft_tdb_index_plan
draft_tdb_query_plan
explain_tdb_backend_fit
inspect_tdb_adapter_surface
describe_real_tdb_opt_in_seam
```

This package remains the deterministic / no-execution / plan/contract layer:

- no real TDB crate linkage (real calls are handled by the `tdb-rust-adapter` opt-in proof)
- no backend open
- no index creation
- no embedding generation
- no vector storage
- no network
- no filesystem access
- no raw backend secret saved or returned

Real TDB wiring is handled by `official/tdb-rust-adapter` and `integrations/tdb/rust-adapter-real-local`; `tdb-retrieval-lab` stays as the default-safe contract/plan layer.

## `official/tdb-rust-adapter`

Explicitly loaded ordinary subprocess package:

```text
examples/packages/tdb-rust-adapter/manifest.yaml
```

Adapter source:

```text
integrations/tdb/rust-adapter
integrations/tdb/rust-adapter-real-local
```

Default adapter:

- can be loaded by the Ygg runtime as an ordinary subprocess package;
- provides `describe_real_tdb_adapter` and `run_real_tdb_smoke`;
- has no `triviumdb` dependency;
- opens no backend;
- makes `run_real_tdb_smoke` return `real_tdb_available=false` and `smoke_executed=false`.

Real local proof:

```bash
cargo test --manifest-path integrations/tdb/rust-adapter-real-local/Cargo.toml --features real-tdb
```

That proof uses the local `/workspace/Yggdrasil/TriviumDB` path dependency and actually calls:

```text
Database::<f32>::open_with_config
insert
link
search
search_hybrid
```

It uses a temporary redacted store, exposes no raw path, performs no network, and does not enter the default workspace build. The real crate is not linked by default not because the work is skipped, but because ordinary clones/CI must not be bound to a local sibling checkout.

Recommended real-mode order:

1. **subprocess adapter package**: preferred. It isolates native dependency, file lock, panic, repair, and compaction lifecycles.
2. **feature-gated in-process adapter**: only when TDB is resolvable in a stable way (published, vendored, submodule, or pinned git rev) and the host explicitly accepts native in-process risk.

Example profile shape:

```text
examples/tdb-provider-profiles/tdb-local.example.json
```

## UI

Forge Storage Inspector calls through public protocol:

```text
official/storage-lab
official/tdb-retrieval-lab
official/tdb-rust-adapter (only when explicitly loaded)
```

It displays:

- event spine / backend classes
- package state / blob / projection contracts
- retrieval provider slot
- TDB adapter contract
- real TDB opt-in seam readiness
- real TDB Rust adapter shell / real-local proof status

The Web shell does not read SQLite/PostgreSQL/TDB, local filesystems, or runtime internals.

## Red lines

Do not add:

```text
kernel.postgres.*
kernel.sql.*
kernel.database.*
kernel.tdb.*
kernel.vector.*
kernel.embedding.*
```

Packages must not receive raw PostgreSQL pools, SQL, DSNs, TDB paths, backend topology, or raw backend errors.

## Validation

At Alpha completion:

- `cargo test --workspace` passes
- `cargo run -p ygg-cli -- conformance` passes with 320 named CLI cases
- `cargo run -p ygg-cli -- conformance --tag storage` passes
- `cargo run -p ygg-cli -- conformance --tag tdb` passes
- `cargo run -p ygg-cli -- package check packages/official/tdb-retrieval-lab/manifest.yaml` passes
- `cargo run -p ygg-cli -- package check examples/packages/tdb-rust-adapter/manifest.yaml` passes
- `cargo test --manifest-path integrations/tdb/rust-adapter/Cargo.toml` passes
- `cargo test --manifest-path integrations/tdb/rust-adapter-real-local/Cargo.toml --features real-tdb` passes
- `cargo check -p ygg-cli --features postgres` passes
- Web TypeScript passes
