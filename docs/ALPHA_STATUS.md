# Alpha Status

> [English](./ALPHA_STATUS.md) · [中文](./ALPHA_STATUS.zh-CN.md)

This is the living snapshot of what Yggdrasil is right now. It is updated whenever a milestone closes. It is not aspirational: every line below has code and conformance behind it (or is explicitly marked partial/deferred).

For the long-term architecture and product stance, see `docs/CHARTER.md`, `docs/architecture/VISION.md`, and `docs/product/PLAY_CREATION_MODEL.md`. For where this is going, see `docs/roadmap/NEXT_STEPS.md`.

## Headline

- **Stage:** Platform Foundation Alpha + Play/Forge Surface Contract Beta.
- **Conformance:** 72 named CLI cases plus crate and service unit tests.
- **Charter discipline:** kernel content-free, official packages no privilege, public protocol only, package equality across entry forms.
- **Code health:** CLI commands/templates/conformance, runtime domain behavior, protocol dispatch, and runtime official in-process handlers are split by domain instead of accumulating in monolithic files.
- **Next stage:** Authoring & Composition Beta+ (see `docs/roadmap/NEXT_STEPS.md`).

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
- `official/composition-lab` — composition validation, launch-plan, permission-preview, surface-graph, and compat-report helpers with v2 descriptor diagnostics (capabilities, permissions, replacements, compatibility notes).
- `official/asset-lab` — generic asset preview, diff, export, and import-plan helpers.
- `official/projection-lab` — projection describe, diff, rebuild-plan, and source-event helpers.
- `official/persona-lab` — persona profile import, normalization, rendering, and compatibility diagnostics.
- `official/knowledge-lab` — structured knowledge collection normalization, matching, injection planning, and diagnostics.
- `official/context-lab` — bounded context block assembly, layer inspection, budget planning, and template rendering.
- `official/text-transform-lab` — deterministic text transform import, validation, preview, pipeline explanation, and diagnostics.
- `official/model-connector-lab` — no-network provider family metadata, profile validation, secret masking, discovery plans, and compatibility reports.
- `official/model-routing-lab` — no-inference consumer-slot binding, route planning, fallback planning, and params normalization.
- `official/assistant-lab` — assistant-action capability that returns approval-gated proposals.
- `official/blank-experience` — minimal experience used by `ygg play-create-demo` to exercise the play-creation loop.
- `official/playable-seed` — reference playable package with entry/play/Forge/assistant surfaces.

The Forge profile (`profiles/forge-alpha.yaml`) autoloads these alongside example fixture packages.

### Web shell (`clients/web`)

- Skeletal Home/Play, Forge, and Assist surfaces over the public protocol.
- Home discovers `experience_entry` surfaces, launches sessions through the package-declared launch capability, supports session fork.
- Forge inspects packages, capabilities, assets, projections, proposals, events, and surface contributions, with package/capability inventory by provider, surface descriptor inventory by slot, composition/authoring diagnostics, manifest/template CLI guidance, and approve/apply controls for proposals.
- No official-package hardcoding. The shell is a public-protocol client like any other.

### Authoring

- `ygg init-package` generates Python or TypeScript subprocess package skeletons. The TypeScript variant uses the SDK runtime under `sdk/typescript/subprocess`.
- `--template basic|experience|play-renderer|forge-panel|assistant-action|asset-editor|full-surface` controls generated surface descriptors. Without `--template`, `--language *-experience` auto-detects a legacy 4-surface experience mode for backward compatibility; otherwise defaults to basic.
- `--language typescript-experience` (without `--template`) still generates the original 4-surface experience descriptors for backward compatibility.
- `ygg init-composition` and `ygg composition check` provide a local composition descriptor flow with v2 fields (title, description, optional packages, required capabilities, default activation, permission expectations, replacement candidates, compatibility notes). `composition check` prints structured diagnostics: loaded required/optional packages, surfaces by slot, capabilities, entry activation, missing required surfaces/capabilities (fail), and warnings for missing optional packages.
- `ygg package check` and `ygg package conformance` validate generated packages locally. `ygg package check` prints structured diagnostics: entry kind, trust level, capability count, surfaces by slot, permissions summary, sandbox policy summary, and warnings for packages with no capabilities or no surfaces.
- `ygg package reload <manifest>` loads a package into an in-memory runtime, restarts it (subprocess only), prints before/after status and logs count, then unloads. Uses existing Runtime::restart_package path; no new protocol methods.
- `ygg package run-fixture` invokes all declared non-streaming capabilities with deterministic canned inputs and prints a structured JSON summary.
- `ygg play-create-demo` orchestrates the blank play-creation loop end-to-end through ordinary public-protocol calls.

### Code organization

- `crates/ygg-cli/src/main.rs` is a thin entry point. CLI types live in `cli.rs`; commands live under `commands/`; package generation templates live under `templates/`; conformance cases live under `conformance/` domain modules.
- `crates/ygg-runtime/src/runtime/` owns runtime domain behavior across session, events, packages, capabilities, hooks, permissions, assets, branches, projections, proposals, and protocol dispatch modules; `runtime/mod.rs` preserves the public `Runtime<S>` API and re-exports moved public request/record types.
- Protocol method metadata and dispatch share the `KernelMethod` source of truth, with unit coverage for registry/dispatch consistency.
- `crates/ygg-runtime/src/inproc.rs` retains the in-process package API and delegates official lab behavior to focused modules under `crates/ygg-runtime/src/inproc/`.
- `crates/ygg-runtime/src/inproc/common.rs` routes shared official in-process handlers by provider package and local capability name rather than suffix-only fallback.
- This split is behavior-preserving and exists to keep future package, conformance, and handler growth reviewable.

### Conformance

- `cargo run -p ygg-cli -- conformance` runs 72 named CLI cases covering: sessions, events, packages, capabilities, hooks, schemas, principals, permissions, subprocess execution, host transports, surfaces, proposals, official packages, composition-lab (with v2 diagnostics and compat-report), asset-lab, projection-lab, persona-lab, knowledge-lab, context-lab, text-transform-lab, model-connector-lab, model-routing-lab, in-process package fallback hardening, playable-seed, blank play-creation loop, asset/branch/projection substrate, generated package authoring (basic, experience, assistant-action, asset-editor, full-surface templates), composition descriptors (v1 and v2), package check diagnostics, and package reload smoke.
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
