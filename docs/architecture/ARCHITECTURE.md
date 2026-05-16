# Architecture

Yggdrasil is organized around a small stable kernel and flexible outer layers.

```text
┌──────────────────────────────────────────────┐
│ Studio / App Layer                            │
│ official UI, editors, inspector, creator UX   │
├──────────────────────────────────────────────┤
│ External Engine Integration Layer             │
│ UE5 / Godot / Unity / Web / custom clients    │
├──────────────────────────────────────────────┤
│ Tavern Runtime Layer                          │
│ built-in SillyTavern-compatible runtime       │
├──────────────────────────────────────────────┤
│ pi Agent Layer                                │
│ agent tasks, proposals, planners, tools       │
├──────────────────────────────────────────────┤
│ Runtime Layer                                 │
│ sessions, turns, actors, context, model flow  │
├──────────────────────────────────────────────┤
│ Capability Fabric                             │
│ discover, describe, invoke, stream, authorize │
├──────────────────────────────────────────────┤
│ Protocol Layer                                │
│ HTTP/JSON-RPC, WebSocket, event streams       │
├──────────────────────────────────────────────┤
│ Core Layer                                    │
│ events, assets, state contracts, IDs, schemas │
└──────────────────────────────────────────────┘
```

## Boundary rules

1. **Core does not depend on Studio.**
   Core types must be usable by a CLI, a service, a game engine, or tests.

2. **Core does not depend on Tavern.**
   SillyTavern resources are imported into native assets/events/projections.

3. **Runtime does not depend on pi internals.**
   pi is integrated through agent tasks/capabilities/proposals.

4. **Studio does not become the kernel.**
   The official Studio should consume the public runtime/protocol boundary.

5. **External engines are not second-class plugins.**
   A game engine may consume Yggdrasil capabilities or provide capabilities to Yggdrasil through protocols.

6. **Agents propose; runtime commits.**
   Memory and state changes are validated and persisted through events.

## First implementation slice

The first slice intentionally avoids the full final crate graph. It starts with four Rust crates:

```text
crates/ygg-core     domain types and event envelopes
crates/ygg-runtime  runtime lifecycle, event store, model provider traits
crates/ygg-service  headless service adapter
crates/ygg-cli      CLI proof path
```

Future modules such as memory, model, capability, tavern, protocol, and pi adapter start as modules inside these crates and split out only after the boundaries stabilize.
