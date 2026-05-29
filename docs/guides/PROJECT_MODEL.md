# 项目模型

> [English](./PROJECT_MODEL.en.md) · [中文](./PROJECT_MODEL.md)

Yggdrasil 是一个平台。平台上跑很多项目。每个项目像 Steam 货架上的一个游戏 ——
独立的入口、独立的状态、可以单独玩或同时跑多个。

## 三层架构

```text
内核 (无内容, 不变)
  ↓ 提供协议/调度/包注册/能力分发/事件流/权限
能力包 (可复用, 跨项目共享)
  ↓ 提供能力 (model-provider-lab / persona-lab / ...)
项目 (使用能力包)
  YdlTavern / 未来 coding agent / 未来 image-gen / ...
```

内核完全不知道项目存在。项目是宿主/运行时概念，不是内核 ontology。

## Steam 类比

| Steam | Yggdrasil |
|---|---|
| Steam 客户端 | Yggdrasil 平台 |
| 游戏库 | Home 屏幕 |
| 游戏卡片 | 项目卡片 |
| 游戏存档目录 | per-project 数据目录 |
| Steam 钱包 | 平台 secret store |
| 游戏自己的 DLC key | 项目 secret store |
| 游戏共用 OS 驱动 | 项目共用能力包 |

## 项目类型

三种 `project.type` 区分项目来源。

### yggdrasil_native

仓库根目录有 `project.yaml`，引用 Yggdrasil 能力包。这是为 Yggdrasil 设计的项目，
是首选形式。

```yaml
schema_version: 1
project:
  id: my-project__abc12345
  title: My Project
  description: 一段简介
  type: yggdrasil_native
  entry_surface_id: my-namespace/play
  packages:
    - packages/foo/manifest.yaml
    - packages/bar/manifest.yaml
  secret_policy:
    fallback_to_platform: true
```

### external_wrapped

外部项目（普通 git/npm 仓库），通过适配器包包装。安装时如果选 “wrap with adapter”，
Yggdrasil 会借助 `adapter-generator-lab` 生成一个适配器包，然后把外部项目接入。

### external_workspace

外部项目作为 agent workspace 接入，不包装。适合临时使用、agent 协助修改的场景。
默认行为（无 TTY、不指定标志）。

## ProjectDescriptor

`project.yaml` 的顶层是 `ProjectDescriptor`。它描述项目实例，不描述某个包本身。

常用字段：

| 字段 | 含义 |
|---|---|
| `id` | 稳定 project id，用于目录、CLI、Home card。 |
| `title` | 给用户看的项目名。 |
| `description` | Home card 和详情页展示。 |
| `type` | `yggdrasil_native` / `external_wrapped` / `external_workspace`。 |
| `entry_surface_id` | 点 Play 后进入的 surface contribution id。 |
| `packages` | 必装 package manifest 路径。 |
| `optional_packages` | 可选 package manifest 路径。 |
| `required_surfaces` | 项目认为必须存在的 surface ids。 |
| `secret_policy` | 项目密钥解析策略。 |

`entry_surface_id` 应该匹配某个 package manifest 中 `slot: experience_entry` 的 surface。
例如 YdlTavern 使用 `ydltavern/play`。

## 项目目录结构

```text
~/.yggdrasil/projects/<project_id>/
├── project.yaml          # ProjectDescriptor 副本
├── secrets.dat           # age 加密的项目 secret store
├── sessions/             # 项目级 session 数据
├── state/                # 包能存的项目级状态
└── lockfile.toml         # 项目锁定的包版本
```

文件权限：0700 目录，0600 文件（Unix）。
加密：同一份 master key（位于 `~/.yggdrasil/secret-store.key` 或 OS keyring）。

## 软隔离 + 平台回退

项目隔离是软的，不是租户级硬隔离。默认行为：

- 项目自己的 secret 优先（`secret_ref:project:NAME`）
- 项目内没有 → 回退平台（`secret_policy.fallback_to_platform: true` 是默认）
- 平台也没有 → fail-closed

这样设计的意图：用户配一次 `OPENAI_API_KEY` 在平台层，所有项目能用。某个项目想用
单独的 key，在该项目设置里 override。两条路径都对用户可见，不是默默 fallback。

强隔离的项目可以关掉 fallback：

```yaml
secret_policy:
  fallback_to_platform: false
  require_per_project:
    - GITHUB_PAT       # 这个名字必须项目级配, 不允许平台 fallback
```

## 生命周期

```text
yg install <url>
  ↓ (检测 project.yaml / 走 wizard)
Installed (注册到 ProjectRegistry, 可见于 Home)
  ↓ yg project start (or Home 点 Play)
Starting → Running
  ↓ yg project stop
Stopping → Stopped
  ↓ yg uninstall
(询问保留数据)
  ├─ Keep: 移到 ~/.yggdrasil/projects/.archived/<id>/
  └─ Delete: 直接 rm -rf
```

任何状态都可以失败 → Failed。

## CLI 命令

```bash
# 安装项目
yg install github.com/user/repo
yg install github.com/user/repo --wrap-as-adapter   # 外部项目: 包装
yg install github.com/user/repo --workspace-only    # 外部项目: 工作区

# 查看项目
yg project list
yg project info <id>
yg project status <id>

# 控制
yg project start <id>
yg project stop <id>
yg update --project-id <id> [--check-only]

# 卸载
yg uninstall <id>                # 交互式问数据怎么办
yg uninstall <id> --keep-data    # 保留 (移到 .archived)
yg uninstall <id> --delete-data  # 立即删除
```

