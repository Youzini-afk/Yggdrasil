# Round 10A.4 — Streaming UX Plan

> Temporary planning document. Removed at Wave 3 once docs converge.

## Mission

YdlTavern surface receives streaming chunks from engine model calls and updates
assistant message progressively. The full stack already exists; the gap is purely
at the surface/client layer.

```text
Audit findings (existing, working):
- kernel.v1.capability.stream protocol (start a stream invocation)
- kernel/v1/stream.{started,chunk,progress,ended,error,cancelled,timeout} events
- SSE delivery: GET /kernel/v1/event.subscribe/:session_id
- engine ydltavern/engine/model.live_call.stream uses kernel.v1.outbound.stream
  with SSE parsing + provider-specific chunk normalization

Audit findings (missing):
- Surface uses unary kernel.v1.capability.invoke + model.live_call (no chunks)
- No SSE subscription client in surface tree
- No streamCapability helper in surface
- Generating indicator + abort button exist as UI but not wired
```

## Architecture

Surface lives in a sandboxed iframe. It cannot directly fetch SSE from the host
(CORS, sandbox restrictions, also bad layering). Cleanest design: host mediates
stream forwarding via the existing postMessage RPC bridge.

### postMessage protocol extension (additive)

```text
Surface → Host: { type: 'stream.subscribe', id: <subscription_id>, stream_id, session_id }
Host → Surface: { type: 'stream.frame',     subscription_id, kind, payload }
Host → Surface: { type: 'stream.ended',     subscription_id }
Host → Surface: { type: 'stream.error',     subscription_id, error: { code, message } }
Surface → Host: { type: 'stream.unsubscribe', subscription_id }
```

Existing `rpc.call`/`rpc.result` messages remain unchanged. New types are
additive; older surface bundles continue working.

### Surface helper API

```ts
// packages/ydltavern-surface/src/host-rpc/stream.ts

export interface StreamFrame {
  kind: 'started' | 'chunk' | 'progress' | 'ended' | 'error' | 'cancelled' | 'timeout';
  payload: unknown;
}

export interface StreamHandle {
  streamId: string;
  frames: AsyncIterable<StreamFrame>;
  cancel(): Promise<void>;
}

export async function streamCapability(
  capabilityId: string,
  input: unknown,
): Promise<StreamHandle>;
```

Implementation pseudo-code:

```ts
async function streamCapability(capabilityId, input) {
  // 1. Start stream via existing kernel.v1.capability.stream RPC
  const start = await callHostRpc('kernel.v1.capability.stream', {
    capability_id: capabilityId,
    input,
  });
  const streamId = start.stream_id;

  // 2. Subscribe to stream frames via new postMessage flow
  const subscriptionId = newId();
  const queue = createAsyncQueue<StreamFrame>();

  const onMessage = (e: MessageEvent) => {
    if (e.data?.subscription_id !== subscriptionId) return;
    if (e.data.type === 'stream.frame')      queue.push(e.data);
    else if (e.data.type === 'stream.ended') { queue.push({ kind: 'ended', payload: null }); queue.close(); }
    else if (e.data.type === 'stream.error') { queue.push({ kind: 'error', payload: e.data.error }); queue.close(); }
  };
  window.addEventListener('message', onMessage);

  window.parent.postMessage({
    type: 'stream.subscribe',
    id: subscriptionId,
    stream_id: streamId,
    session_id: getActiveSessionId(),
  }, '*');

  return {
    streamId,
    frames: queue.iter(),
    cancel: async () => {
      await callHostRpc('kernel.v1.capability.cancel', { stream_id: streamId });
      window.parent.postMessage({ type: 'stream.unsubscribe', subscription_id: subscriptionId }, '*');
      window.removeEventListener('message', onMessage);
      queue.close();
    },
  };
}
```

### Host bridge forwarding

In `clients/web/src/surfaces/surface-host.ts`:

The existing host bridge handles `rpc.call`. Add a `stream.subscribe` handler:

