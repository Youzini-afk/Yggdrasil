# Yggdrasil

> [English](./README.en.md) · [中文](./README.md)

**An extensible creation platform for AI-native worlds, games, stories, and play.**

It has three tiers: a small, restrained, opinion-free kernel; an open ecosystem of capability packages; and projects on Home that can be installed, started, and stopped. Every meaningful concept on the platform — characters, prompts, models, agents, worlds, rules, memory — comes from a package, not the kernel; projects are host-runtime concepts.

```text
┌──────────────────────────────────────────────┐
│  Web shell · CLI · third-party clients         │   public protocol only
├──────────────────────────────────────────────┤
│  Public protocol  ·  /rpc + SSE              │
├──────────────────────────────────────────────┤
│  Projects (Home cards: YdlTavern / ...)       │   install/start/stop
├──────────────────────────────────────────────┤
│  Capability packages (official = third-party)  │   manifest-driven
├──────────────────────────────────────────────┤
│  Kernel: sessions · events · permissions · ... │   content-free
└──────────────────────────────────────────────┘
```

## Why this exists

Most AI-native creative tools today split their users in two: players who consume a finished experience, and developers who build it. **Yggdrasil refuses that split.**

A player can inspect the session, ask an assistant to change something, fork it, swap one capability package for another, and ship the result back. A creator works against the same public protocol, with the same packages, in the same surfaces. The substrate is the same in both directions.

For the full product stance, see [`docs/product/PLAY_CREATION_MODEL.md`](docs/product/PLAY_CREATION_MODEL.en.md).

## Center of gravity

- The kernel hosts capability packages and nothing else.
- Every meaningful concept comes from a capability package.
- Official packages have no privileges — same manifest, same fabric, same permission gate.
- Creators are free to compose, replace, or write their own packages.

The platform's job is to make radical AI-native creation possible — not to give any "official path" a head start.

## Status

The platform substrate is in place. Contract V1 is the public platform spec; see [`docs/spec/KERNEL_V1_CONTRACT.md`](docs/spec/KERNEL_V1_CONTRACT.en.md). The next stage isn't more substrate sprawl — real playable experiences pull what comes next.

- 427 named conformance cases pass, plus crate / service unit tests; 115 v1 schemas validate (63 methods + 45 events + 7 top-level).
- The kernel is content-free, official packages have no privileges, and the public protocol is the only entry.
- Secure execution, proposal approval, capability handles, binding injection, Path A / Path B, the conformance kit, generated SDKs, streaming lifecycle, model integration, and agent infrastructure are all in.
- Path A (`entry.contract: "v1"`) and Path B (`entry.contract: "none"`) are both first-class participation modes.
- SDKs are available through three channels: npm `@yggdrasil/kernel-sdk`, workspace path `file:../yggdrasil/sdk/typescript/kernel-sdk`, or direct generation from `docs/spec/v1/schemas/`.
- The web shell now uses Vite for dev/build; `clients/desktop/` provides a Tauri 2.x desktop wrapper, and `v*` tags build cross-platform installers through GitHub Actions.
- The perf baseline now records p50/p95/p99 + memory + env/git, supports `--compare` + `--threshold-pct`, and commits `perf/baseline.json`.
- `yg install <github-url>` installs capability packages or native projects from GitHub end to end with HTTPS-only fetches, content-addressed storage, optional GPG signature checks, optional strict conformance, and consent prompts.
- Encrypted local secret store through `official/secret-store-lab` — paste API keys in the UI, no env vars required.
- Home is now a project shelf: projects appear as cards and support `yg project list/info/status/start/stop` plus Play lifecycle.
- YdlTavern's real model end-to-end path is wired: SendForm → engine `model.live_call` → host live outbound → provider API → surface reply, with keys coming from the platform or project secret store.
- YdlTavern streaming response UX works: the chat UI updates chunk by chunk and supports Stop to cancel the active generation.

For details, see [`docs/ALPHA_STATUS.md`](docs/ALPHA_STATUS.en.md). For what's next, see [`docs/roadmap/NEXT_STEPS.md`](docs/roadmap/NEXT_STEPS.en.md).

## Repository layout

