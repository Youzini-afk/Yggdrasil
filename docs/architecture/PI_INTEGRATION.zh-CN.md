# pi Integration (deferred)

> [English](./PI_INTEGRATION.md) · [中文](./PI_INTEGRATION.zh-CN.md)

本文档为未来集成 `pi` agent 框架与 Yggdrasil 的能力包家族预留。它不在近期路径上。

## 立场

pi 不是内核的一部分。内核对 agent、planner、proposal、记忆策展或任何其他内容形态关注不持有任何立场。

当 pi 集成被构建时，它将以一个或多个能力包的形式发布，受与任何第三方包相同的 manifest、fabric、权限和 sandbox 规则约束。它不会获得任何内核特权。

## 可能的形态（仅为草稿）

平台契约为每个能力包提供了所需的工具：

- 通过 `kernel.event.subscribe` subscribe 事件。
- 在写入者自己的 kind set 下 append 带 package namespace 的事件（例如 `pi/<...>/proposal.created`）。
- 提供其他能力包可以调用的 capability（例如 `pi/<...>/curate`、`pi/<...>/extract`）。
- 定义自己的 extension point，让其他能力包可以 subscribe pi 内部的阶段。
- 在 manifest 中声明权限、sandbox 限制和副作用。

如果 pi 采用"先 proposal 后 commit"模式，它作为普通事件和 capability 调用在 pi 包和其他包之间实现即可。内核不需要了解它。

## 内核的非目标

内核永远不会：

- 将 "agent" 建模为一等概念，
- 将 "proposal" 建模为一等概念，
- 建模记忆分类体系，
- 提供 pi 专用 hook 或方法，
- 对 pi 能力包给予与其他能力包不同的对待。

## 状态

pi 集成已延后，直到游创平台底座稳固。它所需的底座——event、capability、hook、权限、surface 和 proposal/approval 生命周期——现在已经就位，所以当集成开始时，它可以作为普通能力包发布，无需修改内核。在此之前，本文档仅固定立场：pi 是未来的能力包家族，不是平台层。
