# Performance & Code Health Beta

> [English](./PERFORMANCE_CODE_HEALTH_BETA.en.md) · [中文](./PERFORMANCE_CODE_HEALTH_BETA.md)

这是临时执行计划。完成后删除，长期内容收敛到 `ALPHA_STATUS`、`NEXT_STEPS`、性能指南和相关规范文档。

## 为什么现在做

Experience-Led Platform Beta 0–6 已经把 Yggdrasil 从 foundation-first 推进到可体验闭环。继续做第一个产品前，最大的风险不再是缺少底座，而是增长成本：

- conformance 反馈环太重，245+ cases 会拖慢每次产品迭代。
- Web Forge / Agent panels 仍偏全量重算和全量渲染。
- SQLite event store / replay 在长期产品运行后会成为真实瓶颈。
- official inproc handlers、protocol dispatch、CLI commands 和 TS panels 继续增长后会回到巨型文件风险。
- JSON / clone / string routing 存在成本，但必须 measurement-first，不能凭感觉重写。

外部参考：Rust Performance Book 强调 profiling / benchmarking；React 文档强调用 Profiler 测量慢交互后再 memoization；SQLite 文档强调 WAL、checkpoint 和索引；OpenTelemetry benchmark 文档强调固定场景、重复测量、CPU/内存/吞吐/延迟报告。

## 红线

- 不做官方包 fast path；`official/*` 与第三方包保持同等路由和权限边界。
- 不绕过 permission、hook、schema、redaction、audit。
- 不让 Web 读取 SQLite 或 runtime internals；UI 仍走 public protocol。
- 不新增 `kernel.experience.*`、`kernel.memory.*`、`kernel.sharing.*`、`kernel.marketplace.*`、`kernel.agent.*`、`kernel.model.*`。
- 不做大规模 proc macro / heavy codegen / arena / RawValue rewrite，除非 baseline 证明必要。
- 不把 JSON boundary 全面替换成私有强类型 fast path。

## Phase P0 — Baseline & Measurement（已完成）

目标：先建立事实，不凭感觉优化。

交付：

- `ygg perf baseline` CLI，输出 deterministic baseline JSON / markdown summary。
- 覆盖 inproc invoke、official capability invoke、subprocess echo（可能 skipped）、event store append/list/range（100 events）、composition check、profile load。
- `--iterations` 和 `--format text|json` 参数。
- `docs/performance/BASELINE.md` 与 `.en.md` 记录命令、机器环境、样本规模、预算线。
- 保留默认 no-network；不需要真实 provider。

验收：baseline 命令可重复运行；文档列出后续优化追踪指标；workspace tests、conformance、doc links 通过。

参考：[`docs/performance/BASELINE.md`](../performance/BASELINE.md)

## Phase P1 — Conformance Feedback Loop（已完成）

目标：让 conformance 可筛选、可计时、可定位。

交付：`--list`、`--case <pattern>`、`--tag <tag>`、`--fail-fast`、per-case duration、slowest-N report，以及 case tags（runtime、event、capability、package、subprocess、official、generated、network、outbound、stream、agentic、experience、memory、sharing、secret、composition、replacement、surface、protocol、permission、hook、host、asset、projection、substrate、live、slow 等）。结构化 `ConformanceCase { id, tags, run }` registry 替代原有 `record_case` 调用。默认 `ygg conformance` 仍跑全部 245 cases。

验收：默认 `ygg conformance` 仍跑全部 cases；`--list` 列出 id 和 tags；单 case（`--case sharing_lab.contract_shape`）和 tag-filter（`--tag sharing`）可运行；新增 case 注册点要求 tags；输出包含 per-case duration、slowest-N 报告和失败定位。

参考：[`docs/performance/CONFORMANCE_FEEDBACK.md`](../performance/CONFORMANCE_FEEDBACK.md)

## Phase P2 — Low-risk Structural Split（已完成）

目标：控制增长，不改外部行为。

交付：

