# 下一步

> [English](./NEXT_STEPS.en.md) · [中文](./NEXT_STEPS.md)

这份文档讲 Yggdrasil 接下来要往哪走。已完成的状态见 [`../ALPHA_STATUS.md`](../ALPHA_STATUS.md)，不在这里复述。

## 现在在哪

- 内核对内容无意见，官方包没有特权，公开协议是唯一入口。
- 安全执行底座完整：`secret_ref`、本地加密 secret store、网络声明、外发审计与脱敏、HTTP/WebSocket 出站执行器、流式与取消生命周期。
- 平台底座完整：包安装、项目模型、Home 项目货架、Settings、真实模型端到端、流式 UX、桌面 wrapper、release pipeline、Web shell release closure 与代码组织拆分。
- 多 provider 模型接入、transport-neutral 推理接缝、Agentic Forge、外部项目操作平面、存储中立性、PostgreSQL 事件后端、TDB 真实 Rust adapter——都已落地。
- Contract V1 是公开平台规范，63 methods + 45 events + 7 top-level = 115 个 schema 全部通过校验，427 conformance cases 通过。

下一阶段不再继续摊大表面积，而是由真实可玩体验来牵引剩下的工作。

## 长期方向

平台立场见 [`../product/PLAY_CREATION_MODEL.md`](../product/PLAY_CREATION_MODEL.md)。

要点：

- 用一两个真实可玩体验作为压力源，倒逼底座剩下的工作浮现出来。
- 任何新增基础设施都要回答「哪个真实玩家或创作者循环卡住了」。
- 不再按计划预先堆叠多层路标。

## 评分标准

每条新工作都按章程纪律评：

- 内核保持对内容无意见，不渗入对话 / 模型 / 提示词 / 记忆 / 世界 / 角色 / 导演等语义。
- 任何路径上都不让官方包获得特权。
- 所有能力包与 UI 行为都走公开协议边界。
- 新增的底座必须能回答某个真实可玩体验的压力。

## 接下来会推进的工作

下面这些不构成新阶段，是已知该做、也会真实推进的事项。优先级取决于真实摩擦点。

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
- 内容寻址的 blob 存储与运行时身份层面的资产权限。
- 更广的传输层一致性覆盖。

### 项目与多租户

- 基于 `ProtocolContext.session_id` 的多租户项目范围加固：把项目身份显式传入运行时权限、事件与 resolver 上下文。
- 项目归档超过 30 天自动清理。
- `yg secret put / list / delete` CLI。
- OS keyring 集成（等 CI / 跨平台构建有稳定系统依赖时再恢复）。

### 模型与出站

- 使用本地 mock HTTP / WebSocket server 扩展真实模型出站 conformance，不引入默认公网依赖。
- OpenAI Realtime / Gemini Live 真实 WebSocket smoke，保持显式 opt-in。
- 更多 provider registry、tokenizer / 计费 metadata 适配，仍作为普通能力包实现。
- 单 chat 多并发生成、token-rate UI、Realtime / WebSocket streaming UX。

### 安装与发布

- `yg gc` 孤立 store 回收。
- Tauri UI 安装路径。
- Sigstore keyless 验签。
- 自动更新守护进程。
- 二进制包分发。
- Desktop release code signing / notarization。
- 替换 placeholder desktop icons 为真实应用图标。
- Desktop wrapper 以受控 managed subprocess 启动 / 停止 `host serve`。

### Web shell 与 surface

- Surface lifecycle hooks（`onClose`、`onProposalDraft` 等）。
- Cross-origin surface bundle allowlist（含 CSP 与 origin 校验）。
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
