# 外部项目「构建 + 部署」计划（临时）

> 这是临时执行计划，做完即删，内容收敛进 `docs/guides/DEPLOYMENT_RUNTIME.md`。

## 目标

把当前「部署一个预构建镜像」升级成「克隆 + 构建 + 运行任意 git 项目」，对标 Coolify / Zeabur / Railway 的 build-pack 分层。第一刀就把三层都接进来：

| 策略 | 含义 | 状态 |
|---|---|---|
| `dockerimage` | 项目声明预构建镜像，直接拉 | 已实现（现状） |
| `dockerfile` | 仓库带 Dockerfile，`docker build` | 本计划新增 |
| `nixpacks` | 无 Dockerfile，自动识别语言/框架生成 Dockerfile 再 build | 本计划新增 |
| `compose` | 多服务编排 | 留作后续，明确不在本刀 |

同时放开 env / volume 注入（Q2）：env 支持明文 + secret_ref 宿主侧解析；volume 放开到任意宿主路径，但每条挂载显式批准 + 审计 + 危险路径硬拒。

## 现状事实（来自调查，file:line 见调查记录）

- `docker-runtime-lab`：无 build 能力；只 pull/create/start/stop/status/logs/list_managed。显式拒 env/volumes/binds/mounts/secrets。容器打 4 个 label：`managed-by=yggdrasil`、`yggdrasil.package_id`、`yggdrasil.route_id`、`yggdrasil.port_lease_id`。
- host broker `/host/v1/deploy`：`deny_unknown_fields`，纯预构建镜像（image/container_port/port_name/route_id/health_path/pull_if_missing）。无构建步骤。
- 部署描述符 `project.metadata.deployment.docker`：纯预构建镜像，Web 端解析，禁 env/secrets/volumes。
- `git-tools-lab`：**真实** gix clone，`fetch_tree` 把树写到 dest_dir。这是构建上下文的现成来源。
- `project-intake-lab` / `workspace-lab`：**全是 plan-only 假执行**，不 clone、不 install、不 run、不扫真实文件。本刀的 Docker 构建不依赖它们的「run」（构建在 Docker 内发生）；它们的「在宿主跑 npm install」属于原生路径（Pinokio 式），是后续原生 runtime 的事，不在本刀。
- secret 链：**真实**。`CompositeSecretResolver`（env:/store:/project:）、`ProjectStoreSecretResolver`（每项目加密 store）、outbound `secret_headers` 宿主侧解析注入（原始值不跨包边界）——这是 env 注入要照抄的模式。
- 文件系统：`~/.yggdrasil/projects/<id>/` 有 `state/`，但无 volume / workspace 约定，需要新增 `workspace/` 约定。

## 架构决策（已经 @oracle 复核，下为修正后版本）

1. **构建发生在 `docker-runtime-lab`**（它本就持有 bollard）。新增 `build_image` 能力，带 `strategy` 字段（`dockerfile` | `nixpacks`），从工作区 tar context + Dockerfile 路径用 bollard build，宿主控资源限额 / build 超时 / label / 日志流。strategy 内部模块化可插拔（Railpack/zbpack 后续）。**约束：该包只构建容器镜像，不得变成通用宿主构建器。**
2. **nixpacks 用 `--out` 模式**：无 Dockerfile 时，`build_image` 先跑 `nixpacks build <ctx> --out <ctx>` 生成 Dockerfile + 上下文，再用 bollard build。宿主始终掌控真正的 `docker build`（审批 / 限额 / 审计）。nixpacks 二进制不可用时 fail-closed。版本 pin + 记录；策略标记 experimental；生成的 Dockerfile/context 留存供审计。
3. **工作区 = 克隆的源码**（`git-tools-lab` fetch_tree，真实），落到 `~/.yggdrasil/projects/<id>/workspace/`。本刀不需要让 workspace-lab 变真（那是原生路径的事）。
4. **编排在 host-plane broker**（沿用 Phase C 先例）：新增 `/host/v1/build-deploy`，用 **build job id** 串 clone → 检测策略 → (buildpack 生成 Dockerfile) → `build_image` → 原有 lease/run/proxy/readiness。构建耗时长，不能塞进单个同步 HTTP 请求——broker 持有 job 生命周期。浏览器是瘦客户端。
5. **env 注入（secret 边界关键修正）**：
   - 运行时**明文** env `{ name, value }`：可经包边界，直接进容器。
   - 运行时 **secret** env `{ name, secret_ref }`：**原始值绝不经过 `docker-runtime-lab` 包边界**（否则违反既有 secret_headers 纪律）。secret 由宿主侧解析后，通过**宿主侧执行器路径**注入容器 create 调用——即「最终 Docker create + env 注入」这一步由宿主侧持有 resolved secret，不把原始值交给包代码。具体落地方式在 B4 定（见开放问题 2 的裁决）。
   - **构建时 secret 本刀不做**：build args 会泄进 `docker history`，BuildKit session secret 复杂。任何「构建时 secret」请求 **fail-closed** 明确报错。构建时非密 build args 允许 + 审计。
