# 事件模型

> [English](./EVENT_MODEL.md) · [中文](./EVENT_MODEL.zh-CN.md)

事件日志是内核的真相来源。它按 session 组织、只追加、持久化且有序。

内核不解释事件 payload。意义属于能力包。

## 信封

每个持久化的事件使用相同的信封：

```text
EventEnvelope
- id                  unique event id
- session_id          target session
- sequence            monotonic per session
- timestamp           kernel-assigned
- writer_package_id   the package that produced the event (or "kernel")
- kind                namespaced string, e.g. "kernel/session.opened" or "org/name/event/foo"
- schema_version      payload schema version, owned by the writer
- payload             opaque JSON, validated only against the writer's declared schema
- metadata            opaque JSON; causation_id, correlation_id, trace ids, etc.
```

内核：

- 分配 `id`、`sequence`、`timestamp` 和 `writer_package_id`，
- 要求 `kind` 命名空间在写入方的 id 之下（内核事件使用 `kernel/...`），
- 在写入方声明了 schema 时，依据其声明的 schema 验证 `payload`，
- 将 `metadata` 视为不透明。

## 种类

事件 kind 有两种类型。

### 内核发出的 kind

内核自身产生的一小部分固定集合。它们描述内核操作，而非内容。

Session：

```text
kernel/session.opened
kernel/session.closed
kernel/session.forked
```

能力包 lifecycle：

```text
kernel/package.loading
kernel/package.starting
kernel/package.ready
kernel/package.stopping
kernel/package.stopped
kernel/package.loaded
kernel/package.unloaded
kernel/package.degraded
kernel/package.log
```

Capability 调用（计划的审计形式）：

```text
kernel/capability.invoked
kernel/capability.completed
kernel/capability.failed
```

权限审计：

```text
kernel/permission.granted
kernel/permission.revoked
kernel/permission.denied
```

通用底座：

```text
kernel/asset.put
kernel/projection.updated
```

Proposal 生命周期：

```text
kernel/proposal.created
kernel/proposal.approved
kernel/proposal.rejected
kernel/proposal.applied
kernel/proposal.failed
```

Transport / runtime 错误（计划中）：

```text
kernel/error
```

这些是内核按名称识别的全部事件 kind。它们的 payload 描述内核操作，永远不是内容。

### 能力包发出的 kind

其余一切。每个能力包在自己的 manifest 中定义自己的事件 kind，命名空间在其 package id 之下。示例（仅为说明；不属于内核）：

```text
someorg/conversation/turn.started
someorg/conversation/prompt.rendered
someorg/conversation/model.streamed
someorg/world-sim/tick.completed
someorg/memory-pack/proposal.created
```

内核持久化并排序这些事件。但它不理解它们。

## 权限

追加事件要求写入方 manifest 中有 `events.append`。读取事件流要求 `events.read`（且可限定于特定 session）。

能力包不能在另一个能力包的 namespace 下追加事件。跨能力包的事件协调通过 capability 调用或扩展点进行，而非在日志中冒充对方。

## 持久化规则

- 只追加。日志从不被编辑。
- 按 session 排序是单调的。内核不做跨 session 排序承诺。
- 持久化。`kernel/event.after_append` 触发后，事件即已提交。
- 可 replay。内核可以从 `sequence` 0 开始向前流式输出事件。

## Replay

内核可以将事件 replay 给：

- 新订阅的客户端，
- 请求追赶的新加载能力包，
- 快照工具。

内核原封不动地 replay 信封。意义、projection 和状态重建是能力包的事。

## 版本管理

每个事件 kind 携带 `schema_version`。所属写入方负责迁移。内核不迁移 payload；它持久化写入时的内容。

能力包可以在不改动内核的情况下为自己的 kind 发布新的 `schema_version`。

## 因果与关联

信封的 `metadata` 可以携带 `causation_id`（导致此事件的那条事件）和 `correlation_id`（一个逻辑追踪），但内核将它们视为不透明。能力包决定它们的含义。

## 本模型刻意省略的东西

- 没有聊天历史概念。
- 没有轮次或消息概念。
- 没有 prompt frame、context plan 或 model call 概念。
- 没有记忆或世界状态概念。
- 没有 agent 任务或 proposal 概念。

以上所有对需要它们的能力包来说都是合法的事件 kind。它们都不是内核事件。

## 稳定性

内核发出的 kind 集合刻意保持很小。新增内核 kind 需要与新增内核职责相同的论证：它无法合理地生活在能力包里。
