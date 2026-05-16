# Protocol v0 Draft

The first protocol must be small and stable enough for the official Studio, CLI clients, and external engine experiments to share.

## Transport

Initial transports:

- HTTP/JSON for control requests.
- WebSocket for event streaming.

Future transports may include local IPC, gRPC/ConnectRPC, C ABI, and WASM bindings.

## Session API

```text
session.create
session.get
session.list
session.input
session.close
```

## Turn API

```text
turn.get
turn.list
turn.cancel
turn.regenerate later
```

## Event API

```text
event.list
event.subscribe
```

External systems should not receive unrestricted authority to append trusted internal events. External writes should usually enter as input events, requests, or proposals.

## Prompt inspection API

```text
prompt_frame.get
context_plan.get
model_call.get
```

## Asset API, later in MVP

```text
asset.import
asset.get
asset.list
asset.export
```

## Capability API, later in MVP

```text
capability.discover
capability.describe
capability.invoke
capability.register_provider
```

## Streaming events

The event stream should carry runtime events, not chat-specific deltas only:

```text
event.created
turn.started
context_plan.created
prompt_frame.created
model.delta
message.committed
turn.completed
error
```

Chat UI is a projection over this stream.
