# Public Protocol v0

> [English](./PROTOCOL_V0.en.md) · [中文](./PROTOCOL_V0.md)

The kernel exposes one public protocol. Studio, CLI, in-process packages, subprocess packages, WASM packages, and remote services use the same contract.

There is no private bypass. Official clients use this protocol; third parties use this protocol.

## Transports

All transports eventually surface the same protocol. The current host implements a minimal public subset first. The rest stay marked as deferred until conformance covers them.

- In-process: a Rust API that mirrors the wire shape one-to-one.
- Subprocess: JSON-RPC over stdio. Required for the current host.
- HTTP: request/response for non-streaming methods. Required for the current host.
- Profile-backed HTTP host: `ygg host serve --http 127.0.0.1:8787 --profile profiles/forge-alpha.yaml` starts `/rpc` plus ad hoc SSE routes after autoloading profile packages.
- Host stdio: JSON-RPC for automation and conformance. Required for the current host.
- WebSocket: subscriptions and streaming methods. Planned after sequence-range replay.
- TCP: JSON-RPC over a local socket. Deferred.
- Remote endpoint: HTTP and WebSocket against a declared URL. Deferred.
- WASM host: marshalled calls into the kernel-provided ABI. Deferred.

Transport selection is a host concern. A method is considered implemented only when a public transport path and a conformance case both exercise it without bypassing runtime permission checks.

## Protocol envelope

Canonical request/response transports use this shape:

```json
{
  "id": "request-1",
  "method": "kernel.capability.invoke",
  "params": {}
}
```

The host attaches principal and transport context. Callers cannot self-assert package/admin identity through request JSON.

Success:

```json
{
  "id": "request-1",
  "result": {}
}
```

Failure:

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

## Method shape

Every method has:

- `id`: namespaced under `kernel/...` for kernel methods, or under a package id for package methods.
- `input`: a JSON value validated against a published schema.
- `output`: a JSON value, possibly a stream.
- `errors`: a structured error model with `code`, `message`, `details`.

## Kernel methods

The kernel exposes a minimal set. Anything not listed is owned by a package.

### Sessions

```text
kernel.session.open      open a session with labels and a package set
kernel.session.close     close a session
kernel.session.fork      fork a session at an event sequence
kernel.session.branch.list list branch lineage records
kernel.session.get       get session metadata
kernel.session.list      list sessions visible to the caller
```

The kernel stores no content-level session state. Labels and package set are the only opinions.

### Events

```text
kernel.event.append      append an event under the caller's namespace
kernel.event.list        list events for a session by sequence range
kernel.event.subscribe   stream events as they are appended (resumable)
```

`event.append` requires `events.append` in the caller's manifest. `event.list` and `event.subscribe` require `events.read` for package principals. The current host exposes HTTP SSE as a host-dev stream:

```text
GET /kernel/event.subscribe/:session_id?after_sequence=42&kind_prefix=kernel/&writer_package_id=kernel
```

`kernel.event.list` accepts `session_id`, `after_sequence`, `limit`, `kind_prefix`, and `writer_package_id`.

### Packages

```text
kernel.package.list      list packages visible in the host
kernel.package.describe  fetch a manifest snapshot
kernel.package.load      load a package from a manifest reference
kernel.package.unload    stop and remove a package
kernel.package.status    current state and health
kernel.package.restart   restart a package when its entry form supports restart
kernel.package.logs      read captured package logs
```

Loading a package may be host-policy-restricted.

### Capabilities

```text
kernel.capability.discover    enumerate capabilities, optionally filtered
kernel.capability.describe    fetch input/output schemas and metadata
kernel.capability.invoke      invoke a capability with input
kernel.capability.stream      invoke a capability that streams
kernel.capability.cancel      cancel an in-flight invocation
```

`invoke` resolves to a provider by id, optional `provider_package_id`, optional version constraint, and eventually session package set. If multiple providers match and the caller did not specify `provider_package_id`, the kernel returns an ambiguous-route error. The current host supports exact version or same-major `^x.y` constraints.

### Extension points and hooks

```text
kernel.extension_point.list        list live extension points
kernel.extension_point.describe    fetch payload schema and timing
kernel.hook.list                   list subscribers to a point
```

The kernel does not expose a method to inject hooks at runtime. Subscriptions are declared in manifests. Live registration is allowed only through package lifecycle.

### Assets

```text
kernel.asset.put         store an asset blob under the caller's namespace
kernel.asset.get         fetch an asset by id
kernel.asset.list        list assets visible to the caller
```

The kernel records `mime`, `hash`, `size`, and `origin_package`. It does not parse or interpret asset content.

### Projections

```text
kernel.projection.register  register a generic projection definition
kernel.projection.rebuild   rebuild projection state from event filters
kernel.projection.get       fetch projection state
kernel.projection.list      list projection records
```

The kernel manages projection records and rebuild lifecycle. It does not interpret content-specific state semantics. Package-owned projection execution belongs to packages.

