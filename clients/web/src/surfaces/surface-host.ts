// Iframe-based SurfaceHost.
//
// Mounts third-party surface bundles in a sandboxed iframe with a narrow
// postMessage protocol. The iframe loads /surface-frame.html, which
// dynamically imports the bundle URL passed via mount message.
//
// Security model:
//   - sandbox="allow-scripts" only (no same-origin, no forms, no popups)
//   - host bridge methods are explicit and minimal
//   - bundle URL must be served same-origin or cross-origin with explicit allow
//   - no direct kernel access; surface posts {type: 'rpc.call', method, params, id}
//     and host forwards only a narrow allowlisted bridge policy

const ALLOWED_BRIDGE_METHODS = new Set([
  'kernel.v1.host.info',
  'kernel.v1.host.ping',
  'kernel.v1.capability.invoke',
  'kernel.v1.capability.stream',
  'kernel.v1.capability.cancel',
]);

const CAPABILITY_METHODS = new Set([
  'kernel.v1.capability.invoke',
  'kernel.v1.capability.stream',
]);

const MAX_ID_LENGTH = 128;
const MAX_METHOD_LENGTH = 96;
const MAX_CAPABILITY_ID_LENGTH = 256;
const MAX_OWNED_STREAMS = 32;
const MAX_SUBSCRIPTIONS = 8;

export interface SurfaceHostOptions {
  containerId: string;
  surfaceId: string;
  bundleUrl: string;
  exportName: string;
  wrapperClass?: string;
  hostBridge?: SurfaceHostBridge;
  initialProps?: unknown;
  stylesheets?: string[];
}

export interface SurfaceHostBridge {
  currentSessionId: string;
  allowedCapabilityIds?: Iterable<string>;
  callRpc?(method: string, params: unknown): Promise<unknown>;
  subscribeEvents?(callback: (event: unknown) => void): () => void;
}

export interface SurfaceHostHandle {
  surfaceId: string;
  iframe: HTMLIFrameElement;
  unmount(): Promise<void>;
}

interface MountMessage {
  type: 'mount';
  bundleUrl: string;
  exportName: string;
  wrapperClass?: string;
  initialProps?: unknown;
  stylesheets?: string[];
}

interface RpcCallMessage {
  type: 'rpc.call';
  id: string;
  method: string;
  params: unknown;
}

interface RpcResultMessage {
  type: 'rpc.result';
  id: string;
  result?: unknown;
  error?: { code: string; message: string };
}

interface ReadyMessage {
  type: 'ready';
}

interface StreamSubscribeMessage {
  type: 'stream.subscribe';
  id: string;
  stream_id: string;
  session_id?: string;
}

interface StreamUnsubscribeMessage {
  type: 'stream.unsubscribe';
  subscription_id: string;
}

interface StreamFrameMessage {
  type: 'stream.frame';
  subscription_id: string;
  kind: 'started' | 'chunk' | 'progress';
  payload: unknown;
}

interface StreamEndedMessage {
  type: 'stream.ended';
  subscription_id: string;
}

interface StreamErrorMessage {
  type: 'stream.error';
  subscription_id: string;
  error: { code: string; message: string };
}

type SurfaceMessage =
  | MountMessage
  | RpcCallMessage
  | RpcResultMessage
  | ReadyMessage
  | StreamSubscribeMessage
  | StreamUnsubscribeMessage
  | StreamFrameMessage
  | StreamEndedMessage
  | StreamErrorMessage;

interface OwnedStreamRecord {
  streamId: string;
  invocationId?: string;
}

interface SurfaceBridgeState {
  ownedStreams: Map<string, OwnedStreamRecord>;
  ownedInvocations: Map<string, OwnedStreamRecord>;
}

export class SurfaceBridgeError extends Error {
  constructor(
    readonly code: string,
    message: string,
  ) {
    super(message);
  }
}

