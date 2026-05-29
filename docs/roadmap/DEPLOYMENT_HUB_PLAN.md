# 自托管部署中枢计划

> [English](./DEPLOYMENT_HUB_PLAN.en.md) · [中文](./DEPLOYMENT_HUB_PLAN.md)
>
> 临时计划文档。落地完成后并入长期文档并删除本文件。

## 背景与动机

平台愿景不变：content-free 内核 + 契约、能力包、官方与第三方平等、组合。本阶段调整的是**演化重点**：当前重点把 Yggdrasil 演化成一个**自托管的 AI/agent 项目部署管理中枢**——帮用户部署、运行、管理社区里的项目（Docker 优先，原生后置），从桌面 / Web / 手机客户端访问。

为什么先做这个：

- 能力包生态需要用户才转得起来；用户不会为「还没有生态的平台」而来。先用「即时有用」赢用户，再长生态。
- 「辅助部署」不是新子系统，是又一个**平台能力**，用能力包表达，跑在 content-free 内核上，与其它能力平等。
- 被部署的项目就是 Path B 自包含应用，本来就是平台的平等公民。

这一阶段**不改 README / 愿景定位**。游创一体、能力生态仍是终点，部署中枢是上车口。

## 现状（代码审计结论）

- `kernel.v1.project.start` 是纯状态机：鉴权 → 开 session → 发事件 → 标 Running。不启动进程、不分配端口、不返回 URL、不健康检查、不反代。
- `OutboundExecutor` 是成熟可照抄的模板：trait + `{DenyAll, Fake, LiveHttp}` 配置枚举 + `RuntimeConfig` 注入 + host profile 选择 + dispatch + 审计 + `secret_ref` 注入。
- `subprocess.rs` 能 spawn / kill / 超时，但只有 JSON-RPC-stdio，没有端口 / HTTP 概念。
- 反向代理：整仓零实现。这是最大缺口。
- External Operating Plane 那批包（intake / workspace / install / git-tools / integrity）全是 plan-only / fake-executor。

## 内核边界决定（已拍板：A）

内核新增一组**通用的「受控本地执行 + 端口 + 路由」原语**，与现有 outbound HTTP 执行器同构。内核永远不懂「Docker / 部署 / Tavern」，只提供通用、受审计、受策略门控的本地执行与端口 / 路由 transport。Docker / 原生 / 部署逻辑全部在普通能力包里。

反代数据面**自己实现**（不编排 Traefik），放在 `ygg-service`，因为浏览器流量不走 capability invoke，且自包含单进程对桌面 / 云 / 手机瘦客户端都友好。

## 架构

```
   桌面 / Web / 手机瘦客户端
            │ 都连到
            ▼
   Yggdrasil 控制面（内核 content-free）
   ├ kernel.v1.exec.*    通用进程管理（DenyAll / Fake / LiveLocal）
   ├ kernel.v1.port.*    端口租约（只 loopback）
   ├ kernel.v1.proxy.*   路由注册（上游 = port lease）
   ├ kernel.v1.target.*  执行目标（本地 / 远程 / 隧道）
   └ ygg-service 反代数据面（虚拟主机优先，路径前缀兜底）
            │ 驱动
            ▼
   能力包层（官方与第三方平等）
   ├ official/docker-runtime-lab   （用 bollard）
   ├ official/native-runtime-lab   （后置，危险，trusted-only）
   ├ official/target-registry-lab
   └ official/deployment-plan-lab
            │
            ▼
   本地 Docker / 远程 Docker / 手机原生 — 跑任意社区项目
```

## 内核原语规格

照 `OutboundExecutor` 范式。新增 `KernelMethod` 变体：

```
# 目标域
kernel.v1.target.list / status / register / unregister

# 执行域
kernel.v1.exec.start / stop / status / logs / list

# 端口域
kernel.v1.port.lease / release / status / list

# 路由域
kernel.v1.proxy.register / unregister / status / list
```

执行器配置（照 `OutboundExecutorConfig`）：

```rust
pub enum LocalExecExecutorConfig {
    DenyAll,                       // 默认 fail-closed
    Custom(Arc<dyn LocalExecExecutor>),  // Fake
    LiveLocal(LiveLocalExecConfig),      // opt-in
}
```

注入 `RuntimeConfig`，host profile 选择，dispatch 前做策略检查，审计带脱敏。

关键约束：

- `ExecCommand` 是 `{program, args}`，**没有 shell 字符串**。
- 端口第一版**只允许 loopback 绑定**。
- proxy 上游必须引用 kernel 发的 port lease，不能是任意 URL（防开放中继）。
- env 支持 `Literal / SecretRef / PortRef`；`secret_ref` 必须在 manifest 声明。

新增事件（带脱敏，不持久化原始 env / 日志 / body / secret）：

```
kernel/v1/exec.request / denied / started / stopped / completed / failed
kernel/v1/port.leased / released / denied
kernel/v1/proxy.registered / unregistered / denied / access.summary
```

## 反向代理设计

