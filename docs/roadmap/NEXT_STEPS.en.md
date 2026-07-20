# Next steps

> [English](./NEXT_STEPS.en.md) · [中文](./NEXT_STEPS.md)

This document is about where Yggdrasil goes next. Completed state lives in [`../ALPHA_STATUS.md`](../ALPHA_STATUS.en.md), not here.

## Where we are

- The kernel is content-free. Official packages have no privileges. Public protocol is the only entry.
- The secure-execution layer is complete: `secret_ref`, local encrypted secret store, network declarations, outbound audit and redaction, HTTP/WebSocket outbound executors, streaming and cancel lifecycle.
- The platform substrate is complete: package installation, native project install/mount, profile autoload, installed project surface bundles, surface-bundle freshness safeguards, project update, Home project shelf, structured shell descriptors, standalone project tabs, project-console diagnostics, explicit Docker Deploy broker, target/exec/port/proxy deployment primitives, ygg-service HTTP/WebSocket reverse proxy, Settings, real model end-to-end, streaming UX, the constrained Surface bridge, desktop wrapper, release pipeline, web shell release closure, and the code-organization split.
- Multi-provider model integration, a transport-neutral inference seam, Agentic Forge, the external project operating plane, storage backend neutrality, the PostgreSQL event backend, and the real TDB Rust adapter — all in.
- Contract V1 is the public platform spec; all 147 schemas (80 methods + 59 events + 8 top-level) validate, and 451 conformance cases pass.
- The first three layering-migration steps are complete: v1 factual-drift repair, the Experimental Contract Registry with explicit negotiation, and 36 owner-namespace canonical/legacy dual stacks. The next step in [`CONTRACT_V2_MIGRATION.md`](CONTRACT_V2_MIGRATION.en.md) establishes the object/artifact foundation.

The next stage isn't more substrate sprawl. Real project deployment, human testing, and playable experiences pull what comes next.

> “Complete” here means current v1 operational closure, not that every `kernel.v1.*`
> boundary is permanent constitutional substrate. The long-term layering candidate is
> [`CONSTITUTION_V2.md`](../architecture/CONSTITUTION_V2.en.md), with item-level ownership and temporary implementation order in
> [`CONTRACT_LAYERING_MATRIX.md`](../spec/CONTRACT_LAYERING_MATRIX.en.md) and
> [`CONTRACT_V2_MIGRATION.md`](CONTRACT_V2_MIGRATION.en.md). The candidate changes no current status before explicit adoption.

## Long-term direction

The platform stance lives in [`../product/PLAY_CREATION_MODEL.md`](../product/PLAY_CREATION_MODEL.en.md).

The shape:

- one or two real playable experiences or deployed projects become the pressure source that surfaces the remaining substrate work;
- every new piece of infrastructure has to answer "which real user, player, creator, or deployment loop got stuck here";
- no more pre-planned multi-stage roadmaps stacked in advance.

## Scoring

Every new piece of work is graded against charter discipline:

- The kernel stays content-free — no conversation / model / prompt / memory / world / character / director semantics seep in.
- No path gives official packages a privilege.
- All package and UI behavior crosses the public-protocol boundary.
- New substrate has to answer a real playable experience's pressure.

## What's actively in flight

These are known to-dos. Priority follows real friction.

### Contract frontier

- WIT worlds + the WASM entry form from scaffold to partial: map bindings to resource imports and complete wasm package execution.
- Remote packages: SPIFFE identity, Biscuit token exchange, remote package lifecycle and audit.
- Powerbox: explicit user / host grants, handle delegation, temporary authority, revocable delegation.
- Cross-package delegation, attenuation-chain audit, lease refresh, bulk revoke.
- Extract the conformance kit as an embeddable library that supports project-defined checks.
- Round out SDK distribution: npm publish, Rust crate publish, OpenAPI / codegen documentation.

### Package system and runtime

- Package-owned projection execution.
- `event.subscribe` permission for package principals.
- Timeout and error audit for hook handlers.
- Persistent capability-provider selection policy beyond explicit per-call selection.
- Content-addressed blob storage and runtime-level asset permissions.
- Broader transport-consistency coverage in conformance.

