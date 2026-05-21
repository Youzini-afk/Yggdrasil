# Yggdrasil

> [English](./README.en.md) · [中文](./README.md)

**An extension-driven creation platform for AI-native worlds, games, stories, and play.**

A kernel and a contract — small, stable, opinion-free at the center. Above it, an open ecosystem of capability packages provides every meaningful concept on the platform: characters, prompts, models, agents, worlds, rules, memory — all in packages.

```text
┌──────────────────────────────────────────────┐
│  Web shell · CLI · third-party clients         │   public protocol only
├──────────────────────────────────────────────┤
│  Public protocol  ·  /rpc + SSE              │
├──────────────────────────────────────────────┤
│  Capability packages (official = third-party)  │   manifest-driven
├──────────────────────────────────────────────┤
│  Kernel: sessions · events · permissions · ... │   content-free
└──────────────────────────────────────────────┘
```

## Why this exists

Most AI-native creative tools today divide their users into two: players who consume a finished experience, and developers who build it. **Yggdrasil refuses that split.**

A player can inspect the session, ask an assistant to change it, fork it, replace a package, and ship the result back. A creator works against the same public protocol, with the same packages, in the same surfaces. The substrate is the same in both directions.

For the full product stance, see [`docs/product/PLAY_CREATION_MODEL.md`](docs/product/PLAY_CREATION_MODEL.en.md).

## Center of gravity

- The kernel hosts capability packages and nothing else.
- Capability packages provide every meaningful concept.
- Official packages have no privileges. Same manifest, same fabric, same permission gate.
- Creators are free to compose, replace, or write their own packages.

The platform's job is to make radical AI-native creation possible without privileging an "official path."

## Status

The platform foundation is in place. Yggdrasil is now entering Experience-Led Platform Beta, where real AI-native playable experiences pull the remaining substrate work.

- **320 named conformance cases** + crate / service unit tests, all passing.
- Completed: Platform Foundation Alpha, Play/Forge Surface Contract Beta, Secure Execution Substrate Alpha, Optional Text Engine Alpha, Agent Infrastructure Alpha, Model Provider Integration Alpha, Live Model Calls Alpha, Creative Inference Capability Alpha, Agentic Forge Beta, Experience-Led Platform Beta (Beta 0–6), Performance & Code Health Beta, External Project Operating Plane Alpha, Storage Backend Neutrality Alpha, PostgreSQL + TDB Integration Alpha, Real TDB Rust Adapter Alpha.

For the executable snapshot, see [`docs/ALPHA_STATUS.md`](docs/ALPHA_STATUS.en.md). For what's next, see [`docs/roadmap/NEXT_STEPS.md`](docs/roadmap/NEXT_STEPS.en.md).

## Repository layout

```text
crates/      Rust kernel and runtime
  ygg-core/      Kernel-only contracts and content-free types
  ygg-runtime/   Runtime host: events / packages / capabilities / hooks /
                 surfaces / proposals / assets / branches / projections
  ygg-service/   Public protocol surface (HTTP /rpc, SSE event subscribe)
  ygg-cli/       Host modes, manifest tools, package authoring, conformance

clients/web/   Public-protocol Home / Play, Forge, and Assist shell

packages/official/   Foundation capability packages loaded through ordinary manifests
profiles/            Host profiles for autoloading sets of packages
examples/            Example package manifests and fixtures

sdk/typescript/      Subprocess-package authoring helpers and domain SDKs
docs/                Architecture, protocol, spec, roadmap, product docs
integrations/        Upstream research ledgers (pi, TavernHeadless, pretext, TDB...)
```

## What's in the box

**Kernel and execution**

- Content-free sessions, opaque events, durable SQLite event log, rehydratable substrate
- Real `rust_inproc` and subprocess execution, hook fabric, capability fabric
- Principals with scoped permissions, proposal/approval lifecycle

**Secure execution**

- `secret_ref` references, `EnvSecretResolver` allowlists, host-owned resolution
- Network permission declarations, outbound audit/redaction, public `kernel.outbound.execute`
- `LiveHttpOutboundExecutor` (HTTPS-only, off by default, redirect fail-closed)
- Generic streaming/cancel/timeout lifecycle

**Official capability packages** (all loaded through ordinary manifests, no kernel privilege)

