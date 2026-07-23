# Yggdrasil Desktop

Tauri 2.x shell around the shared `clients/web` platform UI. Desktop owns a
loopback-only `ygg host serve` sidecar and loads the UI from that Host after a
health check, keeping RPC, SSE, project surfaces, and proxies same-origin.

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
npm install --prefix ../web
npm run dev    # builds Web + sidecar, then starts the managed desktop shell
```

Build:
```bash
npm run build
```

The workspace target directory owns the output. With an explicit target triple,
installers are under `../../target/<triple>/release/bundle/`.

For Web-only HMR, run `npm run dev --prefix ../web`; it continues to proxy a
separately started Host on port 8787. The managed Desktop path intentionally
uses a built Web bundle so its random-port, same-origin behavior matches release.

## Managed Host boundary

`scripts/stage-sidecar.mjs` builds the CLI for the selected Rust target and
stages it as Tauri's `ygg-host` external binary. On launch Desktop:

- binds the Host to an OS-assigned `127.0.0.1` port;
- supplies a per-launch token through the child environment, never argv;
- waits for the stable listen handshake and `/healthz`;
- navigates the hidden window to the Host with a one-time bootstrap token;
- terminates the child when the application exits.

Run `npm run smoke:sidecar` after staging to verify the handshake, health route,
unauthenticated denial, and authenticated RPC against the real child process.

## Icons

The v0 wrapper uses generated solid-color placeholder PNG icons. Replace
them with branded icons and generate platform-specific `.icns` and `.ico`
assets before a polished desktop release.

## License

AGPL-3.0. See repository LICENSE.
