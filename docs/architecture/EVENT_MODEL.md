# 事件模型

> [English](./EVENT_MODEL.en.md) · [中文](./EVENT_MODEL.md)

事件日志是内核的真相来源。它按会话组织，只追加、持久化，并保持顺序。

内核不解释事件 payload。意义由能力包拥有。

## 信封

每个持久化事件都使用同一种信封：

```text
EventEnvelope
- id                  unique event id
- session_id          target session
- sequence            monotonic per session
- timestamp           kernel-assigned
- writer_package_id   the package that produced the event (or "kernel")
- kind                namespaced string, e.g. "kernel/v1/session.opened" or "org/name/event/foo"
- schema_version      payload schema version, owned by the writer
- payload             opaque JSON, validated only against the writer's declared schema
- metadata            opaque JSON; causation_id, correlation_id, trace ids, etc.
```

内核：

- 分配 `id`、`sequence`、`timestamp` 和 `writer_package_id`，
- 要求 `kind` 命名空间在写入方的 id 之下（内核事件使用 `kernel/v1/...`），
- 如果写入方声明了 schema，就用该 schema 验证 `payload`，
- 将 `metadata` 视为不透明。

## 种类

事件 kind 分为两类。

### 内核发出的 kind

内核自身只产生一小组固定 kind。它们描述内核操作，不描述内容。

Session：

```text
kernel/v1/session.opened
kernel/v1/session.closed
kernel/v1/session.forked
```

能力包生命周期：

```text
kernel/v1/package.loading
kernel/v1/package.starting
kernel/v1/package.ready
kernel/v1/package.stopping
kernel/v1/package.stopped
kernel/v1/package.loaded
kernel/v1/package.unloaded
kernel/v1/package.degraded
kernel/v1/package.log
```

能力调用（计划中的审计形式）：

```text
kernel/v1/capability.invoked
kernel/v1/capability.completed
kernel/v1/capability.failed
```

权限审计：

```text
kernel/v1/permission.granted
kernel/v1/permission.revoked
kernel/v1/permission.denied
```

通用底座：

```text
kernel/v1/asset.put
kernel/v1/projection.updated
```

提案生命周期：

```text
kernel/v1/proposal.created
kernel/v1/proposal.approved
kernel/v1/proposal.rejected
kernel/v1/proposal.applied
kernel/v1/proposal.failed
```

传输层 / runtime 错误（计划中）：

```text
kernel/v1/error
```

这些是内核按名称识别的全部事件 kind。它们的 payload 描述内核操作，不描述内容。

### 能力包发出的 kind

其余都属于能力包。每个能力包在自己的清单中定义事件 kind，命名空间位于 package id 之下。示例仅用于说明，不属于内核：

```text
someorg/conversation/turn.started
someorg/conversation/prompt.rendered
someorg/conversation/model.streamed
someorg/world-sim/tick.completed
someorg/memory-pack/proposal.created
```

内核持久化并排序这些事件。但它不理解它们。

## 权限

追加事件要求写入方清单中有 `events.append`。读取事件流要求 `events.read`，并且可以限定到特定会话。

能力包不能在另一个能力包的命名空间下追加事件。跨能力包协调应通过能力调用或扩展点完成，不能在日志中冒充对方。

## 持久化规则

- 只追加。日志从不被编辑。
- 会话内排序是单调的。内核不承诺跨会话排序。
- 持久化。`kernel/v1/event.after_append` 触发后，事件即已提交。
- 可 replay。内核可以从 `sequence` 0 开始向前流式输出事件。

## Replay

内核可以将事件 replay 给：

- 新订阅的客户端，
- 请求追赶的新加载能力包，
- 快照工具。

内核原样 replay 信封。意义、projection 和状态重建由能力包负责。

## 版本管理

每个事件 kind 携带 `schema_version`。所属写入方负责迁移。内核不迁移 payload；它只持久化写入时的内容。

能力包可以在不改动内核的情况下为自己的 kind 发布新的 `schema_version`。

## 因果与关联

信封的 `metadata` 可以携带 `causation_id`（导致此事件的那条事件）和 `correlation_id`（一个逻辑追踪）。内核将它们视为不透明字段。能力包决定它们的含义。

## 本模型刻意省略的东西

- 没有聊天历史概念。
- 没有轮次或消息概念。
- 没有 prompt frame、上下文计划或 model call 概念。
- 没有记忆或世界状态概念。
- 没有 agent 任务或提案概念。

需要这些概念的能力包，可以把它们定义成自己的事件 kind。它们都不是内核事件。

## 稳定性

内核发出的 kind 集合刻意保持很小。新增内核 kind 需要和新增内核职责一样被论证：它确实无法合理地放进能力包。
