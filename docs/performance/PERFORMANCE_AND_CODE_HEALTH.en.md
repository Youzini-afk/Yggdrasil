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
