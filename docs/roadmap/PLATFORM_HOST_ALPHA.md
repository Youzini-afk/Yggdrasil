# Platform Host Alpha

Platform Host Alpha is the milestone that proves Yggdrasil can host unprivileged external packages through the same public contract used by every caller. It is not a content runtime, Studio, Tavern compatibility, pi integration, or a game framework.

The implemented slice is in place: a fresh host with zero official packages can load a third-party subprocess package, complete a JSON-RPC-over-stdio handshake, expose and invoke a capability through the public protocol, enforce permissions/schemas/timeouts/teardown, dispatch declared hooks for the implemented extension points, unload cleanly, and pass hostile conformance through in-process and public transport paths.

A subsequent **Play/Forge Surface Contract Beta** layer was built directly on this foundation (see `Implemented slice` below). The remaining Host Alpha partial items continue to be tracked here and roll forward into Phase F (Foundation Alpha Consolidation) in `NEXT_STEPS.md`.

## Milestone definition

Platform Host Alpha is complete when a fresh host with zero official packages can:

1. load a third-party subprocess package from its manifest,
2. complete a JSON-RPC-over-stdio lifecycle handshake,
3. expose and invoke a capability through the public protocol,
4. enforce package permissions, namespace ownership, schemas, timeouts, and process teardown,
5. dispatch declared hooks deterministically for the implemented extension points,
6. unload the package cleanly,
7. pass hostile conformance through in-process and public transport paths.

## Implemented slice

Host Alpha foundation:

1. Protocol and principal foundation: method envelopes, runtime context, structured errors, no caller-supplied package identity spoofing for package-principal paths.
2. Subprocess package execution: JSON-RPC stdio start, handshake, invoke, invoke timeout, degraded state, unload kill.
3. Public transports: canonical HTTP `/rpc` and host JSON-RPC stdio mode for non-streaming methods.
4. Hook fabric slice: event append and capability invoke before/after dispatch, stable ordering, package-owned handler capabilities, metadata mutation, veto, unload cleanup.
5. Package authoring harness: Python and TypeScript subprocess templates, package check, local fixture run, local invoke, package conformance.
6. Release-gate conformance: named hostile cases with docs matrix coverage.
7. Event range replay and host-dev HTTP SSE tailing.
8. Explicit capability provider selection with simple version constraints.
9. Package lifecycle timeline, subprocess restart, stderr log capture, host diagnostics.
10. Event-log-rehydratable asset, projection, and session branch substrate for host-dev protocol callers.
11. Profile-backed `ygg host serve` with autoloaded packages, HTTP `/rpc`, and SSE routes.

Play/Forge Surface Contract Beta (built on the Host Alpha foundation):

12. A public-protocol web shell skeleton with Home/Play, Forge, and Assist surfaces under `clients/web`.
13. First official foundation packages (`official/package-lab`, `official/schema-tools`, `official/event-tools`) are ordinary package manifests under `packages/official` and autoload through the Forge profile.
14. The first assistant package (`official/assistant-lab`) is an ordinary package that contributes an assistant action and returns approval-gated proposals.
15. `ygg play-create-demo` demonstrates the first blank play-creation loop over ordinary packages and public substrate.
16. The web shell Home route discovers `experience_entry` surfaces over public protocol, launches package-backed sessions, and supports branch forking without official-package hardcoding.
17. The web Forge route provides generic public-protocol inspectors for package Forge panels, proposals, assets, projections, capabilities, and event tails.
18. Package authoring includes a generated experience-surface template and local composition descriptor checks.
19. Generic proposal lifecycle methods (`kernel.proposal.create/get/list/approve/reject/apply`) gate assistant/package changes behind explicit approval and append `kernel/proposal.*` audit events.

## Remaining Platform Host Alpha work

These items remain partial. They roll forward into Phase F consolidation and Phase I background hardening (see `NEXT_STEPS.md`).

1. Protocol-dispatched streaming and package-principal subscribe permission checks.
2. Hook timeout/error audit for package-owned handlers.
3. Health checks and richer crash monitoring beyond lifecycle transition events.
4. Persisted provider selection policy beyond per-invocation explicit provider selection.
5. Broader transport parity cases beyond the current core protocol dispatcher/service tests.
6. Richer TypeScript SDK packaging beyond the current thin subprocess helper/template.
7. Persisted permission grant rehydration and richer resource policy coverage.

## Non-goals for this milestone

- conversational runtime,
- model provider package,
- SillyTavern compatibility,
- pi integration,
- Studio / Prompt Inspector UI,
- final UI visual design or content runtime behavior,
- game, world, actor, director, memory, or agent semantics,
- marketplace or package dependency resolution,
- remote package execution,
- WASM package execution,
- full OS sandbox guarantees beyond explicitly tested subprocess timeout/kill behavior.

## Required invariant

No official package, client, service route, or SDK helper may use a privileged kernel path. Official namespaces are ordinary namespaces. If a behavior is unavailable to a third-party package through the public protocol and manifest model, it is not a Platform Host Alpha feature.