export async function mountSurface(options: SurfaceHostOptions): Promise<SurfaceHostHandle> {
  const container = document.getElementById(options.containerId);
  if (!container) throw new Error(`SurfaceHost: container ${options.containerId} not found`);

  const iframe = document.createElement('iframe');
  iframe.className = 'ygg-surface-iframe';
  iframe.setAttribute('sandbox', 'allow-scripts');
  iframe.style.width = '100%';
  iframe.style.height = '100%';
  iframe.style.border = '0';
  iframe.src = '/surface-frame.html';

  // Wait for iframe to send {type: 'ready'}
  const ready = new Promise<void>((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('SurfaceHost: iframe ready timeout')), 5000);
    const onMessage = (e: MessageEvent) => {
      if (e.source !== iframe.contentWindow) return;
      const msg = e.data as SurfaceMessage;
      if (msg.type === 'ready') {
        window.removeEventListener('message', onMessage);
        clearTimeout(timer);
        resolve();
      }
    };
    window.addEventListener('message', onMessage);
  });

  container.appendChild(iframe);
  await ready;

  // Send mount instruction
  iframe.contentWindow!.postMessage({
    type: 'mount',
    bundleUrl: options.bundleUrl,
    exportName: options.exportName,
    wrapperClass: options.wrapperClass,
    initialProps: options.initialProps,
    stylesheets: options.stylesheets,
  } satisfies MountMessage, '*');

  const bridgeState = createSurfaceBridgeState();

  // Wire host bridge for RPC calls from surface
  const bridgeListener = async (e: MessageEvent) => {
    if (e.source !== iframe.contentWindow) return;
    const msg = e.data as SurfaceMessage;
    if (msg.type !== 'rpc.call') return;
    if (!options.hostBridge?.callRpc) {
      iframe.contentWindow!.postMessage({
        type: 'rpc.result',
        id: msg.id,
        error: { code: 'no_bridge', message: 'host did not configure RPC bridge' },
      } satisfies RpcResultMessage, '*');
      return;
    }
    try {
      const result = await callBridgeRpc(options.hostBridge, msg, bridgeState);
      iframe.contentWindow!.postMessage({
        type: 'rpc.result',
        id: safeResponseId(msg.id),
        result,
      } satisfies RpcResultMessage, '*');
    } catch (err) {
      const code = err instanceof SurfaceBridgeError ? err.code : 'rpc_denied';
      iframe.contentWindow!.postMessage({
        type: 'rpc.result',
        id: safeResponseId(msg.id),
        error: {
          code,
          message: sanitizedBridgeMessage(code),
        },
      } satisfies RpcResultMessage, '*');
    }
  };
  window.addEventListener('message', bridgeListener);

  // Wire stream subscriptions from surface to session SSE events.
  const activeSubs = new Map<string, () => void>();
  const streamListener = async (e: MessageEvent) => {
    if (e.source !== iframe.contentWindow) return;
    const msg = e.data as SurfaceMessage;

    if (msg.type === 'stream.subscribe') {
      await handleStreamSubscribe(msg, iframe, options.hostBridge, activeSubs, bridgeState.ownedStreams);
    } else if (msg.type === 'stream.unsubscribe') {
      handleStreamUnsubscribe(msg, activeSubs);
    }
  };
  window.addEventListener('message', streamListener);

  return {
    surfaceId: options.surfaceId,
    iframe,
    async unmount() {
      iframe.contentWindow?.postMessage({ type: 'unmount' }, '*');
      await new Promise((resolve) => setTimeout(resolve, 50));
      window.removeEventListener('message', bridgeListener);
      window.removeEventListener('message', streamListener);
      for (const close of activeSubs.values()) close();
      activeSubs.clear();
      iframe.remove();
    },
  };
}

export function createSurfaceBridgeState(): SurfaceBridgeState {
  return {
    ownedStreams: new Map<string, OwnedStreamRecord>(),
    ownedInvocations: new Map<string, OwnedStreamRecord>(),
  };
}

export async function callSurfaceBridgeForTest(
  hostBridge: SurfaceHostBridge,
  message: { id: string; method: string; params: unknown },
  state: SurfaceBridgeState = createSurfaceBridgeState(),
): Promise<unknown> {
  return callBridgeRpc(hostBridge, { type: 'rpc.call', ...message }, state);
}

