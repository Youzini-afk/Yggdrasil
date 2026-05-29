# 部署运行时

> [English](./DEPLOYMENT_RUNTIME.en.md) · [中文](./DEPLOYMENT_RUNTIME.md)

Yggdrasil 现在可以作为自托管 AI / agent 项目的部署宿主。这里的“部署”不是内核里的 Docker 概念，而是一组通用运行原语，再由普通能力包组合成 Docker、本地进程或后续远程目标。

## 边界

内核只提供四类通用原语：

| 原语 | 协议族 | 作用 |
|---|---|---|
| target | `kernel.v1.target.*` | 描述一个可运行目标。目前内置 `local`。 |
| exec | `kernel.v1.exec.*` | 启动、停止、查询一个受控本地执行。默认全拒。 |
| port | `kernel.v1.port.*` | 向 host 租一个 loopback 端口。 |
| proxy | `kernel.v1.proxy.*` | 把公开 HTTP / WebSocket 路径绑定到某个 port lease。 |

Docker、git、安装、secret store、workspace、adapter 都不是内核概念。它们由普通能力包实现。

## 当前实现

- `LocalExecExecutor` trait：默认 `DenyAllLocalExecExecutor`，profile 显式开启后可用 `LiveLocalExecExecutor`。
- `LiveLocalExecExecutor`：只接受 argv 数组，不接受 shell 字符串；cwd、env、日志、超时、kill 都由 host 控制。
- `ygg-service` 反代：`/p/<route_id>/...` 走 `kernel.v1.proxy.*` 注册的 route，只能指向 active loopback port lease；禁 redirect；剥离危险 header；限制响应体；支持 HTTP 与 WebSocket。
- `official/docker-runtime-lab`：普通官方能力包，使用 `bollard` 管理 Docker 容器。默认无 Docker 时 fail-closed；真实 Docker smoke 需要显式 opt-in。
- Web 项目控制台：显示 target / exec / port / proxy 诊断。若项目声明部署描述符，用户可显式点击 Deploy / Stop，或启动 Build & Deploy 任务。
- 持久化与回放：exec / port / proxy 注册表的变更都写进事件日志，host 重启时回放重建。
- 重启对账：host 重启后，回放出来的记录先降级（exec → unknown、port → reserved、proxy → stale 且 `ready=false`），再与真实世界对账。
- readiness gating：proxy route 注册时 `ready=false`，反代对未就绪的 route 返回 503；只有就绪后才放行。
- 健康监督：host 后台回路定期探测每条 active route 的 upstream，连续失败翻 `ready=false`、恢复翻 `ready=true`，并在状态转换时写审计事件。
- Build & Deploy broker：`POST /host/v1/build-deploy` 创建短期 job，host 侧完成 git clone、Dockerfile / nixpacks 构建、容器启动、proxy 注册、readiness probe。浏览器通过 job status / SSE 查看进度，可取消任务。

## Docker 部署描述符

原生项目可以在 `project.yaml` 的 `project.metadata.deployment.docker` 写入最小部署信息：

```yaml
project:
  metadata:
    deployment:
      docker:
        image: ghcr.io/example/app:latest
        container_port: 3000
        port_name: web        # optional, default: web
        route_id: my-app-web  # optional, default: <project_id>-web
        health_path: /healthz # optional, 用于 readiness probe
        pull_if_missing: false
```

当前 Web broker 只接受这些字段。`env`、`volumes`、`mounts`、`binds`、`secrets` 会被拒绝，避免在第一版部署路径里悄悄扩大权限。

## Build & Deploy 描述符

如果项目没有预构建镜像，可以在 `project.metadata.deployment.build_deploy` 声明从源码构建：

```yaml
project:
  metadata:
    deployment:
      build_deploy:
        source_url: https://github.com/example/app.git
        ref_name: HEAD
        strategy: dockerfile # dockerfile | nixpacks
        dockerfile_path: Dockerfile # optional
        container_port: 3000
        port_name: web
        route_id: my-app-web
        health_path: /healthz
        runtime_env:
          - name: NODE_ENV
            value: production
          - name: OPENAI_API_KEY
            secret_ref: project:OPENAI_API_KEY
        runtime_mounts:
          - source_host_path: /srv/ygg-data/my-app
            container_path: /app/data
            mode: ro
            approved: true
            high_risk_approved: false
            reason: persistent app data
```

`dockerfile` 策略使用仓库里的 Dockerfile。`nixpacks` 策略先运行本机 `nixpacks` 生成 Dockerfile / context，再由 `docker-runtime-lab` 通过 Docker 构建镜像。`nixpacks` 不可用时 fail-closed。

