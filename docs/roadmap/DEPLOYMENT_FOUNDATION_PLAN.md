# 部署地基计划（持久化 + 重启对账 + host broker + 监督）

> 临时计划文档。全部相位完成后删除，长期内容收敛进 `docs/guides/DEPLOYMENT_RUNTIME.md`。

## 背景

上一轮做出了部署运行原语（target/exec/port/proxy）、ygg-service 反代、`LiveLocalExecExecutor`、`official/docker-runtime-lab` 和浏览器侧显式 Deploy broker。

调查（@explorer 实地核对 + @oracle 边界复核）暴露的地基缺口：

1. exec/port/proxy/target 注册表挂在 `RuntimeConfig`，纯内存，host 重启即丢。
2. exec/port/proxy 事件已经在写，但**没有任何地方回放**。
3. 发现先存潜伏 bug：`hydrate_substrate_from_events()` 只在 conformance 测试里被调用，`Runtime::new` 与 host serve 都没调——SQLite-backed host 重启后，asset/branch/projection/grant 也都不回放。
4. `LiveLocalExecExecutor` 进程表内存，`readiness_probe` 字段从没被用，spawn 完立刻 `ready: true`。
5. 反代 route 一 register 就 `Active`，容器没起来就放流量。
6. 无健康检查、无 readiness gating、无 auto-restart、无 host 重启恢复。
7. 编排住在浏览器：关 tab 容器/端口租约成孤儿。

## 边界决策（已与用户确认）

- broker 走 **host-plane（ygg-service / host serve 内）**，不做成普通能力包。理由：编排要调 HostAdmin-only 原语，普通包没权限；长生命监督回路不适合 request/response 的 capability-invoke。
- broker 仍不碰内核语义：Docker 域操作调 `docker-runtime-lab` 包，端口/代理走公开协议原语。
- 不发明"daemon capability"新内核执行模型。host broker 自己持有 async 任务/定时器，包保持无状态 capability 端点。

## 内核纪律红线

- 内核只拥有"原语存活性"：记录、`exec.status/stop/logs`、通用状态、reconcile hook。
- 健康检查 / 重启策略 / backoff 归 broker，**不进内核**。
- 事件溯源 exec/port/proxy 注册表，和现有 asset/grant 一个路子，不违反"内核不定义 disk/store"。
- **回放出来的 active 记录只能标 `unknown/stale`，不能当成"还在跑"**。真实进程/容器由 executor/package 拥有，必须 reconcile。
- 不造特权部署包。包产计划 + 域操作；host broker 持有特权 + 编排 + 监督。

## 相位

### A0 — 修复 substrate 回放潜伏 bug（先决，单独提交）

`ygg host serve` 在 runtime 构造后、**autoload 包之前、serve 之前**调用 `hydrate_substrate_from_events()`（至少 sqlite/postgres 持久后端）。顺序要点（@oracle）：hydrate 会重建并覆盖 runtime map，任何在 hydrate 之前创建的内存 asset/grant/projection 都会被冲掉，所以必须排在 autoload/mutation 之前。`list_all()` 全量扫描此阶段可接受（与现有 substrate 持久模型一致），后续再做 snapshot/checkpoint/增量。**不做成 opt-in**，否则潜伏 bug 仍在。作为**独立提交/PR**，与部署工作分开——它是独立潜伏持久化 bug。补回归：sqlite 重启后 asset/projection/grant 仍在。

### A — exec/port/proxy 持久化 + 回放（不含 target）

修正（@oracle 复核）：**跳过 target 持久化**。`local` target 由 `ExecutionTargetRegistry::new()` 每次启动重建；profile-derived target 也每次重导。持久化 target 反而会存下过时 host 拓扑。只在将来有用户自定义持久 target 配置时再加 `target.*` 事件。

- 扩展回放，覆盖 exec/port/proxy 记录（target 不动）。
- 回放出来的记录按种类区别对待，**不要一刀切标 stale**：
  - exec：标 `unknown`/`stale`。持久 exec 记录不等于进程还在。
  - port：标 reserved/unverified——同端口不被重新分配，但未经 reconcile 不算 routable。
  - proxy：标 pending/stale，未经 upstream 验证不放流量。
- 持久关联元数据（@oracle 第 5 点）：事件 payload 要带 `owner_id`/`correlation_id`/`route_id`/`port_lease_id` 等通用 label，让 broker 能回答"这条 proxy 依赖哪个 lease、这个 lease 属于哪个容器意图、reconcile 失败该清什么"。**不在内核里叫它 deployment**，只是通用 metadata。
- 事件 payload 充分性：port/proxy 事件大概率够重建记录；exec 事件可能只够重建 status，不够重建 restart intent——不假装能从残缺 exec 事件重启。
- conformance：append → 新建 runtime → 回放 → 记录在但状态为 stale/unverified/pending。

### B — 重启对账（Docker 为主）

修正（@oracle 复核）：**local exec 不可恢复**。LiveLocalExec 的 tokio child handle、日志表、控制句柄都随旧 host 进程消失，子进程也随之而去。所以 local exec reconcile 基本就是"进程没了，标 failed/unknown"，恢复 = broker 经审批后起**新**进程。真实跨重启存活只对 Docker 有意义（容器能 outlive host 进程）。

- 新增对账 pass：
  - LiveLocalExec：一律标 failed/unknown/stopped（不尝试 reattach）。
  - Docker：host broker 调 `docker-runtime-lab` 的 status/reconcile 能力，核对容器是否真实存在并绑定到期望的 `127.0.0.1:<lease.port>`。
- 真实存在且绑定正确 → 标 active/ready；否则标 failed/stale。
- conformance：stale 记录在 reconcile 后按真实情况转 active 或 failed。
- `docker-runtime-lab` 补 `reconcile`/`status` 能力（按需）。

### C — host 侧 broker

- 把编排从浏览器搬到 host-plane：一条 broker 路径 `lease → docker run → readiness → proxy ready`。
- 浏览器变成 broker 的客户端，下发"部署这个项目"意图，不再亲自串三步。
- broker 以 HostAdmin/HostDev 调 `kernel.v1.port.lease`/`proxy.register`/`proxy.unregister`/`port.release` + `docker-runtime-lab` 域能力。
- 关 tab 不再漏容器：编排状态归 host。

### D — readiness gating

- proxy route 加状态 `Pending`/`Ready`/`Failed`（在现有 `Active`/`Removed` 之上）。
- `proxy.register` 建为 `Pending`；broker readiness 通过后翻 `Ready`。
- ygg-service 对非 ready route 返 `503`（不是 404）。
- 内核不做 HTTP 健康策略；ygg-service 不自己乱探活。

### E — 监督回路

- host broker 健康 poll 回路 + 有界重启策略 + backoff + 审计事件。
- 只在 A–D 之上做：持久化防丢、对账防假、broker 集权、readiness 防早放流量，监督才是可靠状态上的策略回路。

## 相位顺序理由

持久化防状态丢失 → 对账防"撒谎的状态" → broker 集中权限 → readiness 防过早放流量 → 监督成为可靠状态上的策略回路，而不是新的不一致来源。

## 验证

每相位：`cargo test` 相关 crate + `ygg conformance` + 新增 conformance case；涉及 Web 的相位跑 Web check/test/build。全部完成跑全量 conformance + schema validate + 删临时 plan + 收敛 `DEPLOYMENT_RUNTIME.md`。
