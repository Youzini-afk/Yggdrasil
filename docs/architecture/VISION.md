# Yggdrasil Vision

Yggdrasil is an open AI-native creative gameplay substrate.

It should be useful as all of the following:

1. **Full App** — an official Studio for RP, creation, inspection, and play.
2. **Headless Service** — a local or server runtime driven by protocols.
3. **Embeddable Library** — a runtime that games, tools, or engines can link or embed.
4. **Open Protocol** — a language-neutral way for external engines, agents, tools, and models to participate.

## Non-goals

Yggdrasil is not:

- a direct SillyTavern rewrite,
- a chat-message-only application,
- a walled plugin platform,
- a giant agent graph where everything is an agent,
- or a UI-first product whose API is added later.

## Core product thesis

AI roleplay and creative gameplay should be modeled as long-running, inspectable, replayable, extensible narrative systems — not as a fragile array of chat messages.

The core platform should therefore be built around:

- append-only events,
- derived state projections,
- inspectable context planning,
- structured prompt frames,
- model call traces,
- memory proposals and commits,
- capability providers,
- and public protocols.

## Tavern's role

Tavern compatibility is a first-class built-in runtime profile.

It exists to let the SillyTavern community's resources and habits live inside Yggdrasil, but it must not define the platform kernel.

```text
SillyTavern concepts -> Tavern Runtime -> Yggdrasil native events/assets/state
```

## pi's role

pi provides agentic capabilities such as memory curation, state extraction, planning, consistency checking, and NPC reasoning.

Yggdrasil owns session state and event commits. pi reads events and produces proposals.

```text
AgentTask -> AgentResult -> Proposal -> Validation -> Event append -> State projection
```

## External engines

UE5, Godot, Unity, Web clients, bots, and custom engines should be treated as first-class clients or capability providers.

They may connect through:

- remote HTTP/WebSocket protocol,
- a local sidecar daemon,
- or future embedded Rust/WASM/native bindings.

The official Studio must use the same public runtime boundary as external clients wherever practical.
