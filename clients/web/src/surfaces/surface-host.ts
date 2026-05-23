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
//     and host forwards via existing client.invoke

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
  callRpc?(method: string, params: unknown): Promise<unknown>;
  subscribeEvents?(sessionId: string, callback: (event: unknown) => void): () => void;
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
  session_id: string;
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
      const result = await options.hostBridge.callRpc(msg.method, msg.params);
      iframe.contentWindow!.postMessage({
        type: 'rpc.result',
        id: msg.id,
        result,
      } satisfies RpcResultMessage, '*');
    } catch (err) {
      iframe.contentWindow!.postMessage({
        type: 'rpc.result',
        id: msg.id,
        error: {
          code: 'rpc_failed',
          message: err instanceof Error ? err.message : String(err),
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
      await handleStreamSubscribe(msg, iframe, options.hostBridge, activeSubs);
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
) {
  if (!hostBridge?.subscribeEvents) {
    iframe.contentWindow?.postMessage({
      type: 'stream.error',
      subscription_id: msg.id,
      error: { code: 'no_bridge', message: 'host did not configure event subscription bridge' },
    } satisfies StreamErrorMessage, '*');
    return;
  }

  let close: () => void = () => {};
  try {
    close = hostBridge.subscribeEvents(msg.session_id, (event: unknown) => {
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
        const errMsg = ev.payload && typeof ev.payload === 'object'
          ? JSON.stringify(ev.payload)
          : String(ev.payload ?? '');
        iframe.contentWindow?.postMessage({
          type: 'stream.error',
          subscription_id: msg.id,
          error: { code: kind, message: errMsg },
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
      error: { code: 'subscribe_failed', message: err instanceof Error ? err.message : String(err) },
    } satisfies StreamErrorMessage, '*');
    close();
  }
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
