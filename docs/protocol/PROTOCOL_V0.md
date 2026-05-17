# Public Protocol v0

The kernel exposes one public protocol. Studio, CLI, in-process packages, subprocess packages, WASM packages, and remote services use the same contract.

There is no private bypass. Official clients use this protocol; third parties use this protocol.

## Transports

All transports eventually surface the same protocol. Platform Host Alpha implements a minimal public subset first and marks the rest deferred until conformance covers them.

- In-process: a Rust API that mirrors the wire shape one-to-one.
- Subprocess: JSON-RPC over stdio. Required for Platform Host Alpha.
- HTTP: request/response for non-streaming methods. Required for Platform Host Alpha.
- Host stdio: JSON-RPC for automation and conformance. Required for Platform Host Alpha.
- WebSocket: subscriptions and streaming methods. Planned after sequence-range replay.
- TCP: JSON-RPC over a local socket. Deferred.
- Remote endpoint: HTTP and WebSocket against a declared URL. Deferred.
- WASM host: marshalled calls into the kernel-provided ABI. Deferred.

Transport selection is a host concern. A method is not considered implemented for public callers until at least one public transport path and conformance case exercise it without bypassing runtime permission checks.

## Protocol envelope

Canonical request/response transports use this shape:

```json
{
  "id": "request-1",
  "method": "kernel.capability.invoke",
  "params": {}
}
```

The host attaches principal and transport context. Callers do not self-assert package/admin identity through request JSON.

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

`event.append` requires `events.append` in the caller's manifest. `event.list` and `event.subscribe` require `events.read` for package principals. The current Host Alpha slice exposes HTTP SSE as a host-dev stream:

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

`invoke` resolves to a provider by id, optional `provider_package_id`, optional version constraint, and eventually session package set. If multiple providers match and the caller did not specify `provider_package_id`, the kernel returns an ambiguous-route error. Host Alpha currently supports exact version or same-major `^x.y` constraints.

### Extension points and hooks

```text
kernel.extension_point.list        list live extension points
kernel.extension_point.describe    fetch payload schema and timing
kernel.hook.list                   list subscribers to a point
```

The kernel does not expose a method to inject hooks at runtime; subscriptions are declared in manifests. Live registration is allowed only through package lifecycle.

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
```

The kernel manages projection records and rebuild lifecycle, but does not interpret content-specific state semantics. Package-owned projection execution belongs to packages.

### Health and identity

```text
kernel.host.info         host version, kernel ABI, transports
kernel.host.principal    the calling principal (user, package, remote)
kernel.host.ping         liveness
kernel.host.diagnostics  local host diagnostics for package/capability/hook observability
```

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

Streaming flows over WebSocket or transport-equivalent. Streams carry typed frames whose schema is published with the method.

For `event.subscribe`, frames are event envelopes plus a `cursor` for resume.

For `capability.stream`, frames are provider-defined chunks plus a terminal status frame.

## Authentication and principals

A host enforces authentication at the transport layer. Each connection is associated with a principal: a user, a package, or a remote system. The kernel checks permissions against the principal on every operation.

The kernel does not ship an identity provider. Hosts plug one in.

## Versioning

The protocol carries `protocol_version`. The kernel publishes the schema set per version. Breaking changes require a new version; the kernel may serve multiple concurrently.

Method schemas may evolve in backward-compatible ways within a version (additive fields). Breaking method changes require a new method id.

## Stability

Anything resembling `session.input`, `prompt_frame.get`, `model.call`, or any other content method is out of scope for kernel protocol forever. Adding such methods to the kernel is a charter violation.
