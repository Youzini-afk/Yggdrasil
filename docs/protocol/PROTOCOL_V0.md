# 公开协议 v0

> [English](./PROTOCOL_V0.en.md) · [中文](./PROTOCOL_V0.md)

内核对外暴露一份公开协议。Studio、CLI、in-process 包、子进程包、WASM 包和远端服务使用同一份契约。

不存在私有旁路。官方客户端使用这份协议；第三方也使用这份协议。

## 传输层

所有传输层最终呈现同一份协议。当前 host 先实现最小公开子集。其余传输层先标记为 deferred，等 conformance 覆盖后再开放。

- In-process：与线上格式一一对应的 Rust API。
- Subprocess：基于 stdio 的 JSON-RPC。当前 host 必须实现。
- HTTP：用于非流式方法的 request/response。当前 host 必须实现。
- Profile 支持的 HTTP host：`ygg host serve --http 127.0.0.1:8787 --profile profiles/forge-alpha.yaml` 在自动加载 profile 包之后启动 `/rpc` 以及 ad hoc SSE 路由。
- Host stdio：用于自动化和 conformance 的 JSON-RPC。当前 host 必须实现。
- WebSocket：用于订阅和流式方法。计划在 sequence-range replay 之后实现。
- TCP：基于本地 socket 的 JSON-RPC。Deferred。
- Remote endpoint：对声明 URL 的 HTTP 和 WebSocket。Deferred。
- WASM host：通过内核提供的 ABI 进行编组调用。Deferred。

传输层选择是 host 的职责。一个方法只有在公开传输路径和 conformance 用例都覆盖它时，才被视为已实现。覆盖必须不绕过运行时权限检查。

## 协议信封

规范的 request/response 传输层使用以下格式：

```json
{
  "id": "request-1",
  "method": "kernel.capability.invoke",
  "params": {}
}
```

host 附加身份和传输层上下文。调用者不能通过请求 JSON 自行声明 package/admin 身份。

成功：

```json
{
  "id": "request-1",
  "result": {}
}
```

失败：

```json
{
  "id": "request-1",
  "error": {
    "code": "kernel/error/permission_denied",
    "message": "...",
    "details": {}
  }
}
```

## 方法格式

每个方法具有：

- `id`：内核方法在 `kernel/...` 下命名，包方法在 package id 下命名。
- `input`：根据已发布 schema 验证的 JSON 值。
- `output`：JSON 值，可能是流。
- `errors`：包含 `code`、`message`、`details` 的结构化错误模型。

## 内核方法

内核暴露一个最小集合。未列出的任何内容都归某个包所有。

### Session

```text
kernel.session.open      open a session with labels and a package set
kernel.session.close     close a session
kernel.session.fork      fork a session at an event sequence
kernel.session.branch.list list branch lineage records
kernel.session.get       get session metadata
kernel.session.list      list sessions visible to the caller
```

内核不存储任何内容层面的会话状态。标签和包集是仅有的主观判断。

### 事件

```text
kernel.event.append      append an event under the caller's namespace
kernel.event.list        list events for a session by sequence range
kernel.event.subscribe   stream events as they are appended (resumable)
```

`event.append` 要求调用者清单中包含 `events.append`。`event.list` 和 `event.subscribe` 对 package 身份要求 `events.read`。当前 host 将 HTTP SSE 作为 host-dev 流暴露：

```text
GET /kernel/event.subscribe/:session_id?after_sequence=42&kind_prefix=kernel/&writer_package_id=kernel
```

`kernel.event.list` 接受 `session_id`、`after_sequence`、`limit`、`kind_prefix` 和 `writer_package_id`。

### 包

```text
kernel.package.list      list packages visible in the host
kernel.package.describe  fetch a manifest snapshot
kernel.package.load      load a package from a manifest reference
kernel.package.unload    stop and remove a package
kernel.package.status    current state and health
kernel.package.restart   restart a package when its entry form supports restart
kernel.package.logs      read captured package logs
```

加载包可能受 host 策略限制。

### Capability

```text
kernel.capability.discover    enumerate capabilities, optionally filtered
kernel.capability.describe    fetch input/output schemas and metadata
kernel.capability.invoke      invoke a capability with input
kernel.capability.stream      invoke a capability that streams
kernel.capability.cancel      cancel an in-flight invocation
```

`invoke` 通过 id、可选的 `provider_package_id`、可选版本约束，以及最终的会话包集解析到 provider。如果多个 provider 匹配且调用者未指定 `provider_package_id`，内核返回 ambiguous-route 错误。当前 host 支持精确版本或同主版本 `^x.y` 约束。

### 扩展点和钩子

```text
kernel.extension_point.list        list live extension points
kernel.extension_point.describe    fetch payload schema and timing
kernel.hook.list                   list subscribers to a point
```

内核不暴露在运行时注入钩子的方法。订阅在清单中声明。运行时注册只允许通过包生命周期进行。

### Asset

```text
kernel.asset.put         store an asset blob under the caller's namespace
kernel.asset.get         fetch an asset by id
kernel.asset.list        list assets visible to the caller
```

内核记录 `mime`、`hash`、`size` 和 `origin_package`。它不解析或解释资产内容。

### Projection

```text
kernel.projection.register  register a generic projection definition
kernel.projection.rebuild   rebuild projection state from event filters
kernel.projection.get       fetch projection state
kernel.projection.list      list projection records
```

