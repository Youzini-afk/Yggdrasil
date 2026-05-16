# Kernel v0 Alpha Contract

This document is the implementation contract for the current Yggdrasil kernel alpha. It is intentionally narrower than the long-term architecture documents: if this matrix says a behavior is `implemented`, code and conformance must prove it; if it says `platform-host-alpha`, it is required for the next Platform Host Alpha milestone but no caller may depend on it yet.

The alpha goal is not a playable experience. The goal is a falsifiable, content-free kernel where packages, capabilities, events, permissions, and protocols can be tested without privileged official paths.

## Contract status language

- `implemented`: present in code and covered by tests or CLI conformance.
- `partial`: type or API exists, but behavior is incomplete or conformance is thin.
- `platform-host-alpha`: required for Platform Host Alpha, not yet complete.
- `deferred`: documented target outside the current milestone.

## Kernel object contract

| Object | Alpha status | Contract |
|---|---:|---|
| `KernelSession` | implemented | Holds identity, labels, active package set, principal scope, status, timestamps, metadata. It does not hold messages, turns, prompts, actors, worlds, or memory. |
| `EventEnvelope` | implemented | Append-only opaque JSON payload with per-session sequence, writer package id, namespaced kind, schema version, timestamp, metadata. |
| `PackageManifest` | implemented | Declares identity, entry form, provided capabilities, consumed capabilities, contributed schemas/hooks/extension points/assets, permissions, sandbox policy. |
| `PackageRecord` | partial | Tracks package id, version, entry kind, counts, manifest, trust level, state timestamps. Lifecycle validates and registers manifest declarations; `rust_inproc` entries are resolved through the host catalog before provided capabilities can load. Full start/stop handshakes for all entry forms remain next. |
| `CapabilityDescriptor` | implemented | Declares provider-owned capability id, version, input/output schema refs, streaming, side effects, description. |
| `HookSubscription` | partial | Manifest-declared subscription exists; real handler execution is not yet implemented. |
| `AssetRecord` | planned | Type exists, storage methods are protocol-planned but not implemented. |

## Protocol method matrix

| Method | Status | Notes |
|---|---:|---|
| `kernel.session.open` | implemented | Opens content-free session and writes `kernel/session.opened`. |
| `kernel.session.close` | implemented | Closes session and writes `kernel/session.closed`. |
| `kernel.session.get` | planned | Not exposed in service/CLI yet. |
| `kernel.session.list` | planned | Not exposed in service/CLI yet. |
| `kernel.event.append` | implemented | Enforces writer namespace and `events.append` for non-kernel writers. |
| `kernel.event.list` | partial | Lists whole session; runtime has caller-aware `events.read` gating, while HTTP/CLI host-level list remains unauthenticated for local administration. Sequence-range replay is next. |
| `kernel.event.subscribe` | platform-host-alpha | Declared as streaming method; sequence replay and cursor behavior are required before live subscription is considered complete. |
| `kernel.package.load` | partial | Validates manifest, host policy, resolves `rust_inproc` host entries for capability providers, registers declared capabilities/hooks, writes lifecycle event. Subprocess handshake/start is Platform Host Alpha work. |
| `kernel.package.unload` | partial | Removes registry record and declared capabilities/hooks, writes lifecycle event. Subprocess stop/kill is Platform Host Alpha work. |
| `kernel.package.list` | implemented | Lists in-memory package records. |
| `kernel.package.status` | implemented | Returns registry record for package id. |
| `kernel.package.describe` | planned | Can be derived from status manifest, but not exposed as method yet. |
| `kernel.capability.discover` | implemented | Lists registered descriptors. |
| `kernel.capability.describe` | planned | Registry can inspect descriptors; protocol method not exposed yet. |
| `kernel.capability.invoke` | partial | Enforces caller capability permission when a caller package id is supplied, detects ambiguous providers, validates capability input/output against the supported schema subset, and executes `rust_inproc` providers through the in-process package trait. Subprocess execution is Platform Host Alpha work. |
| `kernel.capability.stream` | planned | Descriptor flag exists; stream execution does not. |
| `kernel.capability.cancel` | planned | No in-flight invocation table yet. |
| `kernel.extension_point.list` | implemented | Lists registered extension points. |
| `kernel.extension_point.describe` | planned | Registry can inspect descriptors; protocol method not exposed yet. |
| `kernel.hook.list` | platform-host-alpha | Hook registry exists; protocol/service method not exposed yet. |
| `kernel.asset.put/get/list` | deferred | Asset types exist; storage is not implemented. |
| `kernel.host.info` | implemented | Returns protocol version and advertised method ids. |
| `kernel.host.ping` | partial | Advertised; direct service route is not yet exposed. |
| `kernel.host.principal` | planned | Identity provider integration deferred. |