```text
crates/                Rust kernel and runtime
  ygg-core/              Kernel-only contracts and content-free types
  ygg-runtime/           Runtime host: sessions, events, packages, capabilities,
                         hooks, surfaces, proposals, assets, branches, projections
  ygg-service/           Public protocol surface (HTTP /rpc, SSE event subscribe)
  ygg-cli/               Host modes, manifest tools, package authoring, conformance

clients/web/           Vite + plain TS Home / Play, Forge, and Assist shell
clients/desktop/       Tauri 2.x desktop wrapper embedding the web shell

packages/official/     Foundation capability packages loaded through ordinary manifests
profiles/              Host profiles for autoloading sets of packages
examples/              Example package manifests and fixtures

sdk/typescript/        Subprocess-package authoring helpers and domain SDKs
docs/                  Architecture, protocol, spec, roadmap, product docs
integrations/          Upstream research notes (pi, TavernHeadless, pretext, TDB...)
```

## What's in the box

**Kernel and execution**

- Content-free sessions, opaque events, a durable SQLite event log, a rehydratable substrate.
- Three-tier model: the kernel provides protocol and scheduling, capability packages provide reusable abilities, and projects compose packages while holding runtime state.
- Real in-process and subprocess package execution, the hook fabric, the capability fabric.
- A principal model with scoped permissions, plus the proposal / approval lifecycle.

**Secure execution**

- `secret_ref:env:` / `secret_ref:store:` references, manifest `permissions.secret_refs` declarations, and host-owned environment-variable plus local encrypted store resolvers.
- `secret_ref:project:` project-level secrets are scoped through the Play session's `metadata.project_id`, with policy-based fallback to the platform store when allowed.
- Network permission declarations, audit and redaction for outbound requests, and the public-protocol outbound trio: unary `kernel.v1.outbound.execute`, SSE/NDJSON/raw `kernel.v1.outbound.stream`, and bidirectional `kernel.v1.outbound.websocket.*`.
- Real live HTTP / WebSocket outbound executors (off by default; require an opt-in profile plus provider env vars; HTTP is HTTPS-only, WebSocket is WSS-only, redirect fail-closed). Real WebSocket smoke also requires `YGG_LIVE_WEBSOCKET_TESTS=1`.
- The subprocess TypeScript SDK `kernelClient` lets subprocess packages issue permission-scoped reverse kernel calls, including `kernelClient.openWebSocket`.
- A generic streaming and cancel lifecycle.

**Official capability packages** (all ordinary packages, no kernel privilege)

- Platform foundation: composition / asset / projection.
- Creative tooling: persona / knowledge / context / text-transform.
- Model integration: model-connector / model-provider / model-routing (OpenAI, Anthropic, Gemini, OpenAI-compatible, OpenRouter, DeepSeek, xAI, Fireworks).
- Agent: pi-agent-runtime / capability-tool-bridge / agentic-forge.
- Experience: playable-creation-board, experience-runtime, experience-observability, memory, sharing, playable-seed.
- Inference: inference-local, inference-playtest.
- Storage and external projects: storage, tdb-retrieval, project-intake, workspace.
- Foundation labs: package, schema-tools, event-tools, assistant, blank-experience.

**TypeScript SDKs**

- `kernel-sdk` — generated public kernel SDK from v1 schemas, usable through npm, workspace path, or independent codegen.
- `subprocess` — subprocess-package scaffolding.
- `secure-execution`, `agentic-forge`, `ygg-agent-adapter`.
- `inference-capability`, `model-provider-adapter`, `experience-runtime`.
- `text-surface` — frontend text-surface helpers.

**Web shell**

- Home / Play, Forge, and Assist — three first-class surfaces, all over the public protocol.
- Plain TypeScript SPA with Vite dev/build/preview; no React or frontend framework in the shell.
- An iframe SurfaceHost can mount third-party surface bundles such as `@ydltavern/surface`, with an explicit postMessage bridge back to the host.
- An optional frontend text engine (a fallback engine plus an optional Pretext loader).
- Forge text preview, plus agent / experience / storage / proposal observability panels.

**Desktop and releases**

- `clients/desktop/` is a Tauri 2.x wrapper that embeds `clients/web/dist` in production.
- v0 does not spawn `ygg-cli host serve`; users still run the host separately.
- `v*` tags trigger the GitHub Actions release workflow and produce draft Linux / macOS / Windows installers. Signing, notarization, and auto-update are not enabled yet.

## Quick start

Run a host:

