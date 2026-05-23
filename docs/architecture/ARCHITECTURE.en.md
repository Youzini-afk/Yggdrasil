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

Characters, scenes, worlds, prompts, models, turns, chats, agents, memory, games, rules, dice, inventories, genres — none of these live in the kernel.v1. If a concept means something to a player or a creator, it belongs in a package.

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

## Contract v1 boundary

The public platform spec is [`../spec/KERNEL_V1_CONTRACT.md`](../spec/KERNEL_V1_CONTRACT.en.md). v1 schemas live under `../spec/v1/schemas/`; method, event, and top-level schemas are the single source of truth for SDK generation, the conformance kit, and third-party implementations.

### Capability handles

Manifest strings declare an authority ceiling; runtime capability handles represent actual authority. The kernel mints handles during package load / handshake / init, and can attenuate, revoke, and expire them. Capability calls, event access, outbound requests, and secret resolution should use handles or equivalent runtime bindings rather than package names or bare strings.

See [`../guides/CAPABILITY_HANDLES.md`](../guides/CAPABILITY_HANDLES.en.md).

### Binding injection

Path A packages (`entry.contract: "v1"`) receive bindings at startup. Subprocess packages receive a bindings dictionary during `package.handshake`; Rust in-process packages initialize through `KernelEnv`; WASM and remote will be completed in Round 10 through WIT resource imports and SPIFFE/Biscuit token exchange. Bindings contain only the package's least granted authority.

### Path B

Path B packages (`entry.contract: "none"`) run self-contained. The kernel still hosts lifecycle, captures logs, and emits events, but does not inject v1 handles, enforce manifest permissions, or turn manifest declarations into platform authority. Path A and Path B are both first-class modes; packages needing capability invoke, network, secrets, or declared-vs-used audit should use Path A.

See [`../guides/PATH_B_SELF_CONTAINED.md`](../guides/PATH_B_SELF_CONTAINED.en.md).

## What's not on this picture

Tavern isn't a kernel layer. It will arrive as a future capability package family.

pi isn't a kernel layer. It will ship as capability packages.

Studio isn't a kernel layer. It's a client of the public protocol, like any other; it may eventually ship as an official package plus a UI shell.

External game engines aren't a kernel layer. They join as remote-entry packages or as protocol clients.

## Client shell and release boundary

### Web client architecture

`clients/web` is a plain TypeScript SPA. Vite provides the dev server, type-check/build flow, and production bundling. The shell does not make React or another frontend framework part of its architecture; Home / Play, Forge, and Assist are public-protocol clients.

The web shell talks to the host only through public transports: HTTP `POST /rpc` for capability and kernel-method calls, plus SSE for event subscriptions. It does not read SQLite, import runtime crates, or use private shortcuts for official packages.

### SurfaceHost

Third-party web surface bundles are mounted through an iframe-based SurfaceHost. The host creates a `sandbox="allow-scripts"` iframe, loads `surface-frame.html`, and sends a mount instruction by `postMessage`. A surface sends `{type: 'rpc.call'}` to the host, and the host returns `{type: 'rpc.result'}` according to the explicit bridge configuration.

By default there is no kernel access; the host must explicitly wire `hostBridge.callRpc`. For the surface bundle contract, iframe CSP, YdlTavern example, and v0 limits, see [`../guides/SURFACE_HOSTING.md`](../guides/SURFACE_HOSTING.en.md).

### Desktop wrapper

`clients/desktop` is a Tauri 2.x wrapper. Production builds embed `clients/web/dist`; development points at the Vite dev server. It is a desktop container for the web shell, not a second protocol or private Studio.

v0 boundary: the desktop wrapper does not spawn `ygg-cli host serve`; users run the host separately. Managed subprocess support can be added later, but should preserve the public-protocol boundary. Build requirements are in [`../../BUILDING.md`](../../BUILDING.md).

### Release pipeline

Releases are triggered by `v*` tags in GitHub Actions. The pipeline builds the web shell, builds cross-platform Tauri installers, and creates a draft GitHub release. `scripts/release-version.sh` synchronizes the version across Cargo, the web package, the desktop package, and Tauri config.

The current release pipeline does not include signing, notarization, or auto-update. Build and release steps are in [`../../BUILDING.md`](../../BUILDING.md); release notes are in [`../../CHANGELOG.md`](../../CHANGELOG.md).

## Repository map

The Yggdrasil Foundation Alpha workspace:

```text
crates/ygg-core      Kernel types: ids, schemas, manifests, principals, opaque events
crates/ygg-runtime   Kernel scheduler: sessions, packages, capabilities, hooks, surfaces,
                     proposals, assets, branches, projections, sandbox, transports
crates/ygg-service   Public protocol surface (HTTP /rpc, SSE event subscribe)
crates/ygg-cli       Host modes, manifest tools, package authoring, conformance
clients/web          Vite + plain TS Home/Play, Forge, and Assist shell
clients/desktop      Tauri 2.x desktop wrapper
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
- [`../spec/KERNEL_V1_CONTRACT.md`](../spec/KERNEL_V1_CONTRACT.en.md) for the public v1 contract and schemas.
- [`../guides/CAPABILITY_HANDLES.md`](../guides/CAPABILITY_HANDLES.en.md) for capability handles and audit.
- [`../protocol/PROTOCOL_V0.md`](../protocol/PROTOCOL_V0.en.md) for the public protocol.
- [`../guides/SURFACE_HOSTING.md`](../guides/SURFACE_HOSTING.en.md) for third-party web surface hosting.
- [`../../BUILDING.md`](../../BUILDING.md) for web / desktop build and release steps.
- [`../../CHANGELOG.md`](../../CHANGELOG.md) for release notes.
