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

## Phase P0 — Baseline & Measurement

目标：先建立事实，不凭感觉优化。

交付：

- `ygg perf baseline` 或等价 CLI，输出 deterministic baseline JSON / markdown summary。
- 覆盖 inproc invoke、official capability invoke、subprocess echo、event append/list/hydrate、composition check、profile load、Web render/TS 指标记录。
- `docs/performance/BASELINE.md` 与 `.en.md` 记录命令、机器环境、样本规模、预算线。
- 保留默认 no-network；不需要真实 provider。

验收：baseline 命令可重复运行；文档列出后续优化追踪指标；workspace tests、conformance、doc links 通过。

## Phase P1 — Conformance Feedback Loop

目标：让 conformance 可筛选、可计时、可定位。

交付：`--list`、`--case <pattern>`、`--tag <tag>`、`--fail-fast`、per-case duration、slowest-N report，以及 case tags（runtime、subprocess、official、generated、network、stream、agentic、experience、memory、sharing、slow 等）。

验收：默认 `ygg conformance` 仍跑全部 cases；单 case 和 tag-filter 可运行；新增 case 注册点要求 tags；输出足以定位失败和慢 case。

## Phase P2 — Low-risk Structural Split

目标：控制增长，不改外部行为。

交付：

- `runtime/protocol_dispatch.rs` 按 domain 拆到 focused modules / functions，保留 `KernelMethod` 单一事实源。
- official inproc 从线性 `try_handle` 链收敛为显式 registry / handler table，保持 package-aware routing。
- 对最大 inproc lab 做行为保持拆分或共享 helper 收敛（优先 playable_creation_board、agentic_forge、sharing/memory 中重复 raw-secret/rejection/contract builder）。
- composition / package diagnostics 中明显 O(n²) 扫描改为 set/index。

验收：public protocol 无变化；replacement/no-official-priority conformance 仍通过；不引入 macro/codegen；workspace tests、conformance、package checks 通过。

## Phase P3 — Event Store & Replay Optimization

目标：为真实产品长期运行做 durable substrate 优化。

交付：SQLite store-level atomic append with sequence；必要索引和 range/audit query；hydrate / audit 路径减少常规 `list_all()`；concurrent append correctness test；baseline 更新 1k/10k/100k event 指标。

验收：并发 append 同 session 稳定；SQLite-backed substrate rehydrate tests 通过；audit/permission/proposal/event conformance 通过；不绕过 redaction/schema/hook。

## Phase P4 — Web Render & UI Organization

目标：让 Forge 能承受真实 product 数据规模。

交付：SSE batching/debounce 或局部 event-tail update；Forge panels 分区渲染；view-model builders / detectors / renderers 分离；大 JSON payload lazy stringify / 默认折叠；事件、proposal、asset 视图分页或 cap；前端性能自测或 diagnostics helper。

验收：Web TypeScript 通过；public protocol-only；无 SQLite/runtime internals；500 events 的 Forge mock/render 指标记录到 baseline。

## Phase P5 — Evidence-based Advanced Optimization & Cleanup

目标：只做证据支持的高级优化，并删除临时计划。

可选交付：capability/surface resolve cache with load/unload invalidation；manifest-derived handler coverage tests；limited registry helper/codegen；RawValue 仅用于被证明的大 payload 透传路径且不得绕过 redaction/schema/hook；performance guide / budgets / CI-friendly commands。

验收：临时计划删除；`docs/performance/`、`ALPHA_STATUS`、`NEXT_STEPS`、`CONFORMANCE_MATRIX` 收敛；最终 workspace tests、conformance、Web TS、doc links、diff check 通过。
