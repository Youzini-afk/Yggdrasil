// Surface frame bootstrap: dynamic bundle loader.
//
// Flow:
//   1. Tell parent we are ready
//   2. Wait for {type: 'mount', bundleUrl, exportName, wrapperClass, initialProps, stylesheets}
//   3. Inject stylesheets
//   4. Dynamic import bundle, find exportName, render
//   5. Surface code can call kernel RPC via window.parent.postMessage({type:'rpc.call'})

const root = document.getElementById('root');

let nextRpcId = 0;
const pendingRpc = new Map();
let bridgeToken = '';
let targetOrigin = '*';

function postToHost(message) {
  window.parent.postMessage(
    bridgeToken ? { ...message, bridge_token: bridgeToken } : message,
    targetOrigin,
  );
}

window.yggHost = {
  async callRpc(method, params) {
    const id = String(++nextRpcId);
    return new Promise((resolve, reject) => {
      pendingRpc.set(id, { resolve, reject });
      postToHost({ type: 'rpc.call', id, method, params });
    });
  },
};

let unmountFn = null;

window.addEventListener('message', async (e) => {
  if (e.source !== window.parent) return;
  const msg = e.data;
  if (!msg || typeof msg !== 'object') return;

  if (msg.type === 'rpc.result') {
    const pending = pendingRpc.get(msg.id);
    if (!pending) return;
    if (bridgeToken && msg.bridge_token !== bridgeToken) return;
    pendingRpc.delete(msg.id);
    if (msg.error) pending.reject(new Error(`${msg.error.code}: ${msg.error.message}`));
    else pending.resolve(msg.result);
    return;
  }

  if (msg.type === 'mount') {
    try {
      bridgeToken = typeof msg.bridge_token === 'string' ? msg.bridge_token : '';
      targetOrigin =
        msg.initialProps && typeof msg.initialProps.targetOrigin === 'string'
          ? msg.initialProps.targetOrigin
          : '*';
      // Inject stylesheets if provided
      if (msg.stylesheets && Array.isArray(msg.stylesheets)) {
        for (const href of msg.stylesheets) {
          const link = document.createElement('link');
          link.rel = 'stylesheet';
          link.href = href;
          document.head.appendChild(link);
        }
      }

      const mod = await import(msg.bundleUrl);
      const mounter = mod[msg.exportName];
      if (!mounter) {
        root.textContent = `Surface bundle missing export ${msg.exportName}`;
        postToHost({ type: 'mount.error', code: 'missing_export', message: 'Surface bundle missing requested export' });
        return;
      }
      if (msg.wrapperClass) {
        root.className = msg.wrapperClass;
      }
      // mounter is a (root, props) => unmountFn function
      unmountFn = mounter(root, msg.initialProps ?? {});
    } catch (err) {
      root.textContent = `Surface bundle failed to load: ${err && err.message || err}`;
      postToHost({ type: 'mount.error', code: 'bundle_load_failed', message: 'Surface bundle failed to load' });
    }
  }

  if (msg.type === 'unmount') {
    if (bridgeToken && msg.bridge_token !== bridgeToken) return;
    if (typeof unmountFn === 'function') {
      try { unmountFn(); } catch { /* noop */ }
      unmountFn = null;
    }
    root.innerHTML = '';
    bridgeToken = '';
    targetOrigin = '*';
  }
});

// Signal ready
window.parent.postMessage({ type: 'ready' }, '*');
