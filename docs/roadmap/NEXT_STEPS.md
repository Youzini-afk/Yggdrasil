# Next Steps

The first implementation slice now proves the runtime spine in memory.

## Completed slice

- Rust workspace with four crates.
- Core event/session/turn/prompt/model types.
- In-memory append-only event store.
- Runtime turn lifecycle.
- Mock streaming model provider.
- CLI demo that prints events and PromptFrame.
- Minimal headless HTTP routes for create session, input, and list events.
- Architecture/protocol/Tavern/pi boundary docs.

## Immediate next work

1. Add SQLite append-only event store.
2. Add tests for event sequence invariants.
3. Add PromptFrame lookup and ContextPlan persistence/projection.
4. Add OpenAI-compatible streaming provider.
5. Add WebSocket event subscription route.
6. Add Character Card V2 import thin slice.
7. Add minimal TypeScript SDK consuming the public service API.

## Keep deferring

- Full Studio UI.
- Full SillyTavern extension compatibility.
- Multi-agent orchestration.
- UE/Godot/Unity native bindings.
- WASM sandbox.
- Full permission system.
