# Performance & Code Health Beta

> [English](./PERFORMANCE_CODE_HEALTH_BETA.en.md) · [中文](./PERFORMANCE_CODE_HEALTH_BETA.md)

This is a temporary execution plan. Delete it when complete and converge durable content into `ALPHA_STATUS`, `NEXT_STEPS`, performance guides, and relevant specs.

## Why now

Experience-Led Platform Beta 0–6 moved Yggdrasil from foundation-first infrastructure into an experience loop. Before the first real product, the largest risk is no longer missing substrate; it is growth cost:

- The conformance feedback loop is heavy; 245+ cases slow every product iteration.
- Web Forge / Agent panels still lean toward full recomputation and full rendering.
- SQLite event store / replay will become a real product bottleneck as sessions grow.
- Official in-process handlers, protocol dispatch, CLI commands, and TS panels can return to giant-file risk.
- JSON / clone / string routing costs exist, but they must be measured first.

External references support this direction: the Rust Performance Book emphasizes profiling and benchmarking; React docs emphasize profiling slow interactions before memoization; SQLite docs emphasize WAL, checkpoints, and indexes; OpenTelemetry benchmark docs emphasize fixed scenarios, repeated measurements, CPU/memory/throughput/latency reports.

## Red lines

- No official-package fast path; `official/*` and third-party packages keep equal routing and permissions.
- Do not bypass permission, hooks, schema validation, redaction, or audit.
- Web must not read SQLite or runtime internals; UI stays public-protocol-only.
- Do not add `kernel.experience.*`, `kernel.memory.*`, `kernel.sharing.*`, `kernel.marketplace.*`, `kernel.agent.*`, or `kernel.model.*`.
- No large proc-macro / heavy codegen / arena / RawValue rewrite unless the baseline proves it is needed.
- Do not replace the JSON boundary with private typed fast paths.

## Phase P0 — Baseline & Measurement (complete)

Goal: establish facts before optimizing.

Deliverables:

- `ygg perf baseline` CLI emitting deterministic baseline JSON / markdown summary.
- Measures in-process invoke, official capability invoke, subprocess echo (may be skipped), event store append/list/range (100 events), composition check, profile load.
- `--iterations` and `--format text|json` parameters.
- `docs/performance/BASELINE.md` and `.en.md` record commands, environment, sample sizes, and budgets.
- Default no-network; no real provider required.

Acceptance: repeatable baseline command; docs list metrics future optimizations must track; workspace tests, conformance, and doc links pass.

Reference: [`docs/performance/BASELINE.en.md`](../performance/BASELINE.en.md)

## Phase P1 — Conformance Feedback Loop (complete)

Goal: make conformance filterable, timed, and diagnosable.

Deliverables: `--list`, `--case <pattern>`, `--tag <tag>`, `--fail-fast`, per-case duration, slowest-N report, and case tags (runtime, event, capability, package, subprocess, official, generated, network, outbound, stream, agentic, experience, memory, sharing, secret, composition, replacement, surface, protocol, permission, hook, host, asset, projection, substrate, live, slow, etc.). Structured `ConformanceCase { id, tags, run }` registry replaces the former `record_case` calls. Default `ygg conformance` still runs all 245 cases.

Acceptance: default `ygg conformance` still runs all cases; `--list` prints ids and tags; single-case (`--case sharing_lab.contract_shape`) and tag-filter (`--tag sharing`) runs work; new cases must declare tags; output includes per-case duration, slowest-N report, and failure location.

Reference: [`docs/performance/CONFORMANCE_FEEDBACK.en.md`](../performance/CONFORMANCE_FEEDBACK.en.md)

## Phase P2 — Low-risk Structural Split (complete)

Goal: control growth without changing external behavior.

Deliverables:

