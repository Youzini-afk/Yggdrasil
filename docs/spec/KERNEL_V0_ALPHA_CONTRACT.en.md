# Kernel v0 Alpha Contract

> [English](./KERNEL_V0_ALPHA_CONTRACT.en.md) · [中文](./KERNEL_V0_ALPHA_CONTRACT.md)

This document is the implementation contract for the current Yggdrasil kernel alpha. It is intentionally narrower than the long-term architecture documents. If this document says a behavior is `implemented`, code and conformance must prove it. If the status is `partial`, the type or API exists but behavior is incomplete. If the status is `planned` or `deferred`, callers must not depend on it yet.

For the executable snapshot of what runs today, see `docs/ALPHA_STATUS.md`. For upcoming work, see `docs/roadmap/NEXT_STEPS.md`.

The alpha goal is not a playable experience. The goal is a falsifiable, content-free kernel.v1. Packages, capabilities, events, permissions, and protocols must be testable without privileged official paths. The Play/Forge surface contract builds on this contract and does not loosen it.

## Contract status language

- `implemented`: present in code and covered by tests or CLI conformance.
- `partial`: type or API exists, but behavior is incomplete or conformance is still thin.
- `planned`: reserved in the contract but not yet implemented.
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
| `AssetRecord` | partial | Opaque asset put/get/list exists with id, origin package, mime, hash, size, metadata, and `kernel/v1/asset.put` audit event. Asset state can be rehydrated from the durable event log; **raw-secret scanning is enforced on asset metadata**. Binary/blob storage and permission enforcement remain next. |

## Protocol method matrix

| Method | Status | Notes |
|---|---:|---|
| `kernel.v1.session.open` | implemented | Opens content-free session and writes `kernel/v1/session.opened`. |
| `kernel.v1.session.close` | implemented | Closes session and writes `kernel/v1/session.closed`. |
| `kernel.v1.session.fork` | partial | Creates a child session from a parent sequence and records branch lineage without interpreting content. |
| `kernel.v1.session.branch.list` | partial | Lists in-memory branch records related to a session. |
| `kernel.v1.session.get` | planned | Not exposed in service/CLI yet. |
| `kernel.v1.session.list` | planned | Not exposed in service/CLI yet. |
| `kernel.v1.event.append` | implemented | Enforces writer namespace and `events.append` for non-kernel writers. |
| `kernel.v1.event.list` | implemented | Lists events by session with `after_sequence`, `limit`, `kind_prefix`, and `writer_package_id`; runtime has caller-aware `events.read` gating, while HTTP/CLI host-level list remains host-dev local administration. |
| `kernel.v1.event.subscribe` | partial | HTTP SSE endpoint replays from `after_sequence` and tails live events. Protocol method dispatch and package-principal subscribe permissions remain next. |
| `kernel.v1.package.load` | partial | Validates manifest, host policy, resolves `rust_inproc` host entries for capability providers, starts subprocess JSON-RPC stdio entries, registers declared capabilities/hooks, writes lifecycle event. Full transition events remain Platform Host Alpha work. |
| `kernel.v1.package.unload` | partial | Stops subprocess handles when present, removes registry record and declared capabilities/hooks, writes lifecycle event. |
| `kernel.v1.package.list` | implemented | Lists in-memory package records. |
| `kernel.v1.package.status` | implemented | Returns registry record for package id. |
| `kernel.v1.package.restart` | partial | Restarts subprocess entries and emits lifecycle events; other entry forms are rejected. |
| `kernel.v1.package.logs` | partial | Drains captured subprocess stderr logs and emits `kernel/v1/package.log` events; stdout remains reserved for JSON-RPC protocol frames. |
| `kernel.v1.package.describe` | planned | Can be derived from status manifest, but not exposed as method yet. |
| `kernel.v1.capability.discover` | implemented | Lists registered descriptors. |
| `kernel.v1.capability.describe` | planned | Registry can inspect descriptors; protocol method not exposed yet. |
| `kernel.v1.capability.invoke` | partial | Enforces caller capability permission when a caller package id is supplied, detects ambiguous providers unless `provider_package_id` is supplied, supports simple exact/major version constraints, validates capability input/output against the supported schema subset, executes `rust_inproc` providers through the in-process package trait, and executes subprocess JSON-RPC stdio providers with timeout/degraded handling. |
| `kernel.v1.capability.stream` | partial | Descriptor flag exists; stream start/cancel lifecycle works with in-memory registry and ordered events. Real network streaming deferred. |
| `kernel.v1.capability.cancel` | partial | In-memory invocation registry tracks in-flight streams; cancel marks invocation cancelled and blocks further chunks. |
| `kernel.v1.extension_point.list` | implemented | Lists registered extension points. |
| `kernel.v1.extension_point.describe` | planned | Registry can inspect descriptors; protocol method not exposed yet. |
| `kernel.v1.hook.list` | partial | Protocol dispatcher can list registered hooks; public docs and richer filtering remain Platform Host Alpha work. |
| `kernel.v1.asset.put/get/list` | partial | Opaque asset substrate exists for host-dev protocol callers and can be rehydrated from SQLite-backed events. Package-principal permission checks and content-addressed blob storage remain next. |
| `kernel.v1.projection.register/rebuild/get` | partial | Generic projection registry exists and can be rehydrated from SQLite-backed events; rebuild currently computes event count/last sequence from filtered event streams. Package-owned projection execution remains next. |
| `kernel.v1.host.info` | implemented | Returns protocol version, advertised methods with statuses, and currently supported transport labels across in-process, HTTP `/rpc`, host stdio, and ad hoc HTTP. |
| `kernel.v1.host.ping` | partial | Advertised; direct service route is not yet exposed. |
| `kernel.v1.host.diagnostics` | partial | Returns package/capability/hook counts and package records for local host observability. |
| `kernel.v1.host.principal` | planned | Identity provider integration deferred. |
| `kernel.v1.permission.grant/revoke/list/audit` | partial | Host-dev callers can grant/revoke scoped permissions to human or assistant principals, list grants, and inspect grant/revoke audit events. **Permission grants now survive rehydrate** from the SQLite event log. Full resource policy coverage remains next. |
| `kernel.v1.proposal.create/get/list/approve/reject/apply` | partial | Generic proposal lifecycle for approval-gated play-creation changes. Initial apply support covers `asset.put` and `projection.rebuild`. **Raw secret scanning is enforced**: proposals containing raw secrets in operation payloads or expected_effects are rejected. Broader transactions and revert/compensation remain next. |
| `kernel.v1.surface.contribution.list` | partial | Lists typed package-declared surface descriptors for experience entry, Home/Play, Forge, asset editor, and assistant slots. The kernel stores descriptors only; UI rendering and content semantics remain package/client work. |
| `kernel.v1.surface.contribution.describe` | partial | Describes one declared surface contribution by id. |

