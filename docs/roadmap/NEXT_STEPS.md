# Next Steps

The current center of gravity is making Yggdrasil a true kernel-and-packages platform. Docs come first; the Rust workspace follows.

## Current deviation

The bootstrapped Rust workspace put conversational concepts (`Turn`, `PromptFrame`, `ModelCall`, message commit) inside what should be a content-free kernel. This violates the charter. The first refactor removes the deviation by moving those concepts into an official capability package and reducing the kernel to its declared responsibilities.

## Phase A — Charter-aligned documentation

Done in this round:

- `docs/CHARTER.md`
- `docs/architecture/PLATFORM_KERNEL.md`
- `docs/architecture/CAPABILITY_PACKAGE.md`
- `docs/architecture/EXTENSION_POINTS.md`
- Rewrite of `docs/architecture/VISION.md`, `ARCHITECTURE.md`, `EVENT_MODEL.md`, `RUNTIME_LIFECYCLE.md`.
- Rewrite of `docs/protocol/PROTOCOL_V0.md`.
- `docs/architecture/PI_INTEGRATION.md` and `docs/tavern/TAVERN_COMPAT.md` reframed as deferred package families.
- Updated `README.md`.

## Phase B — Kernel skeleton in code (next)

Code goal: bring `ygg-core` and `ygg-runtime` in line with the kernel doc set, before any conversational logic comes back.

Targets, not yet implemented:

- Manifest schema: `PackageManifest` with `id`, `version`, `entry`, `provides`, `consumes`, `contributes`, `permissions`, `sandbox_policy`.
- Package registry: load/validate/start/stop, state machine, kernel events.
- Capability fabric: registration, discovery, version-constrained routing, ambiguous-route error, invocation lifecycle, streaming.
- Extension-point dispatch: kernel-emitted points first; subscriber registry; sync/async timing; modifiable/short_circuit semantics.
- Opaque event log: envelope with `writer_package_id`, namespaced kinds, schema validation against writer-declared schemas, kernel-only kinds reserved.
- Permission gate: enforce `events.append`, `events.read`, `network`, `filesystem`, `packages.call` declarations.
- Public protocol surface: `kernel.session.*`, `kernel.event.*`, `kernel.package.*`, `kernel.capability.*`, `kernel.extension_point.*`, `kernel.asset.*`, `kernel.host.*`.
- Entry forms: `rust_inproc` first, with the manifest already typed for `subprocess`, `wasm`, and `remote` so they can be added without a schema change.

Conversational types (`Turn`, `PromptFrame`, `ModelCall`, `MessageCommitted`, `ContextPlan`) leave the kernel during this phase. They will return as the first official capability package, not as kernel types.

## Phase C — First official package: conversational runtime

Once the kernel skeleton is in place, an official package implements the chat-shaped runtime that today's code prototypes:

- own event kinds (`turn.started`, `prompt.rendered`, `model.streamed`, `message.committed`, etc.) under the package's namespace,
- own capabilities (`generate`, `cancel`, `regenerate`),
- own extension points (`before_step`, `after_step`, etc.),
- model provider abstraction inside the package,
- prompt rendering and context planning inside the package.

The kernel will not gain anything to support this package. The package will work the same way a third-party package would.

## Phase D — Demonstrate the equality rule

Ship at least one minimal third-party-style package that reuses the conversational runtime's extension points (e.g., a tiny memory/curator package), to prove on the running platform that nothing the official package does requires kernel privilege.

## Deferred indefinitely from kernel scope

These remain non-goals for the kernel. They may exist as future packages.

- SillyTavern compatibility (see `docs/tavern/TAVERN_COMPAT.md`).
- pi integration (see `docs/architecture/PI_INTEGRATION.md`).
- External game engine bridges.
- Any UI shell, inspector, or studio.
- Any memory model, agent loop, world simulation, or director.
- SQLite schemas, OpenAI providers, vector retrieval, etc., as kernel concerns.

## How to read this list

Phase A is documentation only. Phase B is the kernel refactor in code. Phase C and beyond depend on Phase B landing cleanly. Until the kernel/package separation is real in code, no new content-shaped feature lands in the kernel crates.
