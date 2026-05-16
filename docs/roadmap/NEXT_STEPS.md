# Next Steps

The current center of gravity is hardening Yggdrasil's kernel-and-packages alpha until the platform contract is executable and falsifiable.

## Current status

The initial conversational runtime spike has been removed from the kernel crates. The workspace now contains content-free sessions, opaque events, package manifests, a manifest registry, a capability registry, a hook registry, SQLite event persistence, permission audit events, preview manifests for all entry forms, and a small CLI conformance command.

The new risk is no longer chat-shaped kernel pollution. The risk is a facade kernel: manifests load, but package execution is not yet real; protocol documents list methods that are not all executable; conformance is still too friendly.

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

## Phase B — Kernel v0 alpha contract hardening (current)

Code and documentation goal: make the existing alpha contract precise, executable, and hostile to privilege leaks.

Immediate targets:

- Freeze `docs/spec/KERNEL_V0_ALPHA_CONTRACT.md` as the implemented/partial/planned matrix.
- Keep `docs/spec/CONFORMANCE_MATRIX.md`, `docs/protocol/PROTOCOL_V0.md`, `README.md`, and `crates/ygg-runtime/src/protocol.rs` aligned.
- Implement real `rust_inproc` package execution so capability invocation crosses a package boundary.
- Replace friendly smoke conformance with hostile conformance cases: denied reads/writes/invokes, namespace violations, ambiguous providers, closed sessions, unload behavior, and official no-privilege checks.
- Add practical schema enforcement for manifests, events, and capability input/output.

Conversational types may return only after this phase, and only as a normal package.

## Phase C — First official package: conversational runtime (deferred)

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
