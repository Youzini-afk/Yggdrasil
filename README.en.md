# Yggdrasil

> [English](./README.en.md) · [中文](./README.md)

Yggdrasil is an extension-driven creation platform for AI-native worlds, games, stories, and play.

It is a kernel and a contract — small, stable, opinion-free at the center — over which an open ecosystem of capability packages provides every meaningful concept.

## Why this exists

Most AI-native creative tools today divide their users into players who consume a finished experience and developers who build it. Yggdrasil refuses that split. A player can inspect the session, ask an assistant to change it, fork it, replace a package, and ship the result back. A creator works against the same public protocol, with the same packages, in the same surfaces. The substrate is the same in both directions.

That stance is what the kernel, the public protocol, the official packages, and the web shell collectively serve. See [`docs/product/PLAY_CREATION_MODEL.md`](docs/product/PLAY_CREATION_MODEL.en.md) for the full product stance.

## Center of gravity

- The kernel hosts capability packages and nothing else.
- Capability packages provide every meaningful concept (characters, prompts, models, agents, worlds, rules, memory, anything).
- Official packages have no privileges. Same manifest, same fabric, same permission gate.
- Creators are free to compose, replace, or write their own packages.

The platform's job is to make radical AI-native creation possible without privileging an "official path."

## Status

**Platform Foundation Alpha + Play/Forge Surface Contract Beta + Playable Experience Alpha seed + Creative Capability Kit Alpha + Model Connectivity Kit Alpha + Secure Execution Substrate Alpha + Optional Text Engine Alpha + Agent Infrastructure Alpha + Model Provider Integration Alpha.**

The current foundation includes a content-free kernel, manifest-driven packages, real `rust_inproc` and subprocess execution, hook fabric, SQLite event log, principals and scoped permissions, surface contributions, generic proposal/approval lifecycle, asset/branch/projection substrate, secure execution primitives (`secret_ref`, network declarations, outbound audit/redaction, outbound executor boundary, stream/cancel lifecycle), official platform packages (`composition-lab`, `asset-lab`, `projection-lab`), Creative Capability Kit packages (`persona-lab`, `knowledge-lab`, `context-lab`, `text-transform-lab`), Model Connectivity / Provider packages (`model-connector-lab`, `model-provider-lab`, `model-routing-lab`; covering OpenAI, Anthropic, Gemini, OpenAI-compatible, OpenRouter, DeepSeek, xAI, and Fireworks with no-network normalization, fake/local invoke, and stream normalization), an assistant-as-package, `official/playable-seed`, reference agent runtime package `official/pi-agent-runtime-lab`, capability tool bridge package `official/capability-tool-bridge-lab`, third-party agent runtime replacement proof (`thirdparty/agent-runtime`), a blank play-creation loop, a public-protocol web shell with Home/Play, Forge, and Assist surfaces, an optional frontend text engine substrate (fallback engine, optional Pretext engine, Forge text preview, `sdk/typescript/text-surface`), and an agent runtime package template (`--template agent-runtime`). 114 named conformance cases plus crate and service unit tests cover the boundary.

For the executable snapshot, see [`docs/ALPHA_STATUS.md`](docs/ALPHA_STATUS.en.md).
For where this is going, see [`docs/roadmap/NEXT_STEPS.md`](docs/roadmap/NEXT_STEPS.en.md).

## Repository layout

```text
crates/
  ygg-core/      Kernel-only contracts and content-free types.
  ygg-runtime/   Runtime host: events, packages, capabilities, hooks, surfaces,
                 proposals, assets, branches, projections, sandbox, transports.
  ygg-service/   Public protocol surface (HTTP /rpc, SSE event subscribe).
  ygg-cli/       Host modes, manifest tools, package authoring, conformance.
clients/
  web/           Public-protocol Home / Play, Forge, and Assist shell.
packages/
  official/      Foundation capability packages loaded through ordinary manifests.
sdk/
  typescript/    Subprocess-package authoring helpers and template runtime.
profiles/        Host profiles for autoloading sets of packages.
examples/        Example package manifests and fixtures.
docs/            Architecture, protocol, spec, roadmap, product, and tavern docs.
```

## Quick start

Run a host with the Forge profile, then open the web shell against it:

```bash
cargo run -p ygg-cli -- host serve \
  --http 127.0.0.1:8787 \
  --profile profiles/forge-alpha.yaml
```

In another terminal, type-check the web shell:

```bash
tsc -p clients/web/tsconfig.json --noEmit
```

Run the full conformance suite:

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
```

Demo the blank play-creation loop end-to-end through ordinary public-protocol calls:

```bash
cargo run -p ygg-cli -- play-create-demo
```

## Common commands

```bash
# manifests and packages
cargo run -p ygg-cli -- manifest validate examples/packages/echo-rust-inproc/manifest.yaml
cargo run -p ygg-cli -- package load    examples/packages/echo-rust-inproc/manifest.yaml
cargo run -p ygg-cli -- package check   examples/packages/echo-subprocess-python/manifest.yaml
cargo run -p ygg-cli -- package conformance examples/packages/echo-subprocess-python/manifest.yaml
cargo run -p ygg-cli -- capability invoke examples/packages/echo-rust-inproc/manifest.yaml \
  example/echo-rust-inproc/echo --input '{"hello":"world"}'

