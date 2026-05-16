# Yggdrasil

Yggdrasil is an extension-driven creation platform for AI-native worlds, games, stories, and play.

It is a kernel and a contract — small, stable, opinion-free at the center — over which an open ecosystem of capability packages provides every meaningful concept.

## Center of gravity

- The kernel hosts capability packages and nothing else.
- Capability packages provide every meaningful concept (characters, prompts, models, agents, worlds, rules, memory, anything).
- Official packages have no privileges. Same manifest, same fabric, same permission gate.
- Creators are free to compose, replace, or write their own packages.

The platform's job is to make radical AI-native creation possible without privileging an "official path."

## Read first

- `docs/CHARTER.md` — permanent principles.
- `docs/architecture/VISION.md` — what the platform is for.
- `docs/architecture/ARCHITECTURE.md` — kernel-and-packages layering.
- `docs/architecture/PLATFORM_KERNEL.md` — what the kernel does and does not do.
- `docs/architecture/CAPABILITY_PACKAGE.md` — how a package describes itself and runs.
- `docs/architecture/EXTENSION_POINTS.md` — the hook contract.
- `docs/architecture/EVENT_MODEL.md` — opaque event log model.
- `docs/architecture/RUNTIME_LIFECYCLE.md` — kernel session/event/package lifecycles.
- `docs/protocol/PROTOCOL_V0.md` — the public protocol.
- `docs/spec/KERNEL_V0_ALPHA_CONTRACT.md` — executable alpha contract matrix.
- `docs/spec/CONFORMANCE_MATRIX.md` — hostile conformance roadmap.
- `docs/roadmap/PLATFORM_HOST_ALPHA.md` — current milestone for external package hosting.

## Deferred

- SillyTavern compatibility: a future capability package family. See `docs/tavern/TAVERN_COMPAT.md`.
- pi integration: would ship as a capability package. See `docs/architecture/PI_INTEGRATION.md`.
- External game engines: future packages or remote-entry integrations.

## Repository layout

```text
crates/
  ygg-core/      Will be repurposed as kernel-only types.
  ygg-runtime/   Will be repurposed as the scheduler.
  ygg-service/   Will speak the kernel public protocol.
  ygg-cli/       Will exercise kernel operations against a manifest set.
docs/
  CHARTER.md
  architecture/
  protocol/
  tavern/
  roadmap/
```

## Current kernel commands

```bash
cargo test --workspace
cargo run -p ygg-cli -- demo
cargo run -p ygg-cli -- sqlite-demo /tmp/ygg.db
cargo run -p ygg-cli -- manifest validate examples/packages/echo-rust-inproc/manifest.yaml
cargo run -p ygg-cli -- package load examples/packages/echo-rust-inproc/manifest.yaml
cargo run -p ygg-cli -- capability invoke examples/packages/echo-rust-inproc/manifest.yaml example/echo-rust-inproc/echo --input '{"hello":"world"}'
cargo run -p ygg-cli -- init-package /tmp/ygg-package --id example/new-package --entry rust_inproc
cargo run -p ygg-cli -- conformance
```

## Status

The Rust workspace now follows the platform-as-framework direction: kernel-only events/sessions, manifest-driven packages, capability fabric, hook registry, SQLite event store, permission audits, real `rust_inproc` execution, and conformance-oriented example packages. The current milestone is Platform Host Alpha: subprocess execution, public protocol transports, hook completion, package authoring harnesses, and hostile conformance. Content-shaped runtimes remain deferred packages.

See `docs/roadmap/NEXT_STEPS.md`.
