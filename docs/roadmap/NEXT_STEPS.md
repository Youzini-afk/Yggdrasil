# 下一步

> [English](./NEXT_STEPS.en.md) · [中文](./NEXT_STEPS.md)

这份文档讲 Yggdrasil 接下来要往哪走。已经完成的阶段历史不在这里——那是 [`ALPHA_STATUS.md`](../ALPHA_STATUS.md) 的事。

## 我们现在在哪

平台底座已经搭好。

- 内核对内容无意见、官方包没有特权、入口形式平等。
- 安全执行底座完整：`secret_ref`、`EnvSecretResolver`、`StoreSecretResolver`、本地加密 secret store、网络声明、外发审计与脱敏、live HTTP/WebSocket 出站执行器、出站一元 / SSE-NDJSON-raw 流 / WebSocket 三原语、流式与取消生命周期。
- 体验运行时、可玩纵切片、可观测性、记忆、分享/分发——都以普通能力包的形态落地。
- 多 provider 模型接入、真实出网调用、transport-neutral 推理接缝、Agentic Forge Beta——全部完成。
- 外部项目操作平面、存储中立性、PostgreSQL 事件后端、TDB 真实 Rust adapter——全部完成。
- Web shell 的 Vite 构建、iframe SurfaceHost、Tauri 2.x desktop wrapper、tag 触发的跨平台 release pipeline——全部完成。
- Round 9 Contract Foundation 已完成：Contract V1、能力句柄、bindings 注入、Path B、effect audit、conformance kit、SDK 生成已落地；Round 10A.3 后共有 115 schemas。
- 427 个具名 conformance 用例 + crate / service 单元测试通过。

下一阶段不再继续摊大表面积，而是由真实的 AI 原生可玩体验来牵引剩下的工作。

## 长期方向：体验牵引

平台立场见 [`../product/PLAY_CREATION_MODEL.md`](../product/PLAY_CREATION_MODEL.md)。

要点：

- 用一两个真实的可玩体验作为压力源，倒逼底座剩下的工作浮现出来；
- 任何新增基础设施都要回答「哪个真实的玩家或创作者循环卡住了」；
- 不再按计划预先堆叠多层路标。

## 近期会推进的底座工作

下面这些项目不构成新阶段，但是已知该做、也会真实推进：

- 包安装的基础层已完成；Round 10A.1 已完成默认值简化和本地加密 secret store；Round 10A.2 已完成 Home 项目架、项目生命周期、项目级 secret fallback 和 YdlTavern project.yaml；Round 10A.3 已打通 YdlTavern Send → engine `model.live_call` → live outbound → provider response 的真实路径；Round 10A.4 已补齐 surface streaming response UX。
- 平台 UI 已升级并完成 release-closure：`clients/web` 切到 React 19 + Tailwind v4 + Motion + Radix + Phosphor，所有屏（Home/Settings 五个面板/Install 三步流程/External wizard/Failure modal/Project frame/Toast 系统）已实现。Install 通过普通 `official/install-lab` 能力包接真实 plan/execute，Failure Modal 接 redacted package diagnostics，Disk Usage 接项目 `storage_summary`，并完成暗色模式、响应式、a11y 与 lazy chunk 收口。视觉规则与组件目录见 [`../design/PLATFORM_UI_DESIGN.md`](../design/PLATFORM_UI_DESIGN.md) 与 [`../../clients/web/README.md`](../../clients/web/README.md)。
- 代码组织收口已完成：runtime public protocol handler、`official/install-lab`、Web Install/Home flow、conformance registry、schema exporter 都已拆分到按域模块；这是行为保持的维护性整理，不改变公开协议。
- 后续 distribution polish：Sigstore、Tauri UI 安装路径、`yg gc`、自动更新守护、code signing / notarization、真实应用图标。
- OS keyring 集成延后，等 CI / 跨平台构建环境具备稳定系统依赖时再恢复。
- 包持有的 projection 执行。
- 能力包身份的 `event.subscribe` 权限，以及更广的流式传输一致性。
- 钩子处理器的超时与错误审计。
- 能力 provider 的持久选择策略。
- conformance 里更广的传输层一致性覆盖。
- 使用本地 mock HTTP / WebSocket server 扩展真实模型出站 conformance，不引入默认公网依赖。
- OpenAI Realtime / Gemini Live 等真实 WebSocket smoke，保持显式 opt-in，不进入默认 CI。
- 更多 provider registry、tokenizer / 计费 metadata 适配，仍作为普通能力包实现。
- WASM 与远程包入口的执行。
- 内容寻址的 blob 存储与运行时身份层面的资产权限。
- Desktop release code signing / notarization。
- Desktop auto-updater integration。
- 替换 placeholder desktop icons 为真实应用图标。
- Surface lifecycle hooks（`onClose`、`onProposalDraft` 等）。
- Cross-origin surface bundle allowlist（含 CSP 与 origin 校验）。
- Desktop wrapper 以受控 managed subprocess 启动 / 停止 `host serve`。
- Phase B 优化（next）：使用 [`../../perf/baseline.json`](../../perf/baseline.json) 作为 regression reference，先测量再改动。