```bash
cargo run -p ygg-cli -- host serve \
  --http 127.0.0.1:8787 \
  --profile profiles/forge-alpha.yaml
```

Check or build the web shell:

```bash
npm run check --prefix clients/web
npm run build --prefix clients/web
```

Run the full conformance suite:

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
```

Install and manage capability packages:

```bash
yg install github.com/user/yggdrasil-package#v1.2.0
yg list-installed
yg project list
yg project start <project-id>
yg project stop <project-id>
yg uninstall <package-id-or-project-id>
yg update [<package-id>]
yg lockfile --check
```

Run the blank play-creation loop end to end via the public protocol:

```bash
cargo run -p ygg-cli -- play-create-demo
```

For more commands (manifest, package, composition, host modes, third-party authoring loop, templates), see [`docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md`](docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.en.md).

## Documentation

Every developer doc has both English and Simplified Chinese versions; the bilingual blockquote at the top of each file switches between them. [`docs/`](docs/README.en.md) is grouped by topic: architecture, protocol, spec, product, package authoring, performance, roadmap.

Shortest path by intent:

| If you want to | Read first |
|---|---|
| Understand the platform stance | [`docs/CHARTER.md`](docs/CHARTER.en.md) → [`docs/architecture/VISION.md`](docs/architecture/VISION.en.md) → [`docs/product/PLAY_CREATION_MODEL.md`](docs/product/PLAY_CREATION_MODEL.en.md) |
| Understand the architecture | [`docs/architecture/ARCHITECTURE.md`](docs/architecture/ARCHITECTURE.en.md) → [`docs/architecture/PLATFORM_KERNEL.md`](docs/architecture/PLATFORM_KERNEL.en.md) → [`docs/architecture/CAPABILITY_PACKAGE.md`](docs/architecture/CAPABILITY_PACKAGE.en.md) |
| Use the public protocol | [`docs/protocol/PROTOCOL_V0.md`](docs/protocol/PROTOCOL_V0.en.md) → [`docs/spec/KERNEL_V1_CONTRACT.md`](docs/spec/KERNEL_V1_CONTRACT.en.md) |
| Write your first package | [`docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md`](docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.en.md) |
| Install capability packages/projects | [`docs/guides/PACKAGE_INSTALLATION.md`](docs/guides/PACKAGE_INSTALLATION.en.md) → [`docs/guides/PROJECT_MODEL.md`](docs/guides/PROJECT_MODEL.en.md) |
| Manage API keys / secrets | [`docs/guides/SECRET_MANAGEMENT.md`](docs/guides/SECRET_MANAGEMENT.en.md) |
| Run real model calls end to end | [`docs/guides/REAL_MODEL_END_TO_END.md`](docs/guides/REAL_MODEL_END_TO_END.en.md) |
| Write agent / model / experience packages | [`docs/guides/AGENT_PACKAGE_AUTHORING.md`](docs/guides/AGENT_PACKAGE_AUTHORING.en.md), [`docs/guides/MODEL_PROVIDER_INTEGRATION.md`](docs/guides/MODEL_PROVIDER_INTEGRATION.en.md), [`docs/guides/EXPERIENCE_RUNTIME_AUTHORING.md`](docs/guides/EXPERIENCE_RUNTIME_AUTHORING.en.md) |
| Host third-party web surfaces | [`docs/guides/SURFACE_HOSTING.md`](docs/guides/SURFACE_HOSTING.en.md) |
| See current status | [`docs/ALPHA_STATUS.md`](docs/ALPHA_STATUS.en.md) |
| See what's next | [`docs/roadmap/NEXT_STEPS.md`](docs/roadmap/NEXT_STEPS.en.md) |

## Deferred

These are valuable directions, but they don't belong in the kernel — they will arrive as ordinary capability packages:

- YdlTavern — a separate integration project compatible with SillyTavern's resources and extensions, running on Yggdrasil ([`docs/tavern/TAVERN_COMPAT.md`](docs/tavern/TAVERN_COMPAT.en.md)).
- Production-grade long-running autonomous agents, multi-agent collaboration, production memory systems, world simulation, directors.
- External game-engine integrations (UE5, Godot, Unity, web clients).
- A full Studio, ComfyUI-style node editors, a marketplace.
- Final UI visual design.

## License

Yggdrasil is licensed under the GNU Affero General Public License v3.0 (AGPLv3). See [`LICENSE`](LICENSE).
