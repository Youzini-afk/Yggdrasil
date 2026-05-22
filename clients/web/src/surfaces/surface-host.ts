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

type SurfaceMessage = MountMessage | RpcCallMessage | RpcResultMessage | ReadyMessage;

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

  return {
    surfaceId: options.surfaceId,
    iframe,
    async unmount() {
      window.removeEventListener('message', bridgeListener);
      iframe.remove();
    },
  };
}
