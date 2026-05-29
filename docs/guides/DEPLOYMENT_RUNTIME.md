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
- Web 项目控制台：显示 target / exec / port / proxy 诊断。若项目声明 Docker 部署描述符，用户可显式点击 Deploy / Stop。

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
        health_path: /healthz # optional, display only for now
        pull_if_missing: false
```

当前 Web broker 只接受这些字段。`env`、`volumes`、`mounts`、`binds`、`secrets` 会被拒绝，避免在第一版部署路径里悄悄扩大权限。

## 显式 Deploy 流程

项目控制台里的 Deploy 按钮不会自动触发。用户确认后，Web shell 作为 host broker 执行：

1. `kernel.v1.port.lease`：向 host 租 loopback 端口。
2. `kernel.v1.capability.invoke` → `official/docker-runtime-lab/start_container`：启动 Docker 容器，传入 `approved: true`、`host_port` 与 `port_lease_id`。
3. `kernel.v1.proxy.register`：把 route 绑定到刚租到的 port lease。

任意一步失败后，broker 会尽力回滚：注销 proxy、停止本页已知的 container、释放 port lease。

Stop deployment 只停止当前页面已知的 container id。它不会因为同名 `port_name` 去停止未知容器。

## `project.start` 不自动部署

`kernel.v1.project.start` 仍是项目状态机：打开或复用项目 session，标记 Running，返回 `session_id`。它不启动进程、不分配端口、不注册 proxy。

部署是单独的、显式的 host-broker 行为。这样可以保留“打开项目 UI”和“运行外部服务”之间的可见边界。

## 安全红线

- 默认全拒。没有 profile opt-in，不会真实启动本地进程。
- 端口只绑定 loopback。
- proxy upstream 必须引用 active port lease，并且 `port_name` 必须匹配。
- 反代不跟随上游 redirect，不转发 `Set-Cookie` / `Location` / CORS 等危险响应头。
- 容器部署不接受任意 env / volume / secret。
- Docker 通过普通能力包实现，没有官方快速路径。

## 后续

- 原生执行仍只适合 trusted / dev 场景，不是完整 OS 沙箱。
- Docker 描述符还没有 pull 进度、健康检查轮询、日志归档和 volume 策略。
- 外部项目 wizard 后续可以生成部署描述符，但仍必须让用户显式点击 Deploy。
