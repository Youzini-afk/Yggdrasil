# Yggdrasil Shell + Release Round Plan (S-track, Round 3)

> 临时计划文件，每阶段完成后 push；全部完成后由收尾阶段删除并并入长期文档。

## 目的

把 Yggdrasil 从“可启动 host + 静态 web shell”推进到“可发布桌面应用 + 可挂载第三方 surface bundle 的 web 客户端”。

调研发现：
- `clients/web` 是无框架 plain TS SPA（无 React / Vite）
- 没有 Tauri / Electron 配置
- 没有 GitHub Actions / 发布流水线
- `kernel.surface.contribution.list` 只返回描述符，没有 SurfaceHost 加载器

本轮目标：
1. 给 `clients/web` 加 Vite 打包（保留 plain TS，不强行 React 化）
2. 加 iframe-based SurfaceHost 挂载第三方 surface bundle（如 `@ydltavern/surface`）
3. 加 Tauri 2.x desktop wrapper（Linux/macOS/Windows）
4. 加 GitHub Actions release pipeline + 版本同步脚本

## 不做的事

- 不重写 clients/web 为 React（保留 plain TS 渲染逻辑）
- 不做代码签名（macOS notarization / Windows EV cert 留给手动配置）
- 不做 auto-updater（v0 不要）
- 不真实发布到商店
- 不建立独立 web 域名/CDN

## 阶段

### S0：计划 push

本文件 + 英文版。

### S1：Vite bundling for clients/web + iframe SurfaceHost

**位置**：`clients/web/`

#### Vite 集成

`clients/web/package.json` 加 `vite` 依赖：

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc --noEmit && vite build",
    "preview": "vite preview",
    "check": "tsc --noEmit"
  },
  "devDependencies": {
    "vite": "^5",
    "typescript": "..."
  }
}
```

新增 `clients/web/vite.config.ts`：

```ts
import { defineConfig } from 'vite';

export default defineConfig({
  root: '.',
  build: {
    outDir: 'dist',
    target: 'es2022',
    sourcemap: true,
  },
  server: {
    port: 1420,
    strictPort: true,
    host: '127.0.0.1',
  },
});
```

`index.html` 已经引用 `src/main.ts` —— Vite 会自动处理。验证 `npm run dev` 和 `npm run build` 都能跑通现有所有路由（play / forge / assist drawer）。

#### iframe-based SurfaceHost

新增 `clients/web/src/surfaces/surface-host.ts`：

```ts
export interface SurfaceHostOptions {
  containerId: string;
  surfaceId: string;
  bundleUrl: string;          // 第三方 surface bundle ESM 入口
  exportName: string;         // React 组件名 (TavernPlaySurface 等)
  wrapperClass: string;       // CSS scope class
  // postMessage 桥接：surface 内部能调用宿主接口
  hostBridge?: SurfaceHostBridge;
}

export interface SurfaceHostBridge {
  // 暴露给 surface 的能力（明确定义，不直通 kernel）
  callRpc?(method: string, params: unknown): Promise<unknown>;
  subscribeEvents?(sessionId: string, callback: (e: unknown) => void): () => void;
  // 后续扩展...
}

export function mountSurface(options: SurfaceHostOptions): Promise<SurfaceHostHandle>;
export function unmountSurface(handle: SurfaceHostHandle): Promise<void>;
```

实现细节：
- iframe `src` 指向一个 host-served 静态 page（`/surface-frame.html`），该 page 内部用 ESM 动态 import 加载 bundle
- iframe `sandbox="allow-scripts"` （关闭 same-origin、forms、popups）
- postMessage 协议：surface 发 `{type: 'rpc.call', method, params, id}`，host 转发到 `client.invoke(...)` 然后回 `{type: 'rpc.result', id, result|error}`
- 资源限制：max iframe count、max iframe height、宽度撑满父容器
- CSP：surface frame 不能 fetch 任意网络（只能通过 host bridge）

新增 `clients/web/public/surface-frame.html`：

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self' blob:; style-src 'self' 'unsafe-inline';">
</head>
<body>
  <div id="root"></div>
  <script type="module">
    // 接收来自父窗口的 mount 指令
    // 动态 import bundleUrl
    // mount 到 #root
    // 转发 onFrame / 事件
  </script>
</body>
</html>
```

#### 测试

- `npm run build` 输出 `dist/`
- `npm run dev` 启动后能加载 play / forge / assist drawer 各页面无回归
- 新增最简的 SurfaceHost 单元测试（jsdom + happy path）

### S2：Tauri 2.x desktop wrapper

**位置**：`clients/desktop/`

#### 目录结构

```
clients/desktop/
  src-tauri/
    Cargo.toml
    tauri.conf.json
    build.rs
    src/main.rs
  package.json          (dev deps for tauri-cli)
  README.md
```

#### Cargo.toml

```toml
[package]
name = "ygg-desktop"
version.workspace = true
edition = "2021"
build = "build.rs"

[build-dependencies]
tauri-build = "2"

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
ygg-runtime = { path = "../../../crates/ygg-runtime" }
serde = { workspace = true }
tokio = { workspace = true }
```

把 `clients/desktop/src-tauri` 加入 workspace 的 `members` 列表。

