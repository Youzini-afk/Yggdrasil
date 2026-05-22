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

window.yggHost = {
  async callRpc(method, params) {
    const id = String(++nextRpcId);
    return new Promise((resolve, reject) => {
      pendingRpc.set(id, { resolve, reject });
      window.parent.postMessage({ type: 'rpc.call', id, method, params }, '*');
    });
  },
};

let unmountFn = null;

window.addEventListener('message', async (e) => {
  const msg = e.data;
  if (!msg || typeof msg !== 'object') return;

  if (msg.type === 'rpc.result') {
    const pending = pendingRpc.get(msg.id);
    if (!pending) return;
    pendingRpc.delete(msg.id);
    if (msg.error) pending.reject(new Error(`${msg.error.code}: ${msg.error.message}`));
    else pending.resolve(msg.result);
    return;
  }

  if (msg.type === 'mount') {
    try {
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
        return;
      }
      if (msg.wrapperClass) {
        root.className = msg.wrapperClass;
      }
      // mounter is a (root, props) => unmountFn function
      unmountFn = mounter(root, msg.initialProps ?? {});
    } catch (err) {
      root.textContent = `Surface bundle failed to load: ${err && err.message || err}`;
    }
  }

  if (msg.type === 'unmount') {
    if (typeof unmountFn === 'function') {
      try { unmountFn(); } catch { /* noop */ }
      unmountFn = null;
    }
    root.innerHTML = '';
  }
});

// Signal ready
window.parent.postMessage({ type: 'ready' }, '*');