# package authoring
cargo run -p ygg-cli -- init-package /tmp/ygg-package        --id example/new-package        --entry subprocess --language python
cargo run -p ygg-cli -- init-package /tmp/ygg-ts-package     --id example/new-ts-package     --entry subprocess --language typescript
cargo run -p ygg-cli -- init-package /tmp/ygg-experience-pkg --id example/new-experience     --entry subprocess --language typescript-experience
cargo run -p ygg-cli -- init-composition /tmp/ygg-composition --id example/new-experience
cargo run -p ygg-cli -- composition check /tmp/ygg-composition/composition.yaml

# host modes
cargo run -p ygg-cli -- host serve --http 127.0.0.1:8787 --profile profiles/forge-alpha.yaml
cargo run -p ygg-cli -- host-stdio

# verification and demos
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- play-create-demo
cargo run -p ygg-cli -- demo
cargo run -p ygg-cli -- sqlite-demo /tmp/ygg.db
tsc -p clients/web/tsconfig.json --noEmit

# third-party authoring loop
cargo run -p ygg-cli -- init-package /tmp/ygg-package --id example/package --entry subprocess --language typescript --template full-surface
cargo run -p ygg-cli -- package check /tmp/ygg-package/manifest.yaml
cargo run -p ygg-cli -- package run-fixture /tmp/ygg-package/manifest.yaml
cargo run -p ygg-cli -- package reload /tmp/ygg-package/manifest.yaml
cargo run -p ygg-cli -- init-composition /tmp/ygg-composition --id example/package
cargo run -p ygg-cli -- composition check /tmp/ygg-composition/composition.yaml
```

## Read first

- [`docs/CHARTER.md`](docs/CHARTER.en.md) — permanent principles.
- [`docs/architecture/VISION.md`](docs/architecture/VISION.en.md) — what the platform is for.
- [`docs/architecture/ARCHITECTURE.md`](docs/architecture/ARCHITECTURE.en.md) — kernel-and-packages layering.
- [`docs/architecture/PLATFORM_KERNEL.md`](docs/architecture/PLATFORM_KERNEL.en.md) — what the kernel does and does not do.
- [`docs/architecture/CAPABILITY_PACKAGE.md`](docs/architecture/CAPABILITY_PACKAGE.en.md) — package contract.
- [`docs/architecture/EXTENSION_POINTS.md`](docs/architecture/EXTENSION_POINTS.en.md) — hook contract.
- [`docs/architecture/EVENT_MODEL.md`](docs/architecture/EVENT_MODEL.en.md) — opaque event log model.
- [`docs/architecture/RUNTIME_LIFECYCLE.md`](docs/architecture/RUNTIME_LIFECYCLE.en.md) — kernel-side lifecycles.
- [`docs/protocol/PROTOCOL_V0.md`](docs/protocol/PROTOCOL_V0.en.md) — public protocol.
- [`docs/spec/KERNEL_V0_ALPHA_CONTRACT.md`](docs/spec/KERNEL_V0_ALPHA_CONTRACT.en.md) — executable alpha contract matrix.
- [`docs/spec/CONFORMANCE_MATRIX.md`](docs/spec/CONFORMANCE_MATRIX.en.md) — hostile conformance roadmap.
- [`docs/product/PLAY_CREATION_MODEL.md`](docs/product/PLAY_CREATION_MODEL.en.md) — play-creation product stance.
- [`docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md`](docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.en.md) — third-party package authoring walkthrough.
- [`docs/guides/AGENT_PACKAGE_AUTHORING.md`](docs/guides/AGENT_PACKAGE_AUTHORING.en.md) — agent-like capability package authoring guide.
- [`docs/guides/CREATIVE_CAPABILITY_KIT.md`](docs/guides/CREATIVE_CAPABILITY_KIT.en.md) — Yggdrasil-native creative capability package kit.
- [`docs/guides/MODEL_CONNECTIVITY_KIT.md`](docs/guides/MODEL_CONNECTIVITY_KIT.en.md) — no-network model provider profile and route planning kit.
- [`docs/guides/MODEL_PROVIDER_INTEGRATION.md`](docs/guides/MODEL_PROVIDER_INTEGRATION.en.md) — multi-provider model integration guide.
- [`docs/ALPHA_STATUS.md`](docs/ALPHA_STATUS.en.md) — living snapshot of what is done, partial, and deferred.
- [`docs/roadmap/NEXT_STEPS.md`](docs/roadmap/NEXT_STEPS.en.md) — current and upcoming phases.
- [`sdk/typescript/model-provider-adapter/README.en.md`](sdk/typescript/model-provider-adapter/README.en.md) — Model Provider Adapter SDK (M1).

## Deferred

These are valuable directions but not part of the kernel. They will arrive as ordinary capability packages.

- SillyTavern compatibility — see [`docs/tavern/TAVERN_COMPAT.md`](docs/tavern/TAVERN_COMPAT.en.md).
- pi / agent package infrastructure — see [`docs/architecture/PI_INTEGRATION.md`](docs/architecture/PI_INTEGRATION.en.md) and [`docs/guides/AGENT_PACKAGE_AUTHORING.md`](docs/guides/AGENT_PACKAGE_AUTHORING.en.md). Real agent loops and memory systems remain future ordinary capability packages; model provider integration substrate is documented in [`docs/guides/MODEL_PROVIDER_INTEGRATION.md`](docs/guides/MODEL_PROVIDER_INTEGRATION.en.md).
- External game engines (UE5, Godot, Unity, web clients) — future packages or remote-entry integrations.
- Conversational runtime, production-grade live model calls, memory model, agent loop, world simulation, director.
- Final UI visual design, full Studio, ComfyUI-like node editors, marketplace.

## License

Yggdrasil is licensed under the GNU Affero General Public License v3.0 (AGPLv3). See [`LICENSE`](LICENSE).