## Kernel event kind matrix

| Event kind | Writer | Status | Trigger |
|---|---|---:|---|
| `kernel/session.opened` | kernel | implemented | Session open. |
| `kernel/session.closed` | kernel | implemented | Session close. |
| `kernel/package.loaded` | kernel | implemented | Manifest accepted and registered. |
| `kernel/package.unloaded` | kernel | implemented | Package removed from registry. |
| `kernel/package.degraded` | kernel | planned | Real package execution failure/health loss. |
| `kernel/capability.invoked` | kernel | planned | Invocation lifecycle event. |
| `kernel/capability.completed` | kernel | planned | Invocation success event. |
| `kernel/capability.failed` | kernel | planned | Invocation failure event. |
| `kernel/permission.denied` | kernel | implemented | Permission denial audit. |
| `kernel/error` | kernel | planned | General structured kernel error event. |

Non-kernel event kinds must start with the writer package id followed by `/`. The kernel must reject package attempts to write `kernel/...` or another package's namespace.

## Package entry matrix

| Entry form | Manifest status | Execution status | Trust level |
|---|---:|---:|---|
| `rust_inproc` | implemented | partial | `trusted_inproc` |
| `subprocess` | implemented | platform-host-alpha | `process_isolated` |
| `wasm` | implemented | deferred | `wasm_sandbox` |
| `remote` | implemented | deferred | `remote_boundary` |

Manifest support means the schema can describe the entry and host policy can accept/reject it. Execution support means the kernel actually calls across that boundary. `rust_inproc` now executes through a host-provided package trait and catalog; subprocess execution is required for Platform Host Alpha; WASM and remote execution remain deferred.

## Permission matrix

| Permission | Status | Current enforcement |
|---|---:|---|
| `events.append` | implemented | Required for non-kernel `event.append`. |
| `events.read` | partial | Runtime supports caller-aware read checks; transport-level principal plumbing and subscribe checks are Platform Host Alpha work. |
| `capabilities.invoke` | partial | Required when `caller_package_id` is present. Anonymous host calls are allowed only as host/dev operations and must not become package privilege. |
| `packages.call` | planned | Package-to-package control plane not implemented. |
| `assets.read/write` | planned | Asset store not implemented. |
| `network.hosts` | planned | Applies when subprocess/remote execution exists. |
| `filesystem.read/write` | planned | Applies when subprocess/WASM execution exists. |

## Lifecycle rules

Implemented:

1. Session open/close writes kernel events.
2. Package load validates manifest and host policy, registers manifest-declared capabilities/hooks/extension points, writes a kernel event.
3. Package unload removes registry declarations and writes a kernel event.
4. Event append assigns sequence/timestamp/id and enforces namespace ownership.
5. Permission denials write `kernel/permission.denied` audit events.
6. Closed sessions reject non-kernel appends.
7. Capability input/output and package-declared event payload schemas are validated against the current JSON Schema subset.

Next:

1. Package lifecycle must run actual entry handshake/register/start/stop.
2. Package load should expose explicit discovered/loading/starting/ready transitions rather than a direct ready record.
3. Capability lifecycle must write invoked/completed/failed events.
4. Kernel operations must dispatch before/after hooks according to the extension-point contract.
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
