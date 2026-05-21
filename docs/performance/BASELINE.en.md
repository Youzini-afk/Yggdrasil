# Performance Baseline

> [English](./BASELINE.en.md) · [中文](./BASELINE.md)

This document records usage, measurement scenarios, sample limits, and metric definitions for `ygg perf baseline`. The current baseline is only a developer-machine reference, not a CI budget.

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

All scenarios avoid real network or provider dependencies. Inputs are fixed so developer machines can compare trends.

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
- Each iteration is independently timed; there is no cross-iteration warm-up or cool-down.
- Measurement uses `std::time::Instant`; precision depends on OS (typically 1 µs or better).
- The in-memory event-store scenario appends 100 events per iteration. Scale scenarios cover 1k/10k/100k atomic append. Each iteration uses an independent store and session so accumulated events do not distort fixed-size metrics.
- `event_store_append_list_range_100k` auto-caps to 1 iteration when `--iterations > 1` to avoid excessive runtime.
- `EventStore::append_with_sequence` provides atomic append and prevents duplicate sequences under concurrent same-session access.
- `EventStore::list_kind_prefix` and `list_session_kind_prefix` provide query pushdown. Audit and range queries no longer routinely call `list_all()` and then filter the full result.
- `clients/web/src/performance/render-diagnostics.ts` provides frontend Forge render diagnostics for 50/500 events. The helper is pure TypeScript: no host connection and no SQLite or runtime internals.
- No criterion or statistical framework is used. The goal is a developer-machine reference, not a CI compliance budget.

## Red lines

- No official-package fast path. Official and third-party packages share the same routing and permission boundaries.
- No bypass of permissions, hooks, schema validation, redaction, or audit.
- No real network or provider required.
- No runtime boundary or public protocol changes.

## Metrics for future optimization tracking

Use these metrics for before/after comparisons during later optimization:

1. In-process invoke latency — Watch this if a resolve cache or handler table is introduced.
2. Event-store batch throughput — Compare append/list/range/kind-prefix latency for 100 events, 1k, 10k, and 100k.
3. Event-store scale trend — Use the 1k/10k/100k scenarios to compare growth across versions.
4. Composition check latency — Set/index-based diagnostics should improve this.
5. Profile load latency — Use it as the YAML parsing baseline; re-measure when profiles grow.
6. Subprocess invoke latency — Re-measure with a stable subprocess environment.
7. Forge render diagnostics — Future UI optimization should compare HTML bytes and elapsed_ms.

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

Values above are from a specific developer machine. They are a reference, not a CI compliance budget.