## Round 10A — Package Installation Foundation（完成）

- `yg install <github-url>` 端到端。
- `official/git-tools-lab` + `integrity-lab` + `install-lab` 三个能力包。
- `manifest.requires` 字段 + Lockfile (`yggdrasil.lock.v1`)。
- `~/.yggdrasil` 文件系统约定。
- 交互式同意提示 + 静态 conformance 集成。
- Round 10A.1 follow-up：默认值放宽到 cargo/npm/pip 技术基线；`--require-signed` / `--strict` 改为 opt-in；新增 `official/secret-store-lab`、`StoreSecretResolver` 与 YdlTavern API Connections 加密保存。

## Round 10A.1 — Install Simplification + Secret Store（完成）

- `yg install <url>` 默认不要求签名，conformance 失败默认 warning-only。
- `--require-signed` 和 `--strict` 提供受控环境 opt-in。
- `official/secret-store-lab` 提供 age 加密本地 secret store。
- `StoreSecretResolver` 与 `CompositeSecretResolver` 支持 `secret_ref:store:*` + `secret_ref:env:*`。
- YdlTavern API Connections 抽屉已接入 paste + save → encrypted store。
- OS keyring 与 `yg secret put / list / delete` CLI 延后。


## Round 10A.2 — Steam-Game Project Concept（完成）

- 项目成为一等运行时概念：`ProjectDescriptor`、`ProjectRegistry`、`ProjectType`、`SecretPolicy`。
- `~/.yggdrasil/projects/<id>/` 目录、项目级 secret store、`secret_ref:project:*` 与平台 fallback 已落地。
- 安装检测区分原生 `project.yaml` 与外部项目 wizard（wrap / workspace）。
- `yg project list/info/status/start/stop` 与 `yg uninstall` 归档提示已落地。
- `kernel.v1.project.list/get/start/stop/status` 与项目 lifecycle events 已落地。
- Home 屏幕现在是项目货架；YdlTavern 声明为 `yggdrasil_native` 项目。
- 多租户级 `ProtocolContext.project_id` / 基于 session 的项目范围强隔离推迟到 Round 11+。

## Round 10A.3 — End-to-End Real Path（完成）

- Surface bundle resolution 已由 metadata 驱动，并新增 `kernel.v1.surface.resolve_bundle`。
- `project.start` 会打开项目 session、写 `metadata.project_id`，并返回 `session_id` / `already_running`。
- `project.get` / `status` 在 Running 时返回 `running_session_id`，`project.stop` 会关闭项目 session。
- `clients/web` 将 `sessionId` / `projectId` 注入 surface initialProps，surface RPC 自动带 `session_id`。
- YdlTavern `SendForm` 已接到 engine `model.live_call`，API Connections 支持 platform/project 保存范围，engine manifest 声明 `secret_ref:project:*`。
- 文档收敛见 [`../guides/REAL_MODEL_END_TO_END.md`](../guides/REAL_MODEL_END_TO_END.md)。

## Round 10A.4 — Streaming UX（完成）

- Surface-host stream postMessage protocol 已落地：`stream.subscribe` / `stream.frame` / `stream.ended` / `stream.error` / `stream.unsubscribe`。
- Host bridge 通过 `client.subscribeEvents` 订阅 session SSE，过滤并转发匹配 `stream_id` 的 `kernel/v1/stream.*` 事件。
- YdlTavern `streamCapability` helper、`TavernProvider.sendMessage` streaming branch、chunk delta 累加更新和 Stop/cancelGeneration 已落地。
- 单 chat 多并发生成、token-rate UI、Realtime/WebSocket streaming UX 仍推迟；Round 10B 仍是下一焦点。

## Round 10B — WIT/WASM Contract Frontier（下一焦点）

- WIT worlds + WASM entry form（从 scaffold 推进 partial）。
- Powerbox late-bound provider 选择。
- Cap'n Proto / Biscuit 实验。
- 10A.3 已落地；Round 10B 继续保持现有 Contract Frontier 描述，不再扩展模型/聊天语义进内核。

## Distribution polish（下一批发布收口）

