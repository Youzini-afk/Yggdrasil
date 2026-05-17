# Kernel v0 Alpha Contract

> [English](./KERNEL_V0_ALPHA_CONTRACT.md) · [中文](./KERNEL_V0_ALPHA_CONTRACT.zh-CN.md)

This document is the implementation contract for the current Yggdrasil kernel alpha. It is intentionally narrower than the long-term architecture documents: if this matrix says a behavior is `implemented`, code and conformance must prove it; if it says `partial`, the type or API exists but behavior is incomplete; if it says `planned` or `deferred`, no caller may depend on it yet.

For the executable snapshot of what runs today, see `docs/ALPHA_STATUS.md`. For the upcoming phases, see `docs/roadmap/NEXT_STEPS.md` and `docs/roadmap/PLATFORM_HOST_ALPHA.md`.

The alpha goal is not a playable experience. The goal is a falsifiable, content-free kernel where packages, capabilities, events, permissions, and protocols can be tested without privileged official paths. The Play/Forge Surface Contract Beta builds on this contract; it does not loosen it.

## Contract status language

- `implemented`: present in code and covered by tests or CLI conformance.
- `partial`: type or API exists, but behavior is incomplete or conformance is thin.
- `planned`: reserved in the contract, not yet implemented.
- `deferred`: documented target outside the current milestone.

## Kernel object contract

| Object | Alpha status | Contract |
|---|---:|---|
| `KernelSession` | implemented | Holds identity, labels, active package set, principal scope, status, timestamps, metadata. It does not hold messages, turns, prompts, actors, worlds, or memory. |
| `EventEnvelope` | implemented | Append-only opaque JSON payload with per-session sequence, writer package id, namespaced kind, schema version, timestamp, metadata. |
| `PackageManifest` | implemented | Declares identity, entry form, provided capabilities, consumed capabilities, contributed schemas/hooks/extension points/assets, permissions, sandbox policy. |
| `PackageRecord` | partial | Tracks package id, version, entry kind, counts, manifest, trust level, state timestamps. Lifecycle validates and registers manifest declarations; `rust_inproc` entries are resolved through the host catalog before provided capabilities can load; subprocess entries start a JSON-RPC stdio process and handshake before readiness. Loading/starting/ready/stopping/stopped/unloaded/degraded events are emitted for implemented entry forms. WASM/remote remain next. |
| `CapabilityDescriptor` | implemented | Declares provider-owned capability id, version, input/output schema refs, streaming, side effects, description. |
| `HookSubscription` | partial | Manifest-declared subscription exists; hook dispatch now runs for event append and capability invoke lifecycle points with stable ordering, legacy fixture handlers, package-owned handler capabilities, metadata mutation, and unload cleanup. Rich timeout/error audit remains next. |
| `AssetRecord` | partial | Opaque asset put/get/list exists with id, origin package, mime, hash, size, metadata, and `kernel/asset.put` audit event. Asset state can be rehydrated from the durable event log; binary/blob storage and permission enforcement remain next. |

## Protocol method matrix