运行时 secret 只接受 `store:` / `project:` / `env:` 形式的 `secret_ref`。原始 secret 由 host 私有 Docker runner 注入容器，不经过 `docker-runtime-lab` 包边界，也不写入事件、日志或 job 状态。构建时 secret 暂不支持，遇到 build secret 字段会 fail-closed。

volume 可以指向任意宿主路径，但必须逐条批准。默认建议只读；读写挂载必须额外确认。host 会拒绝 Docker socket、系统目录、密钥目录、Yggdrasil secret store、过宽 home 目录以及这些路径的祖先目录。

## 显式 Deploy 流程

项目控制台里的 Deploy 按钮不会自动触发。用户确认后，请求发到 host-plane 的 `POST /host/v1/deploy`，由 host broker 在服务端串起整条链路（浏览器只是瘦客户端，不再亲自编排）：

1. host 侧重新校验请求（不信任客户端字段）。
2. `kernel.v1.port.lease`：向 host 租 loopback 端口。
3. `kernel.v1.capability.invoke` → `official/docker-runtime-lab/start_container`：启动 Docker 容器，传入 `approved: true`、`host_port` 与 `port_lease_id`。
4. `kernel.v1.proxy.register`：把 route 绑定到刚租到的 port lease（此时 `ready=false`）。
5. readiness probe：对 loopback 端口做 TCP 连接（带可选 health_path 的 HTTP 探测），有界超时内成功才把 route 翻成 `ready=true` 并返回成功。

任意一步失败后，broker 会反向回滚：注销 proxy、停止刚启动的 container、释放 port lease。因为编排在 host 侧，关闭浏览器标签页不会留下孤儿容器或端口租约。

Stop deployment（`POST /host/v1/deploy/stop`）按 Docker label（`route_id`）查找并停止对应容器，不依赖浏览器记住的 container id，也不会因为同名 `port_name` 去停止未知容器。

## Build & Deploy 流程

Build & Deploy 使用 `POST /host/v1/build-deploy`。默认立即返回 `job_id`、status URL 和 SSE events URL，长耗时工作留在 host broker 后台执行：

1. 校验源码 URL、策略、runtime env、runtime mounts 和用户批准。
2. 通过 `git-tools-lab` 克隆到项目工作区。
3. 若策略为 `nixpacks`，先生成 Dockerfile / context。
4. 调用 `official/docker-runtime-lab/build_image` 构建镜像，打上 `project_id`、`build_id`、`source_commit`、`strategy`、`build_descriptor_hash` 等 label。
5. 进入普通部署链路：port lease → 容器启动 → proxy 注册 → readiness probe。
6. 成功后 route 变成 `ready=true`；失败或取消会按已获取资源反向回滚。

job 只保存在内存里，用于 UI 进度和日志环形缓冲。host 重启后 job 日志可能丢失；真实部署状态仍由 Docker label、port/proxy 事件回放和重启对账恢复。

## `project.start` 不自动部署

`kernel.v1.project.start` 仍是项目状态机：打开或复用项目 session，标记 Running，返回 `session_id`。它不启动进程、不分配端口、不注册 proxy。

部署是单独的、显式的 host-broker 行为。这样可以保留“打开项目 UI”和“运行外部服务”之间的可见边界。

## 安全红线

- 默认全拒。没有 profile opt-in，不会真实启动本地进程。
- 端口只绑定 loopback。
- proxy upstream 必须引用 active port lease，并且 `port_name` 必须匹配。
- 反代不跟随上游 redirect，不转发 `Set-Cookie` / `Location` / CORS 等危险响应头。
- 预构建镜像部署不接受 env / volume / secret。源码 Build & Deploy 只接受显式批准的 runtime env / volume；runtime secret 原始值只在 host 私有 runner 内注入，构建时 secret 暂不支持。
- Docker 通过普通能力包实现，没有官方快速路径。

## 后续

- 原生执行仍只适合 trusted / dev 场景，不是完整 OS 沙箱。
- **自动重启**还没做，且是单独的后续阶段。健康监督只做监测 + 翻 readiness + 审计；它不会自动重新部署挂掉的容器。自动重启需要先把「部署意图」（image 等）持久化到 host-plane，且不能让 Docker 语义渗进内核 proxy / port 记录——这个设计单独做。
- 远程目标、多端公开暴露还没做；端口目前只绑 loopback。
- Docker 描述符还没有 pull 进度和长期日志归档。
- 外部项目 wizard 后续可以生成部署描述符，但仍必须让用户显式点击 Deploy。