6. **volume 注入（放开，但加固）**：描述符可声明任意宿主路径挂载；规则：
   - source 路径 canonicalize；拒 symlink 逃逸；首刀要求路径已存在。
   - **每条挂载单独批准**（source / dest / mode / reason）+ 单独审计 + UI 展示。
   - **默认 read-only**；read-write 需更强的高危批准。
   - 拒 mount propagation / 特权 bind。
   - 危险路径**硬拒**（见红线，清单已扩充）；并拒「请求路径是某危险路径的祖先」（如批 `/home/alice` 会暗含 `~/.ssh`）。
7. **构建身份 / provenance（@oracle 标的头号必修）**：引入持久 `build_id`。镜像 tag = 净化项目名 + build_id（不只 short_sha，因为同 commit 不同 Dockerfile/env/build args 会产出不同镜像）。容器/镜像 label 含 `yggdrasil.build_id` / `project_id` / `source_commit` / `strategy` / `build_descriptor_hash`。GC 只删「未被活跃/近期部署引用」的镜像。
8. **内核不变**：无新内核概念。exec/port/proxy 不动。build/workspace/volume 全在普通包 + host broker。审计避免新增大量构建专用内核事件——优先 broker job 状态/日志，非必要不加内核事件。

## 红线

- 生成的 / 用户的 Dockerfile 视为不可信：靠 Docker 隔离 + 资源限额（内存/CPU）+ build 超时；**不**在 Docker 之外另造构建沙箱（业界天花板，Coolify/Zeabur 都不做）。
- **构建密钥本刀不做**：用 build args 会泄进 `docker history`；BuildKit session secret 复杂。任何构建时 secret 请求 fail-closed。运行时 secret 走宿主侧注入（见架构决策 5）。
- 原始 secret 永不跨包边界 / 不进事件 / 不进日志 / 不进 image 层；只在宿主侧解析后注入容器 create。
- nixpacks / Docker 二进制不可用 → fail-closed，明确报错。
- volume 硬拒清单（已 @oracle 扩充）：
  - 容器 socket：`/var/run/docker.sock`、`/run/docker.sock`、rootless docker sock（`$XDG_RUNTIME_DIR` 下）、`/run/containerd/containerd.sock`、`/run/podman/podman.sock`
  - 内核/设备/系统伪文件系统：`/proc`、`/sys`、`/dev`、`/run`、`/var/run`
  - 容器运行时内部：`/var/lib/docker`、`/var/lib/containerd`、`/var/lib/kubelet`
  - 宿主密钥/身份：`/etc/shadow`、`/etc/sudoers`、`/etc/ssh`、`~/.ssh`、`~/.gnupg`、`~/.aws`、`~/.azure`、`~/.config/gcloud`、`~/.kube`、`~/.docker/config.json`
  - Yggdrasil 敏感状态：keys、任意 `secrets.dat`、可能含敏感载荷的事件库
  - **祖先拒绝**：请求路径若是上述任一危险路径的祖先（如 `/home/alice` 暗含 `~/.ssh`），拒绝或要求极高危专门批准（首刀不支持）。
