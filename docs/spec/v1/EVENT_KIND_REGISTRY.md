# 事件类型注册表（v1）

本表列出内核保留的 `kernel/v1/*` 事件类型。非内核包必须写入自己的包 ID 命名空间，不能写入 `kernel/v1/*`。

| 事件类型 | Payload schema | Writer | 触发 | 状态 |
|---|---|---|---|---|
| `kernel/v1/session.opened` | [`./schemas/events/kernel__v1__session.opened.schema.json`](./schemas/events/kernel__v1__session.opened.schema.json) | kernel | Session 开启 | implemented |
| `kernel/v1/session.closed` | [`./schemas/events/kernel__v1__session.closed.schema.json`](./schemas/events/kernel__v1__session.closed.schema.json) | kernel | Session 关闭 | implemented |
| `kernel/v1/session.forked` | [`./schemas/events/kernel__v1__session.forked.schema.json`](./schemas/events/kernel__v1__session.forked.schema.json) | kernel | Session fork 创建分支谱系 | implemented |
| `kernel/v1/package.loaded` | [`./schemas/events/kernel__v1__package.loaded.schema.json`](./schemas/events/kernel__v1__package.loaded.schema.json) | kernel | 包已接受并注册；载荷包含 `contract_mode`（`v1` 或 `none`） | implemented |
| `kernel/v1/package.loading` | [`./schemas/events/kernel__v1__package.loading.schema.json`](./schemas/events/kernel__v1__package.loading.schema.json) | kernel | 包进入加载中 | implemented |
| `kernel/v1/package.starting` | [`./schemas/events/kernel__v1__package.starting.schema.json`](./schemas/events/kernel__v1__package.starting.schema.json) | kernel | 包执行入口启动中 | implemented |
| `kernel/v1/package.ready` | [`./schemas/events/kernel__v1__package.ready.schema.json`](./schemas/events/kernel__v1__package.ready.schema.json) | kernel | 包启动后就绪 | implemented |
| `kernel/v1/package.stopping` | [`./schemas/events/kernel__v1__package.stopping.schema.json`](./schemas/events/kernel__v1__package.stopping.schema.json) | kernel | 包执行停止中 | implemented |
| `kernel/v1/package.stopped` | [`./schemas/events/kernel__v1__package.stopped.schema.json`](./schemas/events/kernel__v1__package.stopped.schema.json) | kernel | 包执行已停止 | implemented |
| `kernel/v1/package.unloaded` | [`./schemas/events/kernel__v1__package.unloaded.schema.json`](./schemas/events/kernel__v1__package.unloaded.schema.json) | kernel | 包从注册表移除 | implemented |
| `kernel/v1/package.degraded` | [`./schemas/events/kernel__v1__package.degraded.schema.json`](./schemas/events/kernel__v1__package.degraded.schema.json) | kernel | 执行失败或健康状态降级 | implemented |
| `kernel/v1/package.log` | [`./schemas/events/kernel__v1__package.log.schema.json`](./schemas/events/kernel__v1__package.log.schema.json) | kernel | 捕获 subprocess stderr 日志行 | implemented |
| `kernel/v1/project.installed` | [`./schemas/events/kernel__v1__project.installed.schema.json`](./schemas/events/kernel__v1__project.installed.schema.json) | kernel | Project 已安装/注册 | implemented |
| `kernel/v1/project.started` | [`./schemas/events/kernel__v1__project.started.schema.json`](./schemas/events/kernel__v1__project.started.schema.json) | kernel | Project 状态转换为 running | implemented |
| `kernel/v1/project.stopped` | [`./schemas/events/kernel__v1__project.stopped.schema.json`](./schemas/events/kernel__v1__project.stopped.schema.json) | kernel | Project 状态转换为 stopped | implemented |
| `kernel/v1/project.uninstalled` | [`./schemas/events/kernel__v1__project.uninstalled.schema.json`](./schemas/events/kernel__v1__project.uninstalled.schema.json) | kernel | Project 已卸载/归档 | implemented |
| `kernel/v1/asset.put` | [`./schemas/events/kernel__v1__asset.put.schema.json`](./schemas/events/kernel__v1__asset.put.schema.json) | kernel | 不透明 asset 已存储 | implemented |
| `kernel/v1/projection.updated` | [`./schemas/events/kernel__v1__projection.updated.schema.json`](./schemas/events/kernel__v1__projection.updated.schema.json) | kernel | projection 状态已重建/更新 | implemented |
| `kernel/v1/proposal.created` | [`./schemas/events/kernel__v1__proposal.created.schema.json`](./schemas/events/kernel__v1__proposal.created.schema.json) | kernel | proposal 已创建 | partial |
| `kernel/v1/proposal.approved` | [`./schemas/events/kernel__v1__proposal.approved.schema.json`](./schemas/events/kernel__v1__proposal.approved.schema.json) | kernel | proposal 已批准 | partial |
| `kernel/v1/proposal.rejected` | [`./schemas/events/kernel__v1__proposal.rejected.schema.json`](./schemas/events/kernel__v1__proposal.rejected.schema.json) | kernel | proposal 已拒绝 | partial |
| `kernel/v1/proposal.applied` | [`./schemas/events/kernel__v1__proposal.applied.schema.json`](./schemas/events/kernel__v1__proposal.applied.schema.json) | kernel | proposal 已应用 | partial |
| `kernel/v1/proposal.failed` | [`./schemas/events/kernel__v1__proposal.failed.schema.json`](./schemas/events/kernel__v1__proposal.failed.schema.json) | kernel | proposal 应用失败 | partial |
| `kernel/v1/capability.invoked` | [`./schemas/events/kernel__v1__capability.invoked.schema.json`](./schemas/events/kernel__v1__capability.invoked.schema.json) | kernel | 能力调用开始 | planned |
| `kernel/v1/capability.completed` | [`./schemas/events/kernel__v1__capability.completed.schema.json`](./schemas/events/kernel__v1__capability.completed.schema.json) | kernel | 能力调用成功 | planned |
| `kernel/v1/capability.failed` | [`./schemas/events/kernel__v1__capability.failed.schema.json`](./schemas/events/kernel__v1__capability.failed.schema.json) | kernel | 能力调用失败 | planned |
| `kernel/v1/permission.denied` | [`./schemas/events/kernel__v1__permission.denied.schema.json`](./schemas/events/kernel__v1__permission.denied.schema.json) | kernel | 权限检查拒绝 | implemented |
| `kernel/v1/permission.granted` | [`./schemas/events/kernel__v1__permission.granted.schema.json`](./schemas/events/kernel__v1__permission.granted.schema.json) | kernel | 权限授予已记录 | implemented |
| `kernel/v1/permission.revoked` | [`./schemas/events/kernel__v1__permission.revoked.schema.json`](./schemas/events/kernel__v1__permission.revoked.schema.json) | kernel | 权限授予已撤销 | implemented |
| `kernel/v1/error` | [`./schemas/events/kernel__v1__error.schema.json`](./schemas/events/kernel__v1__error.schema.json) | kernel | 结构化内核错误 | planned |
| `kernel/v1/outbound.request` | [`./schemas/events/kernel__v1__outbound.request.schema.json`](./schemas/events/kernel__v1__outbound.request.schema.json) | kernel | 出站请求已允许并审计 | partial |
| `kernel/v1/outbound.denied` | [`./schemas/events/kernel__v1__outbound.denied.schema.json`](./schemas/events/kernel__v1__outbound.denied.schema.json) | kernel | 出站请求被拒绝 | partial |
| `kernel/v1/outbound.execute.completed` | [`./schemas/events/kernel__v1__outbound.execute.completed.schema.json`](./schemas/events/kernel__v1__outbound.execute.completed.schema.json) | kernel | 出站 execute 完成 | implemented |
| `kernel/v1/outbound.stream.completed` | [`./schemas/events/kernel__v1__outbound.stream.completed.schema.json`](./schemas/events/kernel__v1__outbound.stream.completed.schema.json) | kernel | 出站 stream 完成 | implemented |
| `kernel/v1/stream.started` | [`./schemas/events/kernel__v1__stream.started.schema.json`](./schemas/events/kernel__v1__stream.started.schema.json) | kernel | streaming 调用开始 | partial |
| `kernel/v1/stream.chunk` | [`./schemas/events/kernel__v1__stream.chunk.schema.json`](./schemas/events/kernel__v1__stream.chunk.schema.json) | kernel | streaming chunk 已发出 | partial |
| `kernel/v1/stream.progress` | [`./schemas/events/kernel__v1__stream.progress.schema.json`](./schemas/events/kernel__v1__stream.progress.schema.json) | kernel | streaming 进度已发出 | partial |
| `kernel/v1/stream.ended` | [`./schemas/events/kernel__v1__stream.ended.schema.json`](./schemas/events/kernel__v1__stream.ended.schema.json) | kernel | streaming 正常结束 | partial |
| `kernel/v1/stream.error` | [`./schemas/events/kernel__v1__stream.error.schema.json`](./schemas/events/kernel__v1__stream.error.schema.json) | kernel | streaming 出错 | partial |
| `kernel/v1/stream.cancelled` | [`./schemas/events/kernel__v1__stream.cancelled.schema.json`](./schemas/events/kernel__v1__stream.cancelled.schema.json) | kernel | streaming 已取消 | partial |
| `kernel/v1/stream.timeout` | [`./schemas/events/kernel__v1__stream.timeout.schema.json`](./schemas/events/kernel__v1__stream.timeout.schema.json) | kernel | streaming 超时 | partial |
| `kernel/v1/outbound.websocket.opened` | [`./schemas/events/kernel__v1__outbound.websocket.opened.schema.json`](./schemas/events/kernel__v1__outbound.websocket.opened.schema.json) | kernel | 出站 WebSocket 已打开 | implemented |
| `kernel/v1/outbound.websocket.frame` | [`./schemas/events/kernel__v1__outbound.websocket.frame.schema.json`](./schemas/events/kernel__v1__outbound.websocket.frame.schema.json) | kernel | 出站 WebSocket frame 已记录 | implemented |
| `kernel/v1/outbound.websocket.error` | [`./schemas/events/kernel__v1__outbound.websocket.error.schema.json`](./schemas/events/kernel__v1__outbound.websocket.error.schema.json) | kernel | 出站 WebSocket 错误 | implemented |
| `kernel/v1/outbound.websocket.completed` | [`./schemas/events/kernel__v1__outbound.websocket.completed.schema.json`](./schemas/events/kernel__v1__outbound.websocket.completed.schema.json) | kernel | 出站 WebSocket 完成/关闭 | implemented |
