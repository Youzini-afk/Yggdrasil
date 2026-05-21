# Platform kernel

> [English](./PLATFORM_KERNEL.en.md) · [中文](./PLATFORM_KERNEL.md)

The kernel is the smallest infrastructure layer that lets capability packages coexist on Yggdrasil. It's small, opinion-free about content, and stable.

This document defines what the kernel does and what it doesn't. Anything not listed as a kernel responsibility has to live in a package.

## What the kernel does

### 1. Identity and schema

- Generates ids for sessions, events, packages, capability invocations, and asset records.
- Maintains `schema_version` on every persisted contract object.
- Validates manifests, hook subscriptions, and capability registrations against published schemas.

### 2. Session shell

- Allocates and addresses sessions.
- Holds per-session metadata (id, created_at, label, status).
- Carries the event stream and the permission scope.
- The kernel doesn't interpret what a session is for. A session is just a labeled event stream with a set of attached packages.

### 3. Append-only event log

- Accepts events from authorized writers.
- Orders them per session.
- Persists them.
- Replays them on demand.
- The kernel treats event payloads as opaque JSON. Meaning belongs to packages.

### 4. Package registry

- Loads, validates, and starts packages from manifests.
- Tracks state (registered, loading, ready, degraded, stopped).
- Unloads cleanly.
- Schedules lifecycle: a session declares which packages are active in its scope.

### 5. Capability routing

- Indexes capabilities by id and version.
- Routes calls and streams to providers.
- Records calls to the event log when configured to.
- Negotiates version constraints between consumers and providers.

### 6. Extension dispatch

- Maintains the extension-point registry.
- Holds subscriber lists.
- Dispatches hook calls in the declared order and timing.
- Enforces timeouts and cancellation.

### 7. Permission gate

- Identifies principals (`host_admin`, `host_dev`, `package`, `human`, `assistant`, `anonymous`).
- Reads each package's manifest-declared permissions.
- Tracks scoped grants for human and assistant principals (`events.read`, `capabilities.invoke`, etc.).
- Enforces all of the above on event writes, capability invocations, cross-package calls, and network / filesystem access.
- Rejects undeclared side effects and writes a `kernel/permission.denied` audit event.

### 8. Surface contributions

- Accepts UI surface descriptors that packages declare in their manifests (slots: `experience_entry`, `home_card`, `play_renderer`, `forge_panel`, `asset_editor`, `assistant_action`).
- Exposes them through the public protocol so any client can discover what's launchable, viewable, or actionable.
- Stores descriptors only. Rendering and content semantics belong to packages and clients.

### 9. Proposal lifecycle

- Schedules generic, approval-gated change proposals (`create`, `get`, `list`, `approve`, `reject`, `apply`).
- Only applies operations the kernel itself understands (`asset.put`, `projection.rebuild`).
- Emits `kernel/proposal.*` audit events on every state transition.
- Rejects applying unapproved proposals or proposals whose operations the kernel doesn't recognize. The kernel never invents domain-specific proposal semantics.

### 10. Assets, branches, projections

- Maintains an opaque asset registry (`id`, `mime`, `hash`, `size`, `origin_package`, `metadata`, content blob).
- Tracks session fork / branch lineage as kernel records.
- Maintains generic projection records, rebuilt by filtering the event log; the kernel doesn't interpret projection state.
- All three can be recovered from the persistent event log.

### 11. Transport layer

- Carries the canonical protocol envelope over: in-process Rust API, HTTP `/rpc`, host JSON-RPC stdio (`ygg host-stdio`), and SSE event subscription.
- Profile-driven `ygg host serve` autoloads packages and exposes the same dispatcher.
- WebSocket and TCP transports are reserved for later.
- All transports present the same conceptual protocol; official packages and clients use it just like third parties.

### 12. Sandbox boundary

- In-process Rust packages run inside the kernel binary (trust level `trusted_inproc`).
- Subprocess packages are launched and supervised over JSON-RPC stdio with handshake, invocation timeout, kill-on-unload, restart, and stderr capture (trust level `process_isolated`).
- WASM (`wasm_sandbox`) and remote (`remote_boundary`) entries are reserved as first-class manifest forms; execution is deferred.

### 13. Public protocol

- The wire-level contract for everything above. The kernel doesn't use a private side door; official packages and clients use the same protocol as third parties.

## What the kernel doesn't do

The kernel takes no opinion on the following. They belong to packages, including official ones.

### Conversation, prompts, models

- No turns, messages, prompt frames, context plans, model calls, sampling, or token usage.
- No prompt rendering, template language, or system / user / assistant roles.
- No model-provider abstraction, streaming chunk format, or chat history.

### Worlds, characters, scenes, rules

- No world model, scene graph, or actor type.
- No character schema, relationship state, inventory, or clock.
- No rule engine, conditions / effects, dice, or combat resolution.

### Memory

- No memory taxonomy, embeddings, or retrieval strategy.
- No summarization, pinning, or merge policy.

### Agents and directors

- No agent loop, planner, or director.
- No propose-and-commit pattern — unless a package chooses to define one.

### Content sources

- No SillyTavern parser, PNG metadata reader, or character-card schema.
- No game-engine bridge, no UE5 / Godot / Unity glue.

### Presentation

- No UI, no chat panel, no inspector, no editor.
- No theme, layout, or asset rendering.

### Storage opinion

- No business tables. The kernel needs to store events, manifests, and asset records, but it doesn't ship an ORM, a query builder, or a content-shaped data model.

## Gray areas

These need an explicit stance to prevent drift.

### Assets

The kernel maintains an asset registry. It records `id`, `mime`, `hash`, `size`, `origin_package`, and the content blob. It doesn't parse, render, or interpret asset contents. Packages own their formats.

### Event ordering

The kernel guarantees monotonic ordering and persistence within a session. It guarantees nothing across sessions, no causal graph, no correlation semantics. Causal / correlation fields are opaque metadata supplied by writers.

### Errors

Kernel errors cover transport, permissions, schema validation, manifests, capacity, and package lifecycle. Package errors flow through capability calls as opaque structured failures; the kernel doesn't classify them.

### Defaults

The kernel ships no default packages. A distribution can bundle official packages, but the kernel binary itself, started without any manifests, runs an empty platform: it accepts sessions, accepts events, but has no capabilities registered and no semantics.

## Stability promise

This document changes by explicit revision. New responsibilities have to argue why they can't live in a package. The default answer is "package, not kernel."
