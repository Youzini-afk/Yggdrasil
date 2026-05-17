# Event Model

> [English](./EVENT_MODEL.md) · [中文](./EVENT_MODEL.zh-CN.md)

The event log is the kernel's source of truth. It is per-session, append-only, durable, and ordered.

The kernel does not interpret event payloads. Meaning is owned by capability packages.

## Envelope

Every persisted event uses the same envelope:

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

The kernel:

- assigns `id`, `sequence`, `timestamp`, and `writer_package_id`,
- enforces that `kind` is namespaced under the writer's id (or `kernel/...` for kernel events),
- validates `payload` against the writer's declared schema if one is declared,
- treats `metadata` as opaque.

## Kinds

There are two flavors of event kinds.

### Kernel-emitted kinds

A small fixed set the kernel itself produces. They describe kernel operations, not content.

Session:

```text
kernel/session.opened
kernel/session.closed
kernel/session.forked
```

Package lifecycle:

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

Capability invocation (planned audit shape):

```text
kernel/capability.invoked
kernel/capability.completed
kernel/capability.failed
```

Permission audit:

```text
kernel/permission.granted
kernel/permission.revoked
kernel/permission.denied
```

Generic substrate:

```text
kernel/asset.put
kernel/projection.updated
```

Proposal lifecycle:

```text
kernel/proposal.created
kernel/proposal.approved
kernel/proposal.rejected
kernel/proposal.applied
kernel/proposal.failed
```

Transport / runtime errors (planned):

```text
kernel/error
```

These are the only event kinds the kernel knows about by name. Their payloads describe kernel operations and never content.

### Package-emitted kinds

Everything else. Each package defines its own event kinds in its manifest, namespaced under its package id. Examples (illustrative; not part of the kernel):

```text
someorg/conversation/turn.started
someorg/conversation/prompt.rendered
someorg/conversation/model.streamed
someorg/world-sim/tick.completed
someorg/memory-pack/proposal.created
```

The kernel persists these and orders them. It does not understand them.

## Permissions

Appending an event requires `events.append` in the writer's manifest. Reading an event stream requires `events.read` (and may be scoped to specific sessions).

A package cannot append events under another package's namespace. Cross-package event coordination flows through capability invocations or extension points, not by impersonating each other in the log.

## Persistence rules

- Append-only. The log is never edited.
- Per-session ordering is monotonic. The kernel makes no cross-session ordering claim.
- Durable. After `kernel/event.after_append` fires, the event is committed.
- Replayable. The kernel can stream events from `sequence` 0 forward.

## Replay

The kernel can replay events to:

- a newly subscribing client,
- a newly loaded package that requested catch-up,
- a snapshot tool.

The kernel replays envelopes verbatim. Meaning, projection, and state reconstruction are package concerns.

## Versioning

Each event kind carries a `schema_version`. The owning writer is responsible for migrations. The kernel does not migrate payloads; it persists what was written.

A package can publish a new `schema_version` for its kind without changing the kernel.

## Causation and correlation

The envelope's `metadata` may carry `causation_id` (the event that caused this one) and `correlation_id` (a logical trace), but the kernel treats them as opaque. Packages decide what they mean.

## What this model deliberately omits

- No chat history concept.
- No turn or message concept.
- No prompt frame, context plan, or model call concept.
- No memory or world-state concept.
- No agent task or proposal concept.

All of those are valid event kinds for the packages that need them. None of them are kernel events.

## Stability

The kernel-emitted kind set is small by design. Adding a new kernel kind requires the same justification as adding a new kernel responsibility: it cannot reasonably live in a package.
