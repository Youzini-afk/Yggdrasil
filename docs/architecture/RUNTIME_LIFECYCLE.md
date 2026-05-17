# Runtime Lifecycle

> [English](./RUNTIME_LIFECYCLE.en.md) · [中文](./RUNTIME_LIFECYCLE.md)

内核运行三种生命周期：package、session 和 capability invocation。它们都不描述 turn、chat、prompt 或任何其他内容形态的操作。那些属于能力包。

## Package 生命周期

```text
discovered  manifest 对 host 可见
loading     manifest 已验证，sandbox 已准备，ABI 已检查
starting    entry point 已启动，内核握手完成，capability 和 hook 已注册
ready       接受调用和 dispatch
degraded    可达但报告能力降低（heartbeat 迟缓、部分功能不可用）
stopping    已发送优雅关闭信号
stopped     资源已释放
unloaded    在 host 中不再活跃
```

每次转换发出一个 `kernel/package.*` 事件。subscriber（可观测性工具、其他能力包）通过公开协议做出反应；内核不暴露 package 状态的私有 hook。

## Session 生命周期

一个 session 是一个带标签的事件流，附带一组 package set 和一个权限作用域。内核不赋予它任何其他含义。

```text
requested   open() 已收到，principal 和 labels 已提供
opening     kernel/session.before_open 已 dispatch（sync，可 veto）
open        kernel/session.opened 已发出
            event 日志接受已授权写入者的 append
            capability invocation 正在针对活跃 package set 进行 dispatch
forking     fork() 已收到，携带 parent session 和 forked-from 序号
forked      kernel/session.forked 已发出；子 session 继承 parent 直至所选序号
closing     kernel/session.before_close 已 dispatch（sync，可 veto）
closed      kernel/session.closed 已发出；日志已冻结，不再接受 append
```

内核不拥有 "当前 turn"、"活跃 actor" 或 session 的任何内容级状态。如果能力包需要这类概念，它从事件中自行推导。

## Proposal 生命周期

内核负责协调通用的需要审批的变更 proposal。该生命周期是内容无关的：它只知道可以执行的操作（`asset.put`、`projection.rebuild`）。

```text
created     proposal 在发起 principal 下记录；kernel/proposal.created 已发出
approved    审批者决定已记录；kernel/proposal.approved 已发出
rejected    审批者决定已记录；kernel/proposal.rejected 已发出
applied     已批准的 proposal 已对内核执行；kernel/proposal.applied 已发出
failed     执行或验证失败；kernel/proposal.failed 已发出
```

能力包或 assistant principal 不能直接 apply proposal：必须先到达 `approved` 状态。内核永远不会发明特定领域的 proposal 语义；更丰富的操作（多步事务、能力包侧补偿）属于构建在其上的能力包。

## Capability invocation 生命周期

```text
requested        invoke(id, version, input) 已收到
authorizing      kernel/capability.before_invoke 已 dispatch（sync，可 veto）
routed           按 id+version+session package set 选择 provider
running          provider 正在执行；streaming chunk 可能正在流动
completed        kernel/capability.completed 已发出，附带 output（或 stream 结束）
failed           kernel/capability.failed 已发出，附带结构化 error
cancelled        取消已被 provider 确认；failed/completed 事件记录结果
```

内核将 invocation 作为 kernel 事件记录。`input` 和 `output` 的内容对内核不透明，仅按 provider 声明的 schema 进行验证。

## 取消与超时

每个长时间运行的操作（capability invocation、hook dispatch、package start）都有一个 deadline，由 manifest sandbox 策略加 host 策略推导而来。超过 deadline 触发取消。内核记录结果。

内核不会为内容发明自己的取消语义（没有 "重新生成"，没有 "停止生成"）。这类操作是能力包的 capability。

## Replay 与引导

当 host 重启时：

1. Manifest 被重新发现。
2. Package 经历 `loading` 和 `starting`。
3. 已存储的 session 立即可用于只读 replay。
4. Session 只有在其所需 package 到达 `ready` 后才恢复写操作。

需要从 event 日志重建内部状态的能力包，通过 `events.read` 和 replay 流来完成。内核不提供其他恢复机制。

## 错误

内核仅在自己的边界上对错误分类：transport、manifest、schema、permission、capacity、lifecycle、ambiguous-route。能力包的错误作为不透明的结构化失败通过 capability invocation 传递，记录在 `kernel/capability.failed` 下。

## 本生命周期未描述的内容

- 没有 turn，没有 message，没有 prompt 循环。
- 没有模型调用编排。
- 没有记忆更新流。
- 没有 agent 任务。
- 没有 world tick。

以上所有都适合存在于能力包内部。它们都不是内核生命周期。
