# Yggdrasil

> [English](./README.en.md) · [中文](./README.md)

**An extensible creation platform for AI-native worlds, games, stories, and play.**

It has two parts: a small, restrained, opinion-free kernel, and an open ecosystem of capability packages. Every meaningful concept on the platform — characters, prompts, models, agents, worlds, rules, memory — comes from a package, not the kernel.

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

The platform substrate is in place. The next stage isn't more substrate sprawl — real playable experiences pull what comes next.

- 320 named conformance cases pass, plus crate / service unit tests.
- The kernel is content-free, official packages have no privileges, and the public protocol is the only entry.
- Secure execution, proposal approval, streaming lifecycle, model integration, and agent infrastructure are all in.

For details, see [`docs/ALPHA_STATUS.md`](docs/ALPHA_STATUS.en.md). For what's next, see [`docs/roadmap/NEXT_STEPS.md`](docs/roadmap/NEXT_STEPS.en.md).

## Repository layout

```text
crates/                Rust kernel and runtime
  ygg-core/              Kernel-only contracts and content-free types
  ygg-runtime/           Runtime host: sessions, events, packages, capabilities,
                         hooks, surfaces, proposals, assets, branches, projections
  ygg-service/           Public protocol surface (HTTP /rpc, SSE event subscribe)
  ygg-cli/               Host modes, manifest tools, package authoring, conformance

clients/web/           Public-protocol Home / Play, Forge, and Assist shell

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
- Real in-process and subprocess package execution, the hook fabric, the capability fabric.
- A principal model with scoped permissions, plus the proposal / approval lifecycle.

**Secure execution**

- `secret_ref` references, a host-owned environment-variable resolver with an allowlist.
- Network permission declarations, audit and redaction for outbound requests, public `kernel.outbound.execute`.
- A real HTTPS outbound executor (off by default, HTTPS only, redirect fail-closed).
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

- `subprocess` — subprocess-package scaffolding.
- `secure-execution`, `agentic-forge`, `ygg-agent-adapter`.
- `inference-capability`, `model-provider-adapter`, `experience-runtime`.
- `text-surface` — frontend text-surface helpers.

**Web shell**

- Home / Play, Forge, and Assist — three first-class surfaces, all over the public protocol.
- An optional frontend text engine (a fallback engine plus an optional Pretext loader).
- Forge text preview, plus agent / experience / storage / proposal observability panels.

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
| Use the public protocol | [`docs/protocol/PROTOCOL_V0.md`](docs/protocol/PROTOCOL_V0.en.md) → [`docs/spec/KERNEL_V0_ALPHA_CONTRACT.md`](docs/spec/KERNEL_V0_ALPHA_CONTRACT.en.md) |
| Write your first package | [`docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md`](docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.en.md) |
| Write agent / model / experience packages | [`docs/guides/AGENT_PACKAGE_AUTHORING.md`](docs/guides/AGENT_PACKAGE_AUTHORING.en.md), [`docs/guides/MODEL_PROVIDER_INTEGRATION.md`](docs/guides/MODEL_PROVIDER_INTEGRATION.en.md), [`docs/guides/EXPERIENCE_RUNTIME_AUTHORING.md`](docs/guides/EXPERIENCE_RUNTIME_AUTHORING.en.md) |
| See current status | [`docs/ALPHA_STATUS.md`](docs/ALPHA_STATUS.en.md) |
| See what's next | [`docs/roadmap/NEXT_STEPS.md`](docs/roadmap/NEXT_STEPS.en.md) |

## Deferred

These are valuable directions, but they don't belong in the kernel — they will arrive as ordinary capability packages:

- The SillyTavern successor project YdlTavern — a separate integration repo that absorbs SillyTavern users and community resources ([`docs/tavern/TAVERN_COMPAT.md`](docs/tavern/TAVERN_COMPAT.en.md)).
- Production-grade long-running autonomous agents, multi-agent collaboration, production memory systems, world simulation, directors.
- External game-engine integrations (UE5, Godot, Unity, web clients).
- A full Studio, ComfyUI-style node editors, a marketplace.
- Final UI visual design.

## License

Yggdrasil is licensed under the GNU Affero General Public License v3.0 (AGPLv3). See [`LICENSE`](LICENSE).
