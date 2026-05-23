# Next steps

> [English](./NEXT_STEPS.en.md) · [中文](./NEXT_STEPS.md)

This document is about where Yggdrasil goes next. Completed phases live in [`ALPHA_STATUS.md`](../ALPHA_STATUS.en.md), not here.

## Where we are

The platform substrate is in place.

- The kernel is content-free. Official packages have no privileges. Entry forms are equal.
- The secure-execution layer is complete: `secret_ref`, `EnvSecretResolver`, network declarations, outbound audit and redaction, live HTTP/WebSocket outbound executors, the outbound trio of unary / SSE-NDJSON-raw stream / WebSocket, plus streaming and cancel lifecycle.
- Experience runtime, a real playable vertical slice, observability, memory, sharing / distribution — all shipped as ordinary capability packages.
- Multi-provider model integration, real outbound calls, a transport-neutral inference seam, and Agentic Forge Beta — all complete.
- External project operating plane, storage backend neutrality, PostgreSQL event backend, real TDB Rust adapter — all complete.
- Vite web-shell builds, iframe SurfaceHost, the Tauri 2.x desktop wrapper, and the tag-triggered cross-platform release pipeline — all complete.
- Round 9 Contract Foundation is complete: Contract V1, capability handles, binding injection, Path B, effect audit, the conformance kit, SDK generation, and 105 schemas are in place.
- 362 named conformance cases pass, plus crate / service unit tests.

The next stage isn't more substrate sprawl. Real AI-native playable experiences pull what comes next.

## Long-term direction: experience-led

See [`../product/EXPERIENCE_LED_PLATFORM_BETA.md`](../product/EXPERIENCE_LED_PLATFORM_BETA.en.md).

The shape:

- one or two real playable experiences become the pressure source that surfaces the remaining substrate work;
- every new piece of infrastructure has to answer "which real player or creator loop got stuck here";
- no more "Alpha + Beta + Phase" stacks planned in advance.

## Background work that doesn't block the main line

Not new phases, but known to-dos that will get done:

- **Round 10+: `official/git-tools-lab`.** Implement the `yg install <github-url>` flow as an ordinary official capability package using `kernel.v1.outbound.execute` (smart-http) plus `permissions.filesystem.write`, with sigstore/GPG signed-tag verification. This replaces the deleted kernel git fetch.
- Package-owned projection execution.
- `event.subscribe` permission for package principals, plus broader streaming-transport parity.
- Timeout and error audit for hook handlers.
- Persistent capability-provider selection policy.
- Broader transport-consistency coverage in conformance.
- Expanded real-model outbound conformance with local mock HTTP / WebSocket servers, without adding default public-internet dependencies.
- Real WebSocket smoke against OpenAI Realtime / Gemini Live, kept explicitly opt-in and out of default CI.
- More provider registries, tokenizer / billing metadata adapters, still as ordinary capability packages.
- Execution for WASM and remote package entries.
- Content-addressed blob storage and runtime-level asset permissions.
- Desktop release code signing / notarization.
- Desktop auto-updater integration.
- Replace placeholder desktop icons with real app icons.
- Surface lifecycle hooks (`onClose`, `onProposalDraft`, and related callbacks).
- Cross-origin surface-bundle allowlist, including CSP and origin checks.
- Desktop wrapper starts / stops `host serve` as a controlled managed subprocess.
- Phase B optimizations (next): use [`../../perf/baseline.json`](../../perf/baseline.json) as the regression reference and measure before changing behavior.

## Round 10: Contract Frontier

Round 10 pushes the v1 contract to farther boundaries without expanding kernel content semantics:

- WASM WIT worlds: map bindings into resource imports and complete wasm package execution.
- Remote packages: SPIFFE identity, Biscuit token exchange, remote package lifecycle, and audit.
- Powerbox: explicit user/host grants, handle delegation, temporary authority, and revocable delegation.
- Advanced authority patterns: cross-package delegation, attenuation-chain audit, lease refresh, and bulk revoke.
- Conformance kit library: extract package conformance as an embeddable library with project custom checks.
- SDK packaging: complete npm publishing, Rust crate publishing, and OpenAPI/codegen documentation.

Items remaining after Round 10 planning: package-owned projection execution, package-principal event.subscribe, hook timeout/error audit, persistent provider selection, broader transport parity, content-addressed blob storage, desktop signing/auto-update, surface lifecycle, and cross-origin allowlists.

These unblock specific scenarios, but none should be the center of the next stage.

## Integration projects (separate repos)

These run on top of Yggdrasil and consume the platform through the public protocol. They don't live in this repo.

- **YdlTavern** — an independent integration project on Yggdrasil, compatible with SillyTavern's character cards, world books, presets, chat history, and extension API, with the engine layer running on Yggdrasil. Repo: <https://github.com/Youzini-afk/Yggdrasil-Tavern>. For Yggdrasil's side of the boundary, see [`../tavern/TAVERN_COMPAT.md`](../tavern/TAVERN_COMPAT.en.md).

## Indefinitely deferred at the kernel level

These don't belong in the kernel.v1. They'll arrive as ordinary capability packages or future work.
- pi as a wholesale product shell embedding — see [`../architecture/PI_INTEGRATION.md`](../architecture/PI_INTEGRATION.en.md). Agent infrastructure can only move forward as ordinary packages and SDKs.
- External game-engine bridges (UE5, Godot, Unity, web clients).
- A privileged built-in Studio, a UI that bypasses the public protocol, or a kernel-owned official inspector. Public-protocol clients and package-contributed surfaces can keep evolving.
- Memory models, world simulation, directors, prompt rendering, and model-provider abstractions in the kernel.v1. Agent loops and production-grade model-provider capabilities stay in ordinary packages.
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
- Outbound WebSocket Substrate
- Shell + Release S-track (Vite web build, iframe SurfaceHost, Tauri desktop wrapper, GitHub Actions release)
- Round 9 Contract Foundation (Contract V1, capability handles, bindings, Path B, audit, conformance kit, SDK generation)
