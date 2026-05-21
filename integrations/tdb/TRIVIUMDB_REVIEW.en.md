# TriviumDB / TDB Integration Review

> [English](./TRIVIUMDB_REVIEW.en.md) · [中文](./TRIVIUMDB_REVIEW.md)

This ledger records what the current `/workspace/Yggdrasil/TriviumDB` source means for Yggdrasil integration.

## Source observations

- `Cargo.toml`: crate name `triviumdb`, version `0.7.1`, `crate-type = ["cdylib", "rlib"]`, no default features, optional `python` / `nodejs` / `cli` features.
- `src/lib.rs`: exposes `Database`, `Config`, `SearchConfig`, `StorageMode`, `VectorType`, `Filter`, `SearchHit`, and related types.
- `src/database/config.rs`: `StorageMode::{Mmap, Rom}`, `Config { dim, sync_mode, storage_mode }`, and `SearchConfig` with `top_k`, `expand_depth`, hybrid text, DPP, PPR, payload filters, and more.
- `src/database/mod.rs`: `Database<T>::open/open_with_config` opens local files, creates directories, lock files, and WAL; supports `insert`, `insert_with_id`, `link`, `begin_tx`, `search`, `search_advanced`, `search_hybrid`, and `search_hybrid_with_context`.
- `src/database/transaction.rs`: transactions use dry-run + WAL-first commit semantics; WAL replay handles insert/link/delete/update operations.
- `triviumdb.d.ts`: Node binding exposes the `TriviumDB` class, vectors, payloads, filters, search config, and search hits.

## Integration judgment

TDB fits Yggdrasil as a **retrieval/multimodal provider adapter**, not as:

- kernel event store;
- canonical asset store;
- projection authority;
- raw package database;
- global memory/chat/agent/world store.

Reason: TDB is valuable as a local embedded multimodal/vector/graph/document hybrid retrieval engine; Yggdrasil events, proposals, permissions, and branch lineage still need the event spine as authority.

## Why the real crate is not linked by default

The current TDB source is a sibling checkout outside the Yggdrasil repository: `/workspace/Yggdrasil/TriviumDB`. If Yggdrasil committed a a sibling path dependency to the local TriviumDB checkout dependency in `Cargo.toml`, ordinary clones and CI would fail when that sibling path is absent.

Therefore the current default only implements:

- `official/tdb-retrieval-lab` deterministic fake / plan-only adapter;
- `describe_real_tdb_opt_in_seam` real wiring notes;
- Forge UI readiness display;
- conformance proving no default linkage, backend open, index creation, or embedding generation.

Real wiring should wait until:

1. TDB is resolvable (crates.io, git rev, submodule/vendor, or independent subprocess adapter);
2. host policy explicitly enables it;
3. backend path is a host ref and never enters events/proposals/logs/public diagnostics;
4. resource limits are declared: dimension, max nodes, payload bytes, query top_k, expand_depth;
5. indexing/query execution has approval/audit/redaction;
6. the adapter remains an ordinary package/provider with no official priority.

## Recommended real-mode order

1. **Subprocess adapter package**: preferred. It isolates native dependency, file lock, panic, repair, and compaction lifecycles, and can use cross-platform binaries or Node/Python bindings.
2. **Feature-gated in-process adapter**: only after TDB is vendored or published and the host explicitly accepts native in-process risk.
