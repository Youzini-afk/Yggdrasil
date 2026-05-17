# Alpha Status

> [English](./ALPHA_STATUS.md) · [中文](./ALPHA_STATUS.zh-CN.md)

This is the living snapshot of what Yggdrasil is right now. It is updated whenever a milestone closes. It is not aspirational: every line below has code and conformance behind it (or is explicitly marked partial/deferred).

For the long-term architecture and product stance, see `docs/CHARTER.md`, `docs/architecture/VISION.md`, and `docs/product/PLAY_CREATION_MODEL.md`. For where this is going, see `docs/roadmap/NEXT_STEPS.md`.

## Headline

- **Stage:** Platform Foundation Alpha + Play/Forge Surface Contract Beta.
- **Conformance:** 55 named CLI cases plus crate and service unit tests.
- **Charter discipline:** kernel content-free, official packages no privilege, public protocol only, package equality across entry forms.
- **Next stage:** Foundation Alpha Consolidation, then Playable Experience Alpha (see `docs/roadmap/NEXT_STEPS.md`).

## What is implemented

### Kernel

- Content-free sessions, append-only opaque events, manifest-driven packages, capability fabric, hook fabric slice, surface contributions, proposal lifecycle, asset/branch/projection substrate.
- SQLite-backed durable event log with per-session monotonic sequencing and rehydratable substrate.
- JSON Schema subset for capability input/output and package-declared event payloads.
- Principals: `host_admin`, `host_dev`, `package`, `human`, `assistant`, `anonymous`. Scoped grants for human and assistant principals.
- Permission audit events: `kernel/permission.granted`, `kernel/permission.revoked`, `kernel/permission.denied`.
- Package lifecycle events: `kernel/package.loading|starting|ready|stopping|stopped|loaded|unloaded|degraded|log`.
- Proposal lifecycle events: `kernel/proposal.created|approved|rejected|applied|failed`.

### Public protocol and transports

- Canonical request/response envelope with host-attached principal context. Callers cannot self-assert package or admin identity.
- HTTP `POST /rpc` and host JSON-RPC stdio (`ygg host-stdio`) call the same dispatcher.
- HTTP SSE event subscribe with `after_sequence` replay and live tailing for host-dev callers.
- Profile-backed `ygg host serve` autoloads packages and exposes `/rpc` plus SSE.
- WebSocket and TCP transports are reserved for future work; remote and WASM entries are reserved as first-class manifest forms with execution deferred.

### Package execution

- `rust_inproc` packages execute through a host-provided package trait and catalog. Manifests whose declared in-process provider is missing from the catalog are rejected.
- `subprocess` packages execute via JSON-RPC over stdio with handshake, invoke, invoke timeout, degraded state, restart, kill-on-unload, and stderr log capture.
- `wasm` and `remote` entries: manifest support yes, execution deferred.
- Capability routing supports explicit provider selection and simple exact / `^x.y` version constraints. Ambiguous routes are rejected unless the caller specifies `provider_package_id`.
- Hook fabric slice: deterministic ordering, package-owned handler capabilities, payload metadata mutation, veto, unload cleanup for `kernel/event.before_append|after_append` and `kernel/capability.before_invoke|after_invoke`.

### Substrate

- Asset registry: opaque `id`/`mime`/`hash`/`size`/`origin_package_id`/`metadata`, rehydratable from SQLite. Permission enforcement and content-addressed blob storage are next.
- Session fork/branch lineage records, rehydratable from the event log.
- Generic projection registry. Rebuild filters events by `kind_prefix` and `writer_package_id` and writes `kernel/projection.updated`. Package-owned projection execution is next.
- Surface contributions: typed descriptors with version, slot, activation, required permissions, approval policy, metadata. Slots: `experience_entry`, `home_card`, `play_renderer`, `forge_panel`, `asset_editor`, `assistant_action`. Discoverable through `kernel.surface.contribution.list` and `.describe`.
- Proposal lifecycle: `kernel.proposal.create|get|list|approve|reject|apply`. Apply currently executes generic `asset.put` and `projection.rebuild` operations. Broader transactions and revert/compensation are next.

### Official packages

All ordinary packages. No kernel privilege. They live under `packages/official/` and load through normal manifests:

- `official/package-lab` — package-authoring helpers exposed as ordinary capabilities and surfaces.
- `official/schema-tools` — schema-validation helpers.
- `official/event-tools` — event filtering and inspection helpers.
- `official/composition-lab` — composition validation, launch-plan, permission-preview, and surface-graph helpers.
- `official/asset-lab` — generic asset preview, diff, export, and import-plan helpers.
- `official/projection-lab` — projection describe, diff, rebuild-plan, and source-event helpers.
- `official/assistant-lab` — assistant-action capability that returns approval-gated proposals.
- `official/blank-experience` — minimal experience used by `ygg play-create-demo` to exercise the play-creation loop.
- `official/playable-seed` — reference playable package with entry/play/Forge/assistant surfaces.