export function canSubscribeSurfaceStreamForTest(
  subscriptionId: string,
  streamId: string,
  activeSubscriptionIds: Iterable<string>,
  ownedStreamIds: Iterable<string>,
): { ok: true } | { ok: false; code: string } {
  if (!isSafeIdentifier(subscriptionId, MAX_ID_LENGTH) || !isSafeIdentifier(streamId, MAX_ID_LENGTH)) {
    return { ok: false, code: 'invalid_request' };
  }
  const active = new Set(activeSubscriptionIds);
  if (active.has(subscriptionId)) return { ok: false, code: 'duplicate_subscription' };
  if (active.size >= MAX_SUBSCRIPTIONS) return { ok: false, code: 'limit_exceeded' };
  if (!new Set(ownedStreamIds).has(streamId)) return { ok: false, code: 'not_owned' };
  return { ok: true };
}

function safeResponseId(value: unknown): string {
  return isBoundedString(value, MAX_ID_LENGTH) ? value : 'invalid';
}

function isPlainRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function isBoundedString(value: unknown, max: number): value is string {
  return typeof value === 'string' && value.length > 0 && value.length <= max;
}

function isSafeIdentifier(value: unknown, max: number): value is string {
  return isBoundedString(value, max) && /^[A-Za-z0-9][A-Za-z0-9._:/@-]*$/.test(value);
}

function sanitizedBridgeMessage(code: string): string {
  switch (code) {
    case 'rpc_denied':
      return 'RPC method is not available to this surface';
    case 'invalid_request':
      return 'Surface bridge request is invalid';
    case 'capability_denied':
      return 'Capability is not available to this surface';
    case 'not_owned':
      return 'Stream is not owned by this surface';
    case 'limit_exceeded':
      return 'Surface bridge limit exceeded';
    default:
      return 'Surface bridge request failed';
  }
}

async function callBridgeRpc(
  hostBridge: SurfaceHostBridge,
  msg: RpcCallMessage,
  state: SurfaceBridgeState,
): Promise<unknown> {
  if (!isBoundedString(msg.method, MAX_METHOD_LENGTH) || !ALLOWED_BRIDGE_METHODS.has(msg.method)) {
    throw new SurfaceBridgeError('rpc_denied', 'method denied');
  }
  if (!isBoundedString(msg.id, MAX_ID_LENGTH)) {
    throw new SurfaceBridgeError('invalid_request', 'invalid id');
  }
  if (!isBoundedString(hostBridge.currentSessionId, MAX_ID_LENGTH)) {
    throw new SurfaceBridgeError('invalid_request', 'invalid session');
  }
  if (msg.method === 'kernel.v1.host.ping') {
    return { ok: true };
  }
  if (!hostBridge.callRpc) {
    throw new SurfaceBridgeError('rpc_denied', 'bridge unavailable');
  }

  if (CAPABILITY_METHODS.has(msg.method)) {
    const params = sanitizeCapabilityParams(msg.params, hostBridge);
    const result = await hostBridge.callRpc(msg.method, params);
    if (msg.method === 'kernel.v1.capability.stream') {
      recordOwnedStream(result, state.ownedStreams, state.ownedInvocations);
    }
    return result;
  }

  if (msg.method === 'kernel.v1.capability.cancel') {
    const params = sanitizeCancelParams(msg.params, hostBridge, state.ownedStreams, state.ownedInvocations);
    const result = await hostBridge.callRpc(msg.method, params);
    removeOwnedStream(params, state.ownedStreams, state.ownedInvocations);
    return result;
  }

  return hostBridge.callRpc(msg.method, {});
}