## Kernel event kind matrix

| Event kind | Writer | Status | Trigger |
|---|---|---:|---|
| `kernel/v1/session.opened` | kernel | implemented | Session open. |
| `kernel/v1/session.closed` | kernel | implemented | Session close. |
| `kernel/v1/session.forked` | kernel | implemented | Session fork creates branch lineage. |
| `kernel/v1/package.loaded` | kernel | implemented | Manifest accepted and registered. |
| `kernel/v1/package.loading` | kernel | implemented | Package record enters loading. |
| `kernel/v1/package.starting` | kernel | implemented | Subprocess package process is about to start/handshake. |
| `kernel/v1/package.ready` | kernel | implemented | Package is ready after entry-specific startup. |
| `kernel/v1/package.stopping` | kernel | implemented | Unload/restart is stopping package execution. |
| `kernel/v1/package.stopped` | kernel | implemented | Package execution has stopped. |
| `kernel/v1/package.unloaded` | kernel | implemented | Package removed from registry. |
| `kernel/v1/package.degraded` | kernel | implemented | Real package execution failure/health loss. |
| `kernel/v1/package.log` | kernel | implemented | Captured subprocess stderr log line. |
| `kernel/v1/asset.put` | kernel | implemented | Opaque asset stored. |
| `kernel/v1/projection.updated` | kernel | implemented | Generic projection state rebuilt. |
| `kernel/v1/capability.invoked` | kernel | planned | Invocation lifecycle event. |
| `kernel/v1/capability.completed` | kernel | planned | Invocation success event. |
| `kernel/v1/capability.failed` | kernel | planned | Invocation failure event. |
| `kernel/v1/stream.started` | kernel | partial | Streaming invocation started. |
| `kernel/v1/stream.chunk` | kernel | partial | Streaming chunk frame emitted. |
| `kernel/v1/stream.progress` | kernel | partial | Streaming progress indication. |
| `kernel/v1/stream.ended` | kernel | partial | Streaming invocation ended normally. |
| `kernel/v1/stream.error` | kernel | partial | Streaming invocation errored. |
| `kernel/v1/stream.cancelled` | kernel | partial | Streaming invocation cancelled by caller. |
| `kernel/v1/stream.timeout` | kernel | partial | Streaming invocation timed out. |
| `kernel/v1/permission.denied` | kernel | implemented | Permission denial audit. |
| `kernel/v1/permission.granted` | kernel | implemented | Permission grant audit. |
| `kernel/v1/permission.revoked` | kernel | implemented | Permission revoke audit. |
| `kernel/v1/proposal.*` | kernel | partial | Proposal lifecycle audit events. |
| `kernel/v1/outbound.request` | kernel | partial | Outbound network request allowed and audited. |
| `kernel/v1/outbound.denied` | kernel | partial | Outbound network request denied. |
| `kernel/v1/error` | kernel | planned | General structured kernel error event. |