内核管理 projection 记录和 rebuild 生命周期，但不解释内容相关的状态语义。包拥有的 projection 执行归包所有。

### 健康与身份

```text
kernel.host.info         host version, kernel ABI, transports
kernel.host.principal    the calling principal (user, package, remote)
kernel.host.ping         liveness
kernel.host.diagnostics  local host diagnostics for package/capability/hook observability
```

### Outbound

```text
kernel.outbound.execute    unary HTTP-style outbound through the host executor
kernel.outbound.stream     streaming outbound through SSE / NDJSON / raw frames
kernel.outbound.audit      list redacted outbound audit records for a package
kernel.outbound.git_fetch  public HTTPS git fetch under host policy
```

`execute` 和 `stream` 的请求 shape 以运行时类型为准：`OutboundExecutorRequest`、`OutboundExecutorResponse`、`KernelOutboundStreamResponse`、`OutboundStreamFrame`（见 `crates/ygg-runtime/src/runtime/outbound.rs`）以及分发解析（见 `crates/ygg-runtime/src/runtime/protocol_dispatch.rs`）。核心字段包括 `capability_id`、`destination_host`、`method`、可选 `path`、`body_shape`、`metadata`、`secret_headers`、`static_headers`、`timeout_ms`；`stream` 额外接受 `stream_format`（`sse` / `ndjson` / `raw`）与帧/时长上限。

出站请求按两层 fail-closed 校验：能力包 manifest 必须声明匹配的 `permissions.network.declarations`，并且所有 `secret_headers` / `secret_refs` 必须声明在 `permissions.secret_refs`。host profile 还必须显式启用 outbound execute/stream，目标 host 必须精确匹配 allowlist（支持 `*.suffix`），HTTPS-only 不能关闭，redirect 默认拒绝。`capability_id` 必须属于调用包 namespace；subprocess reverse kernel calls 也使用 host 绑定的 package principal，不能 spoof。

`kernel.outbound.audit` 只返回脱敏审计记录：package、capability、destination host、method、purpose、使用的 `secret_ref` 与 redaction state。raw header/body/secret/response 不进入审计或协议响应。

## 包方法

每个包通过能力注册和扩展点声明贡献自己的协议方法。它们的 schema 可以通过 `kernel.capability.describe` 和 `kernel.extension_point.describe` 发现。

内核不预定义 `session.input`、`prompt_frame.get`、`model.call`、`memory.search` 等方法。如果它们存在，它们属于特定的包。

## 错误

```text
kernel/error/transport
kernel/error/schema_validation
kernel/error/manifest
kernel/error/permission_denied
kernel/error/ambiguous_route
kernel/error/not_found
kernel/error/timeout
kernel/error/cancelled
kernel/error/capacity
kernel/error/package_state
```

包错误作为 `package_error` 携带 provider 定义的详情，在 `capability.invoke` 响应中传递。

## Streaming

流式输出通过 WebSocket 或等效传输层进行。流携带类型化帧，其 schema 随方法发布。

对于 `event.subscribe`，帧是事件信封加上用于恢复的 `cursor`。

对于 `capability.stream`，帧是 provider 定义的块加上终端状态帧。

## 认证和 principal

host 在传输层强制执行认证。每个连接关联一个身份：用户、assistant、包、host 工具、匿名调用者或远端系统。内核在每次操作时根据身份检查权限。

内核不附带 identity provider。Host 自行接入。

当前身份种类：

```text
host_admin
host_dev
package { package_id }
human { user_id }
assistant { assistant_id, delegated_user_id? }
anonymous
```

Human 和 assistant 身份对敏感操作需要显式的有范围授权：

```text
kernel.permission.grant
kernel.permission.revoke
kernel.permission.list
kernel.permission.audit
```

## Surface 贡献

包可以在其清单中声明 UI surface 描述符。内核不渲染或解释这些描述符的内容；它只将它们暴露给公开客户端：

```text
kernel.surface.contribution.list
kernel.surface.contribution.describe
```

初始 slot 为 `experience_entry`、`home_card`、`play_renderer`、`forge_panel`、`asset_editor` 和 `assistant_action`。

Surface 描述符可以包含版本、启动能力、会话模板、input schema、权限 UX 元数据和 approval 策略。这些始终只是描述符；内核不会将它们转化为内建的体验/游戏语义。

## 提案生命周期

Assistant 和包驱动的变更使用通用提案信封，而不是特权变更路径：

```text
kernel.proposal.create
kernel.proposal.get
kernel.proposal.list
kernel.proposal.approve
kernel.proposal.reject
kernel.proposal.apply
```

提案状态为 `created`、`approved`、`rejected`、`applied` 和 `failed`。初始操作支持刻意保持通用，例如 `asset.put` 和 `projection.rebuild`。它们必须产生内核审计/提案事件。

## 版本控制

协议携带 `protocol_version`。内核按版本发布 schema 集。破坏性变更需要新版本；内核可以同时服务多个版本。

方法 schema 可以在一个版本内以向后兼容的方式演进（增量字段）。破坏性方法变更需要新的方法 id。

## 稳定性

任何类似 `session.input`、`prompt_frame.get`、`model.call` 或其他内容方法的东西，永远不在内核协议的范围内。向内核添加此类方法是 charter 违规。
