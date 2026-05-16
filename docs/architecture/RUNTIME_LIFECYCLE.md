# Runtime Lifecycle v0

The first runtime goal is to prove a single session turn without depending on UI, HTTP, SQLite, Tavern compatibility, or pi.

## Minimal turn lifecycle

```text
session.create
  -> SessionCreated

session.input(text)
  -> UserInputReceived
  -> TurnStarted
  -> build ContextPlan
  -> ContextPlanCreated
  -> build PromptFrame
  -> PromptFrameCreated
  -> call ModelProvider
  -> ModelCallStarted
  -> ModelStreamDelta*
  -> ModelCallCompleted
  -> MessageCommitted
  -> TurnCompleted
```

## ContextPlan

The first ContextPlan can be trivial: selected recent input and a basic system block. It must still exist as a first-class object so Prompt Inspector is not a later hack.

## PromptFrame

PromptFrame is the final structured input to a model provider. It is not just a string.

It records:

- model target,
- message blocks,
- system/developer/user blocks when applicable,
- sampling parameters,
- token estimate when available,
- render trace.

## ModelProvider

The runtime should call a trait-like model boundary.

Phase 1 can use:

- a mock provider for tests and demos,
- then one OpenAI-compatible streaming provider.

## Cancellation

The first implementation can be simple, but statuses must include:

```text
pending
running
completed
cancelled
failed
```

Cancellation should eventually produce:

```text
TurnCancelled
ModelCallCancelled
CapabilityCancelled
```

## Storage progression

Recommended order:

1. in-memory event store,
2. CLI proof path,
3. prompt/model capture,
4. SQLite append-only event store,
5. headless service,
6. Studio.