Non-kernel event kinds must start with the writer package id followed by `/`. The kernel must reject package attempts to write `kernel/v1/...` or another package's namespace.

## Package entry matrix

| Entry form | Manifest status | Execution status | Trust level |
|---|---:|---:|---|
| `rust_inproc` | implemented | partial | `trusted_inproc` |
| `subprocess` | implemented | partial | `process_isolated` |
| `wasm` | implemented | deferred | `wasm_sandbox` |
| `remote` | implemented | deferred | `remote_boundary` |

Manifest support means the schema can describe the entry and host policy can accept or reject it. Execution support means the kernel actually calls across that boundary. `rust_inproc` now executes through a host-provided package trait and catalog. Subprocess JSON-RPC stdio execution supports handshake, invoke, timeout, and unload kill. Fuller lifecycle event sequencing remains next. WASM and remote execution remain deferred.

## Permission matrix

| Permission | Status | Current enforcement |
|---|---|---:|
| `events.append` | implemented | Required for non-kernel `event.append`. |
| `events.read` | partial | Runtime supports package manifest checks and scoped grants for human/assistant principals. SSE subscribe is currently host-dev only. |
| `capabilities.invoke` | partial | Runtime supports package manifest checks and scoped grants for human/assistant principals. Anonymous host calls are allowed only as host/dev operations and must not become package privilege. |
| `packages.call` | planned | Package-to-package control plane not implemented. |
| `assets.read/write` | planned | Asset store not implemented. |
| `projections` | planned | Projection registration is host-dev only; package permission model remains next. |
| `network.hosts` | partial | Packages declare allowed outbound hosts in manifest; runtime `check_network_policy` and `check_and_audit_outbound` enforce allowlists for Ygg-provided network helpers. Flat `hosts` list and structured `declarations` (host, methods, purpose) are supported. Official packages have no bypass. Denied requests write `kernel/v1/outbound.denied`; allowed requests write `kernel/v1/outbound.request` with redacted audit. |
| `filesystem.read/write` | planned | Applies when subprocess/WASM execution exists. |

## Secret reference contract

| Contract element | Status | Notes |
|---|---|---:|
| `SecretRef` type and validation | implemented | Recognizes `secret_ref:<vault>:<key>`, `secretRef:`, `secret-ref:`, and `host:` patterns. |
| `HostSecretResolver` trait | implemented | Async trait for runtime secret resolution. `DenyAllSecretResolver` placeholder rejects all. Production vault integrations are host-level packages, not kernel.v1. |
| `SecretResolverConfig` on `RuntimeConfig` | implemented | Default uses `DenyAllSecretResolver`; hosts can provide custom resolver. |
| Raw-secret blocking in proposals | implemented | Proposals with raw secrets in operation payloads or expected_effects are rejected. `secret_ref` references are accepted. |
| Raw-secret blocking in asset metadata | implemented | Asset metadata with raw secrets is rejected. Asset content is excluded from scanning (arbitrary user data). `secret_ref` references are accepted. |
| Official-package no-secret-bypass | implemented | Secret scanning applies uniformly; official packages have no bypass. |
| Permission grant rehydrate | implemented | `kernel/v1/permission.granted` and `kernel/v1/permission.revoked` events are replayed during `hydrate_substrate_from_events`. Grants survive runtime reconstruction against the same SQLite store. |
| Resolved secrets never written to event log | implemented by contract | The `HostSecretResolver` trait is only used at runtime; no kernel path writes resolved secrets back to events/proposals/logs/audit. |

## Lifecycle rules

Implemented:

