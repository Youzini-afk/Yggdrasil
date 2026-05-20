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

## Phase P2 — Low-risk Structural Split

Goal: control growth without changing external behavior.

Deliverables:

- Split `runtime/protocol_dispatch.rs` by domain into focused modules/functions while keeping `KernelMethod` as source of truth.
- Replace linear official in-process `try_handle` chain with explicit registry / handler table while preserving package-aware routing.
- Behavior-preserving split/helper extraction for the largest in-process labs, especially repeated raw-secret/rejection/contract builders.
- Replace obvious O(n²) composition/package diagnostics scans with sets/indexes.

Acceptance: public protocol unchanged; replacement/no-official-priority conformance still passes; no macro/codegen; workspace tests, conformance, and package checks pass.

## Phase P3 — Event Store & Replay Optimization

Goal: harden durable substrate for real product sessions.

Deliverables: SQLite store-level atomic append with sequence; needed indexes and range/audit queries; reduced routine `list_all()` in hydrate/audit paths; concurrent append correctness test; baseline update for 1k/10k/100k event metrics.

Acceptance: concurrent same-session append is stable; SQLite-backed substrate rehydrate tests pass; audit/permission/proposal/event conformance passes; no redaction/schema/hook bypass.

## Phase P4 — Web Render & UI Organization

Goal: make Forge usable under realistic product data.

Deliverables: SSE batching/debounce or local event-tail updates; partitioned Forge panels; separated view-model builders / detectors / renderers; lazy stringify / collapsed-by-default large JSON payloads; pagination or caps for events, proposals, and assets; frontend performance self-test or diagnostics helper.

Acceptance: Web TypeScript passes; public-protocol-only; no SQLite/runtime internals; 500-event Forge mock/render metrics recorded in baseline.

## Phase P5 — Evidence-based Advanced Optimization & Cleanup

Goal: apply only evidence-backed advanced optimizations and remove this temporary plan.

Optional deliverables: capability/surface resolve cache with load/unload invalidation; manifest-derived handler coverage tests; limited registry helpers/codegen; RawValue only for proven large-payload pass-through paths and never bypassing redaction/schema/hooks; performance guide, budgets, and CI-friendly commands.

Acceptance: temporary plan deleted; `docs/performance/`, `ALPHA_STATUS`, `NEXT_STEPS`, and `CONFORMANCE_MATRIX` converged; final workspace tests, conformance, Web TS, doc links, and diff check pass.
