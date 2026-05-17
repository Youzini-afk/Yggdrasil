# Optional Text Engine Alpha

> [English](./OPTIONAL_TEXT_ENGINE_ALPHA.md) · [中文](./OPTIONAL_TEXT_ENGINE_ALPHA.zh-CN.md)

This temporary execution plan turns the current Text Surface Proof into an optional frontend text engine track. Pretext is treated as an optional client-side layout engine, not a kernel feature and not an official capability package.

## Invariants

- No `kernel.text.*`, `kernel.model.*`, `kernel.agent.*`, or `kernel.prompt.*` methods.
- No `official/pretext-*` package.
- Fallback text layout remains always available.
- Pretext, if used, is behind a Web client engine abstraction and dynamic selection.
- Assistant/Forge/Play consume generic stream/text surfaces, not model/agent semantics.

## Phase T2 — Engine abstraction and fallback registry ✅ COMPLETE

Goals:

- Introduce a `TextEngine` interface, engine registry, config, and fallback engine implementation.
- Refactor the existing text-layout adapter so current Assistant proof behavior is preserved.
- Add stream-frame-to-text-buffer adapter helpers for generic stream frames.

Delivered:

- **`engine.ts`**: `TextEngine` interface, `EngineConfig`/`TextEngineConfig`/`TextEngineName`/`TextEngineState`/`TextEngineDiagnostics` types.
- **`fallback-engine.ts`**: `FallbackTextEngine implements TextEngine` wrapping the original canvas adapter. Backward-compatible function exports (`prepareText`, `layoutPreparedText`, `createStreamingBuffer`, etc.) preserved. Bounded width cache (default 4096 entries, FIFO eviction).
- **`registry.ts`**: `registerTextEngine`/`activateTextEngine`/`getActiveTextEngine`/`selectTextEngine`/`getTextEngineState`/`getTextEngineDiagnostics`/`unregisterTextEngine`. Default is fallback. Supports localStorage/URL param/env string preference resolution (T3 will wire to Pretext feature flags).
- **`stream-adapter.ts`**: `feedStreamFrame(buffer, frame)` generic adapter supporting `start`/`chunk`/`progress`/`end`/`error`/`cancelled`/`timeout`. No model/agent semantics. Convenience frame constructors provided.
- **`index.ts`**: Updated re-exports — all original function names unchanged; new types and functions exported alongside.
- **Assistant Drawer**: Shows active engine name, version, and state badge in the Text Proof metadata row.
- **`clients/web/README.md`**, **`integrations/pretext/ui-map.yaml`**: Updated to document T2 additions.

Validation:

- `tsc -p clients/web/tsconfig.json --noEmit` passes.
- Existing Rust/conformance checks unaffected.
- No kernel/package/protocol changes.

## Phase T3 — Optional Pretext engine and feature flags ✅ COMPLETE

Goals:

- Add optional `PretextEngine` behind dynamic import / runtime engine selection.
- Keep the repo buildable without installing Pretext.
- Add runtime controls via URL/localStorage/build environment fallbacks.
- Update `integrations/pretext` ledger and client README.

Delivered:

- **`pretext-shim.ts`**: Local type definitions mirroring the `@chenglou/pretext` API surface (v0.0.7). Defines `PretextModuleShape` interface for safe dynamic import casting. Allows TypeScript compilation without the package installed.
- **`pretext-bridge.ts`**: Isolated mapping between Ygg text-layout types and Pretext shapes. Option conversion (`toPretextOptions`), result mapping (`fromPretextLayoutResult`, `fromPretextLayoutLinesResult`, `fromPretextLineStats`, `fromPretextLineRange`), and opaque handle bridging (`bridgePrepared`, `bridgePreparedWithSegments`, `unbridgePrepared`, `unbridgePreparedWithSegments`). If the real module is unavailable, the type skeleton and adapter functions still compile.
- **`pretext-engine.ts`**: `PretextTextEngine implements TextEngine` with async `initialize()`. Uses dynamic import with unknown-safe casting (`import("@chenglou/pretext")` → cast to `PretextModuleShape`). If the module is unavailable, `initialize()` throws a diagnostic error with the load failure reason for registry fallback. Exports `isPretextAvailable()`, `getPretextLoadError()`, `resetPretextLoadState()`.
- **`config.ts`**: Runtime engine preference resolution. `TextEnginePreference` type: `"auto" | "fallback" | "pretext"`. Resolved from URL param `?text-engine=`, localStorage `ygg_text_engine`, or `globalThis.__YGG_TEXT_ENGINE__`. Default is `"auto"` (use Pretext if available, otherwise fallback). Exports `TextEngineInitializationResult` with preferred/active engine, fallback reason, and Pretext availability.
- **`registry.ts` (T3 additions)**: `initializeTextEnginePreference()` — async init that resolves preference, tries to load and activate Pretext, falls back gracefully with recorded reason. `getInitializationResult()` — returns the last init result for diagnostics. `isPretextEngineAvailable()` — checks Pretext module availability. `getPretextAvailabilityError()` — returns the load error if any. Existing sync `getActiveTextEngine()` still works unchanged.
- **Assistant Drawer (T3 additions)**: Additional diagnostic badges — engine preference (`pref auto`/`pref fallback`/`pref pretext`), Pretext availability (`pretext available`/`pretext unavailable`), and fallback reason (with full reason in tooltip).
- **`index.ts`**: Updated re-exports for all new T3 types and functions.
- **`clients/web/README.md`**: Updated with T3 section documenting PretextTextEngine, bridge, shim, config, async init, and Assistant Drawer additions.
- **`integrations/pretext/README.md`**: Updated with T3 integration details (shim, bridge, engine, config, registry, drawer).
- **`integrations/pretext/upstream.lock.toml`**: Updated with T3 notes (dynamic import, fallback guarantee).
- **`integrations/pretext/ui-map.yaml`**: Updated to T3-alpha-1 with bridge/shim/config/engine entries and updated constraints.

Validation:

- `tsc -p clients/web/tsconfig.json --noEmit` passes (without `@chenglou/pretext` installed).
- Fallback works when Pretext is unavailable.
- Engine selection diagnostics visible in Assistant proof (preference, availability, fallback reason badges).
- No kernel/package/protocol changes.

## Phase T4 — Forge/Assistant stream text integration

Goals:

- Wire the text buffer adapter to generic stream frame shapes.
- Add a bounded Forge text preview for stream/proposal/tool/audit-like long text without replacing JSON inspectors.
- Keep Play unchanged except for documented future optional hint design.

Validation:

- Web TypeScript passes.
- UI behavior remains public-protocol-only.

## Phase T5 — SDK extraction, tests, and hardening

Goals:

- Extract reusable text-surface helpers under `sdk/typescript/text-surface`.
- Add lightweight unit tests for fallback engine, registry, stream adapter, and engine selection.
- Add cache limits and font-loading helpers.
- Document third-party client usage.

Validation:

- TypeScript tests pass.
- Existing Rust/conformance/play demo pass.

## Final phase — durable docs and cleanup

Goals:

- Update durable docs/status/roadmap.
- Delete this temporary plan after completion.
- Run full validation.

Required checks:

```bash
tsc -p clients/web/tsconfig.json --noEmit
cargo test --workspace
cargo run -p ygg-cli -- conformance
cargo run -p ygg-cli -- play-create-demo
```
