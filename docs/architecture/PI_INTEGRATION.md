# pi Integration Plan

pi is integrated as an agent/capability layer. It should not own Yggdrasil's session state.

## Ownership boundary

Yggdrasil owns:

- sessions,
- turns,
- event commits,
- state projections,
- validation,
- prompt/model traces.

pi owns:

- agent reasoning,
- task execution,
- tool orchestration,
- analysis,
- proposal generation.

## Integration stages

### Stage 1: capability provider

pi exposes capabilities such as:

```text
agent.memory_curate
agent.state_extract
agent.consistency_check
```

Yggdrasil invokes them and receives proposals.

### Stage 2: event subscriber

pi subscribes to events such as:

```text
TurnCompleted
MessageCommitted
StatePatchCommitted
```

It asynchronously returns:

```text
MemoryProposal
StatePatchProposal
NarrativeThreadProposal
```

### Stage 3: agent-in-the-loop runtime

Specific runtime profiles may call pi during:

```text
before_context_plan
after_context_plan
before_model_call
after_turn
```

Every hook needs timeout, cancellation, permission, tracing, and fallback behavior.

## Commit rule

pi never directly mutates Yggdrasil state.

```text
AgentResult -> Proposal -> Validation -> Event append -> Projection
```