The Forge profile (`profiles/forge-alpha.yaml`) autoloads these alongside example fixture packages.

### Web shell (`clients/web`)

- Skeletal Home/Play, Forge, and Assist surfaces over the public protocol.
- Home discovers `experience_entry` surfaces, launches sessions through the package-declared launch capability, supports session fork.
- Forge inspects events, capabilities, assets, projections, proposals, and Forge-panel surface contributions, with approve/apply controls for proposals.
- No official-package hardcoding. The shell is a public-protocol client like any other.

### Authoring

- `ygg init-package` generates Python or TypeScript subprocess package skeletons. The TypeScript variant uses the SDK runtime under `sdk/typescript/subprocess`.
- `--language typescript-experience` generates a manifest with experience-entry, play-renderer, Forge-panel, and assistant-action surface descriptors.
- `ygg init-composition` and `ygg composition check` provide a local composition descriptor flow.
- `ygg package check` and `ygg package conformance` validate generated packages locally.
- `ygg play-create-demo` orchestrates the blank play-creation loop end-to-end through ordinary public-protocol calls.

### Conformance

- `cargo run -p ygg-cli -- conformance` runs 55 named CLI cases covering: sessions, events, packages, capabilities, hooks, schemas, principals, permissions, subprocess execution, host transports, surfaces, proposals, official packages, composition-lab, asset-lab, projection-lab, playable-seed, blank play-creation loop, asset/branch/projection substrate, generated package authoring, and composition descriptors.
- Plus crate and service unit tests under `cargo test --workspace`.
- `tsc -p clients/web/tsconfig.json --noEmit` checks the web shell.

## What is partial

- Capability invocation lifecycle events (`kernel/capability.invoked|completed|failed`) reserved in contract; not emitted yet.
- Streaming protocol dispatch and package-principal `event.subscribe` permissions.
- Hook handler timeout/error audit for package-owned handlers.
- Persisted capability provider selection policy beyond per-invocation explicit selection.
- Persisted permission grant rehydration and richer resource policy coverage (network/filesystem/packages/projections enforcement matrices).
- Content-addressed asset blob storage and package-principal asset permission checks.
- Package-owned projection execution.
- Richer crash monitoring and health-check beyond lifecycle events.
- Broader transport parity coverage in conformance beyond the current core protocol dispatcher and service tests.
- Richer TypeScript SDK packaging beyond the current thin subprocess helper.
- Full `kernel.session.get|list`, `kernel.package.describe`, `kernel.capability.describe`, `kernel.extension_point.describe`, `kernel.host.principal`, `kernel.host.ping` route exposure.

## What is deferred

These are non-goals for the kernel and are expected to ship as ordinary packages or future work:

- Conversational runtime, prompts, models, sampling, message/turn semantics.
- Memory model, retrieval, summarization, agent loop, director.
- World, scene, actor, rule, dice, inventory semantics.
- SillyTavern resource and behavior compatibility (see `docs/tavern/TAVERN_COMPAT.md`).
- pi integration (see `docs/architecture/PI_INTEGRATION.md`).
- External game engine bridges (UE5, Godot, Unity, web clients).
- Marketplace, package signing, dependency resolver.
- Final UI visual design, full Studio, ComfyUI-like node editors.
- WASM and remote package execution.

## How to verify this snapshot

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- play-create-demo
tsc -p clients/web/tsconfig.json --noEmit
```

If any of the above fails, this document is wrong; the code is right. Update this document.

## Where to read next

- `docs/CHARTER.md` — what does not change.
- `docs/architecture/VISION.md` — what the platform is for.
- `docs/architecture/ARCHITECTURE.md` — kernel-and-packages layering.
- `docs/architecture/PLATFORM_KERNEL.md` — what the kernel does and does not do.
- `docs/architecture/CAPABILITY_PACKAGE.md` — package contract.
- `docs/architecture/EVENT_MODEL.md` — opaque event log.
- `docs/architecture/EXTENSION_POINTS.md` — hook contract.
- `docs/architecture/RUNTIME_LIFECYCLE.md` — kernel-side lifecycles.
- `docs/protocol/PROTOCOL_V0.md` — public protocol.
- `docs/spec/KERNEL_V0_ALPHA_CONTRACT.md` — executable alpha contract matrix.
- `docs/spec/CONFORMANCE_MATRIX.md` — hostile conformance roadmap.
- `docs/product/PLAY_CREATION_MODEL.md` — play-creation product stance.
- `docs/roadmap/NEXT_STEPS.md` — current and upcoming phases.
- `docs/roadmap/PLATFORM_HOST_ALPHA.md` — Host Alpha + Play/Forge Surface Beta result.
