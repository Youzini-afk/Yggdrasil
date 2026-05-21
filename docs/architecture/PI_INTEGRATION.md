# pi 集成边界

> [English](./PI_INTEGRATION.en.md) · [中文](./PI_INTEGRATION.md)

本文档固定 Yggdrasil 如何吸收 [pi](https://github.com/earendil-works/pi) 的 agent 框架能力。结论是：pi 是 agent package infrastructure 的重要参考来源和可选包内实现来源，但不是 Yggdrasil 内核、协议或产品壳。

## 当前立场

Yggdrasil 要能托管、约束、观察和替换 agent-like capability packages；Yggdrasil 本身不要变成内置 agent runtime。

Agent 基础设施必须落在现有公开原语上：

- `kernel.capability.invoke` / `kernel.capability.stream` 启动或推进 agent-like package capability。
- `kernel.capability.cancel` 取消运行中的 stream invocation。
- `kernel.capability.discover` / `kernel.capability.describe` 发现可映射为 agent tool 的能力。
- `kernel.proposal.create/get/list/approve/reject/apply` 承载 agent 产生的变更提案。
- `kernel.event.append/list/subscribe` 承载 package-owned trace、tool-call 和 run events。
- `kernel.surface.contribution.*` 让 Assist / Forge 通过公开协议发现 agent actions 和 trace panels。
- 权限、`secret_ref`、network declarations、outbound audit/redaction 和 stream/cancel lifecycle 继续由安全执行底座约束。

## pi 分层吸收策略

| pi 层 | Yggdrasil 处理方式 | 理由 |
|---|---|---|
| `pi-ai` | 参考 + 未来普通 model/inference package 内部可选使用 | provider registry、stream/tool-call、faux provider 很有价值，但真实模型调用仍必须等待 host policy、secret、network、audit、usage、redaction 契约更成熟。 |
| `pi-agent-core` | adapter-now + package-internal optional | `AgentEvent`、`AgentTool`、before/after tool-call、parallel/sequential execution、steer/followUp queues 可吸收；但 model/message/systemPrompt/thinkingLevel 不能进 kernel。 |
| `pi-coding-agent` | reference only | 它是完整 coding-agent 产品，带 TUI、bash/read/write/edit tools、session JSONL、model resolver、skills/extensions、coding workflow；不适合作为 Ygg 平台依赖或产品壳。 |

更细的 ledger 见 [`../../integrations/pi/README.md`](../../integrations/pi/README.md)。

## Agent 概念到 Ygg 原语的映射

| Agent 概念 | Yggdrasil public primitive | 规则 |
|---|---|---|
| run / turn / step | package capability invocation 或 stream invocation | Kernel 不新增 agent lifecycle。 |
| cancellation | `kernel.capability.cancel` | 使用通用 stream/cancel 生命周期。 |
| tool discovery | `kernel.capability.discover` / `describe` | Tool 是 capability 的 adapter view。 |
| tool execution | `kernel.capability.invoke` / `stream` | 必须保留 caller principal、provider package、permission gate 和 audit。 |
| tool ambiguity | 显式 `provider_package_id` | 禁止自动偏向 official provider。 |
| proposal | `kernel.proposal.*` | Agent 不直接 mutate trusted state。 |
| trace | package-owned events 或 stream frames | Kernel 不理解 trace payload。 |
| state | package-owned asset/projection/get_state capability | 不新增 `kernel.agent.state`。 |
| memory/prompt/model | future ordinary packages | 不进入 kernel。 |
| UI | surface contributions + public protocol | Assist/Forge 不读 runtime internals。 |

## SDK 与包边界

Agent Infrastructure Alpha 可以新增：

- `sdk/typescript/ygg-agent-adapter`：把 Ygg capability 映射为 pi-style tool，提供 proposal、trace、stream/cancel、permission/provider diagnostics helpers。
- `ygg init-package --template agent-runtime`：生成 deterministic/no-network subprocess agent package。
- `official/pi-agent-runtime-lab`：普通参考包，默认 no-network/faux，不真实调用模型。
- `official/capability-tool-bridge-lab`：普通 tool bridge 包，发现能力、预览权限、显式 provider selection、通过公开协议调用。
- Forge/Assist 的 agent trace/tool/proposal 观察面。
- 第三方 replacement proof，证明官方 agent 包无优先级。

这些组件不能：

- import runtime private modules；
- bypass package/capability/permission/proposal boundaries；
- hardcode official package IDs in UI；
- expose raw secrets in events/proposals/audit；
- provide default bash/edit/write tools；
- make real model calls in Alpha。

## 内核非目标

内核不会新增或标准化：

- `kernel.agent.*`
- `kernel.model.*`
- `kernel.prompt.*`
- `kernel.memory.*`
- `kernel.turn.*`
- agent state、chat transcript、prompt template、model provider、thinking/reasoning 或 memory taxonomy。

## 反模式

- 把 `pi-coding-agent` 嵌成 Ygg 产品壳。
- `Assist` 通过 private runtime path 启动 agent。
- Tool bridge 自动选择第一个匹配 provider 或偏向 official provider。
- Agent package 直接写 asset/projection/session trusted state。
- 把 pi `AgentState` 存成 kernel state。
- 为了 trace viewer 新增 kernel trace ontology。
- 先接真实 OpenAI/Anthropic，再补 secret/network/audit/redaction。

## 当前状态

Agent Infrastructure Alpha 已进入执行阶段。J0 固定边界和 ledger；后续阶段将先做 adapter SDK、deterministic/no-network template、普通官方参考包、tool bridge、Forge/Assist 观察面和第三方 replacement proof。真实 model inference 继续延后到专门 package 和 host policy 完成之后。