| Method | Status | Notes |
|---|---:|---|
| `kernel.session.open` | implemented | Opens content-free session and writes `kernel/session.opened`. |
| `kernel.session.close` | implemented | Closes session and writes `kernel/session.closed`. |
| `kernel.session.fork` | partial | Creates a child session from a parent sequence and records branch lineage without interpreting content. |
| `kernel.session.branch.list` | partial | Lists in-memory branch records related to a session. |
| `kernel.session.get` | planned | Not exposed in service/CLI yet. |
| `kernel.session.list` | planned | Not exposed in service/CLI yet. |
| `kernel.event.append` | implemented | Enforces writer namespace and `events.append` for non-kernel writers. |
| `kernel.event.list` | implemented | Lists events by session with `after_sequence`, `limit`, `kind_prefix`, and `writer_package_id`; runtime has caller-aware `events.read` gating, while HTTP/CLI host-level list remains host-dev local administration. |
| `kernel.event.subscribe` | partial | HTTP SSE endpoint replays from `after_sequence` and tails live events. Protocol method dispatch and package-principal subscribe permissions remain next. |
| `kernel.package.load` | partial | Validates manifest, host policy, resolves `rust_inproc` host entries for capability providers, starts subprocess JSON-RPC stdio entries, registers declared capabilities/hooks, writes lifecycle event. Full transition events remain Platform Host Alpha work. |
| `kernel.package.unload` | partial | Stops subprocess handles when present, removes registry record and declared capabilities/hooks, writes lifecycle event. |
| `kernel.package.list` | implemented | Lists in-memory package records. |
| `kernel.package.status` | implemented | Returns registry record for package id. |
| `kernel.package.restart` | partial | Restarts subprocess entries and emits lifecycle events; other entry forms are rejected. |
| `kernel.package.logs` | partial | Drains captured subprocess stderr logs and emits `kernel/package.log` events; stdout remains reserved for JSON-RPC protocol frames. |
| `kernel.package.describe` | planned | Can be derived from status manifest, but not exposed as method yet. |
| `kernel.capability.discover` | implemented | Lists registered descriptors. |
| `kernel.capability.describe` | planned | Registry can inspect descriptors; protocol method not exposed yet. |
| `kernel.capability.invoke` | partial | Enforces caller capability permission when a caller package id is supplied, detects ambiguous providers unless `provider_package_id` is supplied, supports simple exact/major version constraints, validates capability input/output against the supported schema subset, executes `rust_inproc` providers through the in-process package trait, and executes subprocess JSON-RPC stdio providers with timeout/degraded handling. |
| `kernel.capability.stream` | planned | Descriptor flag exists; stream execution does not. |
| `kernel.capability.cancel` | planned | No in-flight invocation table yet. |
| `kernel.extension_point.list` | implemented | Lists registered extension points. |
| `kernel.extension_point.describe` | planned | Registry can inspect descriptors; protocol method not exposed yet. |
| `kernel.hook.list` | partial | Protocol dispatcher can list registered hooks; public docs and richer filtering remain Platform Host Alpha work. |
| `kernel.asset.put/get/list` | partial | Opaque asset substrate exists for host-dev protocol callers and can be rehydrated from SQLite-backed events. Package-principal permission checks and content-addressed blob storage remain next. |
| `kernel.projection.register/rebuild/get` | partial | Generic projection registry exists and can be rehydrated from SQLite-backed events; rebuild currently computes event count/last sequence from filtered event streams. Package-owned projection execution remains next. |
| `kernel.host.info` | implemented | Returns protocol version, advertised methods with statuses, and currently supported transport labels across in-process, HTTP `/rpc`, host stdio, and ad hoc HTTP. |
| `kernel.host.ping` | partial | Advertised; direct service route is not yet exposed. |
| `kernel.host.diagnostics` | partial | Returns package/capability/hook counts and package records for local host observability. |
| `kernel.host.principal` | planned | Identity provider integration deferred. |
| `kernel.permission.grant/revoke/list/audit` | partial | Host-dev callers can grant/revoke scoped permissions to human or assistant principals, list grants, and inspect grant/revoke audit events. Durable grant rehydration and full resource policy coverage remain next. |
| `kernel.proposal.create/get/list/approve/reject/apply` | partial | Generic proposal lifecycle for approval-gated play-creation changes. Initial apply support covers `asset.put` and `projection.rebuild`; broader transactions and revert/compensation remain next. |
| `kernel.surface.contribution.list` | partial | Lists typed package-declared surface descriptors for experience entry, Home/Play, Forge, asset editor, and assistant slots. The kernel stores descriptors only; UI rendering and content semantics remain package/client work. |
| `kernel.surface.contribution.describe` | partial | Describes one declared surface contribution by id. |

## Kernel event kind matrix

| Event kind | Writer | Status | Trigger |
|---|---|---:|---|
| `kernel/session.opened` | kernel | implemented | Session open. |
| `kernel/session.closed` | kernel | implemented | Session close. |
| `kernel/session.forked` | kernel | implemented | Session fork creates branch lineage. |
| `kernel/package.loaded` | kernel | implemented | Manifest accepted and registered. |
| `kernel/package.loading` | kernel | implemented | Package record enters loading. |
| `kernel/package.starting` | kernel | implemented | Subprocess package process is about to start/handshake. |
| `kernel/package.ready` | kernel | implemented | Package is ready after entry-specific startup. |
| `kernel/package.stopping` | kernel | implemented | Unload/restart is stopping package execution. |
| `kernel/package.stopped` | kernel | implemented | Package execution has stopped. |
| `kernel/package.unloaded` | kernel | implemented | Package removed from registry. |
| `kernel/package.degraded` | kernel | implemented | Real package execution failure/health loss. |
| `kernel/package.log` | kernel | implemented | Captured subprocess stderr log line. |
| `kernel/asset.put` | kernel | implemented | Opaque asset stored. |
| `kernel/projection.updated` | kernel | implemented | Generic projection state rebuilt. |
| `kernel/capability.invoked` | kernel | planned | Invocation lifecycle event. |
| `kernel/capability.completed` | kernel | planned | Invocation success event. |
| `kernel/capability.failed` | kernel | planned | Invocation failure event. |
| `kernel/permission.denied` | kernel | implemented | Permission denial audit. |
| `kernel/permission.granted` | kernel | implemented | Permission grant audit. |
| `kernel/permission.revoked` | kernel | implemented | Permission revoke audit. |
| `kernel/proposal.*` | kernel | partial | Proposal lifecycle audit events. |
| `kernel/error` | kernel | planned | General structured kernel error event. |

Non-kernel event kinds must start with the writer package id followed by `/`. The kernel must reject package attempts to write `kernel/...` or another package's namespace.

## Package entry matrix

| Entry form | Manifest status | Execution status | Trust level |
|---|---:|---:|---|
| `rust_inproc` | implemented | partial | `trusted_inproc` |
| `subprocess` | implemented | partial | `process_isolated` |
| `wasm` | implemented | deferred | `wasm_sandbox` |
| `remote` | implemented | deferred | `remote_boundary` |

