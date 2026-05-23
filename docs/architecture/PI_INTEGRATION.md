# pi 集成边界

> [English](./PI_INTEGRATION.en.md) · [中文](./PI_INTEGRATION.md)

本文档固定 Yggdrasil 如何吸收 [pi](https://github.com/earendil-works/pi) 的 agent 框架能力。pi 是 agent 能力包基础设施的重要参考，也可以作为包内实现来源。它不是 Yggdrasil 的内核、协议或产品壳。

## 当前立场

Yggdrasil 要能托管、约束、观察和替换 agent 类能力包。Yggdrasil 本身不应变成内置 agent runtime。

Agent 基础设施必须落在现有公开原语上：

- `kernel.v1.capability.invoke` / `kernel.v1.capability.stream` 启动或推进 agent 类包能力。
- `kernel.v1.capability.cancel` 取消运行中的流式调用。
- `kernel.v1.capability.discover` / `kernel.v1.capability.describe` 发现可映射为 agent 工具的能力。
- `kernel.v1.proposal.create/get/list/approve/reject/apply` 承载 agent 产生的变更提案。
- `kernel.v1.event.append/list/subscribe` 承载包拥有的 trace、工具调用和 run events。
- `kernel.v1.surface.contribution.*` 让 Assist / Forge 通过公开协议发现 agent 动作和 trace 面板。
- 权限、`secret_ref`、网络声明、出站审计/脱敏和流式取消生命周期继续由安全执行底座约束。

## pi 分层吸收策略

| pi 层 | Yggdrasil 处理方式 | 理由 |
|---|---|---|
| `pi-ai` | 参考 + 未来普通 model/inference 包内可选使用 | provider registry、流式/tool-call 和 faux provider 很有价值。真实模型调用仍要等 host 策略、secret、网络、审计、usage 和脱敏契约成熟。 |
| `pi-agent-core` | 现在做 adapter + 包内可选 | `AgentEvent`、`AgentTool`、before/after tool-call、parallel/sequential execution、steer/followUp queues 可吸收。model/message/systemPrompt/thinkingLevel 不能进入内核。 |
| `pi-coding-agent` | 仅作参考 | 它是完整 coding-agent 产品，带 TUI、bash/read/write/edit tools、会话 JSONL、model resolver、skills/extensions 和 coding workflow。不适合作为 Ygg 平台依赖或产品壳。 |

更细的 ledger 见 [`../../integrations/pi/README.md`](../../integrations/pi/README.md)。

## Agent 概念到 Ygg 原语的映射

| Agent 概念 | Yggdrasil 公开原语 | 规则 |
|---|---|---|
| run / turn / step | 包能力调用或流式调用 | 内核不新增 agent 生命周期。 |
| cancellation | `kernel.v1.capability.cancel` | 使用通用流式取消生命周期。 |
| tool discovery | `kernel.v1.capability.discover` / `describe` | 工具是能力的 adapter view。 |
| tool execution | `kernel.v1.capability.invoke` / `stream` | 必须保留调用者身份、provider 包、权限门禁和审计。 |
| tool ambiguity | 显式 `provider_package_id` | 禁止自动偏向官方 provider。 |
| proposal | `kernel.v1.proposal.*` | Agent 不直接修改受信状态。 |
| trace | 包拥有的事件或流式帧 | 内核不理解 trace payload。 |
| state | 包拥有的 asset/projection/get_state 能力 | 不新增 `kernel.v1.agent.state`。 |
| memory/prompt/model | 未来的普通包 | 不进入内核。 |
| UI | surface contributions + public protocol | Assist/Forge 不读取 runtime 内部状态。 |

## SDK 与包边界

后续 agent 基础设施可以新增：

- `sdk/typescript/ygg-agent-adapter`：把 Ygg 能力映射为 pi-style tool，提供提案、trace、流式取消和权限/provider diagnostics helpers。
- `ygg init-package --template agent-runtime`：生成子进程 agent 包模板，默认不联网。
- `official/pi-agent-runtime-lab`：普通参考包，默认 no-network/faux，不真实调用模型。
- `official/capability-tool-bridge-lab`：普通 tool bridge 包，用于发现能力、预览权限、显式选择 provider，并通过公开协议调用。
- Forge/Assist 的 agent trace/tool/proposal 观察面。
- 第三方 replacement proof，用来证明官方 agent 包无优先级。

这些组件不能：

- import runtime private modules；
- bypass package/capability/permission/proposal boundaries；
- hardcode official package IDs in UI；
- expose raw secrets in events/proposals/audit；
- provide default bash/edit/write tools；
- 在当前阶段发起真实模型调用。

## 内核非目标

内核不会新增或标准化：

- `kernel.v1.agent.*`
- `kernel.v1.model.*`
- `kernel.v1.prompt.*`
- `kernel.v1.memory.*`
- `kernel.v1.turn.*`
- agent state、chat transcript、prompt template、model provider、thinking/reasoning 或 memory taxonomy。

## 反模式

- 把 `pi-coding-agent` 嵌成 Ygg 产品壳。
- `Assist` 通过 private runtime path 启动 agent。
- Tool bridge 自动选择第一个匹配 provider，或偏向官方 provider。
- Agent 包直接写 asset/projection/session trusted state。
- 把 pi `AgentState` 存成 kernel state。
- 为了 trace viewer 新增 kernel trace ontology。
- 先接真实 OpenAI/Anthropic，再补 secret、网络、审计和脱敏。

## 当前状态

Agent 基础设施已进入执行阶段。当前文档固定边界和 ledger。接下来会先做 adapter SDK、默认不联网的模板、普通官方参考包、tool bridge、Forge/Assist 观察面和第三方 replacement proof。真实模型推理继续延后，等专门能力包和 host 策略准备好之后再接入。
