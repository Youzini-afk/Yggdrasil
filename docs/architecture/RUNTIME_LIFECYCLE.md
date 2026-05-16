# Runtime Lifecycle

The kernel runs three lifecycles: package, session, and capability invocation. None of them describes a turn, a chat, a prompt, or any other content-shaped operation. Those belong to packages.

## Package lifecycle

```text
discovered  manifest visible to the host
loading     manifest validated, sandbox prepared, ABI checked
starting    entry point booted, kernel handshake, capabilities and hooks registered
ready       accepting calls and dispatches
degraded    reachable but reporting reduced ability (heartbeat slow, partial features)
stopping    graceful shutdown signal sent
stopped     resources released
unloaded    no longer active in the host
```

Each transition emits a `kernel/package.*` event. Subscribers (observability tools, other packages) react via the public protocol; the kernel exposes no private hook for package state.

## Session lifecycle

A session is a labeled event stream with an attached package set and a permission scope. The kernel does not assign any other meaning to it.

```text
requested   open() received, principal and labels supplied
opening     kernel/session.before_open dispatched (sync, vetoable)
open        kernel/session.opened emitted
            event log accepting appends from authorized writers
            capability invocations dispatching against the active package set
closing     kernel/session.before_close dispatched (sync, vetoable)
closed      kernel/session.closed emitted; log frozen for further appends
```

The kernel does not own a "current turn," "active actor," or any content-level state of the session. If a package wants such a notion, it derives it from events.

## Capability invocation lifecycle

```text
requested        invoke(id, version, input) received
authorizing      kernel/capability.before_invoke dispatched (sync, vetoable)
routed           provider selected by id+version+session package set
running          provider executing; streaming chunks may flow
completed        kernel/capability.completed emitted with output (or stream end)
failed           kernel/capability.failed emitted with structured error
cancelled        cancellation acknowledged by provider; failed/completed event records the outcome
```

The kernel records invocations as kernel events. The contents of `input` and `output` are opaque to the kernel and validated only against the provider's declared schemas.

## Cancellation and timeouts

Every long-running operation (capability invocation, hook dispatch, package start) has a deadline derived from manifest sandbox policy plus host policy. Exceeding the deadline triggers cancellation. The kernel records the outcome.

The kernel does not invent its own cancellation semantics for content (no "regenerate," no "stop generating"). Such operations are package capabilities.

## Replay and bootstrap

When a host restarts:

1. Manifests are rediscovered.
2. Packages move through `loading` and `starting`.
3. Stored sessions are accessible for read-only replay immediately.
4. A session resumes write operations only after its required packages reach `ready`.

Packages that need to rebuild internal state from the event log do so via `events.read` and the replay stream. The kernel offers no other recovery mechanism.

## Errors

The kernel classifies errors only at its own boundary: transport, manifest, schema, permission, capacity, lifecycle, ambiguous-route. Package errors flow through capability invocations as opaque structured failures and are recorded under `kernel/capability.failed`.

## What this lifecycle does not describe

- No turn, no message, no prompt cycle.
- No model call orchestration.
- No memory update flow.
- No agent task.
- No world tick.

All of the above are appropriate inside packages. None of them are kernel lifecycles.
