# Next steps

> [English](./NEXT_STEPS.en.md) · [中文](./NEXT_STEPS.md)

This document is about where Yggdrasil goes next. Completed phases live in [`ALPHA_STATUS.md`](../ALPHA_STATUS.en.md), not here.

## Where we are

The platform substrate is in place.

- The kernel is content-free. Official packages have no privileges. Entry forms are equal.
- The secure-execution layer is complete: `secret_ref`, `EnvSecretResolver`, network declarations, outbound audit and redaction, `LiveHttpOutboundExecutor`, streaming and cancel lifecycle.
- Experience runtime, a real playable vertical slice, observability, memory, sharing / distribution — all shipped as ordinary capability packages.
- Multi-provider model integration, real outbound calls, a transport-neutral inference seam, and Agentic Forge Beta — all complete.
- External project operating plane, storage backend neutrality, PostgreSQL event backend, real TDB Rust adapter — all complete.
- 347 named conformance cases pass, plus crate / service unit tests.

The next stage isn't more substrate sprawl. Real AI-native playable experiences pull what comes next.

## Long-term direction: experience-led

See [`../product/EXPERIENCE_LED_PLATFORM_BETA.md`](../product/EXPERIENCE_LED_PLATFORM_BETA.en.md).

The shape:

- one or two real playable experiences become the pressure source that surfaces the remaining substrate work;
- every new piece of infrastructure has to answer "which real player or creator loop got stuck here";
- no more "Alpha + Beta + Phase" stacks planned in advance.

## Background work that doesn't block the main line

Not new phases, but known to-dos that will get done:

- **Automatic resolve / pin / apply for git package installation.** Controlled git fetch, `kernel.outbound.git_fetch`, `official/package-installer-lab`, the profile-scoped lockfile, and manual-pin CLI are in place; next is wiring `ygg package install <github-url>` into automatic commit/content-hash resolution, approval, lockfile write, and package load. Current capability: [`../guides/GIT_PACKAGE_INSTALLATION.md`](../guides/GIT_PACKAGE_INSTALLATION.en.md).
- Package-owned projection execution.
- `event.subscribe` permission for package principals, plus broader streaming-transport parity.
- Timeout and error audit for hook handlers.
- Persistent capability-provider selection policy.
- Broader transport-consistency coverage in conformance.
- Expanded real-model outbound conformance with local mock HTTP servers, without adding default public-internet dependencies.
- More provider tokenizer / billing metadata adapters, still as ordinary capability packages.
- Execution for WASM and remote package entries.
- Content-addressed blob storage and runtime-level asset permissions.

These unblock specific scenarios, but none should be the center of the next stage.

## Integration projects (separate repos)

These run on top of Yggdrasil and consume the platform through the public protocol. They don't live in this repo.

- **YdlTavern** — an independent integration project on Yggdrasil, compatible with SillyTavern's character cards, world books, presets, chat history, and extension API, with the engine layer running on Yggdrasil. Repo: <https://github.com/Youzini-afk/Yggdrasil-Tavern>. For Yggdrasil's side of the boundary, see [`../tavern/TAVERN_COMPAT.md`](../tavern/TAVERN_COMPAT.en.md).

## Indefinitely deferred at the kernel level

These don't belong in the kernel. They'll arrive as ordinary capability packages or future work.
- pi as a wholesale product shell embedding — see [`../architecture/PI_INTEGRATION.md`](../architecture/PI_INTEGRATION.en.md). Agent infrastructure can only move forward as ordinary packages and SDKs.
- External game-engine bridges (UE5, Godot, Unity, web clients).
- A privileged built-in Studio, a UI that bypasses the public protocol, or a kernel-owned official inspector. Public-protocol clients and package-contributed surfaces can keep evolving.
- Memory models, world simulation, directors, prompt rendering, and model-provider abstractions in the kernel. Agent loops and production-grade model-provider capabilities stay in ordinary packages.
- Marketplace, package signing networks, dependency-resolver economy. The local sharing proof is done — see [`../guides/SHARING_DISTRIBUTION.md`](../guides/SHARING_DISTRIBUTION.en.md).

## Scoring

Every new phase is graded against charter discipline:

- no content-shaped concept leaking into the kernel;
- no privilege for official packages along any path;
- all package and UI behavior on the public-protocol boundary;
- new substrate has to answer real-experience pressure.

## Completed phases at a glance

In rough order. Each one has support in `ALPHA_STATUS` and conformance. For details, see [`../ALPHA_STATUS.md`](../ALPHA_STATUS.en.md).

- Platform Foundation Alpha
- Play / Forge Surface Contract Beta
- Code Health Split Alpha
- Authoring & Composition Beta+
- Secure Execution Substrate Alpha
- Optional Text Engine Alpha
- Agent Infrastructure Alpha
- Model Provider Integration Alpha
- Live Model Calls Alpha
- Creative Inference Capability Alpha
- Agentic Forge Beta
- Experience Beta 0–6 (thin runtime → playable slice → state/asset pipeline → observability → memory/knowledge → creator loop → sharing/distribution)
- Performance & Code Health Beta
- External Project Operating Plane Alpha
- Storage Backend Neutrality Alpha
- PostgreSQL + TDB Integration Alpha
- Real TDB Rust Adapter Alpha
- Git Package Installation Substrate
