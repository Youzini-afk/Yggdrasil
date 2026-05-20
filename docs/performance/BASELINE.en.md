# Performance Baseline

> [English](./BASELINE.en.md) · [中文](./BASELINE.md)

This document records the usage, measurement scenarios, sample limitations, and metric definitions for `ygg perf baseline`. The current baseline is a **developer-machine reference**, not a CI budget.

Performance/code-health guide: [`PERFORMANCE_AND_CODE_HEALTH.en.md`](./PERFORMANCE_AND_CODE_HEALTH.en.md).

## Command

```bash
# Default 10 iterations, text output
cargo run -p ygg-cli -- perf baseline

# Custom iteration count
cargo run -p ygg-cli -- perf baseline --iterations 20

# JSON output (stdout contains JSON only, machine-parseable)
cargo run -p ygg-cli -- perf baseline --format json
```

## Measurement scenarios

All scenarios are **no-network / deterministic** — no real network or provider dependency.

| scenario_id | Description |
|---|---|
| `inproc_echo_invoke` | Rust inproc package echo capability invocation. Uses `examples/packages/echo-rust-inproc/manifest.yaml`. |
| `official_capability_invoke` | Official package capability invocation. Uses `official/composition-lab/describe`. |
| `event_store_append_list_range` | In-memory event store batch append (100 events), full list, range query. |
| `event_store_append_list_range_1k` | In-memory event store atomic append (1,000 events), full list, kind-prefix query. Added in P3. |
| `event_store_append_list_range_10k` | In-memory event store atomic append (10,000 events), full list, kind-prefix query. Added in P3. |
| `event_store_append_list_range_100k` | In-memory event store atomic append (100,000 events), full list, kind-prefix query. Auto-capped to 1 iteration when iterations > 1. Added in P3. |
| `composition_check` | Composition descriptor validation and package loading. Uses `examples/compositions/playable-seed-replacement/`. |
| `profile_load` | Profile YAML parsing. Uses `profiles/forge-alpha.yaml`. |
| `subprocess_echo_invoke` | Subprocess echo capability invocation (requires Python; status=skipped if unavailable). |
| `forge_render_diagnostics_50/500` | Web Forge pure TS render diagnostics helper. Uses mock public-protocol events and does not read runtime internals; added in P4. |

## Output fields

| Field | Description |
|---|---|
| `scenario_id` | Scenario identifier |
| `iterations` | Number of iterations |
| `total_ms` | Total wall time (ms) |
| `avg_ms` | Average per iteration |
| `min_ms` | Minimum iteration time |
| `max_ms` | Maximum iteration time |
| `status` | `ok` / `skipped` / `error` |
| `notes` | Additional context |

## Sample limitations

- Default 10 iterations. Adjustable via `--iterations`.
- `--iterations 0` is rejected; every scenario must run at least once.
- Each iteration is independently timed; no cross-iteration warm-up or cool-down.
- Measurement uses `std::time::Instant`; precision depends on OS (typically 1 µs or better).
- Event store scenario appends 100 events per iteration; P3 extends to 1k/10k/100k atomic append scenarios. Scale scenarios use an independent store/session per iteration so fixed-size metrics are not distorted by accumulated events.
- `event_store_append_list_range_100k` auto-caps to 1 iteration when `--iterations > 1` to avoid excessive runtime.
- P3 adds `EventStore::append_with_sequence` atomic append API, guaranteeing no duplicate sequences under concurrent same-session access.
- P3 adds `EventStore::list_kind_prefix` and `list_session_kind_prefix` query pushdown APIs; audit/range queries no longer routinely call `list_all()` + full filter.
- P4 adds `clients/web/src/performance/render-diagnostics.ts` for front-end 50/500 event Forge render diagnostics. The helper is pure TypeScript: no host connection, no SQLite/runtime internals.
- No criterion or statistical framework; the goal is a developer-machine reference, not CI compliance budgets.

## Red lines

- **No official-package fast path.** Official and third-party packages share equal routing and permission boundaries.
- **No bypass of permission / hook / schema / redaction / audit.**
- **No real network or provider required.**
- **No runtime boundary or public protocol changes.**

## Metrics for future optimization tracking

These metrics serve as comparison baselines for later optimization phases:

1. **inproc invoke latency** — Should be watched if resolve cache or handler table is introduced in P2/P5.
2. **event store batch throughput** — After P3 atomic append + query pushdown, 100-event / 1k / 10k / 100k append+list+range+kind-prefix latencies are quantifiable.
3. **event store scale metrics** — P3 adds 1k/10k/100k event scale scenarios for cross-version trend comparison.
4. **composition check latency** — Should improve after P2 replaces O(n²) diagnostics with sets/indexes.
5. **profile load latency** — Serves as YAML parsing baseline; re-measure if profiles grow.
6. **subprocess invoke latency** — Will be re-measured in P1/P3 with a stable subprocess environment.
7. **Forge render diagnostics** — P4 adds a front-end render helper for 50/500 mock events; future UI optimization should compare HTML bytes and elapsed_ms.

## Sample reference output

```
scenario                       iterations   total_ms     avg_ms     min_ms   max_ms  status
------------------------------------------------------------------------------------------
inproc_echo_invoke                     10       0.17      0.017      0.009    0.074  ok
official_capability_invoke             10       0.19      0.019      0.012    0.056  ok [official/composition-lab/describe]
event_store_append_list_range          10      24.85      2.485      1.920    3.092  ok [100 events per iteration]
composition_check                      10       4.18      0.418      0.388    0.565  ok [playable-seed-replacement]
profile_load                           10       1.25      0.125      0.118    0.135  ok [forge-alpha.yaml parse]
subprocess_echo_invoke                 10       0.73      0.073      0.054    0.184  ok

baseline: 6 ok, 0 skipped, 0 error (6 scenarios)
```

Values above are from a specific developer machine and serve only as a reference, not as CI compliance budgets.
