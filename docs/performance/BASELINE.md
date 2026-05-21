# Performance Baseline

> [English](./BASELINE.en.md) · [中文](./BASELINE.md)

本文档记录 `ygg perf baseline` 的用法、测量场景、样本限制和指标定义。当前基线只作为开发机参考，不是 CI 预算。

性能与代码健康指南见 [`PERFORMANCE_AND_CODE_HEALTH.md`](./PERFORMANCE_AND_CODE_HEALTH.md)。

## 命令

```bash
# 默认 10 次迭代，文本输出
cargo run -p ygg-cli -- perf baseline

# 自定义迭代次数
cargo run -p ygg-cli -- perf baseline --iterations 20

# JSON 输出（stdout 仅 JSON，可程序化处理）
cargo run -p ygg-cli -- perf baseline --format json
```

## 测量场景

所有场景都不依赖真实网络或 provider。输入固定，方便在开发机上比较趋势。

| scenario_id | 说明 |
|---|---|
| `inproc_echo_invoke` | Rust inproc 包 echo 能力调用。使用 `examples/packages/echo-rust-inproc/manifest.yaml`。 |
| `official_capability_invoke` | 官方包能力调用。使用 `official/composition-lab/describe`。 |
| `event_store_append_list_range` | 内存 event store 批量追加（100 events）、全量 list、range 查询。 |
| `event_store_append_list_range_1k` | 内存 event store 原子追加（1,000 events）、全量 list、kind-prefix 查询。P3 新增。 |
| `event_store_append_list_range_10k` | 内存 event store 原子追加（10,000 events）、全量 list、kind-prefix 查询。P3 新增。 |
| `event_store_append_list_range_100k` | 内存 event store 原子追加（100,000 events）、全量 list、kind-prefix 查询。当 iterations > 1 时自动限制为 1 次迭代。P3 新增。 |
| `composition_check` | Composition descriptor 验证与包加载。使用 `examples/compositions/playable-seed-replacement/`。 |
| `profile_load` | Profile YAML 解析。使用 `profiles/forge-alpha.yaml`。 |
| `subprocess_echo_invoke` | Subprocess echo 能力调用（需要 Python；不可用时 status=skipped）。 |
| `forge_render_diagnostics_50/500` | Web Forge pure TS render diagnostics helper。使用 mock public-protocol events，不读 runtime internals；P4 新增。 |

## 输出字段

| 字段 | 说明 |
|---|---|
| `scenario_id` | 场景标识符 |
| `iterations` | 迭代次数 |
| `total_ms` | 总耗时（毫秒） |
| `avg_ms` | 平均耗时 |
| `min_ms` | 最小耗时 |
| `max_ms` | 最大耗时 |
| `status` | `ok` / `skipped` / `error` |
| `notes` | 附加说明 |

## 样本限制

- 默认 10 次迭代。可通过 `--iterations` 调整。
- `--iterations 0` 会被拒绝；所有场景必须至少运行一次。
- 每次迭代独立计时；不做跨迭代预热或冷却。
- 测量使用 `std::time::Instant`，精度取决于 OS（通常 1 µs 或更优）。
- 内存事件存储场景每次迭代追加 100 个事件。扩展规模场景覆盖 1k/10k/100k 原子追加。每次迭代使用独立 store 和会话，避免累计事件影响固定规模指标。
- `event_store_append_list_range_100k` 当 `--iterations > 1` 时自动限制为 1 次迭代以避免过慢。
- `EventStore::append_with_sequence` 提供原子追加，保证同一会话并发时不会产生重复 sequence。
- `EventStore::list_kind_prefix` 和 `list_session_kind_prefix` 提供查询下推。审计和范围查询不再常规执行 `list_all()` 后全量过滤。
- `clients/web/src/performance/render-diagnostics.ts` 用于前端侧 50/500 个事件的 Forge 渲染诊断。该 helper 是纯 TypeScript，不连接 host，不读取 SQLite 或 runtime internals。
- 不使用 criterion 或统计框架；当前目标是建立开发机参考，不是 CI 合规预算。

## 红线

- 不做官方包 fast path。官方包和第三方包走同一路由与权限边界。
- 不绕过权限、钩子、schema、脱敏或审计。
- 不需要真实网络或 provider。
- 不修改 runtime 边界或公开协议。

## 后续优化追踪指标

后续优化可以用这些指标做前后对比：

1. in-process 调用延迟 — 如果引入 resolve cache 或 handler table，应观察该指标变化。
2. 事件存储批量吞吐 — 100 个事件、1k、10k、100k 的 append/list/range/kind-prefix 延迟都可比较。
3. 事件存储规模趋势 — 用 1k/10k/100k 场景观察跨版本增长曲线。
4. composition check 延迟 — 诊断扫描改用 set/index 后应有改善。
5. profile load 延迟 — 作为 YAML 解析基线；profile 变大后应重新测量。
6. 子进程调用延迟 — 需要稳定的子进程环境再做比较。
7. Forge 渲染诊断 — 后续 UI 优化应比较 HTML bytes 和 elapsed_ms。

## 样本参考输出

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

以上数值来自特定开发机，仅作参考。不作为 CI 合规预算。
