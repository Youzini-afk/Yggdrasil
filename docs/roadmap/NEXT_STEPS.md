# 下一步

> [English](./NEXT_STEPS.en.md) · [中文](./NEXT_STEPS.md)

这份文档讲 Yggdrasil 接下来要往哪走。已经完成的阶段历史不在这里——那是 [`ALPHA_STATUS.md`](../ALPHA_STATUS.md) 的事。

## 我们现在在哪

平台底座已经搭好。

- 内核对内容无意见、官方包没有特权、入口形式平等。
- 安全执行底座完整：`secret_ref`、`EnvSecretResolver`、网络声明、外发审计与脱敏、`LiveHttpOutboundExecutor`、流式与取消生命周期。
- 体验运行时、可玩纵切片、可观测性、记忆、分享/分发——都以普通能力包的形态落地。
- 多 provider 模型接入、真实出网调用、transport-neutral 推理接缝、Agentic Forge Beta——全部完成。
- 外部项目操作平面、存储中立性、PostgreSQL 事件后端、TDB 真实 Rust adapter——全部完成。
- 329 个具名 conformance 用例 + crate / service 单元测试通过。

下一阶段不再继续摊大表面积，而是由真实的 AI 原生可玩体验来牵引剩下的工作。

## 长期方向：体验牵引

详见 [`../product/EXPERIENCE_LED_PLATFORM_BETA.md`](../product/EXPERIENCE_LED_PLATFORM_BETA.md)。

要点：

- 用一两个真实的可玩体验作为压力源，倒逼底座剩下的工作浮现出来；
- 任何新增基础设施都要回答「哪个真实的玩家或创作者循环卡住了」；
- 不再凭计划堆「Alpha + Beta + Phase」。

## 近期会推进的底座工作

下面这些项目不构成新阶段，但是已知该做、也会真实推进：

- **git 安装能力包的自动 resolve / pin / apply。** 受控 git fetch、`kernel.outbound.git_fetch`、`official/package-installer-lab`、profile 级 lockfile 与手动 pin CLI 已落地；下一步是把 `ygg package install <github-url>` 接成自动解析 commit/content hash、审批后写 lockfile 并加载包。当前能力见 [`../guides/GIT_PACKAGE_INSTALLATION.md`](../guides/GIT_PACKAGE_INSTALLATION.md)。
- 包持有的 projection 执行。
- 能力包身份的 `event.subscribe` 权限，以及更广的流式传输一致性。
- 钩子处理器的超时与错误审计。
- 能力 provider 的持久选择策略。
- conformance 里更广的传输层一致性覆盖。
- WASM 与远程包入口的执行。
- 内容寻址的 blob 存储与运行时身份层面的资产权限。

这些项目解除某些场景的阻塞，但都不应该成为下一阶段的中心。

## 接入项目（独立仓库）

下面这些是跑在 Yggdrasil 之上、通过公开协议消费平台的独立项目。它们不在本仓库里：

- **YdlTavern** —— 一个跑在 Yggdrasil 之上、兼容 SillyTavern 资源与扩展的独立接入项目：支持 SillyTavern 的角色卡、世界书、预设、聊天历史和扩展 API，底层走 Yggdrasil。仓库：<https://github.com/Youzini-afk/Yggdrasil-Tavern>。Yggdrasil 这边的边界见 [`../tavern/TAVERN_COMPAT.md`](../tavern/TAVERN_COMPAT.md)。

## 内核范围内的无限期延后

下面这些不会进内核，会以普通能力包或后续工作出现：
- pi 作为产品壳的整包嵌入 —— 见 [`../architecture/PI_INTEGRATION.md`](../architecture/PI_INTEGRATION.md)。Agent 基础设施只能以普通能力包 / SDK 形态推进。
- 外部游戏引擎桥接（UE5 / Godot / Unity / Web 客户端）。
- 享受特权的内置 Studio、绕过公开协议的 UI、由内核拥有的官方审查器。公开协议的客户端和能力包贡献的 surface 可以继续演化。
- 内核里的记忆模型、世界模拟、导演、提示词渲染、模型 provider 抽象。Agent 循环、生产级真实模型调用、模型 provider，都只能作为普通能力包存在。
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
- Git Package Installation Substrate