- 数据面在 `ygg-service`，在 SPA fallback 之前插入代理路由。
- 路由注册表 + 策略 / 审计权威在 runtime。
- **虚拟主机优先**：`<route_id>.apps.<host>` / `<route_id>.localhost:<port>`，让社区 app 以为自己拥有 `/`（`fetch('/api/*')`、cookie、WebSocket 才不碎）。
- 路径前缀 `/_ygg/app/<route_id>/` 仅兜底，给支持 base path 的 app。
- 支持 HTTP/1.1 + WebSocket upgrade + 流式 + body 上限 + idle 超时 + per-route auth。
- 浏览器 iframe 用一次性 launch token bootstrap → 设 route-scoped cookie → 重定向到干净 URL；转发上游前剥掉 Ygg 自己的 auth header / cookie。
- 绝不把宿主 `access_token` 透传给被代理 app。

## 执行目标模型

first-class 但 content-free：

```rust
pub struct ExecutionTargetDescriptor {
    target_id, display_name,
    reachability: { LocalHost, RemoteAgent, ReverseTunnel },
    capabilities: [ LocalExec, PortLease, HttpProxyUpstream, WebSocketProxyUpstream ],
    status, registered_by_package_id, metadata,
}
```

内核不认识 `DockerTarget`。Docker 是能力包 / metadata 词汇。运行中项目用一个派生的 `ProjectRunInstance` 视图暴露给客户端（run_id / project_id / target_id / exec_id / port_lease_ids / proxy_route_ids / status / urls）。

## 安全姿态（红线）

跑任意社区项目 = 跑任意代码，默认全部当作敌意，除非隔离。

1. 默认拒绝所有本地执行 / 端口租约 / 路由注册。
2. 端口只绑 loopback。
3. 只通过受鉴权的 ygg-service 反代暴露。
4. 无 host-admin 显式批准不得公开暴露。
5. 审计 / 日志 / env 持久化 / crash 记录里绝无原始 secret。
6. 没有 shell 命令字符串，只有 argv。
7. proxy 上游必须引用 port lease。
8. Docker 默认禁：`--privileged`、`--network host`、挂 `/`、挂 docker.sock、挂凭证目录、root 运行、直接暴露 `0.0.0.0`、`latest` 不锁 digest。危险项只能 host-admin 显式 override + 大声审计。
9. 原生执行对不可信社区项目不安全，只给 trusted / dev 模式，直到有真 OS 沙箱。

## 阶段（每阶段验证 + 提交 + 推送）

### Phase 1 — 内核原语骨架（deny / fake only）
- 加 `kernel.v1.target.* / exec.* / port.* / proxy.*` 协议方法 + schema。
- `DenyAllLocalExecExecutor` + `FakeLocalExecExecutor` + 内存注册表。
- 审计事件、权限声明、host profile 配置。
- 不真启动进程。
- 验收：测试包能拿到确定性 fake handle；被拒调用不进执行器；审计无原始 secret / env / 日志。

### Phase 2 — ygg-service 通用反代（虚拟主机优先）
- SPA fallback 前插入动态代理路由，从 runtime proxy 注册表查路由。
- 虚拟主机模式 + 路径前缀兜底 + WebSocket + launch token + header/cookie 剥离 + 路由禁用/过期。
- 验收：fake 上游能从 iframe 打开；虚拟主机模式下 `fetch('/api/*')` 工作；WebSocket echo 通过；被代理 app 无法用继承凭证调 `/kernel/v1/*`。

### Phase 3 — LiveLocal exec + Docker-first 包
- `LiveLocalExecExecutor`（`tokio::process::Command` + 长生命进程表 + stdout/stderr ring buffer + stop/kill 超时 + readiness probe）。
- `official/docker-runtime-lab`（用 bollard）。
- 最小验证里程碑：部署一个 Docker 社区 Web 项目 → 租 loopback 端口 → 注册 proxy → 从 Web 客户端打开 → 看日志 → 停止 → 审计齐全。先用极小已知 Docker Web app 跑绿，再拿 YdlTavern / 社区酒馆 dogfood。

### Phase 4 — 项目运行 UX + 目标模型
- 目标 list/status UI、项目运行卡、启停/日志/打开操作、`ProjectRunInstance` 投影、路由健康、per-project secrets UI、本地执行/端口暴露/docker run 审批提示。

### Phase 5 — intake → install → deploy 管道
- 连 project intake → install-lab → integrity-lab → deployment-plan-lab → docker-runtime-lab。
- 验收：真实非 Yggdrasil 外部项目能被安装并部署，无需手写一次性代码；plan 执行前可审查；权限/提案可读。

### Phase 6 — 文档收敛
- 删本临时 plan，更新长期文档（NEXT_STEPS / ALPHA_STATUS / 相关 guide）。
- 远程目标 / 原生 / 手机瘦客户端列为后续方向，不在本轮强行做完。

## 不做（本轮）
- 编排 Traefik / Caddy。
- 原生执行作为「安全」路径推广。
- 远程 target agent / mTLS / 反向隧道的完整实现（只留接缝）。
- 手机原生执行（手机先做远程瘦客户端）。
- 在内核引入任何 docker / container / deploy / tavern 语义。
