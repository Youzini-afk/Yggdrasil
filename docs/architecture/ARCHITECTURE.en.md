# Architecture

> [English](./ARCHITECTURE.en.md) · [中文](./ARCHITECTURE.md)

Yggdrasil has two layers: a kernel that hosts capability packages, and the packages themselves. The kernel is small and content-free; everything meaningful lives in packages.

```text
┌─────────────────────────────────────────────────────────────────┐
│ Capability packages (every meaningful concept lives here)        │
│                                                                  │
│   official packages          third-party packages                │
│   ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────┐ │
│   │ conversation │ │ tavern compat│ │ world sim    │ │  ...   │ │
│   │ runtime      │ │ (future)     │ │ (community)  │ │        │ │
│   └──────────────┘ └──────────────┘ └──────────────┘ └────────┘ │
│   ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────┐ │
│   │ memory pack  │ │ agent pack   │ │ inspector ui │ │  ...   │ │
│   └──────────────┘ └──────────────┘ └──────────────┘ └────────┘ │
│                                                                  │
│   no privilege difference between official and third-party       │
└─────────────────────────────────────────────────────────────────┘
                          ▲    same contract    ▲
                          │                     │
┌─────────────────────────────────────────────────────────────────┐
│ Yggdrasil kernel (content-free)                                  │
│                                                                  │
│   sessions      events       packages       capabilities         │
│   permissions   sandbox      hooks          assets               │
│                                                                  │
│   schemas, ids, ordering, replay, transports                     │
└─────────────────────────────────────────────────────────────────┘
                          ▲    public protocol    ▲
                          │                       │
┌─────────────────────────────────────────────────────────────────┐
│ Transports                                                       │
│   in-process • stdio JSON-RPC • TCP JSON-RPC • HTTP • WebSocket  │
│   (WASM host • remote endpoint)                                  │
└─────────────────────────────────────────────────────────────────┘
```

## The two layers

### The kernel

The kernel hosts capability packages and nothing else. The full responsibility list is in [`PLATFORM_KERNEL.md`](PLATFORM_KERNEL.en.md). Briefly: identity, sessions, an opaque event log, the package registry, capability routing, extension dispatch, permissions, and transports.

### Capability packages

Capability packages provide every meaningful concept on the platform: characters, prompts, models, agents, worlds, rules, memory, presentation, and so on. See [`CAPABILITY_PACKAGE.md`](CAPABILITY_PACKAGE.en.md).

A package can be a Rust in-process crate, a subprocess, a WASM module, or a remote service. The kernel treats all four the same.

## Boundary rules

These aren't preferences; they're invariants.

### 1. The kernel knows nothing about content

Characters, scenes, worlds, prompts, models, turns, chats, agents, memory, games, rules, dice, inventories, genres — none of these live in the kernel. If a concept means something to a player or a creator, it belongs in a package.

### 2. Official packages have no privileges

Anything an official package can do, a third-party can do too. Same manifest, same fabric, same hooks, same permission gate. No kernel shortcuts based on package id.

### 3. Protocol first

The kernel exposes one public contract. Studio, the CLI, in-process packages, subprocess packages, WASM packages, and remote services all use it. No private side door.

### 4. Entry forms are equal

A package can be `rust_inproc`, `subprocess`, `wasm`, or `remote`. Packaging form is an implementation detail; the fabric treats them all the same.

### 5. Events are truth, but opaque to the kernel

The kernel orders and persists events. It doesn't interpret payloads — meaning belongs to packages.

### 6. Sandboxing is declarative

Side effects, network reach, filesystem reach, cross-package calls — all declared in the manifest. The kernel enforces them. An undeclared side effect is a violation.

### 7. Composition over containment

Multiple packages can coexist in one session. There is no canonical "main experience." Conflicts are resolved by host-configured priority, not kernel defaults.

## What's not on this picture

Tavern isn't a kernel layer. It will arrive as a future capability package family.

pi isn't a kernel layer. It will ship as capability packages.

Studio isn't a kernel layer. It's a client of the public protocol, like any other; it may eventually ship as an official package plus a UI shell.

External game engines aren't a kernel layer. They join as remote-entry packages or as protocol clients.

## Repository map

The Yggdrasil Foundation Alpha workspace:

```text
crates/ygg-core      Kernel types: ids, schemas, manifests, principals, opaque events
crates/ygg-runtime   Kernel scheduler: sessions, packages, capabilities, hooks, surfaces,
                     proposals, assets, branches, projections, sandbox, transports
crates/ygg-service   Public protocol surface (HTTP /rpc, SSE event subscribe)
crates/ygg-cli       Host modes, manifest tools, package authoring, conformance
clients/web          Public-protocol Home/Play, Forge, and Assist shell
packages/official    Foundation capability packages loaded through ordinary manifests
sdk/typescript       Subprocess-package authoring helpers and template runtime
profiles/            Host profiles for autoloading sets of packages
examples/            Example package manifests and fixtures
```

The kernel crate is content-free. Conversation, worlds, agents, memory, and model behavior — when they arrive — come as ordinary capability packages with no kernel privilege.

## Where to read next

- [`CHARTER.md`](../CHARTER.en.md) for principles.
- [`PLATFORM_KERNEL.md`](PLATFORM_KERNEL.en.md) for what the kernel does and doesn't do.
- [`CAPABILITY_PACKAGE.md`](CAPABILITY_PACKAGE.en.md) for the package contract.
- [`EXTENSION_POINTS.md`](EXTENSION_POINTS.en.md) for the hook contract.
- [`EVENT_MODEL.md`](EVENT_MODEL.en.md) for the opaque event log.
- [`RUNTIME_LIFECYCLE.md`](RUNTIME_LIFECYCLE.en.md) for kernel-side lifecycles.
- [`../protocol/PROTOCOL_V0.md`](../protocol/PROTOCOL_V0.en.md) for the public protocol.
