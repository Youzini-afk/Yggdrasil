# Architecture

> [English](./ARCHITECTURE.en.md) · [中文](./ARCHITECTURE.md)

Yggdrasil has two architectural strata: a kernel that hosts capability packages, and the packages themselves. The kernel is small and content-free. Everything meaningful lives in packages.

```text
┌─────────────────────────────────────────────────────────────────┐
│ Capability Packages (every meaningful concept lives here)        │
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
│ Yggdrasil Kernel (content-free)                                  │
│                                                                  │
│   sessions      events       packages       capabilities         │
│   permissions   sandbox      hooks          assets               │
│                                                                  │
│   schemas, IDs, ordering, replay, transports                     │
└─────────────────────────────────────────────────────────────────┘
                          ▲    public protocol    ▲
                          │                       │
┌─────────────────────────────────────────────────────────────────┐
│ Transports                                                       │
│   in-process • stdio JSON-RPC • TCP JSON-RPC • HTTP • WebSocket  │
│   (WASM host • remote endpoint)                                  │
└─────────────────────────────────────────────────────────────────┘
```

## The two strata

### Kernel

The kernel hosts capability packages and nothing else. See `PLATFORM_KERNEL.md` for the exhaustive list of responsibilities. In short: identity, sessions, opaque event log, package registry, capability fabric, extension-point dispatch, permissions, transports.

### Capability packages

Capability packages provide every meaningful concept on the platform: characters, prompts, models, agents, worlds, rules, memory, presentation, anything. See `CAPABILITY_PACKAGE.md`.

Packages can be Rust in-process, subprocess, WASM, or remote. The kernel treats all four the same way.

## Boundary rules

These are not preferences. They are invariants.

### 1. The kernel knows nothing about content

No characters, scenes, worlds, prompts, models, turns, chats, agents, memories, games, rules, dice, inventories, or genres are part of the kernel. If a concept is meaningful to a creator or a player, it lives in a package.

### 2. Official packages have no privileges

Anything an official package can do, a third-party package can do. Same manifest, same fabric, same hooks, same permission gate. There is no kernel shortcut based on package id.

### 3. Protocol-first

The kernel exposes one public contract. Studio, CLI, in-process packages, subprocess packages, WASM packages, and remote services use the same contract. No private bypass.

### 4. Many entry forms, equal status

A package can be `rust_inproc`, `subprocess`, `wasm`, or `remote`. Packaging form is implementation detail. The fabric treats them identically.

### 5. Events are the truth, but opaque to the kernel

The kernel orders and persists events. It does not interpret payloads. Packages own meaning.

### 6. Sandbox by declaration

Side effects, network reach, filesystem reach, and cross-package calls are declared in manifest. The kernel enforces. Undeclared effects are violations.

### 7. Composition over containment

Multiple packages can coexist in a session. There is no canonical "main experience." Conflicts are resolved by host-configured precedence, not by kernel defaults.

## What is not in this picture

Tavern is not a kernel layer. It will be a future capability package family.

pi is not a kernel layer. It would ship as a capability package.

Studio is not a kernel layer. It is a client of the public protocol, just like any other client. It may ship as official packages plus a UI shell.

External game engines are not a kernel layer. They participate as remote-entry packages or as protocol clients.

## Repository map

The Yggdrasil Foundation Alpha workspace:

```text
crates/ygg-core      kernel types: ids, schemas, manifests, principals, opaque events
crates/ygg-runtime   kernel scheduler: sessions, packages, capabilities, hooks, surfaces,
                     proposals, assets, branches, projections, sandbox, transports
crates/ygg-service   public protocol surface (HTTP /rpc, SSE event subscribe)
crates/ygg-cli       host modes, manifest tools, package authoring, conformance
clients/web          public-protocol Home/Play, Forge, and Assist shell
packages/official    foundation capability packages loaded through ordinary manifests
sdk/typescript       subprocess-package authoring helpers and template runtime
profiles/            host profiles for autoloading sets of packages
examples/            example package manifests and fixtures
```

The kernel crates are content-free. Conversational, world, agent, memory, and model behavior — when added — arrives as ordinary capability packages and gets no kernel privilege.

## How to read the rest of the docs

- `CHARTER.md` for the principles.
- `PLATFORM_KERNEL.md` for what the kernel does and does not do.
- `CAPABILITY_PACKAGE.md` for the package contract.
- `EXTENSION_POINTS.md` for the hook contract.
- `EVENT_MODEL.md` for the opaque event log.
- `RUNTIME_LIFECYCLE.md` for kernel-side lifecycles.
- `protocol/PROTOCOL_V0.md` for the public protocol.
