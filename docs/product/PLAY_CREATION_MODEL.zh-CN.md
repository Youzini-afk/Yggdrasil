# 游创模型

> [English](./PLAY_CREATION_MODEL.md) · [中文](./PLAY_CREATION_MODEL.zh-CN.md)

本文档固定 Yggdrasil 的产品立场。平台不是聊天工具，不是游戏引擎，不是 Tavern 兼容层，也不是开发者工作台。它是一个游创（play-creation）平台，目的是让此前不存在的 AI 原生体验成为可能，并让游玩这些体验的人可以审视、修改和 fork 它们。

这里固定的立场是内核、公开协议、官方包、Web shell 和 SDK 共同服务的目标。当某个未来功能看起来与这一立场冲突时，立场优先。

## 游创前提

今天大多数 AI 原生创作工具都把用户切成两个角色：消费成品体验的玩家，和构建体验的开发者。Yggdrasil 拒绝这种切分。

Yggdrasil 上的玩家可以：

- 启动一个 session，
- 审视正在发生什么，
- 让 assistant 修改某处，
- fork 这个 session 尝试另一种走向，
- 用一个包替换另一个包，
- 保存自己的创作并分享。

Yggdrasil 上的创作者可以：

- 写一个可以在任意 host 上加载的包，
- 声明 entry point、能力、hook 和 surface，
- 用和玩家同一套协议在线调试，
- 在自己的包上看着游创循环发生，不需要单独的「开发者模式」。

底层底座在两个方向上完全相同。不存在 Yggdrasil 的「开发者版本」和「玩家版本」。只有一个跑着公开协议的 host，其余都是包的选择。

## 三个第一等 surface

平台围绕三个 surface 组织自身。内核知道它们的 slot 名；内核不知道它们的含义。

### Home / Play

类主机的启动器与游玩 surface。它从包声明的 `experience_entry` surface 发现可游玩内容，在 session 中渲染包声明的 `play_renderer` surface。Home/Play 是大多数用户度过大部分时间的地方。

Home 不是应用商店，也不是路由器。它是一个向公开协议询问「现在这里有什么可启动的？」然后信任包来回答的 surface。

### Forge

Agentic 创作工作区。它诚实地暴露底座：事件、能力、asset、projection、branch、proposal、surface、包、hook、权限。它承载包声明的 `forge_panel` surface，让包能在通用检查器旁边提供自己的创作/检查面板。

Forge 是游创创作者变成创作者-创作者而不用离开平台的地方。视觉编辑器、节点编辑器、提示词编辑器、lorebook、世界地图及类似工具属于 Forge，以包贡献的 Forge 编辑器模式存在，而不是内核功能。

### Assist

跨模式 assistant 抽屉。在 Play 中，它提供小的实时修改和 proposal。在 Forge 中，它做更深的工作 —— 提议操作、起草包、叙述 diff、建议变更。在两种模式下，每一次修改都经过 `kernel.proposal.*` 并在落定前获得 approval。

Assist 是 proposal lifecycle 的薄客户端。它不是特权修改路径。一个第三方 assistant 包可以替换 `official/assistant-lab` 并以相同方式运作。

## 游创创作者流程

游创循环跑在已有底座上。端到端流程如下：

```text
Home discovers experience_entry surfaces over the public protocol.
Player launches an experience.
Kernel opens a session bound to the package set the experience needs.
Package writes its own events, drives its own play_renderer surface.
Player asks Assist to change something.
Assist (a package) calls kernel.proposal.create with generic operations.
Player reviews the proposal and approves it.
Kernel applies approved operations and writes kernel/proposal.applied.
Player optionally forks the session at a sequence to try another path.
Player optionally opens Forge to inspect events, assets, projections, branches.
Player optionally edits a package or composition through Forge editor modes.
Cycle continues.
```

内核从不为这些步骤中的任何一个发明领域语义。语义属于包。这个循环之所以成立，是因为包可以声明自己贡献的 surface、自己提议的操作、自己拥有的事件 —— 并且内核以通用方式居中调停。

## 平台提供与不提供

平台提供：

- 一个内容无关的内核，
- 一个包的 manifest 模型，
- 一个面向人类、assistant、包和 host 的权限与 principal 模型，
- 一份所有人使用的公开协议，
- 供 Home/Play、Forge、asset 编辑器和 assistant action 使用的通用 surface contributions，
- 任何变更的通用 proposal/approval lifecycle，
- 通用的 asset、branch 和 projection 底座，
- 展示而非特权的官方基础包。

平台不提供：

- 一个聊天体验或任何其他题材，
- 一个模型 provider 抽象，
- 一个记忆模型、检索策略或 director，
- 一个 SillyTavern 兼容层，
- 一个外部游戏引擎桥接，
- 一个受眷顾的视觉编辑器或 asset 编辑器，
- 一个市场。

以上每一项都欢迎以包的形式到来。没有任何一项欢迎作为内核。

## 对 Tavern、agent、引擎的立场

SillyTavern 资源、agent 循环和外部引擎桥接是有价值的，但它们是包家族，不是平台家族。

当它们到来时，将是普通能力包，受同一套 manifest、fabric、权限和 sandbox 规则约束，与任何第三方包无异。它们不会获得内核特权。游创循环在它们上面运行的方式和在小型 fixture 体验上完全一样：发现、启动、提议、审批、应用、fork。

如果某天一个 Tavern 形态的 runtime 以官方包形式交付，一个第三方世界模拟包必须能在同一 session 中与它共存。如果不能，出错的不是第三方包，而是内核。

## 对激进创作的立场

Yggdrasil 的目标不是交付一个更好的 Tavern。目标是让平台作者未预见到的体验成为可能，并让尝试这些体验的玩家可以 fork、审视、修改和分享他们的发现。

底座天然偏向这个方向。事件是只追加的、内核拥有排序。Branch 是第一等公民。Proposal 可审计。Surface 是描述符，不是硬编码 UI。包不论来源或入口形式一律平等。

当一个功能决策让激进创作变得更难 —— 通过给官方路径特权、通过隐藏状态不让审视、通过强迫单一形态 —— 那就是 charter 退化，功能让步。

## 对「发布」的立场

不存在「1.0 聊天体验」目标。平台的发布形态是：

- Foundation Alpha —— 底座内容无关且可信（当前）。
- Playable Experience Alpha —— 至少一个体验端到端跑在底座上，可替换、可 fork、可被 assistant 辅助。
- Authoring Beta —— 第三方可以以和官方包同等地位交付包。
- Substrate v1 —— 底座停止快速变动，承诺公开协议稳定性。

Substrate v1 之后的都是产品范围。平台从不拥有它。