- Platform foundation: `composition-lab`, `asset-lab`, `projection-lab`
- Creative tooling: `persona-lab`, `knowledge-lab`, `context-lab`, `text-transform-lab`
- Model integration: `model-connector-lab`, `model-provider-lab`, `model-routing-lab` (OpenAI, Anthropic, Gemini, OpenAI-compatible, OpenRouter, DeepSeek, xAI, Fireworks)
- Agent infrastructure: `pi-agent-runtime-lab`, `capability-tool-bridge-lab`, `agentic-forge-lab`
- Experience: `playable-creation-board`, `experience-runtime-lab`, `experience-observability-lab`, `memory-lab`, `sharing-lab`, `playable-seed`
- Inference: `inference-local-lab`, `inference-playtest-lab`
- Storage / external projects: `storage-lab`, `tdb-retrieval-lab`, `project-intake-lab`, `workspace-lab`
- Foundation labs: `package-lab`, `schema-tools`, `event-tools`, `assistant-lab`, `blank-experience`

**SDKs (TypeScript)**

- `subprocess` subprocess-package scaffolding
- `secure-execution`, `agentic-forge`, `ygg-agent-adapter`
- `inference-capability`, `model-provider-adapter`, `experience-runtime`
- `text-surface` (frontend text-surface helpers)

**Web shell**

- Home / Play, Forge, Assist — three deep surfaces, all over public protocol
- Optional frontend text engine (fallback + optional Pretext via dynamic import)
- Forge text preview, agent / experience / storage / proposal observability panels

## Quick start

Run a host:

```bash
cargo run -p ygg-cli -- host serve \
  --http 127.0.0.1:8787 \
  --profile profiles/forge-alpha.yaml
```

Type-check the web shell:

```bash
tsc -p clients/web/tsconfig.json --noEmit
```

Run the full conformance suite:

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
```

Demo the blank play-creation loop end-to-end via public protocol calls:

```bash
cargo run -p ygg-cli -- play-create-demo
```

For more commands (manifest, package, composition, host modes, third-party authoring loop, templates), see [`docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md`](docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.en.md).

## Documentation

Every developer doc has both English and Simplified Chinese versions. The bilingual blockquote at the top of each file switches between them.

[`docs/`](docs/README.en.md) is grouped by topic: architecture, protocol, spec, product, package authoring, performance, roadmap.

Shortest path by intent:

| If you want to | Read first |
|---|---|
| Understand the platform stance | [`docs/CHARTER.md`](docs/CHARTER.en.md) → [`docs/architecture/VISION.md`](docs/architecture/VISION.en.md) → [`docs/product/PLAY_CREATION_MODEL.md`](docs/product/PLAY_CREATION_MODEL.en.md) |
| Understand the architecture | [`docs/architecture/ARCHITECTURE.md`](docs/architecture/ARCHITECTURE.en.md) → [`docs/architecture/PLATFORM_KERNEL.md`](docs/architecture/PLATFORM_KERNEL.en.md) → [`docs/architecture/CAPABILITY_PACKAGE.md`](docs/architecture/CAPABILITY_PACKAGE.en.md) |
| Use the public protocol | [`docs/protocol/PROTOCOL_V0.md`](docs/protocol/PROTOCOL_V0.en.md) → [`docs/spec/KERNEL_V0_ALPHA_CONTRACT.md`](docs/spec/KERNEL_V0_ALPHA_CONTRACT.en.md) |
| Write your first package | [`docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md`](docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.en.md) |
| Write agent / model / experience packages | [`docs/guides/AGENT_PACKAGE_AUTHORING.md`](docs/guides/AGENT_PACKAGE_AUTHORING.en.md), [`docs/guides/MODEL_PROVIDER_INTEGRATION.md`](docs/guides/MODEL_PROVIDER_INTEGRATION.en.md), [`docs/guides/EXPERIENCE_RUNTIME_AUTHORING.md`](docs/guides/EXPERIENCE_RUNTIME_AUTHORING.en.md) |
| See current status | [`docs/ALPHA_STATUS.md`](docs/ALPHA_STATUS.en.md) |
| See what's next | [`docs/roadmap/NEXT_STEPS.md`](docs/roadmap/NEXT_STEPS.en.md) |

## Deferred

These are valuable directions but not part of the kernel — they will arrive as ordinary capability packages:

- SillyTavern compatibility ([`docs/tavern/TAVERN_COMPAT.md`](docs/tavern/TAVERN_COMPAT.en.md))
- Production-grade long-running autonomous agents, multi-agent collaboration, production memory systems, world simulation, director
- External game-engine integrations (UE5, Godot, Unity, web clients)
- Full Studio, ComfyUI-like node editors, marketplace
- Final UI visual design

## License

Yggdrasil is licensed under the GNU Affero General Public License v3.0 (AGPLv3). See [`LICENSE`](LICENSE).
