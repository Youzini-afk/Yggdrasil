# Yggdrasil Outbound WebSocket Evolution Plan (English)

> Temporary plan file. Push after each phase. Delete in Z8 once durable docs absorb its content.

## Purpose

Close the gap in Yggdrasil's outbound protocol — the missing bidirectional streaming primitive (WebSocket). After this round, the outbound triad is complete:

| Capability | Public method | Status |
|------------|---------------|--------|
| Unary HTTPS | `kernel.outbound.execute` | Y1 |
| Unidirectional stream (SSE/NDJSON) | `kernel.outbound.stream` | Y3 |
| Bidirectional WebSocket | `kernel.outbound.websocket.*` | **This round (Z)** |

Driving real scenarios:

1. **OpenAI Realtime API / Gemini Live API** — bidirectional voice/text; SSE alone cannot carry it
2. **Remote package entry form** — the fourth entry-form's first real path
3. **Outbound completion audit gap** — current `kernel.outbound.*` records request start but emits no per-call summary; both stream and websocket need it

## Out of scope

- Audio codecs (PCM/Opus/G.711) — caller's responsibility
- WebRTC / DataChannel (not outbound)
- Remote package real loading (separate entry-form work)
- Replacement for SSE (they coexist)
- Plain `ws://` allowed by default (only `wss://`)
- Frame payloads in audit (only shape/size/seq)

## Design principles

- **Reuse existing constraints**: HTTPS-only → wss-only; strict host match; manifest-declared secret_refs (Y2); capability_id must be in caller package namespace; same redaction rules
- **connection_id == stream_id**: `open` returns an id usable with `kernel.capability.cancel` for graceful close
- **Dedicated event channel**: WS frames mix text/binary which clashes with SSE chunk shape — use `kernel/outbound.websocket.*` events
- **send is unary RPC**: avoids complex bidirectional stream abstraction in subprocess SDK; backpressure expressed via executor buffer limits
- **deny-all default**: profile must explicitly enable + select executor

## Phases

### Z0: Plan push (this file + Chinese counterpart)

### Z1: Protocol method registration

| Method | Shape | Notes |
|--------|-------|-------|
| `kernel.outbound.websocket.open` | streaming (subscribe to events) | establish wss; returns connection_id |
| `kernel.outbound.websocket.send` | unary | send a frame (text or binary) |
| `kernel.outbound.websocket.close` | unary | send a close frame |

`kernel.capability.cancel(connection_id)` = `close(1001, "canceled")`.

Touch:
- `crates/ygg-runtime/src/protocol.rs` (KernelMethod, registry, streaming flag)
- `crates/ygg-runtime/src/runtime/protocol_dispatch.rs` (three dispatch fns)

### Z2: WebSocketExecutor trait + three impls

```rust
#[async_trait]
pub trait WebSocketExecutor: Send + Sync {
    async fn open(&self, req: OutboundWebSocketOpenRequest) -> Result<OutboundWebSocketSession>;
    async fn send(&self, connection_id: &str, frame: OutboundWebSocketFrame) -> Result<SendStatus>;
    async fn close(&self, connection_id: &str, code: u16, reason: Option<String>) -> Result<()>;
}
```

| Executor | Behavior |
|----------|----------|
| `DenyAllWebSocketExecutor` | always permission_denied |
| `FakeWebSocketExecutor` | scriptable local echo / canned frames; for conformance |
| `LiveWebSocketExecutor` | `tokio-tungstenite` + `rustls`, wss-only, strict host |

Live executor enforces:
- wss only (unless `allow_insecure_ws_for_tests` + loopback)
- destination_host equality with handshake URL
- secret_headers injected during HTTP upgrade
- `max_frame_bytes` / `max_total_bytes_inbound|outbound`
- `max_idle_ms`, `max_duration_ms`
- automatic ping/pong (not surfaced)
- close-code normalization (4000-4999 user range, others mapped to standard)

Per-connection actor (`tokio::spawn`):
- receives commands via `mpsc::Sender<Command>`
- inbound frames → emit `kernel/outbound.websocket.frame`
- close/error → emit corresponding events + cleanup

### Z3: Manifest schema

`network.declarations[].methods` accepts a new string `"WEBSOCKET"`:

```yaml
permissions:
  network:
    declarations:
      - host: api.openai.com
        methods: [POST, WEBSOCKET]
```

Dispatch:
- HTTP methods → outbound.execute / outbound.stream
- `WEBSOCKET` → outbound.websocket.*
- Mismatch → fail-closed

Touch:
- `crates/ygg-core/src/manifest.rs` validation
- `crates/ygg-runtime/src/runtime/network.rs` matcher

### Z4: HostProfile.outbound.websocket

