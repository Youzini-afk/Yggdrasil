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

## Status

The docs are being aligned with the platform-as-framework direction first. The current Rust workspace contains conversational concepts inside what should be the kernel; that is a known deviation, and the refactor follows the docs.

See `docs/roadmap/NEXT_STEPS.md`.