function sanitizeCapabilityParams(params: unknown, hostBridge: SurfaceHostBridge): Record<string, unknown> {
  if (!isPlainRecord(params)) {
    throw new SurfaceBridgeError('invalid_request', 'params must be an object');
  }
  const capabilityId = params.capability_id;
  if (!isSafeIdentifier(capabilityId, MAX_CAPABILITY_ID_LENGTH)) {
    throw new SurfaceBridgeError('invalid_request', 'invalid capability id');
  }
  const allowed = new Set(hostBridge.allowedCapabilityIds ?? []);
  if (!allowed.has(capabilityId)) {
    throw new SurfaceBridgeError('capability_denied', 'capability denied');
  }
  const out: Record<string, unknown> = {
    capability_id: capabilityId,
    session_id: hostBridge.currentSessionId,
  };
  if ('input' in params) out.input = params.input;
  if (isSafeIdentifier(params.provider_package_id, MAX_CAPABILITY_ID_LENGTH)) out.provider_package_id = params.provider_package_id;
  if (isBoundedString(params.version, 64)) out.version = params.version;
  if (isPlainRecord(params.metadata)) out.metadata = params.metadata;
  return out;
}

function sanitizeCancelParams(
  params: unknown,
  hostBridge: SurfaceHostBridge,
  ownedStreams: Map<string, OwnedStreamRecord>,
  ownedInvocations: Map<string, OwnedStreamRecord>,
): { session_id: string; stream_id?: string; invocation_id?: string } {
  if (!isPlainRecord(params)) {
    throw new SurfaceBridgeError('invalid_request', 'params must be an object');
  }
  const streamId = isSafeIdentifier(params.stream_id, MAX_ID_LENGTH) ? params.stream_id : undefined;
  const invocationId = isSafeIdentifier(params.invocation_id, MAX_ID_LENGTH) ? params.invocation_id : undefined;
  if (!streamId && !invocationId) {
    throw new SurfaceBridgeError('invalid_request', 'cancel requires stream or invocation id');
  }
  if (streamId && !ownedStreams.has(streamId)) {
    throw new SurfaceBridgeError('not_owned', 'stream not owned');
  }
  if (invocationId && !ownedInvocations.has(invocationId)) {
    throw new SurfaceBridgeError('not_owned', 'invocation not owned');
  }
  return {
    session_id: hostBridge.currentSessionId,
    ...(streamId ? { stream_id: streamId } : {}),
    ...(invocationId ? { invocation_id: invocationId } : {}),
  };
}

function recordOwnedStream(
  result: unknown,
  ownedStreams: Map<string, OwnedStreamRecord>,
  ownedInvocations: Map<string, OwnedStreamRecord>,
) {
  if (ownedStreams.size >= MAX_OWNED_STREAMS) {
    throw new SurfaceBridgeError('limit_exceeded', 'too many streams');
  }
  const streamId = extractStringField(result, 'stream_id');
  const invocationId = extractStringField(result, 'invocation_id');
  if (!isSafeIdentifier(streamId, MAX_ID_LENGTH)) {
    throw new SurfaceBridgeError('invalid_request', 'stream result missing stream id');
  }
  const record = { streamId, ...(invocationId ? { invocationId } : {}) };
  ownedStreams.set(streamId, record);
  if (invocationId) ownedInvocations.set(invocationId, record);
}

function extractStringField(value: unknown, field: string): string | undefined {
  if (!isPlainRecord(value)) return undefined;
  const direct = value[field];
  if (typeof direct === 'string') return direct;
  for (const nestedKey of ['frame', 'invocation', 'record']) {
    const nested = value[nestedKey];
    if (isPlainRecord(nested) && typeof nested[field] === 'string') return nested[field];
  }
  return undefined;
}

function removeOwnedStream(
  params: { stream_id?: string; invocation_id?: string },
  ownedStreams: Map<string, OwnedStreamRecord>,
  ownedInvocations: Map<string, OwnedStreamRecord>,
) {
  const record = params.stream_id
    ? ownedStreams.get(params.stream_id)
    : params.invocation_id
      ? ownedInvocations.get(params.invocation_id)
      : undefined;
  if (!record) return;
  ownedStreams.delete(record.streamId);
  if (record.invocationId) ownedInvocations.delete(record.invocationId);
}