```yaml
outbound:
  execute: ...    # Y1
  websocket:      # NEW
    enabled: false
    executor: deny_all                   # | fake | live
    allowed_hosts: []
    wss_only: true
    max_idle_ms: 60000
    max_duration_ms: 1800000
    max_frame_bytes: 65536
    max_total_bytes_inbound: 10485760
    max_total_bytes_outbound: 10485760
    max_concurrent_connections: 8
    allow_insecure_ws_for_tests: false
```

Touch: `cli.rs` HostProfileOutbound, `host.rs` profile→runtime config.

### Z5: TypeScript SDK kernelClient.openWebSocket

```typescript
export interface KernelWebSocketHandle {
  readonly connectionId: string;
  readonly subprotocol?: string;
  send(frame: { kind: 'text'; data: string } | { kind: 'binary'; data: Uint8Array }): Promise<void>;
  close(code?: number, reason?: string): Promise<void>;
}

export interface KernelClient {
  // existing
  openWebSocket(params: WebSocketOpenParams, callbacks: {
    onOpen?: () => void;
    onFrame: (frame: { kind: 'text'; data: string } | { kind: 'binary'; data: Uint8Array }) => void;
    onClose?: (info: { code: number; reason: string }) => void;
    onError?: (err: { code: string; message: string }) => void;
  }): Promise<KernelWebSocketHandle>;
}
```

Internally: open → reverse `kernel.outbound.websocket.open`; subscribe ws events filtered by connection_id; route frames to `onFrame`; `handle.send` → reverse `send`; `handle.close` → reverse `close` or `capability.cancel`.

### Z6: Outbound completion audit (small gap)

| Event | When | Key fields |
|-------|------|------------|
| `kernel/outbound.execute.completed` | unary terminates | status / executor_kind / total_bytes / duration_ms |
| `kernel/outbound.stream.completed` | stream.ended/error/cancelled/timeout | status / total_chunks / total_bytes / duration_ms |
| `kernel/outbound.websocket.completed` | ws close completes | code / reason / total_frames_in/out / total_bytes_in/out / duration_ms |

WS lifecycle events:

| Event | Trigger | Payload (redacted) |
|-------|---------|--------------------|
| `kernel/outbound.websocket.opened` | handshake ok | connection_id / destination_host / capability_id / package_id / subprotocol |
| `kernel/outbound.websocket.frame` | each frame in/out | connection_id / direction / frame_kind / bytes / seq (no payload) |
| `kernel/outbound.websocket.error` | any error | connection_id / error_code / redacted message |

Touch:
- `crates/ygg-core/src/event.rs` new event types
- `crates/ygg-runtime/src/runtime/network.rs` emit completion events

### Z7: Conformance (~12 cases)

```text
outbound_websocket_default_deny_all
outbound_websocket_fake_executor_open_send_close
outbound_websocket_secret_ref_undeclared_fails
outbound_websocket_capability_namespace_enforced
outbound_websocket_wss_only_default
outbound_websocket_idle_timeout_emits_error_and_completed
outbound_websocket_max_total_bytes_inbound_terminates
outbound_websocket_max_concurrent_connections_enforced
outbound_websocket_cancel_via_capability_cancel
outbound_execute_completed_audit_emitted
outbound_stream_completed_audit_emitted
outbound_websocket_completed_audit_emitted
```

Total: 347 → ~359.

### Z8: Profile + docs convergence + delete temp plan

- Update `profiles/forge-with-live-models.example.yaml` with `outbound.websocket` example (default disabled)
- Update `docs/protocol/PROTOCOL_V0.md`/`.en.md` outbound section
- Update `docs/spec/CONFORMANCE_MATRIX.md`/`.en.md`
- Update `README.md`/`.en.md` outbound triad
- Update `docs/ALPHA_STATUS.md`/`.en.md`
- Update `docs/roadmap/NEXT_STEPS.md`/`.en.md`
- Delete this plan file + Chinese counterpart

## Safety checklist

- [ ] Default deny_all
- [ ] wss:// only by default
- [ ] handshake host equality
- [ ] manifest-declared secret_refs (Y2)
- [ ] capability_id namespace lock
- [ ] Frame payloads NEVER in audit
- [ ] Secrets NEVER in frame/completion events
- [ ] Idle/duration/total-bytes/concurrent: 4 hard limits
- [ ] capability.cancel == close(1001)
- [ ] Real wss smoke gated by `YGG_LIVE_WEBSOCKET_TESTS=1` + creds

## Done criteria

- `cargo test --workspace` passes
- `cargo run -p ygg-cli -- conformance` 347 → ~359
- `forge-alpha.yaml` still parses
- `forge-with-live-models.example.yaml` parses with new section
- Live wss smoke is opt-in only; default CI offline
- Three completion events fire in both fake/live paths and are covered by conformance
- Temporary plan deleted; durable docs updated
