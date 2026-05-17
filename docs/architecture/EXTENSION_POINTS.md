# Extension Points

> [English](./EXTENSION_POINTS.en.md) · [中文](./EXTENSION_POINTS.md)

Extension point 是内核或能力包在运行时发出的具名 hook。其他能力包可以 subscribe 它。内核负责路由调用，不负责赋予含义。

本文档涵盖少量由内核发出的 extension point，以及所有 extension point 遵循的规则。

## Hook 契约

每个 extension point 包含：

- `id`：带 namespace 的、不可变的标识。
- `payload_schema`：调用的 JSON 结构。
- `timing`：`sync` 或 `async`。Sync handler 会阻塞操作；async handler 不会。
- `modifiable`：subscriber 是否可以返回变异后的 payload，供下一个 subscriber 看到该变异。
- `short_circuit`：subscriber 是否可以 veto 该操作。
- `ordering`：dispatcher 如何排列 subscriber 的顺序（声明的 precedence，相同 precedence 时按稳定顺序打破平局）。

内核为每个内核发出的 extension point 发布 schema。能力包为自己声明的 extension point 发布 schema。

## Subscription

subscriber 在 manifest 中声明：

```yaml
contributes:
  hooks:
    - extension_point: kernel/event.before_append
      handler: my_handler
      timing: sync
      precedence: 100
```

内核验证 subscriber 的 manifest 是否声明了 hook 所隐含的权限（例如，`event.before_append` 要求 event 读取权限；修改 payload 需要 event append 权限）。

当且仅当 `short_circuit: true` 时，返回错误的 subscriber 会 short-circuit 该操作。否则错误被记录，dispatch 继续。

## 取消与超时

Sync handler 在操作的 deadline 内运行。Async handler 收到的 deadline 由能力包 sandbox 策略推导而来。超过 deadline 的 handler 会被取消，并被视为一次失败的 handler 调用。

## 实现状态

内核发出的 extension point 集合在设计上是固定的。Foundation Alpha 实现了 event append 和 capability invoke 的可执行切片（确定性排序、包级 handler 能力、payload 元数据修改、veto、unload 清理）。Session 和 package 生命周期 hook 在契约中已预留；它们的 dispatch 目前通过 `kernel/session.*` 和 `kernel/package.*` 事件传递，随着内核演进将获得完整的 sync/async hook 处理。新的 extension point 由能力包贡献添加，而不是通过扩展内核。

## 内核发出的 extension point

内核发出少量固定的 extension point。新的 extension point 由能力包贡献添加，而不是通过扩展内核。

### Session 生命周期

- `kernel/session.before_open` — sync，modifiable false，short_circuit true。
  打开权限在此执行。subscriber 可以 veto。
- `kernel/session.after_open` — async。
- `kernel/session.before_close` — sync，modifiable false，short_circuit true。
- `kernel/session.after_close` — async。

Payload：session id、请求的 labels、package set、发起请求的 principal。

### Event 日志

- `kernel/event.before_append` — sync，modifiable true，short_circuit true。
  权限和 schema 校验在此执行。subscriber 可以修改 metadata 或 veto。
- `kernel/event.after_append` — async。
  subscriber 收到已持久化的 envelope。

Payload：event envelope。内核不解释 payload 字段；它只在写入者的 manifest 引用了该 event kind 的 payload schema 时检查声明的 schema。

### Capability 调用

- `kernel/capability.before_invoke` — sync，modifiable true，short_circuit true。
  权限、路由解析和配额执行在此发生。
- `kernel/capability.after_invoke` — async。
  subscriber 收到 input、output（或 error）、延迟和 provider id。
- `kernel/capability.error` — async。
  subscriber 收到结构化失败信息。

Payload：invocation envelope。

### Package 生命周期

- `kernel/package.loaded` — async。
- `kernel/package.unloaded` — async。
- `kernel/package.degraded` — async。
- `kernel/package.heartbeat_lost` — async。

### Hook 注册表

- `kernel/hook.registered` — async。
- `kernel/hook.unregistered` — async。

这些让可观测性能力包发现当前活跃的 extension 拓扑。

## 能力包发出的 extension point

能力包可以通过在 `contributes.extension_points` 下列出自有的 extension point 来发布它们。该能力包即成为 schema 的拥有者。

内核路由调用但不验证语义。如果拥有者能力包被 unload，内核拒绝 dispatch 该 extension point，并为所有孤立的 subscriber 发出 `kernel/hook.unregistered`。

示例（仅作示意；不属于内核）：

```yaml
contributes:
  extension_points:
    - id: someorg/conversation/before_step
      payload_schema: ...
      timing: sync
      modifiable: true
      short_circuit: true
```

其他能力包可以 subscribe：

```yaml
contributes:
  hooks:
    - extension_point: someorg/conversation/before_step
      handler: ...
```

内核不知道 `conversation/before_step` 意味着什么。拥有者能力包知道。

## 发现

客户端可以向内核查询当前活跃的 extension point 及其 subscriber。Schema 会被暴露。创作者工具、可观测性 dashboard 和其他能力包就是通过这种方式探索当前运行中的 host 里有哪些可扩展之处的。

## 版本管理

每个 extension point 都有一个 `version`。subscriber 声明自己目标的 version。内核拒绝向声明的 version 与当前活跃 extension point 不兼容的 subscriber 进行 dispatch。

对 extension point 的破坏性变更需要新的 id。拥有者能力包可以在过渡期间同时发出两个版本。

## 稳定性

内核发出的 extension point 集合在设计上是小的。新增内核 extension point 需要与新增内核职责相同的理由：它无法合理地存在于能力包中。