#### tauri.conf.json

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Yggdrasil",
  "version": "../../../Cargo.toml",
  "identifier": "com.yggdrasil.desktop",
  "build": {
    "beforeDevCommand": "npm --prefix ../../web run dev",
    "beforeBuildCommand": "npm --prefix ../../web run build",
    "devUrl": "http://127.0.0.1:1420",
    "frontendDist": "../../web/dist"
  },
  "app": {
    "windows": [{
      "title": "Yggdrasil",
      "width": 1280,
      "height": 800,
      "minWidth": 800,
      "minHeight": 600
    }],
    "security": {
      "csp": "default-src 'self' asset: tauri:; connect-src 'self' http://127.0.0.1:8787 ipc: http://ipc.localhost; img-src 'self' asset: blob: data:; style-src 'self' 'unsafe-inline'; script-src 'self'; frame-src 'self'"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["deb", "rpm", "appimage", "msi", "nsis", "dmg"],
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.icns", "icons/icon.ico"]
  },
  "plugins": {}
}
```

#### main.rs

```rust
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 后续：在子线程启动 ygg-runtime host serve
            // v0：先只显示窗口，host 由用户单独启动
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

v0 暂不在 desktop 内部 spawn host serve，让用户单独跑 `ygg-cli host serve`。后续可以加 sidecar / embed runtime。

#### 图标占位

`clients/desktop/src-tauri/icons/`：用占位图标（生成纯色 PNG / icns / ico）。后续替换。

#### 测试

```bash
cd clients/desktop
npm install
npm run tauri dev    # Linux 上跑通，能打开窗口加载 web
npm run tauri build  # 输出 deb/rpm/appimage
```

CI 上跑 `tauri build` 验证不出错。

### S3：GitHub Actions release pipeline + 版本同步脚本

**位置**：`.github/workflows/`、`scripts/`

#### 版本同步脚本

新增 `scripts/release-version.sh`：

```bash
#!/bin/bash
set -euo pipefail

VERSION=${1:?"version required"}

# Stamp Cargo workspace version
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Stamp clients/web/package.json
node -e "
  const fs = require('fs');
  const pkg = JSON.parse(fs.readFileSync('clients/web/package.json', 'utf-8'));
  pkg.version = '$VERSION';
  fs.writeFileSync('clients/web/package.json', JSON.stringify(pkg, null, 2) + '\n');
"

# Stamp clients/desktop/src-tauri/tauri.conf.json (handled by version=Cargo.toml ref already)

echo "Stamped version $VERSION"
```

#### Release workflow

新增 `.github/workflows/release.yml`：

```yaml
name: release

on:
  push:
    tags:
      - "v*"

permissions:
  contents: write

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu-22.04
            rust_target: x86_64-unknown-linux-gnu
            tauri_args: ""
          - os: macos-14
            rust_target: aarch64-apple-darwin
            tauri_args: "--target aarch64-apple-darwin"
          - os: macos-13
            rust_target: x86_64-apple-darwin
            tauri_args: "--target x86_64-apple-darwin"
          - os: windows-2022
            rust_target: x86_64-pc-windows-msvc
            tauri_args: ""

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.rust_target }}

      - uses: swatinem/rust-cache@v2

      - name: Install Linux deps
        if: matrix.os == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libgtk-3-dev libsoup-3.0-dev

      - name: Install web deps
        run: npm ci --prefix clients/web

      - name: Install desktop deps
        run: npm ci --prefix clients/desktop

      - name: Build & release
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          projectPath: clients/desktop
          tagName: ${{ github.ref_name }}
          releaseName: "Yggdrasil ${{ github.ref_name }}"
          releaseBody: |
            ## Yggdrasil ${{ github.ref_name }}

            See CHANGELOG.md for details. Source code is licensed under AGPL-3.0.

            ### Build instructions
            - Rust: stable
            - Node: 20.x
            - Build script: `cd clients/desktop && npm run tauri build`
          releaseDraft: true
          prerelease: false
          args: ${{ matrix.tauri_args }}
```

#### CI workflow（test，非 release）

新增 `.github/workflows/ci.yml`：

```yaml
name: ci

on:
  push:
    branches: [main]
  pull_request:

jobs:
  rust:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: swatinem/rust-cache@v2
      - run: cargo check --workspace
      - run: cargo test -p ygg-runtime --lib
      - run: cargo test -p ygg-cli
      - run: cargo run -p ygg-cli -- conformance

  web:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm
      - run: npm ci --prefix clients/web
      - run: npm run check --prefix clients/web
      - run: npm run build --prefix clients/web
```

#### CHANGELOG 与 BUILDING 文档

新增 `CHANGELOG.md`（v0.1.0 占位）和 `BUILDING.md`（构建说明）。

### S4：临时计划删除 + 长期文档收敛

S 全部完成后：

```bash
rm docs/YGGDRASIL_SHELL_RELEASE_PLAN.md
rm docs/YGGDRASIL_SHELL_RELEASE_PLAN.en.md
```

更新长期文档：
- `README.md` / `.en.md`：加 desktop 安装/构建段落 + Web shell 章节更新
- `docs/ARCHITECTURE.md` / `.en.md`：加 SurfaceHost / desktop wrapper 章节
- `docs/roadmap/NEXT_STEPS.md` / `.en.md`：S 已完成
- `docs/ALPHA_STATUS.md` / `.en.md`：补 desktop 状态
- 新增 `docs/guides/SURFACE_HOSTING.md` / `.en.md`：iframe surface 协议
- 新增 `BUILDING.md`：构建说明

## 完成判据

- `cargo check --workspace` 通过
- `cargo test -p ygg-runtime --lib` 460+ 通过
- `cargo run -p ygg-cli -- conformance` 360 通过
- `npm run build --prefix clients/web` 输出 dist
- `npm run tauri build --prefix clients/desktop`（Linux）成功
- GitHub Actions workflow 文件 lint 通过（actionlint 或人工检查）
- 临时计划删除
- 长期文档同步