- Split `runtime/protocol_dispatch.rs` by domain into focused helper functions while keeping `KernelMethod` as source of truth. Top-level match delegates to host/surface/outbound/permission/proposal/session/event/package/capability/extension/hook/asset/projection helpers.
- Replace linear official in-process `try_handle` chain with `provider_package_id` indexed dispatch. `dispatch_official` matches `provider_package_id.as_str()` for direct dispatch to the corresponding module; unknown official packages fall through to `common::try_handle`; non-official packages are never served by the official fallback.
- Safety helper convergence: the 5 inproc labs (agentic_forge, experience_observability, memory, playable_creation_board, sharing) with identical `is_secret_ref_value` / `looks_like_raw_secret_value` / `contains_raw_secret` now share a `inproc/safety.rs` module. Marketplace/billing/signing field checks remain local to `sharing_lab`. No change to rejection output text or JSON shape.
- Replace obvious `.iter().any()` in composition/package diagnostics with `BTreeSet`/`BTreeMap`/`HashSet` index lookups. Suffix/contains semantics carefully preserved via helpers.
- Web `forge.ts` uses `surfacesByPackage` group index for `packagesWithoutSurfaces`, avoiding repeated O(packages × surfaces) filter.

Acceptance: public protocol unchanged; replacement/no-official-priority conformance still passes; no macro/codegen; workspace tests, conformance, and package checks pass.

## Phase P3 — Event Store & Replay Optimization (complete)

Goal: harden durable substrate for real product sessions.

Deliverables:
- `EventStore::append_with_sequence` atomic append API: inputs are session_id, writer_package_id, kind, schema_version, payload_json, metadata_json; output is the inserted EventEnvelope. Default implementation uses `next_sequence + append`; `SqliteEventStore` override reads max sequence, constructs event, and inserts within the same connection mutex; `InMemoryEventStore` override allocates sequence and pushes within the same write lock. Guarantees no duplicate sequences under concurrent same-session access.
- `EventStore::list_kind_prefix` and `list_session_kind_prefix` query APIs: default implementation lists and filters; SQLite override uses SQL range/LIKE pushdown; InMemory override uses single read+filter. Stable ordering preserved.
- SQLite indexes: `kind`, `session+kind+sequence` for audit/range query pushdown.
- `append_event_unchecked` uses store-level atomic append; hook veto/schema failure no longer consumes a sequence.
- `dispatch_permission_audit()` uses `list_kind_prefix("kernel/permission")` pushdown instead of `list_all()` + filter.
- `list_outbound_audit()` uses `list_session_kind_prefix(session, "kernel/outbound")` pushdown instead of `list_session()` + full filter.
- Concurrent append correctness test: 50 concurrent appends to the same session produce contiguous sequences with no duplicates.
- `ygg perf baseline` extended with event scale scenarios: `event_store_append_list_range_1k` (1,000 events), `event_store_append_list_range_10k` (10,000 events), `event_store_append_list_range_100k` (100,000 events, auto-capped to 1 iteration when >1 requested).
- `docs/performance/BASELINE*` updated with new event scale metrics.

Acceptance: concurrent same-session append is stable; SQLite-backed substrate rehydrate tests pass; audit/permission/proposal/event conformance passes; no redaction/schema/hook bypass.

## Phase P4 — Web Render & UI Organization

Goal: make Forge usable under realistic product data.

Delivered:
- `clients/web/src/main.ts` adds a 16ms render scheduler so SSE / action bursts no longer trigger immediate repeated full renders.
- `clients/web/src/utils/html.ts` adds a bounded JSON preview helper limiting depth, array items, object keys, and string length so large payloads are not fully stringified by default.
- `clients/web/src/surfaces/forge.ts` caps displayed events, proposals, assets, projections, and surfaces, and renders event/proposal/surface/projection payloads as preview details.
- `clients/web/src/performance/render-diagnostics.ts` adds a pure TS Forge render diagnostics helper that records HTML bytes and elapsed_ms for 50/500 mock events.

Acceptance: Web TypeScript passes; public-protocol-only; no SQLite/runtime internals; 500-event Forge mock/render metrics recorded in the baseline docs.

## Phase P5 — Evidence-based Advanced Optimization & Cleanup

Goal: apply only evidence-backed advanced optimizations and remove this temporary plan.

Optional deliverables: capability/surface resolve cache with load/unload invalidation; manifest-derived handler coverage tests; limited registry helpers/codegen; RawValue only for proven large-payload pass-through paths and never bypassing redaction/schema/hooks; performance guide, budgets, and CI-friendly commands.

Acceptance: temporary plan deleted; `docs/performance/`, `ALPHA_STATUS`, `NEXT_STEPS`, and `CONFORMANCE_MATRIX` converged; final workspace tests, conformance, Web TS, doc links, and diff check pass.
