# Surface Hosting Guide

> [English](./SURFACE_HOSTING.en.md) · [中文](./SURFACE_HOSTING.md)

This guide describes how `clients/web` hosts external React / Web surface bundles through sandboxed iframes. It documents the v0 host boundary: the web shell remains a plain TypeScript SPA, while third-party surfaces interact with Yggdrasil through the public protocol and an explicit host bridge.

## Purpose

Yggdrasil capability packages can contribute surface descriptors through their manifests. `clients/web` turns those descriptors into visible UI. When a third-party surface brings its own frontend bundle, the web shell does not load that code directly into the main window. Instead, `SurfaceHost` creates an iframe:

- the main shell keeps control of navigation, sessions, the public-protocol client, and permission prompts;
- the third-party bundle runs inside an isolated frame;
- the frame and host communicate only through a narrow `postMessage` protocol;
- the surface cannot reach the kernel directly and only gets bridge methods explicitly wired by the host.

The host implementation is in `clients/web/src/surfaces/surface-host.ts`. The frame bootstrap is in `clients/web/public/surface-frame.html`.

## Host API

```ts
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

export function mountSurface(options: SurfaceHostOptions): Promise<SurfaceHostHandle>;

// Unmount operation shape:
export function unmountSurface(handle: SurfaceHostHandle): Promise<void>;
```

`mountSurface(options)`:

1. finds the DOM container named by `options.containerId`;
2. creates a `sandbox="allow-scripts"` iframe and loads `/surface-frame.html`;
3. waits for the frame to send `{type: 'ready'}`;
4. sends `{type: 'mount', bundleUrl, exportName, wrapperClass, initialProps}` to the frame;
5. registers an `rpc.call` listener scoped to that iframe.

The current implementation exposes unmounting as `SurfaceHostHandle.unmount()`; the equivalent `unmountSurface(handle)` shape is `handle.unmount()`. It removes the message listener and removes the iframe from the DOM.

## Surface bundle expectations

A surface bundle must be an ESM module loadable via dynamic `import(bundleUrl)`, and it must expose a named export. `exportName` comes from surface metadata, for example `YdlTavernPlaySurface`.

The frame accepts two mount contracts:

```ts
export async function YdlTavernPlaySurface(root: HTMLElement, props: unknown) {
  // render into root
}

export const YdlTavernPlaySurface = {
  async mount(root: HTMLElement, props: unknown) {
    // render into root
  },
};
```

A React surface normally calls `createRoot(root).render(...)` inside the mount function. A plain DOM surface may update `root` directly.

CSS must be scoped under a wrapper class so it does not leak across nodes in the frame and so the host can size or theme by surface type:

```css
.ydltavern-play-surface {
  min-height: 100%;
}

.ydltavern-play-surface .message-row {
  /* scoped styles */
}
```

`wrapperClass` is applied to the frame's `#root` element.

## Iframe security model

The host creates the iframe with only:

```html
<iframe sandbox="allow-scripts" src="/surface-frame.html"></iframe>
```

There is no `allow-same-origin`, `allow-forms`, `allow-popups`, or other sandbox capability. This means:

- surface scripts can run;
- the frame does not get host same-origin authority;
- form submission, popups, and top-level navigation are unavailable by default;
- every host capability must go through the `postMessage` bridge.

`surface-frame.html` currently uses this CSP:

```text
default-src 'self'; script-src 'self' blob:; connect-src 'self'
```

The page also allows the minimal inline style and local/data/blob images needed for basic rendering. Network connections remain limited by `connect-src 'self'`; third-party bundles should not fetch arbitrary networks directly.

## postMessage protocol

After load, the frame first notifies the host:

```ts
// frame → host
{ type: 'ready' }
```

The host then sends the mount instruction:

```ts
// host → frame
{
  type: 'mount',
  bundleUrl,
  exportName,
  wrapperClass,
  initialProps,
}
```

When a surface needs host RPC, code in the frame calls `window.yggHost.callRpc(method, params)`, which sends:

```ts
// frame → host
{ type: 'rpc.call', id, method, params }
```

The host answers after the call finishes:

```ts
// host → frame
{ type: 'rpc.result', id, result }

// or
{ type: 'rpc.result', id, error: { code, message } }
```

The frame allocates `id` values to match pending promises. The host only accepts messages whose source is the expected iframe `contentWindow`.

## Host bridge

`hostBridge.callRpc(method, params)` is opt-in. If `mountSurface` does not receive `hostBridge.callRpc`, surface RPC calls receive:

```ts
{ type: 'rpc.result', id, error: { code: 'no_bridge', message: 'host did not configure RPC bridge' } }
```

By default, a third-party surface has no kernel access. The host must explicitly decide which public-protocol methods may be forwarded, which principal is used, and how approval or permission state is displayed. Do not pass internal runtime objects or unfiltered admin methods to a surface.

`subscribeEvents` is also an explicit bridge capability. The v0 host API defines the shape; the concrete event subscription wiring belongs to the host-side surface integration.

Future lifecycle callbacks can be added on the same boundary, for example:

- `onClose`
- `onProposalDraft`
- `onDirtyStateChanged`
- `onFocusRequest`

These callbacks should stay explicit and auditable. They must not become an implicit kernel side door.

## YdlTavern surface example

YdlTavern is an independent integration project that runs on top of Yggdrasil. Its `manifest.yaml` can declare three surfaces:

```yaml
surfaces:
  - id: ydltavern.play
    slot: play_renderer
    metadata:
      framework: react
      bundle_url: /surfaces/ydltavern/index.js
      export_name: YdlTavernPlaySurface
      wrapper_class: ydltavern-play-surface

  - id: ydltavern.settings
    slot: forge_panel
    metadata:
      framework: react
      bundle_url: /surfaces/ydltavern/index.js
      export_name: YdlTavernSettingsSurface
      wrapper_class: ydltavern-settings-surface

  - id: ydltavern.extensions
    slot: assistant_action
    metadata:
      framework: react
      bundle_url: /surfaces/ydltavern/index.js
      export_name: YdlTavernExtensionsSurface
      wrapper_class: ydltavern-extensions-surface
```

The web shell reads descriptors and metadata through `kernel.surface.contribution.list` / `.describe`, chooses the surface for the target slot, resolves `bundle_url`, `export_name`, and `wrapper_class`, then calls `mountSurface`. The host can pass the session id, descriptor, and read-only configuration in `initialProps`, and can decide whether to wire `hostBridge.callRpc` based on permissions.

## v0 limitations

- **Same-origin bundles:** the iframe currently loads same-origin bundle URLs only. Cross-origin bundles need an explicit allowlist, CSP changes, and origin checks.
- **No persistent frame state:** mount/unmount discards iframe memory. The host should own recoverable state and pass it back through `initialProps`.
- **No direct Tauri API:** iframe code cannot use Tauri APIs directly. Desktop capabilities must be exposed through controlled host bridge methods.
- **No implicit kernel access:** every RPC is explicitly wired by the host and should continue to use the public protocol and permission boundary.
- **Lifecycle callbacks are not complete:** `onClose`, `onProposalDraft`, and related callbacks are future work.

## Related docs

- [`../../BUILDING.md`](../../BUILDING.md) — web / desktop build and release notes.
- [`../architecture/ARCHITECTURE.md`](../architecture/ARCHITECTURE.en.md) — where the web shell, SurfaceHost, and desktop wrapper fit.
- [`../ALPHA_STATUS.md`](../ALPHA_STATUS.en.md) — current completion status.
- [`../roadmap/NEXT_STEPS.md`](../roadmap/NEXT_STEPS.en.md) — follow-up work.
