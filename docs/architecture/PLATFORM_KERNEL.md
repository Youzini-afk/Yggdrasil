# Platform Kernel

The kernel is the minimum infrastructure that lets capability packages coexist on Yggdrasil. It is small, content-free, and stable.

This document fixes what the kernel does and does not do. Anything not listed as a kernel responsibility must live in a capability package.

## What the kernel does

### 1. Identity and schemas

- Generate IDs for sessions, events, packages, capability invocations, asset records.
- Maintain `schema_version` on every persisted contract object.
- Validate manifests, hook subscriptions, and capability registrations against published schemas.

### 2. Session shell

- Allocate and address sessions.
- Hold per-session metadata (id, created_at, label, status).
- Carry an event stream and a permission scope.
- The kernel does not interpret what a session is for. A session is a labeled event stream with an attached package set.

### 3. Append-only event log

- Accept events from authorized writers.
- Order them per session.
- Persist them durably.
- Replay them on demand.
- The kernel treats event payloads as opaque JSON. Meaning is owned by packages.

### 4. Package registry

- Load, validate, and start packages from manifest.
- Track package state (registered, loading, ready, degraded, stopped).
- Unload cleanly.
- Mediate lifetime: a session declares which packages are active in its scope.

### 5. Capability fabric

- Index capabilities by id and version.
- Route invocation calls and streams to providers.
- Record invocations in the event log when configured.
- Negotiate version constraints between consumer and provider.

### 6. Extension-point dispatch

- Maintain the registry of extension points.
- Hold subscriber lists.
- Dispatch hook calls in declared order with declared timing.
- Enforce timeout and cancellation.

### 7. Permission gate

- Read manifest-declared permissions for each package.
- Enforce them on event writes, capability invocations, cross-package calls, network/filesystem access.
- Refuse undeclared side effects.

### 8. Transport layer

- Speak HTTP, WebSocket, JSON-RPC over stdio, JSON-RPC over TCP, and a process-internal channel.
- Support remote endpoints.
- Surface a single conceptual protocol over all transports.

### 9. Sandbox boundaries

- Run in-process Rust packages within the kernel binary.
- Spawn and supervise subprocess packages.
- Host WASM packages with declared resource limits.
- Authenticate remote packages.

### 10. Public protocol

- The wire-level contract for the above. The kernel uses no private bypass; official packages and clients use the same protocol as third parties.

## What the kernel does not do

The kernel ships zero opinion on these. They are reserved for capability packages, including official ones.

### Conversation, prompts, and models

- No notion of turn, message, prompt frame, context plan, model call, sampling, or token usage.
- No prompt rendering, no template language, no system/user/assistant roles.
- No model provider abstraction, no streaming chunk format, no chat history.

### Worlds, characters, scenes, rules

- No world model, scene graph, or actor type.
- No character schema, no relationship state, no inventory, no clock.
- No rule engine, no condition/effect, no dice, no combat resolution.

### Memory

- No memory taxonomy, no embedding, no retrieval policy.
- No summary, no pin, no consolidation strategy.

### Agents and directors

- No agent loop, no planner, no director.
- No proposal-and-commit pattern other than what packages choose to define.

### Content sources

- No SillyTavern parser, no PNG metadata reader, no character card schema.
- No game engine bridge, no UE5/Godot/Unity glue.

### Presentation

- No UI, no chat panel, no inspector, no editor.
- No theme, no layout, no asset rendering.

### Storage opinion

- No business tables. The kernel needs storage for events, manifests, and asset records. It does not provide ORM, query builders, or data models for content.

## Gray zones

These need explicit positions to avoid drift.

### Assets

The kernel maintains an asset registry. It records `id`, `mime`, `hash`, `size`, `origin_package`, and a content blob. It does not parse, render, or interpret asset content. Packages own their formats.

### Event ordering

The kernel guarantees per-session monotonic ordering and durable persistence. It does not guarantee any cross-session ordering, causation graph, or correlation semantics. Causation/correlation fields are opaque metadata supplied by writers.

### Errors

Kernel errors cover: transport, permission, schema validation, manifest, capacity, package lifecycle. Package errors flow through capability invocations as opaque structured failures; the kernel does not classify them.

### Defaults

The kernel ships no default packages. A distribution may bundle official packages, but the kernel binary itself, when started with no manifests, runs an empty platform: it accepts sessions, accepts events, but no capability is registered and no semantics exist.

## Stability commitment

This document changes by explicit revision. New responsibilities require justification that they cannot live in a package. The default answer is "package, not kernel."
