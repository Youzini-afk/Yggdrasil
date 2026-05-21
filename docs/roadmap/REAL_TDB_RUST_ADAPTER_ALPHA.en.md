# Real TDB Rust Adapter Alpha (Temporary Plan)

> [English](./REAL_TDB_RUST_ADAPTER_ALPHA.en.md) · [中文](./REAL_TDB_RUST_ADAPTER_ALPHA.md)

This plan corrects the previous overly conservative TDB integration. The TriviumDB README explicitly recommends Rust-side `cargo add triviumdb`, so this line must deliver a **real Rust API adapter proof**, not another plan-only placeholder.

## Goals

- Actually call TriviumDB Rust APIs: `Database::open/open_with_config`, `insert`, `link`, `search/search_hybrid`.
- Keep TDB as an ordinary retrieval / multimodal provider adapter, not kernel ontology and not `EventStore`.
- Do not make default Yggdrasil builds / CI depend on a local TDB checkout.
- Allow local opt-in proof through a path dependency to `/workspace/Yggdrasil/TriviumDB`.

## Red lines

- Do not add `kernel.tdb.*`, `kernel.vector.*`, `kernel.embedding.*`, `kernel.database.*`, or `kernel.sql.*`.
- Do not make TDB the event authority, permission audit, proposal lifecycle, branch lineage, or canonical asset store.
- Do not let Web read TDB, local files, or runtime internals directly.
- Do not accept raw paths / backend topology in package input; real adapter uses bounded temp stores or host-ref shapes.
- Default `cargo test --workspace` must not require a TriviumDB checkout.

## Phase R0 — Plan / API correction

- Re-read TriviumDB README / APIs and acknowledge that Rust crate integration is the intended route.
- Add this plan and switch status entry points.
- Keep the previous `official/tdb-retrieval-lab` plan/contract capabilities, but reposition them as prerequisites for a real adapter route.

## Phase R1 — Independent adapter crate shell

- Add `integrations/tdb/rust-adapter/` outside the main workspace members.
- Default build provides a safe stub / JSON-RPC stdio package handler with no TriviumDB dependency.
- Add an explicit subprocess package manifest (not autoloaded), proving an ordinary package can load the adapter shell.
- Validate that default builds/conformance do not need TDB.

## Phase R2 — Real TriviumDB API proof

- Add a `real-tdb` feature and local path-dependency config to the adapter crate.
- Use TriviumDB `Database<f32>` for a real proof: open temp `.tdb`, insert two nodes, link them, search, and search_hybrid.
- Keep the real proof in adapter-crate opt-in tests / smoke command, outside the default main workspace.
- Validate: `cargo test --manifest-path integrations/tdb/rust-adapter/Cargo.real-tdb.local.toml --features real-tdb`.

## Phase R3 — Package capability + conformance boundary

- Add adapter stdio capabilities: `describe_real_tdb_adapter`, `run_real_tdb_smoke`.
- When real feature is disabled, return `real_tdb_available=false` instead of pretending success.
- When real feature is enabled locally, return a real open/insert/link/search summary without raw paths.
- Add CLI conformance for default safe shell, package check, disabled real proof; real opt-in conformance only runs when the environment allows it.

## Phase R4 — Docs / UI / cleanup

- Forge Storage Inspector shows real Rust adapter available/disabled/opt-in proof status.
- Update `docs/guides/POSTGRES_TDB_INTEGRATION*.md` and `integrations/tdb/TRIVIUMDB_REVIEW*.md`.
- Delete this temporary plan and fold durable content into guides.
- Run final validation, commit, and push.
