# SillyTavern 后继项目

> [English](./TAVERN_COMPAT.en.md) · [中文](./TAVERN_COMPAT.md)

SillyTavern 的下一代项目叫 **YdlTavern**，作为独立项目存在，不在 Yggdrasil 仓库里。

- 仓库：<https://github.com/Youzini-afk/Yggdrasil-Tavern>
- 定位：跑在 Yggdrasil 之上的接入项目，对 SillyTavern 用户、扩展、角色卡、世界书、预设、聊天形成承接。
- 目标：基本 100% 承接 SillyTavern API 与社区资源；前端可以重写，但 UI 结构、样式、操作对 ST 老用户保持熟悉。

YdlTavern 通过公开协议消费 Yggdrasil。它不读 Yggdrasil 内部，不依赖私有 API，跟其他第三方项目没有差别。

## 为什么不放在 Yggdrasil 仓库

Yggdrasil 是平台。把一个产品级的、几百个扩展兼容层、6 年 API 表面要承接的项目塞进 `packages/official/`，会立刻撞到章程：「官方包没有特权」。

YdlTavern 体量很大、产品决策很多、跟 SillyTavern 社区直接对接。这些都是产品话题，不是平台话题。两件事必须分开。

## Yggdrasil 这边继续提供什么

下面这些是 Yggdrasil 已经有、YdlTavern 会直接用上的：

- 公开协议（HTTP `/rpc` + SSE 事件订阅）。
- `secret_ref`、网络声明、外发审计、流式与取消生命周期。
- 模型接入能力包：OpenAI、Anthropic、Gemini、OpenAI-compatible、OpenRouter、DeepSeek、xAI、Fireworks。
- 通用创作能力包：persona-lab、knowledge-lab、context-lab、text-transform-lab、memory-lab。
- 提案与审批生命周期、资产、分支、projection。
- 后续会做「通过 GitHub 地址安装能力包」的能力——YdlTavern 的扩展生态会受益于此。

## 内核绝不会做的事

无论 YdlTavern 多大、多重要：

- 内核不会理解角色卡、世界书、预设、提示词渲染。
- 内核不会硬编码 `{{char}}` / `{{user}}` 替换。
- 内核不会提供 Tavern 专属的钩子或方法。
- 内核不会区别对待 Tavern 形态的能力包和其他能力包。

## TavernHeadless 调研

[`integrations/tavern-headless/`](../../integrations/tavern-headless/) 仍然作为研究记录留在 Yggdrasil 仓库里——它是 Yggdrasil 通用能力包（persona / knowledge / context / model-provider）的灵感来源。这一层是平台话题，跟 YdlTavern 这个接入项目分开。

YdlTavern 的具体兼容路线、扩展桥设计、UI 结构选择，都在 YdlTavern 仓库里讨论。
