# Event Model v0

The event log is Yggdrasil's spine. Chat history, prompt inspection, state, timeline views, replay, and external synchronization are projections over events.

## Event envelope

Every persisted event uses the same envelope:

```text
EventEnvelope
- id
- stream_id
- session_id
- turn_id?
- actor_id?
- kind
- schema_version
- timestamp
- causation_id?
- correlation_id?
- source
- payload
- metadata
```

## Initial event kinds

Phase 1 only needs a small set:

```text
SessionCreated
UserInputReceived
TurnStarted
ContextPlanCreated
PromptFrameCreated
ModelCallStarted
ModelStreamDelta
ModelCallCompleted
MessageCommitted
TurnCompleted
TurnCancelled
ErrorOccurred
```

## Turn event sequence

The minimal runtime spine should produce this sequence:

```text
SessionCreated
UserInputReceived
TurnStarted
ContextPlanCreated
PromptFrameCreated
ModelCallStarted
ModelStreamDelta*
ModelCallCompleted
MessageCommitted
TurnCompleted
```

## Persistence rules

- Events are append-only.
- State is derived from events.
- Prompt frames and model call information must be inspectable after the turn.
- Errors and cancellations are events, not just logs.
- Event payloads are versioned.

## Replay target

Yggdrasil cannot make LLM sampling fully deterministic, but the runtime should preserve enough data to debug and replay the runtime decisions:

- user input,
- context plan,
- prompt frame,
- model provider/model/parameters,
- model stream/final output,
- committed message,
- error/cancellation state.
