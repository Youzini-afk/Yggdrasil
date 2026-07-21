# 下一步

> [English](./NEXT_STEPS.en.md) · [中文](./NEXT_STEPS.md)

这份文档讲 Yggdrasil 接下来要往哪走。已完成的状态见 [`../ALPHA_STATUS.md`](../ALPHA_STATUS.md)，不在这里复述。

## 现在在哪

- 内核对内容无意见，官方包没有特权，公开协议是唯一入口。
- 安全执行底座完整：`secret_ref`、本地加密 secret store、网络声明、外发审计与脱敏、HTTP/WebSocket 出站执行器、流式与取消生命周期。
- 平台底座完整：包安装、原生项目安装/挂载、profile autoload、installed project surface bundle、surface bundle freshness 防护、项目更新、Home 项目货架、结构化 shell descriptor、独立项目标签页、项目控制台诊断、显式 Docker Deploy broker、target/exec/port/proxy 部署原语、ygg-service HTTP/WebSocket 反代、Settings、真实模型端到端、流式 UX、受限 Surface bridge、桌面 wrapper、release pipeline、Web shell release closure 与代码组织拆分。
- 多 provider 模型接入、transport-neutral 推理接缝、Agentic Forge、外部项目操作平面、存储中立性、PostgreSQL 事件后端、TDB 真实 Rust adapter——都已落地。
- Contract V1 是公开平台规范，80 methods + 59 events + 22 top-level = 161 个 schema 全部通过校验，474 conformance cases 通过。
- 分层迁移已完成前八步：v1 事实漂移修复、Experimental Contract Registry / 显式协商、36 条 owner-namespace canonical/legacy 双栈、SHA-256 object/artifact 基础、EffectReceipt / Change primitives、Protocol Commons 脚手架、独立 package-envelope/component identity、trust 与 composition pin，以及真实 World Bundle 可移植性证明；[`CONTRACT_V2_MIGRATION.md`](CONTRACT_V2_MIGRATION.md) 剩余一步是把客户端迁到 canonical API，并完成一次真实弃用转换。

下一阶段不再继续摊大表面积，而是由真实项目部署、人测和可玩体验来牵引剩下的工作。

> 这里的「完整」指当前 v1 运行闭环，不代表现有 `kernel.v1.*` 边界已经成为永久宪法。
> 长期分层候选见 [`CONSTITUTION_V2.md`](../architecture/CONSTITUTION_V2.md)，逐项归属与临时实施顺序见
> [`CONTRACT_LAYERING_MATRIX.md`](../spec/CONTRACT_LAYERING_MATRIX.md) 和
> [`CONTRACT_V2_MIGRATION.md`](CONTRACT_V2_MIGRATION.md)。候选方案在显式采纳前不改变当前工作状态。

## 长期方向

平台立场见 [`../product/PLAY_CREATION_MODEL.md`](../product/PLAY_CREATION_MODEL.md)。

要点：

- 用一两个真实可玩体验或真实部署项目作为压力源，倒逼底座剩下的工作浮现出来。
- 任何新增基础设施都要回答「哪个真实用户、玩家、创作者或部署循环卡住了」。
- 不再按计划预先堆叠多层路标。

## 评分标准

每条新工作都按章程纪律评：

- 内核保持对内容无意见，不渗入对话 / 模型 / 提示词 / 记忆 / 世界 / 角色 / 导演等语义。
- 任何路径上都不让官方包获得特权。
- 所有能力包与 UI 行为都走公开协议边界。
- 新增的底座必须能回答某个真实可玩体验的压力。

## 接下来会推进的工作

下面这些是已知该做、也会真实推进的事项。优先级取决于真实摩擦点。

### 契约前沿

- WIT worlds + WASM entry form 从 scaffold 推进到 partial：把 bindings 映射成 resource imports，补齐 wasm 包执行。
- Remote 包：SPIFFE 身份、Biscuit token 兑换、远端包生命周期与审计。
- Powerbox：显式 user/host 授权、句柄转授、临时权威、可撤销 delegation。
- 跨包委派、衰减链审计、租约刷新、批量撤销。
- Conformance kit 抽成可嵌入库，支持项目自定义检查。
- SDK 发布渠道完善：npm 发布、Rust crate 发布、OpenAPI/codegen 文档。

### 包系统与运行时

- 包持有的 projection 执行。
- 能力包身份的 `event.subscribe` 权限。
- 钩子处理器的超时与错误审计。
- 能力 provider 的持久选择策略（超出单次调用显式选择）。
- object/artifact 的运行时权限、配额与可达性 GC；内容寻址 blob 存储本身已完成。
- 更广的传输层一致性覆盖。