- `yg gc` 孤立 store 回收。
- Tauri UI 安装路径。
- Sigstore keyless 验签。
- 自动更新守护进程。
- 二进制包分发。
- 基于 `ProtocolContext.session_id` 的多租户项目范围加固：把项目身份显式传入运行时权限、事件与 resolver 上下文。

## Round 10：Contract Frontier

Round 10 的中心是把 v1 契约推进到更远的边界，而不是扩大内容语义：

- WASM WIT worlds：把 bindings 映射成 resource imports，补齐 wasm package execution。
- Remote packages：SPIFFE 身份、Biscuit token 兑换、远端 package lifecycle 与 audit。
- Powerbox：显式用户/host 授权、句柄转授、临时权威与可撤销 delegation。
- Advanced authority patterns：跨包委派、衰减链审计、租约刷新、批量撤销。
- Conformance kit library：把 package conformance 抽成可嵌入库，支持项目自定义检查。
- SDK packaging：完善 npm 发布、Rust crate 发布与 OpenAPI/codegen 文档。

Round 10 之后仍保留的底座项：package-owned projection 执行、package-principal event.subscribe、hook timeout/error audit、持久 provider selection、更广传输一致性、content-addressed blob 存储、桌面签名/自动更新、surface lifecycle 与跨源 allowlist。

这些项目解除某些场景的阻塞，但都不应该成为下一阶段的中心。

## 接入项目（独立仓库）

下面这些是跑在 Yggdrasil 之上、通过公开协议消费平台的独立项目。它们不在本仓库里：

- **YdlTavern** —— 一个跑在 Yggdrasil 之上、兼容 SillyTavern 资源与扩展的独立接入项目：支持 SillyTavern 的角色卡、世界书、预设、聊天历史和扩展 API，底层走 Yggdrasil。仓库：<https://github.com/Youzini-afk/Yggdrasil-Tavern>。Yggdrasil 这边的边界见 [`../tavern/TAVERN_COMPAT.md`](../tavern/TAVERN_COMPAT.md)。

## 内核范围内的无限期延后

下面这些不会进内核，会以普通能力包或后续工作出现：
- pi 作为产品壳的整包嵌入 —— 见 [`../architecture/PI_INTEGRATION.md`](../architecture/PI_INTEGRATION.md)。Agent 基础设施只能以普通能力包 / SDK 形态推进。
- 外部游戏引擎桥接（UE5 / Godot / Unity / Web 客户端）。
- 享受特权的内置 Studio、绕过公开协议的 UI、由内核拥有的官方审查器。公开协议的客户端和能力包贡献的 surface 可以继续演化。
- 内核里的记忆模型、世界模拟、导演、提示词渲染、模型 provider 抽象。Agent 循环、生产级模型 provider 能力，都只能作为普通能力包存在。
- 市场、包签名网络、依赖解析经济。本地分享 proof 已完成，见 [`../guides/SHARING_DISTRIBUTION.md`](../guides/SHARING_DISTRIBUTION.md)。

## 评分标准

每个新阶段都按章程纪律来评：

- 不让内容形态的概念渗进内核；
- 任何路径上都不让官方包获得特权；
- 所有能力包和 UI 行为都走公开协议边界；
- 新增的底座必须能回答某个真实可玩体验的压力。

## 已完成阶段一览

按时间顺序，每条都有 ALPHA_STATUS 与 conformance 支撑。详细描述见 [`../ALPHA_STATUS.md`](../ALPHA_STATUS.md)。

- Platform Foundation Alpha
- Play / Forge Surface Contract Beta
- Code Health Split Alpha
- Authoring & Composition Beta+
- Secure Execution Substrate Alpha
- Optional Text Engine Alpha
- Agent Infrastructure Alpha
- Model Provider Integration Alpha
- Live Model Calls Alpha
- Creative Inference Capability Alpha
- Agentic Forge Beta
- Experience Beta 0–6（thin runtime → playable slice → state/asset pipeline → observability → memory/knowledge → creator loop → sharing/distribution）
- Performance & Code Health Beta
- External Project Operating Plane Alpha
- Storage Backend Neutrality Alpha
- PostgreSQL + TDB Integration Alpha
- Real TDB Rust Adapter Alpha
- Package Installation Foundation (Round 10A)
- Install Simplification + Secret Store (Round 10A.1)
- Steam-Game Project Concept (Round 10A.2)
- Streaming UX (Round 10A.4)
- Outbound WebSocket Substrate
- Shell + Release S-track（Vite Web build、iframe SurfaceHost、Tauri desktop wrapper、GitHub Actions release）
- Round 9 Contract Foundation（Contract V1、capability handles、bindings、Path B、audit、conformance kit、SDK generation）
