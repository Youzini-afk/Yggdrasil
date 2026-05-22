// Yggdrasil desktop entry point.
//
// v0 scope: open a window pointing at the bundled clients/web app.
// User runs `ygg-cli host serve` separately and the web app connects via
// HTTP /rpc + SSE on 127.0.0.1:8787 by default.
//
// Future: optionally spawn `ygg-cli host serve` as a managed subprocess
// or embed the runtime directly.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ygg_desktop_lib::run();
}
