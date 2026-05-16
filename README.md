# Yggdrasil

Yggdrasil is an open substrate for AI-native roleplay, storytelling, and interactive worlds.

It is not a SillyTavern clone and not a walled plugin host. The project is designed to exist as:

- a complete creator/player application,
- a headless runtime service,
- an embeddable engine/library,
- and an open protocol for external engines and tools.

The first implementation slice focuses on the platform spine:

```text
Event log -> Runtime turn -> Context plan -> Prompt frame -> Model call -> Streamed output -> Inspectable trace
```

## Initial implementation direction

- Rust core/runtime/service/CLI first.
- TypeScript Studio/SDK later, consuming the same public protocol.
- Tavern compatibility as a built-in runtime profile, not the kernel.
- pi integration as an agent/capability layer that proposes changes; Yggdrasil commits them through events.

## Repository layout

```text
crates/
  ygg-core/      Stable domain types and event envelopes.
  ygg-runtime/   Runtime lifecycle, event stores, model provider traits.
  ygg-service/   Headless HTTP/WebSocket adapter.
  ygg-cli/       CLI for proving the runtime spine.
docs/
  architecture/  Vision, architecture, event model, runtime lifecycle.
  protocol/      Protocol v0.
  tavern/        Tavern compatibility plan.
```

## Current phase

Phase 0/1: contract-first architecture plus a minimal in-memory runtime demo.