### Health and identity

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
kernel.outbound.websocket.open   open an outbound WebSocket stream and return connection_id
kernel.outbound.websocket.send   send one outbound WebSocket frame
kernel.outbound.websocket.close  close an outbound WebSocket connection
kernel.outbound.audit      list redacted outbound audit records for a package
kernel.outbound.git_fetch  public HTTPS git fetch under host policy
```

The outbound protocol has three outbound primitives: `execute` is a unary HTTP-style request, `stream` is an SSE / NDJSON / raw one-way stream, and `kernel.outbound.websocket.*` is bidirectional WebSocket. `websocket.open` is a streaming method that establishes a WSS connection and returns `connection_id`; `websocket.send` and `websocket.close` are unary methods. `connection_id` is also the `stream_id`; passing it to `kernel.capability.cancel` uses the same cancel/close path.

Request/response shapes are defined by runtime types and protocol dispatch parsing, not repeated in full here: HTTP/stream types live in `crates/ygg-runtime/src/runtime/outbound.rs`, WebSocket types live in `crates/ygg-runtime/src/runtime/outbound_websocket.rs`, and protocol parsing lives in `crates/ygg-runtime/src/runtime/protocol_dispatch.rs`. Core fields include `capability_id`, `destination_host`, `method`, optional `path`, `body_shape`, `metadata`, `secret_headers`, `static_headers`, and `timeout_ms`; `stream` also accepts `stream_format` (`sse` / `ndjson` / `raw`) and frame/duration limits; `websocket.open` accepts destination host/path, optional subprotocols, headers, `secret_refs`, and connection/frame/byte limits.

Outbound requests pass two fail-closed gates: the package manifest must declare matching `permissions.network.declarations` (WebSocket uses the `WEBSOCKET` method), and every `secret_headers` / `secret_refs` entry must be declared in `permissions.secret_refs`. The host profile must also explicitly enable the relevant outbound primitive, the destination host must match the allowlist by equality (or `*.suffix`), HTTP/SSE use HTTPS-only, WebSocket defaults to WSS-only, and redirects are rejected by default. `capability_id` must be in the caller package namespace; subprocess reverse kernel calls use the host-bound package principal and cannot spoof another package.

WebSocket-specific events use `kernel/outbound.websocket.*`: `opened` records handshake success and connection/subprotocol metadata; `frame` records inbound/outbound direction, frame kind, byte count, and sequence number without payload; `error` records a redacted error; `completed` records close code, reason, frame/byte counts, duration, executor kind, network_performed, redaction state, and secret_ref references.

All three outbound primitives emit completion audit events: `kernel/outbound.execute.completed`, `kernel/outbound.stream.completed`, and `kernel/outbound.websocket.completed`. These events record only status, counts, duration, executor kind, network_performed, redaction state, and `secret_ref` references; they do not record raw headers, bodies, secrets, frame payloads, or responses.

`kernel.outbound.audit` returns only redacted audit records: package, capability, destination host, method, purpose, used `secret_ref`s, and redaction state. Raw headers, bodies, secrets, and responses are not written to audit or protocol responses.

## Package methods

Each package contributes its own protocol methods through capability registrations and extension-point declarations. Their schemas are discoverable via `kernel.capability.describe` and `kernel.extension_point.describe`.

The kernel does not predefine methods like `session.input`, `prompt_frame.get`, `model.call`, `memory.search`. If those exist, they belong to specific packages.

## Errors

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

Package errors travel inside `capability.invoke` responses as `package_error` with provider-defined details.

## Streaming

Streaming flows over WebSocket or an equivalent transport. Streams carry typed frames whose schema is published with the method.

For `event.subscribe`, frames are event envelopes plus a `cursor` for resume.

For `capability.stream`, frames are provider-defined chunks plus a terminal status frame.

## Authentication and principals

A host enforces authentication at the transport layer. Each connection is associated with a principal: a user, assistant, package, host tool, anonymous caller, or remote system. The kernel checks permissions against the principal on every operation.

The kernel does not ship an identity provider. Hosts plug one in.

Alpha principal kinds:

```text
host_admin
host_dev
package { package_id }
human { user_id }
assistant { assistant_id, delegated_user_id? }
anonymous
```

Human and assistant principals require explicit scoped grants for sensitive operations:

```text
kernel.permission.grant
kernel.permission.revoke
kernel.permission.list
kernel.permission.audit
```

## Surface contributions

Packages may declare UI surface descriptors in their manifests. The kernel does not render or interpret these descriptors as content; it only exposes them for public clients:

```text
kernel.surface.contribution.list
kernel.surface.contribution.describe
```

Initial slots are `experience_entry`, `home_card`, `play_renderer`, `forge_panel`, `asset_editor`, and `assistant_action`.

Surface descriptors may include a version, launch capability, session template, input schema, permission UX metadata, and an approval policy. They remain descriptors; the kernel does not turn them into built-in experience/game semantics.

## Proposal lifecycle

Assistant and package-driven changes use generic proposal envelopes instead of privileged mutation paths:

```text
kernel.proposal.create
kernel.proposal.get
kernel.proposal.list
kernel.proposal.approve
kernel.proposal.reject
kernel.proposal.apply
```

Proposal statuses are `created`, `approved`, `rejected`, `applied`, and `failed`. Initial operation support is intentionally generic, such as `asset.put` and `projection.rebuild`. These operations must produce kernel audit/proposal events.

## Versioning

The protocol carries `protocol_version`. The kernel publishes the schema set per version. Breaking changes require a new version; the kernel may serve multiple concurrently.

Method schemas may evolve in backward-compatible ways within a version (additive fields). Breaking method changes require a new method id.

## Stability

Anything resembling `session.input`, `prompt_frame.get`, `model.call`, or any other content method is out of scope for kernel protocol forever. Adding such methods to the kernel is a charter violation.
