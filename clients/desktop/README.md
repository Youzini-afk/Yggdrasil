# Yggdrasil Desktop

Tauri 2.x wrapper around `clients/web`. Opens a desktop window for the
Yggdrasil web app.

## Development

Prerequisites:
- Rust stable
- Node.js 20+
- Linux: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libgtk-3-dev libsoup-3.0-dev`
- macOS: Xcode command-line tools
- Windows: Visual Studio Build Tools + WebView2

Run:
```bash
npm install
npm run dev    # opens window pointing at Vite dev server
```

Build:
```bash
npm run build  # produces installers in src-tauri/target/release/bundle/
```

## Boundary

The desktop wrapper does NOT spawn `ygg-cli host serve` internally in v0.
Run host separately:
```bash
cargo run -p ygg-cli -- host serve --http 127.0.0.1:8787 --profile profiles/forge-alpha.yaml
```

The desktop window points at `clients/web/dist` (production) or the Vite
dev server at 127.0.0.1:1420 (development), and the web app connects to
the host via HTTP /rpc + SSE on 127.0.0.1:8787.

## Icons

The v0 wrapper uses generated solid-color placeholder PNG icons. Replace
them with branded icons and generate platform-specific `.icns` and `.ico`
assets before a polished desktop release.

## License

AGPL-3.0. See repository LICENSE.