```ts
async function handleStreamSubscribe(msg: StreamSubscribeMessage, iframe: HTMLIFrameElement) {
  const { id: subscriptionId, stream_id, session_id } = msg;

  // Subscribe to session events; filter for stream_id
  const close = await client.subscribeEvents(session_id, (event) => {
    if (!event.kind.startsWith('kernel/v1/stream.')) return;
    const payload = event.payload as { stream_id?: string };
    if (payload.stream_id !== stream_id) return;

    if (event.kind === 'kernel/v1/stream.chunk') {
      iframe.contentWindow!.postMessage({
        type: 'stream.frame',
        subscription_id: subscriptionId,
        kind: 'chunk',
        payload: event.payload,
      }, '*');
    } else if (event.kind === 'kernel/v1/stream.ended') {
      iframe.contentWindow!.postMessage({
        type: 'stream.ended',
        subscription_id: subscriptionId,
      }, '*');
      activeSubs.delete(subscriptionId);
      close();
    } else if (event.kind === 'kernel/v1/stream.error' || event.kind === 'kernel/v1/stream.cancelled' || event.kind === 'kernel/v1/stream.timeout') {
      iframe.contentWindow!.postMessage({
        type: 'stream.error',
        subscription_id: subscriptionId,
        error: { code: event.kind, message: JSON.stringify(event.payload) },
      }, '*');
      activeSubs.delete(subscriptionId);
      close();
    }
    // started + progress: forward as 'frame' too (caller can ignore)
  });

  activeSubs.set(subscriptionId, close);
}

async function handleStreamUnsubscribe(msg: StreamUnsubscribeMessage) {
  const close = activeSubs.get(msg.subscription_id);
  if (close) { close(); activeSubs.delete(msg.subscription_id); }
}
```

Track subscriptions per iframe; clean up on unmount.

## Waves

### Wave 1 — Host bridge + Surface streamCapability helper (~1-1.5 day)

#### Yggdrasil side

- `clients/web/src/surfaces/surface-host.ts`:
  - Extend MountMessage protocol with stream.subscribe / unsubscribe handling
  - Subscribe to client.subscribeEvents(session_id, ...) on demand
  - Filter and forward kernel/v1/stream.* events to iframe
  - Clean up on unmount

- `clients/web/src/protocol/client.ts`:
  - Verify subscribeEvents accepts a sessionId filter (per Wave 4 of 10A.2 it should)
  - Add lightweight helper if needed

#### YdlTavern side

- New `packages/ydltavern-surface/src/host-rpc/stream.ts`:
  - `streamCapability(capabilityId, input)` API
  - AsyncIterable queue helper
  - Tests with mocked window.parent + window.addEventListener

- Tests in `packages/ydltavern-surface/test/stream-capability.test.tsx`

### Wave 2 — TavernProvider streaming sendMessage + abort (~1 day)

- Update `packages/ydltavern-surface/src/app/TavernProvider.tsx::sendMessage`:
  - When `settings.streaming === true`, branch to streaming path
  - Call streamCapability('ydltavern/engine/model.live_call.stream', ...)
  - For each chunk frame: parse content delta from frame.payload, append to assistant message
  - On ended: mark message complete
  - On error: replace content with error message
  - Track active stream handle in state for cancel

- New `cancelGeneration()` action on TavernProvider:
  - Calls `streamHandle.cancel()` on the active stream
  - Updates UI generating state

- Wire abort button (per audit, exists as UI stub) to `cancelGeneration`

- Provider-specific chunk delta extraction:
  - OpenAI Chat: `choices[0].delta.content`
  - Anthropic: content_block_delta with text type
  - Gemini: candidates[0].content.parts[0].text
  - The engine's normalize_stream may already produce uniform shape — check
    `packages/ydltavern-engine/src/capabilities/model-live-call.ts` stream output
  - Fallback to attempting all known shapes

- Tests:
  - Mock streamCapability returning controlled frame sequence
  - Verify assistant message updates progressively
  - Verify cancel stops further updates
  - Verify error frame produces error UI

### Wave 3 — Docs + delete plan (~half day)

- Update `docs/guides/REAL_MODEL_END_TO_END.{md,en.md}` with streaming branch
- Update `PROJECT_MODEL.{md,en.md}` if streaming changes Play flow description
- Update `ALPHA_STATUS.{md,en.md}` — mark streaming UX implemented
- Update `CONFORMANCE_MATRIX.{md,en.md}` if any new conformance cases added
- Update `NEXT_STEPS.{md,en.md}` — Wave 3.6 streaming UX outlook → done
- Delete `docs/STREAMING_UX_PLAN.md`

## Push cadence

```
Plan written + pushed                        (now)
Wave 1 done + pushed                         (~1-1.5 day)
Wave 2 done + pushed                         (~1 day after Wave 1)
Wave 3 done + pushed (deletes plan)          (~half day after Wave 2)
Final report                                 (after Wave 3)
```

## Constraints

- AGPL-3.0 compatible
- Backward compatible: old surface bundles using only rpc.call continue working
  (postMessage extension is additive)
- No new kernel ontology — all changes in clients/web + YdlTavern surface
- Existing non-streaming sendMessage path stays as fallback when settings.streaming === false
- Stream cancel must be robust: cancel button in UI → cancelGeneration() → kernel.v1.capability.cancel + postMessage stream.unsubscribe + cleanup

## Out of scope (defer)

- Streaming for the realtime/WebSocket path (separate capability, separate UX)
- Multi-stream concurrency in single surface (one active generation per chat)
- Streaming progress UI beyond message append (e.g., token rate indicator)
- Server-side rate limiting / backpressure (existing kernel handles this)