- `runtime/protocol_dispatch.rs` 按 domain 拆到 focused helper functions，保留 `KernelMethod` 单一事实源。顶层 match delegate 到 host/surface/outbound/permission/proposal/session/event/package/capability/extension/hook/asset/projection 等 helper。
- official inproc 从线性 `try_handle` 链收敛为 `provider_package_id` indexed dispatch。`dispatch_official` 按 `provider_package_id.as_str()` 直接 dispatch 到对应模块；unknown official package 走 `common::try_handle`；non-official 不得走 official fallback。
- 安全 helper 收敛：5 个 inproc lab（agentic_forge、experience_observability、memory、playable_creation_board、sharing）中完全一致的 `is_secret_ref_value` / `looks_like_raw_secret_value` / `contains_raw_secret` 抽取到 `inproc/safety.rs` 共享模块。marketplace/billing/signing 字段检查保留在 `sharing_lab` 本地。不改变 rejection output 文案/JSON shape。
- composition / package diagnostics 中明显 `.iter().any()` 改为 `BTreeSet`/`BTreeMap`/`HashSet` index lookup。suffix/contains 语义谨慎保留 helper。
- Web forge.ts 中 `packagesWithoutSurfaces` 改用 `surfacesByPackage` group index，避免 packages × surfaces 重复 filter。

验收：public protocol 无变化；replacement/no-official-priority conformance 仍通过；不引入 macro/codegen；workspace tests、conformance、package checks 通过。

## Phase P3 — Event Store & Replay Optimization（已完成）

目标：为真实产品长期运行做 durable substrate 优化。

交付：
- `EventStore::append_with_sequence` 原子追加 API：输入 session_id、writer_package_id、kind、schema_version、payload_json、metadata_json，输出插入后的 EventEnvelope。默认实现走 `next_sequence + append`；`SqliteEventStore` override 在同一把 connection mutex 内读取 max sequence、构造 event、insert；`InMemoryEventStore` override 在同一 write lock 内分配 sequence 并 push。保证同 session 并发不重复 sequence。
- `EventStore::list_kind_prefix` 和 `list_session_kind_prefix` 查询 API：默认实现 list+filter；SQLite override 用 SQL range/LIKE pushdown；InMemory override 用单次 read+filter。保持排序稳定。
- SQLite 索引：`kind`、`session+kind+sequence`，用于 audit/range 查询 pushdown。
- `append_event_unchecked` 改用 store-level atomic append；hook veto/schema fail 不消耗 sequence。
- `dispatch_permission_audit()` 改用 `list_kind_prefix("kernel/permission")` pushdown。
- `list_outbound_audit()` 改用 `list_session_kind_prefix(session, "kernel/outbound")` pushdown。
- 并发 append correctness test：同 session 并发 50 个 append 后 sequence 连续且无重复。
- `ygg perf baseline` 扩展 event scale scenarios：`event_store_append_list_range_1k`（1,000 events）、`event_store_append_list_range_10k`（10,000 events）、`event_store_append_list_range_100k`（100,000 events，>1 迭代时自动限制为 1 次）。
- `docs/performance/BASELINE*` 更新新增 event scale 指标。

验收：并发 append 同 session 稳定；SQLite-backed substrate rehydrate tests 通过；audit/permission/proposal/event conformance 通过；不绕过 redaction/schema/hook。

## Phase P4 — Web Render & UI Organization

目标：让 Forge 能承受真实 product 数据规模。

交付：SSE batching/debounce 或局部 event-tail update；Forge panels 分区渲染；view-model builders / detectors / renderers 分离；大 JSON payload lazy stringify / 默认折叠；事件、proposal、asset 视图分页或 cap；前端性能自测或 diagnostics helper。

验收：Web TypeScript 通过；public protocol-only；无 SQLite/runtime internals；500 events 的 Forge mock/render 指标记录到 baseline。

## Phase P5 — Evidence-based Advanced Optimization & Cleanup

目标：只做证据支持的高级优化，并删除临时计划。

可选交付：capability/surface resolve cache with load/unload invalidation；manifest-derived handler coverage tests；limited registry helper/codegen；RawValue 仅用于被证明的大 payload 透传路径且不得绕过 redaction/schema/hook；performance guide / budgets / CI-friendly commands。

验收：临时计划删除；`docs/performance/`、`ALPHA_STATUS`、`NEXT_STEPS`、`CONFORMANCE_MATRIX` 收敛；最终 workspace tests、conformance、Web TS、doc links、diff check 通过。
