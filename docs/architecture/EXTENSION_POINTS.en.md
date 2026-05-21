# Extension Points

> [English](./EXTENSION_POINTS.en.md) · [中文](./EXTENSION_POINTS.md)

An extension point is a named hook emitted by the kernel or a package during operation. Other packages may subscribe to it. The kernel routes the call; it does not assign meaning.

This document covers the small set of kernel-emitted points and the shared rules for all extension points.

## Hook contract

Every extension point has:

- `id`: namespaced, immutable.
- `payload_schema`: the JSON shape of the call.
- `timing`: `sync` or `async`. Synchronous handlers block the operation. Asynchronous handlers do not.
- `modifiable`: whether subscribers may return a changed payload that the next subscriber sees.
- `short_circuit`: whether a subscriber may veto the operation.
- `ordering`: how the dispatcher orders subscribers. Declared precedence is used first; ties use a stable order.

The kernel publishes a schema for each kernel-emitted point. Packages publish schemas for the points they declare.

## Subscription

A subscriber is declared in a manifest:

```yaml
contributes:
  hooks:
    - extension_point: kernel/event.before_append
      handler: my_handler
      timing: sync
      precedence: 100
```

The kernel verifies that the subscriber's manifest declares the permissions implied by the hook. For example, `event.before_append` requires event read; modifying the payload requires event append.

A subscriber that returns an error stops the operation only when `short_circuit: true`. Otherwise the error is logged and dispatch continues.

## Cancellation and timeout

Synchronous handlers run within the operation's deadline. Asynchronous handlers receive a deadline derived from the package sandbox policy. Exceeding the deadline cancels the handler and counts as a failed call.

## Implementation status

The kernel-emitted point set is fixed by design. The current implementation covers the core paths for event append and capability invoke: stable ordering, package-owned handlers, payload metadata mutation, veto, and unload cleanup. Session and package lifecycle hooks are reserved in the contract. Today they are delivered through `kernel/session.*` and `kernel/package.*` events; later they will gain synchronous and asynchronous hook handling. New points should come from package contributions, not from growing the kernel.

## Kernel-emitted points

The kernel emits a small fixed set of points. New points come from package contributions.

### Session lifecycle

- `kernel/session.before_open` — sync, modifiable false, short_circuit true.
  Permission to open is enforced here. Subscribers may veto.
- `kernel/session.after_open` — async.
- `kernel/session.before_close` — sync, modifiable false, short_circuit true.
- `kernel/session.after_close` — async.

Payload: session id, requested labels, package set, requesting principal.

### Event log

- `kernel/event.before_append` — sync, modifiable true, short_circuit true.
  Permission and schema enforcement happen here. Subscribers may amend metadata or veto.
- `kernel/event.after_append` — async.
  Subscribers receive the persisted envelope.

Payload: event envelope. The kernel does not interpret the payload field. It only checks declared schemas when the writer's manifest references a payload schema for that event kind.

### Capability invocation

- `kernel/capability.before_invoke` — sync, modifiable true, short_circuit true.
  Permission, route resolution, and quota enforcement happen here.
- `kernel/capability.after_invoke` — async.
  Subscribers receive input, output (or error), latency, and provider id.
- `kernel/capability.error` — async.
  Subscribers receive the structured failure.

Payload: invocation envelope.

### Package lifecycle

- `kernel/package.loaded` — async.
- `kernel/package.unloaded` — async.
- `kernel/package.degraded` — async.
- `kernel/package.heartbeat_lost` — async.

### Hook registry

- `kernel/hook.registered` — async.
- `kernel/hook.unregistered` — async.

These let observability packages discover the live extension topology.

## Package-emitted points

A package may publish its own extension points by listing them under `contributes.extension_points`. The package becomes the owner of the schema.

The kernel routes calls but does not validate semantics. If the owning package is unloaded, the kernel refuses to dispatch the point and emits `kernel/hook.unregistered` for any orphaned subscribers.

Example (illustrative; not part of the kernel):

```yaml
contributes:
  extension_points:
    - id: someorg/conversation/before_step
      payload_schema: ...
      timing: sync
      modifiable: true
      short_circuit: true
```

A different package can subscribe:

```yaml
contributes:
  hooks:
    - extension_point: someorg/conversation/before_step
      handler: ...
```

The kernel does not know what `conversation/before_step` means. The owning package does.

## Discovery

A client may query the kernel for live extension points and their subscribers. Schemas are exposed. Creator tools, observability dashboards, and other packages use this to see what is currently extensible in a running host.

## Versioning

Each extension point has a `version`. Subscribers declare the version they target. The kernel refuses to dispatch to a subscriber whose declared version is incompatible with the live point.

Breaking changes to a point require a new id. The owning package may emit both versions during transition.

## Stability

The kernel-emitted point set is small by design. Adding a kernel point needs the same justification as adding a kernel responsibility: it truly cannot live in a package.
