# Surface Hosting 指南

> [English](./SURFACE_HOSTING.en.md) · [中文](./SURFACE_HOSTING.md)

本指南说明 `clients/web` 如何用 sandboxed iframe 承载外部 React / Web surface bundle。它描述的是 v0 宿主边界：Web shell 仍然是 plain TypeScript SPA，第三方 surface 通过公开协议和显式 host bridge 与 Yggdrasil 交互。

## 目的

Yggdrasil 的能力包可以通过 manifest 贡献 surface 描述符。`clients/web` 负责把这些描述符变成可见 UI。对于需要自带前端 bundle 的第三方 surface，Web shell 不把代码直接加载进主窗口，而是通过 `SurfaceHost` 创建 iframe：

- 主 shell 保持对导航、会话、公开协议客户端和权限提示的控制；
- 第三方 bundle 在隔离 frame 内运行；
- frame 与宿主只通过窄 `postMessage` 协议通信；
- surface 不能直接访问 kernel，只能使用宿主显式接线的 bridge。

实现入口见 `clients/web/src/surfaces/surface-host.ts`，frame bootstrap 见 `clients/web/public/surface-frame.html`。

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

`mountSurface(options)` 会：

1. 查找 `options.containerId` 指向的 DOM 容器；
2. 创建 `sandbox="allow-scripts"` iframe，加载 `/surface-frame.html`；
3. 等待 frame 发送 `{type: 'ready'}`；
4. 向 frame 发送 `{type: 'mount', bundleUrl, exportName, wrapperClass, initialProps}`；
5. 为该 iframe 注册 `rpc.call` 监听器。

当前实现把 unmount 操作挂在 `SurfaceHostHandle.unmount()` 上；`unmountSurface(handle)` 的等价形状是 `handle.unmount()`。它会移除 message listener，并从 DOM 中移除 iframe。

## Surface bundle 约定

Surface bundle 必须是可被动态 `import(bundleUrl)` 加载的 ESM module，并暴露一个具名 export。`exportName` 来自 surface metadata，例如 `YdlTavernPlaySurface`。

frame 接受两种 mount contract：

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

React surface 通常在 mount function 内调用 `createRoot(root).render(...)`。Plain DOM surface 可以直接修改 `root`。

CSS 必须限制在 wrapper class 之下，避免污染 frame 内其他节点，也便于宿主按 surface 类型控制尺寸和主题：

```css
.ydltavern-play-surface {
  min-height: 100%;
}

.ydltavern-play-surface .message-row {
  /* scoped styles */
}
```

`wrapperClass` 会被设置到 frame 的 `#root` 元素上。

## Iframe 安全模型

宿主创建 iframe 时只设置：

```html
<iframe sandbox="allow-scripts" src="/surface-frame.html"></iframe>
```

没有 `allow-same-origin`、`allow-forms`、`allow-popups` 或其他权限。结果是：

- surface script 可以运行；
- frame 不能取得宿主同源权限；
- form submit、popup、顶层导航等能力默认不可用；
- 所有宿主能力都必须走 `postMessage` bridge。

`surface-frame.html` 当前使用的 CSP 是：

```text
default-src 'self'; script-src 'self' blob:; connect-src 'self'
```

页面还允许必要的 inline style 和本地/data/blob 图片，用于基础渲染。网络连接仍限制为 `connect-src 'self'`；第三方 bundle 不应直接访问任意外网。

## postMessage 协议

Frame load 后先通知宿主：

```ts
// frame → host
{ type: 'ready' }
```

宿主随后发送 mount 指令：

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

Surface 如需调用宿主 RPC，frame 内代码通过 `window.yggHost.callRpc(method, params)` 发送：

```ts
// frame → host
{ type: 'rpc.call', id, method, params }
```

宿主完成调用后返回：

```ts
// host → frame
{ type: 'rpc.result', id, result }

// or
{ type: 'rpc.result', id, error: { code, message } }
```

`id` 由 frame 分配，用于匹配 pending promise。宿主只处理来自对应 iframe `contentWindow` 的 message。

## Host bridge

`hostBridge.callRpc(method, params)` 是 opt-in。如果 `mountSurface` 没有收到 `hostBridge.callRpc`，surface 调用 RPC 会得到：

```ts
{ type: 'rpc.result', id, error: { code: 'no_bridge', message: 'host did not configure RPC bridge' } }
```

默认状态下，第三方 surface 没有 kernel access。宿主必须显式决定哪些公开协议方法可以转发、使用哪个 principal、如何显示审批或权限状态。不要把内部 runtime object 或未过滤的 admin 方法传入 surface。

`subscribeEvents` 也属于显式 bridge 能力；v0 host API 只定义形状，具体事件订阅接线由宿主 surface integration 决定。

后续可以在同一边界上增加 surface lifecycle callback，例如：

- `onClose`
- `onProposalDraft`
- `onDirtyStateChanged`
- `onFocusRequest`

这些 callback 应保持显式、可审计，不应变成隐式 kernel 旁路。

## YdlTavern surface 示例

YdlTavern 是独立接入项目，运行在 Yggdrasil 之上。它的 `manifest.yaml` 可以声明三个 surface：

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

Web shell 通过 `kernel.v1.surface.contribution.list` / `.describe` 读取描述符和 metadata，选择目标 slot 的 surface，解析 `bundle_url`、`export_name`、`wrapper_class`，然后调用 `mountSurface`。宿主可以把 session id、surface descriptor、只读配置等放入 `initialProps`，并按权限决定是否接线 `hostBridge.callRpc`。

## v0 限制

- **同源 bundle：** iframe 当前只加载同源 bundle URL。跨源 bundle 需要显式 allowlist、CSP 更新和来源校验。
- **无持久 frame 状态：** mount/unmount 会丢弃 iframe 内存状态。宿主应持有可恢复状态，并通过 `initialProps` 传回 surface。
- **无 Tauri 直通 API：** iframe 内不能直接访问 Tauri API。需要桌面能力时，通过宿主 bridge 暴露受控方法。
- **无隐式 kernel access：** 所有 RPC 都由宿主显式接线，并应继续走公开协议和权限边界。
- **生命周期 callback 未完成：** `onClose`、`onProposalDraft` 等仍是后续工作。

## 相关文档

- [`../../BUILDING.md`](../../BUILDING.md) — Web / desktop 构建与 release 说明。
- [`../architecture/ARCHITECTURE.md`](../architecture/ARCHITECTURE.md) — Web shell、SurfaceHost、desktop wrapper 的架构位置。
- [`../ALPHA_STATUS.md`](../ALPHA_STATUS.md) — 当前完成状态。
- [`../roadmap/NEXT_STEPS.md`](../roadmap/NEXT_STEPS.md) — 后续工作。