## Home 屏幕

`clients/web` 的 Home 路由显示所有已安装项目的卡片：

```text
┌─────────────────┐  ┌─────────────────┐
│   YdlTavern     │  │  Coding Agent   │
│   ●Running      │  │  ◯Stopped       │
│   [Play]        │  │  [Play]         │
└─────────────────┘  └─────────────────┘
┌─────────────────┐
│  + Install      │
└─────────────────┘
```

状态指示：

- ● Running (绿)
- ◯ Stopped / Installed (灰)
- ⏳ Starting / Stopping (黄)
- ❌ Failed (红)

点 Play 调用 `kernel.v1.project.start`，启动后导航到项目的 `entry_surface`。

项目页带平台侧控制台：显示 bundle、包、最近事件、更新诊断与部署诊断；更新检查与执行通过 `official/install-lab/check_for_updates` / `update_project`，仍走公开协议 `kernel.v1.capability.invoke`。

## Play 流程

Home 点 Play 后，Web shell 与 host 走固定的公开协议序列：

1. 用户点项目卡上的 Play。
2. `clients/web` 调 `kernel.v1.project.start`。
3. host 把项目状态转为 Running，创建或复用项目 session。
4. session 写入 `metadata.project_id`，并加上 `project:<id>` label。
5. `project.start` 返回 `session_id` 与 `already_running`。
6. `clients/web` 调 `kernel.v1.surface.resolve_bundle`，用项目的 `entry_surface_id` 拿 surface 包 URL。
7. `mountSurface` 挂载 sandboxed iframe。
8. iframe `initialProps` 注入 `sessionId` 与 `projectId`。
9. surface 内的 `callHostRpc` / `invokeCapability` 自动带 `session_id`。
10. host 后续把 `ProtocolContext.session_id` 传到 capability 与 outbound dispatch。

这条链路让项目级 secret 解析能从 session metadata 找到项目范围，也让真实模型调用回到同一个项目 session。端到端说明见 [`REAL_MODEL_END_TO_END.md`](REAL_MODEL_END_TO_END.md)。

注：这个 `sessionId` 之后被用于：

- 所有 RPC 调用自动附带（`callHostRpc` 通过 `setActiveSessionId` 读取）。
- 流式调用（`streamCapability`）用它作为订阅范围，接收 `kernel/v1/stream.*` 事件。

## 显式部署

`project.start` 不启动外部进程。它只打开项目 session，并把项目标记为 Running。

若项目需要启动 Docker HTTP 服务，可以在 `project.metadata.deployment.docker` 声明最小部署描述符。Web 项目控制台会显示 Deploy / Stop 按钮，用户确认后由 Web shell 作为 host broker 串联：

1. `kernel.v1.port.lease` 租 loopback 端口。
2. `official/docker-runtime-lab/start_container` 启动容器。
3. `kernel.v1.proxy.register` 注册 HTTP/WebSocket 反代 route。

这条路径是显式操作，不会在打开项目时自动执行。完整说明见 [`DEPLOYMENT_RUNTIME.md`](DEPLOYMENT_RUNTIME.md)。

## 协议

宿主管理项目的协议（HostAdmin/HostDev only，普通包不能调）：

```text
kernel.v1.project.list      列出已安装项目
kernel.v1.project.get       项目详情
kernel.v1.project.start     启动项目
kernel.v1.project.stop      停止项目
kernel.v1.project.status    项目状态
```

部署运行时协议（HostAdmin/HostDev only，普通包不能调）：

```text
kernel.v1.target.*   运行目标
kernel.v1.exec.*     受控本地执行
kernel.v1.port.*     loopback 端口租约
kernel.v1.proxy.*    HTTP/WebSocket route
```

生命周期事件：

```text
kernel/v1/project.installed
kernel/v1/project.started
kernel/v1/project.stopped
kernel/v1/project.uninstalled
```

## 与 Composition 的区别

| Composition (现有) | Project (新) |
|---|---|
| 静态 package-set 描述符 | 运行时实例 + 状态 |
| `ygg composition check` 校验 | `yg project list/start/stop` |
| 用于 share/import bundles | 用于 Home + 安装 lifecycle |
| `ygg-cli` 内部类型 | `ygg-core` 公开类型 |

未来一个 composition 模板可以实例化为多个项目（不同 id，同一个包集）。当前
版本不强求这一点 —— 一个项目通常就是一个 composition 的具体实例。

## 安装检测

`yg install <url>` 先看仓库根目录是否有 `project.yaml`。

- 有且 `type: yggdrasil_native`：按原生项目安装。
- 没有：进入外部项目 wizard。
- 有但解析失败：fail-closed，要求修正 descriptor。

无 TTY 且没有显式 flag 时，外部项目默认进入 `external_workspace`，避免自动生成包装代码。

## 不做的事 (推迟)

- 多用户/项目成员/access control
- 项目导入/导出 bundle (sharing-lab 已有 bundle 格式)
- 多并发租户 (`project_id` 进 `ProtocolContext`) — 延后 / 计划中
- 项目自动归档清理 (留给用户手动)
- 项目市场/marketplace (违反平台开放原则)
