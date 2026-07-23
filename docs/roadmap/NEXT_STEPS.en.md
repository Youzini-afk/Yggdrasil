# Next steps

> [English](./NEXT_STEPS.en.md) · [中文](./NEXT_STEPS.md)

This document is about where Yggdrasil goes next. Completed state lives in [`../ALPHA_STATUS.md`](../ALPHA_STATUS.en.md), not here.

## Where we are

- The kernel is content-free. Official packages have no privileges. Public protocol is the only entry.
- The secure-execution layer is complete: `secret_ref`, local encrypted secret store, network declarations, outbound audit and redaction, HTTP/WebSocket outbound executors, streaming and cancel lifecycle.
- The platform substrate is complete: package installation, native project install/mount, profile autoload, installed project surface bundles, surface-bundle freshness safeguards, project update, Home project shelf, structured shell descriptors, standalone project tabs, project-console diagnostics, the explicit Docker Deploy broker, durable deployment jobs, controlled Host development ChangeSets, target/exec/port/proxy deployment primitives, ygg-service HTTP/WebSocket reverse proxy, action-scoped revocable Host device identity, mobile PWA control, private-by-default and explicitly public application routes, Settings, real model end-to-end, streaming UX, the constrained Surface bridge, the managed-Host desktop wrapper, release pipeline, web shell release closure, and the code-organization split.
- Multi-provider model integration, a transport-neutral inference seam, Agentic Forge, the external project operating plane, storage backend neutrality, the PostgreSQL event backend, and the real TDB Rust adapter — all in.
- Contract V1 is the public platform spec; all 161 schemas (80 methods + 59 events + 22 top-level) validate, and 474 conformance cases pass.
- All nine Contract v2 layering-migration phases are complete: on top of the first eight substrate milestones, clients now use canonical APIs and Contract Registry `0.5.0` completes the real Deprecated → Legacy Adapter transition for `kernel.v1.host.info` and `kernel.v1.target.list`. See [`CONTRACT_V2_MIGRATION.md`](CONTRACT_V2_MIGRATION.en.md) for the implementation record.

The next stage isn't more substrate sprawl. Real project deployment, human testing, and playable experiences pull what comes next.

The dependency order among project authority, reliable deployment, operational safety, remote targets, and unified clients is fixed in
[`HOST_OPERATIONS_IMPLEMENTATION.en.md`](HOST_OPERATIONS_IMPLEMENTATION.en.md). Implementation must satisfy project isolation and local recovery gates
before enabling remote targets; real-project pressure still drives this work, and none of it becomes kernel content ontology.

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
- Runtime object/artifact permissions, quotas, and reachability GC; content-addressed blob storage itself is complete.
- Broader transport-consistency coverage in conformance.

### Project and multi-tenancy

- Multi-tenant project scoping based on `ProtocolContext.session_id`: make project identity explicit in runtime permission, event, and resolver context.
- Project archive auto-cleanup beyond 30 days.
- `yg secret put / list / delete` CLI.
- OS keyring integration (deferred until CI / cross-platform builds have stable system dependencies).
- Next for Host device identity: project-level multi-tenant scopes, delegation chains / bulk revoke, finer audit, and dedicated remote-CLI UX. Action scopes, remote identity, per-grant revoke, and mobile PWA control now use the same Host API; the root token remains the root credential.
- Remote CLI continues to reuse the same Host API and Bearer device token, with no local side-channel mutation interface. Mobile already uses HTTPS pairing plus a Secure cookie.
- Development-artifact read authority, encryption/retention policy, reachability GC, and more declarative verifier / sandbox backends.
- Deployment auto-restart (separate phase): first persist "deploy intent" (image, etc.) in host-plane terms, then add bounded-retry + backoff self-healing without leaking Docker semantics into the kernel proxy / port records. Today's health supervision only monitors, flips readiness, and audits — it does not re-deploy.
- Deployment descriptor polish: Docker pull progress, long-term log archival, artifact retention/cleanup, and external-project wizard generation.
- Remote execution targets: ports currently bind to loopback only. Multi-client control of one Host and explicit public vhosts are complete, but neither is a remote target nor an application identity system.

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
- Managed-Host Desktop follow-up: richer crash recovery guidance, sidecar-update coordination, and diagnostic export. Controlled start/stop, a random loopback port, one-time bootstrap, and a durable SQLite profile are complete.

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