function payloadStreamId(payload: unknown): string | undefined {
  if (!payload || typeof payload !== 'object') return undefined;
  const streamId = (payload as { stream_id?: unknown }).stream_id;
  return typeof streamId === 'string' ? streamId : undefined;
}

async function handleStreamSubscribe(
  msg: StreamSubscribeMessage,
  iframe: HTMLIFrameElement,
  hostBridge: SurfaceHostBridge | undefined,
  activeSubs: Map<string, () => void>,
  ownedStreams: Map<string, OwnedStreamRecord>,
) {
  if (!hostBridge?.subscribeEvents) {
    iframe.contentWindow?.postMessage({
      type: 'stream.error',
      subscription_id: msg.id,
      error: { code: 'no_bridge', message: 'host did not configure event subscription bridge' },
    } satisfies StreamErrorMessage, '*');
    return;
  }
  const decision = canSubscribeSurfaceStreamForTest(msg.id, msg.stream_id, activeSubs.keys(), ownedStreams.keys());
  if (!decision.ok) {
    postStreamError(iframe, safeResponseId(msg.id), decision.code);
    return;
  }

  let close: () => void = () => {};
  try {
    close = hostBridge.subscribeEvents((event: unknown) => {
      const ev = event as { kind?: string; payload?: unknown };
      const kind = ev.kind ?? '';
      if (!kind.startsWith('kernel/v1/stream.')) return;
      if (payloadStreamId(ev.payload) !== msg.stream_id) return;

      if (kind === 'kernel/v1/stream.started') {
        iframe.contentWindow?.postMessage({
          type: 'stream.frame',
          subscription_id: msg.id,
          kind: 'started',
          payload: ev.payload,
        } satisfies StreamFrameMessage, '*');
      } else if (kind === 'kernel/v1/stream.chunk') {
        iframe.contentWindow?.postMessage({
          type: 'stream.frame',
          subscription_id: msg.id,
          kind: 'chunk',
          payload: ev.payload,
        } satisfies StreamFrameMessage, '*');
      } else if (kind === 'kernel/v1/stream.progress') {
        iframe.contentWindow?.postMessage({
          type: 'stream.frame',
          subscription_id: msg.id,
          kind: 'progress',
          payload: ev.payload,
        } satisfies StreamFrameMessage, '*');
      } else if (kind === 'kernel/v1/stream.ended') {
        iframe.contentWindow?.postMessage({
          type: 'stream.ended',
          subscription_id: msg.id,
        } satisfies StreamEndedMessage, '*');
        const c = activeSubs.get(msg.id);
        if (c) {
          c();
          activeSubs.delete(msg.id);
        }
      } else if (
        kind === 'kernel/v1/stream.error' ||
        kind === 'kernel/v1/stream.cancelled' ||
        kind === 'kernel/v1/stream.timeout'
      ) {
        iframe.contentWindow?.postMessage({
          type: 'stream.error',
          subscription_id: msg.id,
          error: { code: kind, message: 'Stream failed' },
        } satisfies StreamErrorMessage, '*');
        const c = activeSubs.get(msg.id);
        if (c) {
          c();
          activeSubs.delete(msg.id);
        }
      }
    });
    activeSubs.set(msg.id, close);
  } catch (err) {
    iframe.contentWindow?.postMessage({
      type: 'stream.error',
      subscription_id: msg.id,
      error: { code: 'subscribe_failed', message: 'Stream subscription failed' },
    } satisfies StreamErrorMessage, '*');
    close();
  }
}

function postStreamError(iframe: HTMLIFrameElement, subscriptionId: string, code: string) {
  iframe.contentWindow?.postMessage({
    type: 'stream.error',
    subscription_id: subscriptionId,
    error: { code, message: sanitizedBridgeMessage(code) },
  } satisfies StreamErrorMessage, '*');
}

function handleStreamUnsubscribe(
  msg: StreamUnsubscribeMessage,
  activeSubs: Map<string, () => void>,
) {
  const close = activeSubs.get(msg.subscription_id);
  if (close) {
    close();
    activeSubs.delete(msg.subscription_id);
  }
}