### Project and multi-tenancy

- Multi-tenant project scoping based on `ProtocolContext.session_id`: make project identity explicit in runtime permission, event, and resolver context.
- Project archive auto-cleanup beyond 30 days.
- `yg secret put / list / delete` CLI.
- OS keyring integration (deferred until CI / cross-platform builds have stable system dependencies).
- Deployment auto-restart (separate phase): first persist "deploy intent" (image, etc.) in host-plane terms, then add bounded-retry + backoff self-healing without leaking Docker semantics into the kernel proxy / port records. Today's health supervision only monitors, flips readiness, and audits — it does not re-deploy.
- Deployment descriptor polish: Docker pull progress, long-term log archival, Build & Deploy job persistence, and external-project wizard generation.
- Remote targets and multi-client public exposure: ports currently bind to loopback only.

### Models and outbound

- Expand real-model outbound conformance with local mock HTTP / WebSocket servers, without adding default public-internet dependencies.
- Real WebSocket smoke against OpenAI Realtime / Gemini Live, kept explicitly opt-in.
- More provider registries, tokenizer / billing metadata adapters, still as ordinary capability packages.
- Multi-concurrent generation in one chat, token-rate UI, Realtime / WebSocket streaming UX.

### Install and release

- Update-flow follow-up is mostly polish: clearer failure recovery, external wrapped adapter updates, and more UI progress detail.
- Tauri UI install polish and release integration.
- Sigstore keyless verification.
- Auto-update daemon.
- Binary package distribution.
- Desktop release code signing / notarization.
- Replace placeholder desktop icons with real app icons.
- Desktop wrapper starts / stops `host serve` as a controlled managed subprocess.

### Web shell and surfaces

- Executable wiring for structured shell descriptors: package-contributed `quick_action` / `workshop_card` entries are discovery affordances today. If they become executable later, they must go through proposal / permission / audit and must not silently invoke capabilities.
- Surface lifecycle hooks (`onClose`, `onProposalDraft`, and related callbacks).
- Cross-origin surface-bundle allowlist, including CSP and origin checks.
- Community-marketplace surface allowlists, integrity pins, version pins, and audit metadata; installed project bundles remain same-origin by default.
- The project-console update entry already uses `check_for_updates` / `update_project`; next steps are richer update progress, failure recovery, and history.
- Wire up real stderr / exit metadata for the Failure modal, project `size_bytes` for Disk usage, and a more precise `storage_summary` measurement state once the host exposes them.
- Richer failure and health monitoring.

### Performance

The baseline lives in [`../performance/BASELINE.md`](../performance/BASELINE.en.md) and [`../../perf/baseline.json`](../../perf/baseline.json). Future optimizations use it as the regression reference: measure before changing behavior.

## Integration projects (separate repos)

These run on top of Yggdrasil and consume the platform through the public protocol. They don't live in this repo.

- **YdlTavern** — an independent integration project on Yggdrasil, compatible with SillyTavern's character cards, world books, presets, chat history, and extension API, with the engine layer running on Yggdrasil. Repo: <https://github.com/Youzini-afk/Yggdrasil-Tavern>. For Yggdrasil's side of the boundary, see [`../tavern/TAVERN_COMPAT.md`](../tavern/TAVERN_COMPAT.en.md).

## Indefinitely deferred at the kernel level

These don't belong in the kernel. They'll arrive as ordinary capability packages or future work:

- pi as a wholesale product shell embedding — see [`../architecture/PI_INTEGRATION.md`](../architecture/PI_INTEGRATION.en.md). Agent infrastructure can only move forward as ordinary packages and SDKs.
- External game-engine bridges (UE5, Godot, Unity, web clients).
- A privileged built-in Studio, a UI that bypasses the public protocol, or a kernel-owned official inspector. Public-protocol clients and package-contributed surfaces can keep evolving.
- Conversation runtime, prompts, models / sampling, message / turn semantics, memory models, world simulation, directors in the kernel. These stay in ordinary packages.
- Marketplace, package signing networks, dependency-resolver economy. The local sharing proof is done — see [`../guides/SHARING_DISTRIBUTION.md`](../guides/SHARING_DISTRIBUTION.en.md).
