# Next steps

> [English](./NEXT_STEPS.en.md) · [中文](./NEXT_STEPS.md)

This document is about where Yggdrasil goes next. Completed phases live in [`ALPHA_STATUS.md`](../ALPHA_STATUS.en.md), not here.

## Where we are

The platform substrate is in place.

- The kernel is content-free. Official packages have no privileges. Entry forms are equal.
- The secure-execution layer is complete: `secret_ref`, `EnvSecretResolver`, `StoreSecretResolver`, local encrypted secret store, network declarations, outbound audit and redaction, live HTTP/WebSocket outbound executors, the outbound trio of unary / SSE-NDJSON-raw stream / WebSocket, plus streaming and cancel lifecycle.
- Experience runtime, a real playable vertical slice, observability, memory, sharing / distribution — all shipped as ordinary capability packages.
- Multi-provider model integration, real outbound calls, a transport-neutral inference seam, and Agentic Forge Beta — all complete.
- External project operating plane, storage backend neutrality, PostgreSQL event backend, real TDB Rust adapter — all complete.
- Vite web-shell builds, iframe SurfaceHost, the Tauri 2.x desktop wrapper, and the tag-triggered cross-platform release pipeline — all complete.
- Round 9 Contract Foundation is complete: Contract V1, capability handles, binding injection, Path B, effect audit, the conformance kit, and SDK generation are in place; after Round 10A.3 there are 115 schemas.
- 427 named conformance cases pass, plus crate / service unit tests.

The next stage isn't more substrate sprawl. Real AI-native playable experiences pull what comes next.

## Long-term direction: experience-led

See [`../product/EXPERIENCE_LED_PLATFORM_BETA.md`](../product/EXPERIENCE_LED_PLATFORM_BETA.en.md).

The shape:

- one or two real playable experiences become the pressure source that surfaces the remaining substrate work;
- every new piece of infrastructure has to answer "which real player or creator loop got stuck here";
- no more "Alpha + Beta + Phase" stacks planned in advance.

## Background work that doesn't block the main line

Not new phases, but known to-dos that will get done:

- The package-installation foundation is complete; Round 10A.1 also completed default simplification and the local encrypted secret store; Round 10A.2 completed the Home project shelf, project lifecycle, project-level secret fallback, and YdlTavern project.yaml. Round 10A.3 completed the real path from YdlTavern Send → engine `model.live_call` → live outbound → provider response. Round 10A.4 completed surface streaming response UX. Only distribution polish remains, such as Sigstore, Tauri UI, `yg gc`, and an auto-update daemon.
- OS keyring integration is deferred until CI / cross-platform builds have stable system dependencies.
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

## Round 10A — Package Installation Foundation (complete)

- `yg install <github-url>` end to end.
- `official/git-tools-lab` + `integrity-lab` + `install-lab` capability packages.
- `manifest.requires` field + Lockfile (`yggdrasil.lock.v1`).
- `~/.yggdrasil` filesystem convention.
- Interactive consent prompts + static conformance integration.
- Round 10A.1 follow-up: defaults relaxed to the cargo/npm/pip technical baseline; `--require-signed` / `--strict` are opt-in; added `official/secret-store-lab`, `StoreSecretResolver`, and YdlTavern API Connections encrypted saving.

## Round 10A.1 — Install Simplification + Secret Store (complete)

- `yg install <url>` no longer requires signatures by default, and conformance failures are warning-only by default.
- `--require-signed` and `--strict` provide controlled-environment opt-ins.
- `official/secret-store-lab` provides an age-encrypted local secret store.
- `StoreSecretResolver` and `CompositeSecretResolver` support `secret_ref:store:*` plus `secret_ref:env:*`.
- YdlTavern API Connections is wired for paste + save → encrypted store.
- OS keyring and `yg secret put / list / delete` CLI are deferred.


## Round 10A.2 — Steam-Game Project Concept (complete)

- Projects are first-class runtime concepts: `ProjectDescriptor`, `ProjectRegistry`, `ProjectType`, and `SecretPolicy`.
- `~/.yggdrasil/projects/<id>/`, project-level secret stores, `secret_ref:project:*`, and platform fallback are implemented.
- Install detection distinguishes native `project.yaml` from the external-project wizard (wrap / workspace).
- `yg project list/info/status/start/stop` and `yg uninstall` archival prompts are implemented.
- `kernel.v1.project.list/get/start/stop/status` and project lifecycle events are implemented.
- Home is now a project shelf; YdlTavern declares itself as a `yggdrasil_native` project.
- Multi-tenant-grade `ProtocolContext.project_id` / session-based project scoping hardening is deferred to Round 11+.

## Round 10A.3 — End-to-End Real Path (complete)

- Surface bundle resolution is metadata-driven, with the new `kernel.v1.surface.resolve_bundle` method.
- `project.start` opens a project session, writes `metadata.project_id`, and returns `session_id` / `already_running`.
- `project.get` / `status` return `running_session_id` while Running, and `project.stop` closes the project session.
- `clients/web` injects `sessionId` / `projectId` into surface initialProps, and surface RPCs automatically carry `session_id`.
- YdlTavern `SendForm` is wired to engine `model.live_call`; API Connections supports platform/project save scope; the engine manifest declares `secret_ref:project:*`.
- Documentation convergence is in [`../guides/REAL_MODEL_END_TO_END.md`](../guides/REAL_MODEL_END_TO_END.en.md).

## Round 10A.4 — Streaming UX (complete)

- The surface-host stream postMessage protocol is implemented: `stream.subscribe` / `stream.frame` / `stream.ended` / `stream.error` / `stream.unsubscribe`.
- The host bridge subscribes to session SSE through `client.subscribeEvents`, filters `kernel/v1/stream.*` events by `stream_id`, and forwards matching frames.
- YdlTavern now has the `streamCapability` helper, the `TavernProvider.sendMessage` streaming branch, chunk-delta accumulation updates, and Stop/cancelGeneration.
- Multi-concurrent generation in one chat, token-rate UI, and Realtime/WebSocket streaming UX remain deferred; Round 10B remains the next focus.

## Round 10B — WIT/WASM Contract Frontier (next focus)

- WIT worlds + WASM entry form (move from scaffold toward partial).
- Powerbox late-bound provider selection.
- Cap'n Proto / Biscuit experiments.
- 10A.3 has landed; Round 10B keeps the existing Contract Frontier focus and does not add model/chat semantics to the kernel.

## Round 11 — Distribution polish (not started)

- `yg gc` orphaned-store cleanup.
- Tauri UI install path.
- Sigstore keyless verification.
- Auto-update daemon.
- Binary package distribution.
- Multi-tenant project scoping based on `ProtocolContext.session_id`: make project identity explicit in runtime permission, event, and resolver context.

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
- Package Installation Foundation (Round 10A)
- Install Simplification + Secret Store (Round 10A.1)
- Steam-Game Project Concept (Round 10A.2)
- Streaming UX (Round 10A.4)
- Outbound WebSocket Substrate
- Shell + Release S-track (Vite web build, iframe SurfaceHost, Tauri desktop wrapper, GitHub Actions release)
- Round 9 Contract Foundation (Contract V1, capability handles, bindings, Path B, audit, conformance kit, SDK generation)