1. Session open/close writes kernel events.
2. Package load validates manifest and host policy, registers manifest-declared capabilities/hooks/extension points, writes a kernel event.
3. Package unload removes registry declarations and writes stopping/stopped/unloaded kernel events.
4. Event append assigns sequence/timestamp/id and enforces namespace ownership.
5. Permission denials write `kernel/v1/permission.denied` audit events.
6. Closed sessions reject non-kernel appends.
7. Capability input/output and package-declared event payload schemas are validated against the current JSON Schema subset.
8. Protocol contexts distinguish host/dev calls from package-principal calls. Package-principal operations ignore caller-supplied package identity fields.
9. Canonical protocol envelopes can be dispatched in-process and through HTTP `/rpc`. `ygg host-stdio` exposes the same envelope over stdin/stdout for automation.
10. Subprocess JSON-RPC stdio packages can handshake, invoke capabilities, time out, degrade, restart, capture stderr logs, and unload with process kill.
11. The first hook fabric slice dispatches event/capability before/after points. It supports stable ordering, legacy veto fixtures, package-owned handler capabilities, metadata mutation, and unload cleanup.
12. Event range replay is implemented for in-process protocol and HTTP ad hoc list. HTTP SSE can replay from `after_sequence` and tail new events.
13. Capability routing supports explicit provider selection and a simple exact/major version constraint.
14. Asset, branch, and generic projection substrate exists for host-dev protocol callers and can rehydrate from the durable event log.
15. Human and assistant principals can receive scoped grants for event reads and capability invocation, with grant/revoke audit events.
16. First official foundation packages (`official/package-lab`, `official/schema-tools`, `official/event-tools`) load through ordinary manifests and route through ordinary capabilities/surface descriptors.
17. `official/assistant-lab` is an ordinary assistant capability package that returns approval-gated proposals rather than mutating trusted state directly.
18. The first blank play-creation loop demo proves package launch, assistant proposal, branch fork, asset write, and projection rebuild without adding content semantics to the kernel.v1.
19. Generic proposal lifecycle methods gate assistant/package changes behind explicit approval and append audit events.
20. Permission grants are persisted through `kernel/v1/permission.granted` and `kernel/v1/permission.revoked` events and rehydrated during `hydrate_substrate_from_events`.
21. Secret references follow the `secret_ref:<vault>:<key>` contract. Raw secrets in proposal payloads and asset metadata are rejected by the kernel.v1. Content/description/title/reason fields are excluded from value-pattern scanning to avoid false positives on ordinary text.
22. The `HostSecretResolver` trait provides runtime-only secret resolution. Resolved raw secrets must never be written back to events, proposals, logs, or audit records. Official packages have no bypass.
23. Network permission declarations: packages declare allowed outbound destinations in `permissions.network`. Structured `declarations` include host, methods, and purpose; flat `hosts` remains for backward compatibility. The runtime policy checker enforces allowlists for Ygg-provided network helpers. Official packages have no bypass.
24. Outbound audit records: `OutboundAuditRecord` captures principal, package_id, capability_id, destination_host, method, purpose, redaction_state, secret_refs_used, usage/cost placeholders, and status/error. Raw body/header/prompt/response is never saved. `redaction_state` defaults to `redacted`.
25. Denied outbound requests write `kernel/v1/outbound.denied` events; allowed requests write `kernel/v1/outbound.request` events. Both are inspectable via `kernel.v1.outbound.audit`.
26. Streaming invocation registry: `StreamRegistry` tracks in-flight streaming capability invocations with start/append/end/cancel/timeout lifecycle. `StreamFrameEnvelope` defines generic content-free frame types (start/chunk/progress/end/error/cancelled/timeout) with invocation_id, stream_id, sequence, redaction_state, and timestamp/metadata. It carries no model/prompt/agent semantics.
27. `kernel.v1.capability.stream` validates `streaming=true` in the capability descriptor before starting a streaming invocation. Non-streaming capabilities (descriptor `streaming=false`) are rejected.
28. Cancel marks an active streaming invocation `Cancelled` and blocks further chunk/progress frames. Timeout marks an invocation `Timeout` and blocks further frames. Error terminal frame sets state to `Error` and blocks further frames. Normal end sets state to `Ended`.
29. Streaming lifecycle emits ordered kernel events: `kernel/v1/stream.started` on start, `kernel/v1/stream.chunk` on chunk, `kernel/v1/stream.progress` on progress, `kernel/v1/stream.ended` on normal end, `kernel/v1/stream.error` on error, `kernel/v1/stream.cancelled` on cancel, `kernel/v1/stream.timeout` on timeout.
30. `StreamInvocationRecord` tracks invocation_id, stream_id, capability_id, provider_package_id, session_id, state, frame_count, timestamps, and metadata. Terminal states block further frame appends.

Still partial:

1. Event subscribe lacks protocol-dispatch streaming and package-principal subscribe permissions.
2. Hook handler timeout/error audit is thin.
3. Package lifecycle emits transitions for implemented entry forms; lifecycle health checks and richer crash monitoring remain partial.
4. Capability routing has simple explicit provider/version constraints but no persisted provider selection policy.
5. Transport conformance covers core `/rpc` and host stdio behavior, but not a full method parity matrix.
6. Asset/projection/branch substrate persists through the event log, but does not yet enforce package-principal permissions or use dedicated blob storage.
7. Production secret vault integration is deferred to host-level packages; `DenyAllSecretResolver` is the default.
8. Network permission enforcement covers Ygg-provided network/request helpers; arbitrary subprocess OS-level outbound interception is not claimed.

Next:

1. Package lifecycle must run actual entry handshake/register/start/stop.
2. Package load should expose explicit discovered/loading/starting/ready transitions rather than a direct ready record.
3. Capability lifecycle must write invoked/completed/failed events.
4. Kernel operations must dispatch before/after hooks according to the extension-point contract. Event append and capability invoke already have the first executable slice.
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