- 镜像 tag 带净化项目名 + build_id（见架构决策 7），避免污染用户 Docker 命名空间 + 避免重复构建 tag 冲突。
- 工作区路径做 canonical containment，拒 `..` / symlink 逃逸（沿用 git-tools-lab 已有的写出防护）。
- **构建上下文限额**：max 大小 + max 文件数；尊重 `.dockerignore`；tar context 路径安全（无绝对路径、无 `..`、无 symlink 意外）。
- **克隆 URL 策略**：至少拒 `file://` 和本地路径；SSH 谨慎对待（首刀倾向只 HTTPS，沿用 git-tools-lab）。
- **build 超时 + 并发构建上限 + 工作区/生成 context 的磁盘配额与清理 + 构建可取消**。

## 阶段（每阶段验证 + 提交 + 推送；顺序已按 @oracle 调整，提早打通端到端）

- **B0** — 本计划 + @oracle 复核 plan + 向用户汇报。（已完成）
- **B1** — 工作区克隆：broker 经 `git-tools-lab` 把 git 源 clone 到 `~/.yggdrasil/projects/<id>/workspace/`，新增 `workspace/` 路径约定；canonical containment 防护；克隆 URL 策略（HTTPS only）。
- **B2** — `docker-runtime-lab/build_image`（dockerfile 策略）：bollard 从 tar context build，资源限额、build 超时、label（含 build_id/provenance）、日志流；构建上下文限额 + `.dockerignore`；Docker 不可用 fail-closed。引入持久 `build_id` + 镜像 tag/provenance 模型。
- **B2.5** — **最小 broker 端到端纵切**：clone → Dockerfile build → 走现有 deploy 路径 run → proxy 通。先证明完整链路打通，再叠加 nixpacks/env/volume 复杂度。
- **B3** — nixpacks 策略：`build_image` 在无 Dockerfile 时跑 `nixpacks --out` 生成 Dockerfile + context 再 build；版本 pin；不可用 fail-closed；策略可插拔。
- **B4** — 运行时 env 注入：明文经包边界；secret 经宿主侧执行器路径注入容器 create（原始值不过 docker-runtime-lab 包边界）；构建时 secret fail-closed。
- **B5** — volume 注入（放开 + 加固）：任意宿主路径、逐条批准、默认只读、危险路径硬拒 + 祖先拒绝、审计。
- **B6** — host broker `/host/v1/build-deploy` 富编排：build job 生命周期 + broker 侧 SSE 日志 + 取消 + 并发上限；描述符扩展（`source: git` + build 配置 + env/volume）；回滚含构建镜像清理；镜像 GC（只删未被活跃部署引用的）。
- **B7** — Web UI：build-deploy 流程、构建日志流式（broker SSE）、env/volume/批准 UI；保持瘦客户端。
- **B8** — conformance + 文档收敛（写进 DEPLOYMENT_RUNTIME guide）+ 删本计划。

## @oracle 复核裁决（记录）

1. **包切分** → `build_image` 放 `docker-runtime-lab`（它持 bollard，"源→镜像"仍属 Docker 运行时域）；内部模块化；约束它不得变成通用宿主构建器。
2. **构建时 secret** → 首刀**只做运行时 secret**，构建时 secret fail-closed。但运行时 secret 的原始值**不得经过 docker-runtime-lab 包边界**——必须宿主侧执行器注入（这是必修，已写进架构决策 5）。
3. **volume 放开** → 用户需求明确，做法 = 任意路径 + 逐条批准 + 审计 + 默认只读 + 危险清单（已扩充）+ 祖先拒绝。read-write 需高危批准。
4. **nixpacks 维护模式** → 现在用 nixpacks（emit 工作流最干净，MIT），策略可插拔，Railpack/zbpack 后续；不在首刀上 Railpack。
5. **编排归属 + 日志** → host broker 持 build job 生命周期，不新开 build-lab；UI 走 **broker 侧 SSE**（`/host/v1/build-deploy/:job_id/events`，环形缓冲 + 脱敏），不把 `kernel.v1.capability.stream` 直接暴露给部署 UI。
6. **头号风险** → **构建身份 / provenance / 镜像 tag / GC**（已写进架构决策 7，列为首刀必做，不是后补）。

## 内核边界确认（@oracle）

无 `kernel.v1.build.*` / `kernel.v1.git.*`；exec/port/proxy 不加 build/workspace/volume 字段；proxy 路由记录不嵌 Docker spec；build/deploy 描述符住项目 metadata / broker 状态 / 包载荷，不进内核协议；避免新增大量构建专用内核事件。