Manifest support means the schema can describe the entry and host policy can accept/reject it. Execution support means the kernel actually calls across that boundary. `rust_inproc` now executes through a host-provided package trait and catalog. Subprocess JSON-RPC stdio execution now supports handshake/invoke/timeout/unload kill; fuller lifecycle event sequencing is still Platform Host Alpha work. WASM and remote execution remain deferred.

## Permission matrix

| Permission | Status | Current enforcement |
|---|---:|---|
| `events.append` | implemented | Required for non-kernel `event.append`. |
| `events.read` | partial | Runtime supports package manifest checks and scoped grants for human/assistant principals. SSE subscribe is currently host-dev only. |
| `capabilities.invoke` | partial | Runtime supports package manifest checks and scoped grants for human/assistant principals. Anonymous host calls are allowed only as host/dev operations and must not become package privilege. |
| `packages.call` | planned | Package-to-package control plane not implemented. |
| `assets.read/write` | planned | Asset store not implemented. |
| `projections` | planned | Projection registration is host-dev only; package permission model remains next. |
| `network.hosts` | planned | Applies when subprocess/remote execution exists. |
| `filesystem.read/write` | planned | Applies when subprocess/WASM execution exists. |

## Lifecycle rules

Implemented:

1. Session open/close writes kernel events.
2. Package load validates manifest and host policy, registers manifest-declared capabilities/hooks/extension points, writes a kernel event.
3. Package unload removes registry declarations and writes stopping/stopped/unloaded kernel events.
4. Event append assigns sequence/timestamp/id and enforces namespace ownership.
5. Permission denials write `kernel/permission.denied` audit events.
6. Closed sessions reject non-kernel appends.
7. Capability input/output and package-declared event payload schemas are validated against the current JSON Schema subset.
8. Protocol contexts distinguish host/dev calls from package-principal calls, and package-principal operations ignore caller-supplied package identity fields.
9. Canonical protocol envelopes can be dispatched in-process and through HTTP `/rpc`; `ygg host-stdio` exposes the same envelope over stdin/stdout for automation.
10. Subprocess JSON-RPC stdio packages can handshake, invoke capabilities, time out, degrade, restart, capture stderr logs, and unload with process kill.
11. The first hook fabric slice dispatches event/capability before/after points with stable ordering, legacy veto fixtures, package-owned handler capabilities, metadata mutation, and unload cleanup.
12. Event range replay is implemented for in-process protocol and HTTP ad hoc list; HTTP SSE can replay from `after_sequence` and tail new events.
13. Capability routing supports explicit provider selection and a simple exact/major version constraint.
14. Asset, branch, and generic projection substrate exists for host-dev protocol callers and can rehydrate from the durable event log.
15. Human and assistant principals can receive scoped grants for event reads and capability invocation, with grant/revoke audit events.
16. First official foundation packages (`official/package-lab`, `official/schema-tools`, `official/event-tools`) load through ordinary manifests and route through ordinary capabilities/surface descriptors.
17. `official/assistant-lab` is an ordinary assistant capability package that returns approval-gated proposals rather than mutating trusted state directly.
18. The first blank play-creation loop demo proves package launch, assistant proposal, branch fork, asset write, and projection rebuild without adding content semantics to the kernel.
19. Generic proposal lifecycle methods gate assistant/package changes behind explicit approval and append audit events.

Still partial for Platform Host Alpha:

1. Event subscribe lacks protocol-dispatch streaming and package-principal subscribe permissions.
2. Hook handler timeout/error audit is thin.
3. Package lifecycle emits transitions for implemented entry forms; lifecycle health checks and richer crash monitoring remain partial.
4. Capability routing has simple explicit provider/version constraints but no persisted provider selection policy.
5. Transport conformance covers core `/rpc` and host stdio behavior but not a full method parity matrix.
6. Asset/projection/branch substrate persists through the event log, but does not yet enforce package-principal permissions or use dedicated blob storage.

Next:

1. Package lifecycle must run actual entry handshake/register/start/stop.
2. Package load should expose explicit discovered/loading/starting/ready transitions rather than a direct ready record.
3. Capability lifecycle must write invoked/completed/failed events.
4. Kernel operations must dispatch before/after hooks according to the extension-point contract; event append and capability invoke have the first executable slice.
5. Session package sets must constrain routing.
6. Schema validation must grow from the current practical subset into a published full schema dialect.

## Schema validation subset

The alpha validates a deliberately small JSON Schema-compatible subset:

- `null` or `{}` means accept any JSON value.
- `type` may be `object`, `array`, `string`, `number`, `integer`, `boolean`, or `null`.
- `required` is enforced for object fields.

This is enough to make schema declarations executable in conformance without freezing a full schema dialect too early.

## Content-free invariant

The kernel crates must not define or require content-shaped concepts such as `Turn`, `Message`, `PromptFrame`, `ModelCall`, `Agent`, `World`, `Scene`, `Director`, or `Memory`. Any such concept belongs to a package.
