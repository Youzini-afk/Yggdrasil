# Yggdrasil

> [English](./README.md) · [中文](./README.zh-CN.md)

Yggdrasil is an extension-driven creation platform for AI-native worlds, games, stories, and play.

It is a kernel and a contract — small, stable, opinion-free at the center — over which an open ecosystem of capability packages provides every meaningful concept.

## Why this exists

Most AI-native creative tools today divide their users into players who consume a finished experience and developers who build it. Yggdrasil refuses that split. A player can inspect the session, ask an assistant to change it, fork it, replace a package, and ship the result back. A creator works against the same public protocol, with the same packages, in the same surfaces. The substrate is the same in both directions.

That stance is what the kernel, the public protocol, the official packages, and the web shell collectively serve. See [`docs/product/PLAY_CREATION_MODEL.md`](docs/product/PLAY_CREATION_MODEL.md) for the full product stance.

## Center of gravity

- The kernel hosts capability packages and nothing else.
- Capability packages provide every meaningful concept (characters, prompts, models, agents, worlds, rules, memory, anything).
- Official packages have no privileges. Same manifest, same fabric, same permission gate.
- Creators are free to compose, replace, or write their own packages.

The platform's job is to make radical AI-native creation possible without privileging an "official path."

## Status

**Platform Foundation Alpha + Play/Forge Surface Contract Beta.**

The current foundation includes a content-free kernel, manifest-driven packages, real `rust_inproc` and subprocess execution, hook fabric, SQLite event log, principals and scoped permissions, surface contributions, generic proposal/approval lifecycle, asset/branch/projection substrate, official foundation packages, an assistant-as-package, a blank play-creation loop, and a public-protocol web shell with Home/Play and Forge surfaces. 51 named conformance cases plus crate and service unit tests cover the boundary.

For the executable snapshot, see [`docs/ALPHA_STATUS.md`](docs/ALPHA_STATUS.md).
For where this is going, see [`docs/roadmap/NEXT_STEPS.md`](docs/roadmap/NEXT_STEPS.md).

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
```

## Read first

- [`docs/CHARTER.md`](docs/CHARTER.md) — permanent principles.
- [`docs/architecture/VISION.md`](docs/architecture/VISION.md) — what the platform is for.
- [`docs/architecture/ARCHITECTURE.md`](docs/architecture/ARCHITECTURE.md) — kernel-and-packages layering.
- [`docs/architecture/PLATFORM_KERNEL.md`](docs/architecture/PLATFORM_KERNEL.md) — what the kernel does and does not do.
- [`docs/architecture/CAPABILITY_PACKAGE.md`](docs/architecture/CAPABILITY_PACKAGE.md) — package contract.
- [`docs/architecture/EXTENSION_POINTS.md`](docs/architecture/EXTENSION_POINTS.md) — hook contract.
- [`docs/architecture/EVENT_MODEL.md`](docs/architecture/EVENT_MODEL.md) — opaque event log model.
- [`docs/architecture/RUNTIME_LIFECYCLE.md`](docs/architecture/RUNTIME_LIFECYCLE.md) — kernel-side lifecycles.
- [`docs/protocol/PROTOCOL_V0.md`](docs/protocol/PROTOCOL_V0.md) — public protocol.
- [`docs/spec/KERNEL_V0_ALPHA_CONTRACT.md`](docs/spec/KERNEL_V0_ALPHA_CONTRACT.md) — executable alpha contract matrix.
- [`docs/spec/CONFORMANCE_MATRIX.md`](docs/spec/CONFORMANCE_MATRIX.md) — hostile conformance roadmap.
- [`docs/product/PLAY_CREATION_MODEL.md`](docs/product/PLAY_CREATION_MODEL.md) — play-creation product stance.
- [`docs/ALPHA_STATUS.md`](docs/ALPHA_STATUS.md) — living snapshot of what is done, partial, and deferred.
- [`docs/roadmap/NEXT_STEPS.md`](docs/roadmap/NEXT_STEPS.md) — current and upcoming phases.
- [`docs/roadmap/PLATFORM_HOST_ALPHA.md`](docs/roadmap/PLATFORM_HOST_ALPHA.md) — Host Alpha + Play/Forge Surface Beta result.

## Deferred

These are valuable directions but not part of the kernel. They will arrive as ordinary capability packages.

- SillyTavern compatibility — see [`docs/tavern/TAVERN_COMPAT.md`](docs/tavern/TAVERN_COMPAT.md).
- pi integration — see [`docs/architecture/PI_INTEGRATION.md`](docs/architecture/PI_INTEGRATION.md).
- External game engines (UE5, Godot, Unity, web clients) — future packages or remote-entry integrations.
- Conversational runtime, model providers, memory model, agent loop, world simulation, director.
- Final UI visual design, full Studio, ComfyUI-like node editors, marketplace.

## License

Yggdrasil is licensed under the GNU Affero General Public License v3.0 (AGPLv3). See [`LICENSE`](LICENSE).