### 项目与多租户

- 基于 `ProtocolContext.session_id` 的多租户项目范围加固：把项目身份显式传入运行时权限、事件与 resolver 上下文。
- 项目归档超过 30 天自动清理。
- `yg secret put / list / delete` CLI。
- OS keyring 集成（等 CI / 跨平台构建有稳定系统依赖时再恢复）。
- 部署自动重启（单独阶段）：先把「部署意图」（image 等）持久化到 host-plane，再做有界重试 + backoff 的自愈，且不让 Docker 语义渗进内核 proxy / port 记录。当前健康监督只监测、翻 readiness、写审计，不自动重新部署。
- 部署描述符 polish：Docker pull 进度、长期日志归档、Build & Deploy job 持久化，以及外部项目 wizard 自动生成描述符。
- 远程目标与多端公开暴露：当前端口只绑 loopback。

### 模型与出站

- 使用本地 mock HTTP / WebSocket server 扩展真实模型出站 conformance，不引入默认公网依赖。
- OpenAI Realtime / Gemini Live 真实 WebSocket smoke，保持显式 opt-in。
- 更多 provider registry、tokenizer / 计费 metadata 适配，仍作为普通能力包实现。
- 单 chat 多并发生成、token-rate UI、Realtime / WebSocket streaming UX。

### 安装与发布

- 更新链路的下一步主要是 polish：更细的失败恢复提示、外部 wrapped adapter 更新、更多 UI 进度细节。
- Tauri UI 安装 polish 与发行集成。
- Sigstore keyless 验签。
- 自动更新守护进程。
- 二进制包分发。
- Desktop release code signing / notarization。
- 替换 placeholder desktop icons 为真实应用图标。
- Desktop wrapper 以受控 managed subprocess 启动 / 停止 `host serve`。

### Web shell 与 surface

- 结构化 shell descriptor 的下一步执行接线：包贡献的 `quick_action` / `workshop_card` 现在是发现入口，后续若要可执行，必须走 proposal / permission / audit，不得直接静默调用能力。
- Surface lifecycle hooks（`onClose`、`onProposalDraft` 等）。
- Cross-origin surface bundle allowlist（含 CSP 与 origin 校验）。
- 社区 marketplace 的 surface allowlist / integrity pin / version pin / audit metadata；默认 installed project bundle 仍走 same-origin 静态路由。
- 项目控制台更新入口已接 `check_for_updates` / `update_project`；后续补更丰富的更新进度、失败恢复和历史记录。
- 当 host 暴露后接入：真实 stderr / exit metadata 给 Failure modal、项目 `size_bytes` 给 Disk usage、更精确的 storage_summary 测量状态。
- Failure 与 health 监控更丰富。

### 性能

性能基线见 [`../performance/BASELINE.md`](../performance/BASELINE.md) 与 [`../../perf/baseline.json`](../../perf/baseline.json)。后续优化以基线为 regression reference，先测量再改。

## 接入项目（独立仓库）

下面这些跑在 Yggdrasil 之上，通过公开协议消费平台。它们不在本仓库里：

- **YdlTavern** —— 跑在 Yggdrasil 之上、兼容 SillyTavern 资源与扩展的独立接入项目：支持 SillyTavern 的角色卡、世界书、预设、聊天历史和扩展 API，引擎层走 Yggdrasil。仓库：<https://github.com/Youzini-afk/Yggdrasil-Tavern>。Yggdrasil 这边的边界见 [`../tavern/TAVERN_COMPAT.md`](../tavern/TAVERN_COMPAT.md)。

## 内核范围内的无限期延后

这些不会进内核，会以普通能力包或后续工作的形式出现：

- pi 作为产品壳的整包嵌入 —— 见 [`../architecture/PI_INTEGRATION.md`](../architecture/PI_INTEGRATION.md)。Agent 基础设施只能以普通能力包 / SDK 形态推进。
- 外部游戏引擎桥接（UE5 / Godot / Unity / Web 客户端）。
- 享受特权的内置 Studio、绕过公开协议的 UI、由内核拥有的官方审查器。公开协议的客户端和能力包贡献的 surface 可以继续演化。
- 内核里的对话运行时、提示词、模型 / 采样、消息 / 回合语义、记忆模型、世界模拟、导演。
- 市场、包签名网络、依赖解析经济。本地分享 proof 已完成，见 [`../guides/SHARING_DISTRIBUTION.md`](../guides/SHARING_DISTRIBUTION.md)。
