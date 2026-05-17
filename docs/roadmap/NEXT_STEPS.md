# Next Steps

> [English](./NEXT_STEPS.md) · [中文](./NEXT_STEPS.zh-CN.md)

The platform foundation is in place. Yggdrasil now has a content-free kernel, manifest-driven packages, real `rust_inproc` and subprocess execution, a permission/principal system, the hook fabric slice, surface contributions, the proposal/approval lifecycle, asset/branch/projection substrate, official platform packages, an assistant package, `official/playable-seed`, a blank play-creation loop, and a public-protocol web shell with Home/Play and Forge surfaces.

The next center of gravity is **not** more substrate. It is making the first reference packages useful enough that third-party packages can replace them on the same path.

## Where we are

- Platform Foundation Alpha: complete.
- Play/Forge Surface Contract Beta: complete.
- First Real Capability Package Track: complete seed (`composition-lab`, `asset-lab`, `projection-lab`, `playable-seed`; 55 conformance cases).
- Platform Host Alpha: implemented slice complete; remaining items (streaming dispatch, hook timeout audit, persisted provider policy, broader transport parity, richer SDK packaging) are tracked in `PLATFORM_HOST_ALPHA.md`.

See `docs/ALPHA_STATUS.md` for a detailed snapshot.

## Phase F — Foundation Alpha Consolidation (complete)

Goal: stop expanding surface area. Sand the rough edges, lock the contract, and make the existing foundation easy to demo, document, and extend.

- Documentation refresh across `README.md`, `README.zh-CN.md`, and the docs tree.
- Add `docs/product/PLAY_CREATION_MODEL.md` to fix the play-creation product stance.
- Add `docs/ALPHA_STATUS.md` as the living snapshot of what is done, partial, and deferred.
- Resolve remaining Platform Host Alpha partial items where they are cheap.
- @oracle-led review pass for content-shape leaks, official-privilege leaks, and YAGNI cleanups.
- A single canonical end-to-end demo path documented and validated through conformance.

This phase finishes when a new contributor can clone the repo, read one README, run one host serve command, and reach the blank play-creation loop without surprises.

## Phase G — Playable Experience Alpha seed (complete)

Goal: prove the substrate by building reference packages that are launchable, inspectable, forkable, and assistant-assistable, entirely as ordinary packages.

This is the first time the platform produces something a player-creator can sit with for more than a demo. It is not a SillyTavern, not a chat-only runtime, not a director — it is the smallest experience that exercises every substrate primitive honestly.

This seed is intentionally not a canonical game runtime. `official/playable-seed` proves the package path; `official/composition-lab`, `official/asset-lab`, and `official/projection-lab` prove the surrounding authoring and inspection loops.

Constraints carried into this phase:

- Kernel changes are last resort. If the experience needs a new primitive, redesign the experience first.
- The official package implementing the experience must remain replaceable by a third-party package.
- The assistant must propose changes through `kernel.proposal.*`, not through privileged paths.
- Forge must be able to inspect, fork, and edit the experience using only the public protocol.
- Conformance grows alongside the package: at least one hostile case proves third-party experience packages reach the same surfaces.

## Phase H — Authoring & Composition Beta+ (next)

Goal: turn the current authoring slice (`init-package`, `init-composition`, `composition check`, generated experience template) into a real authoring loop someone outside this repo can use to ship a package.

- Richer composition descriptors (multi-package bundles, optional capabilities, default activation).
- Template variants per surface slot (play renderer, forge panel, assistant action) beyond the current "experience template."
- Local dev-loop ergonomics: watch mode, fast reload, manifest diff, surface preview.
- Keep expanding `docs/guides/PACKAGE_AUTHORING_WALKTHROUGH.md` from a thin walkthrough into a complete contributor path.
- Optional package registry-shaped surface, still on top of the public protocol.

## Phase I — Substrate hardening (concurrent, low-priority)

Carried forward as background work, not the headline:

- Persisted permission grants and richer resource policy coverage.
- Content-addressed asset blobs.
- Package-owned projection execution.
- Streaming protocol dispatch + package-principal subscribe permissions.
- Hook handler timeout/error audit.
- Persisted capability provider selection policy.
- Broader transport parity coverage in conformance.
- WASM and remote package entry execution.

These items unblock specific use cases. They do not gate the headline phases above.

## Deferred indefinitely from kernel scope

These remain non-goals for the kernel. They may exist as future packages.

- SillyTavern compatibility — see `docs/tavern/TAVERN_COMPAT.md`.
- pi integration — see `docs/architecture/PI_INTEGRATION.md`.
- External game engine bridges (UE5/Godot/Unity, web clients).
- Any UI shell, inspector, or studio beyond the public-protocol web shell skeleton.
- Memory model, agent loop, world simulation, director, prompt rendering, model provider abstraction.
- Marketplace, package signing, dependency resolver.

## How to read this list

Phase F, the seed form of Phase G, and Creative Capability Kit Alpha are complete. Phase H is next: make third-party package authoring and composition feel real using the same interfaces proven by the official labs. Phase I runs in the background and is graded on charter discipline (no content shapes leaking into the kernel, no official privilege leaking through any path).
