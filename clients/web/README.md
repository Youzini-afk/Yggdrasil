# Yggdrasil web shell

> English only for now. The bilingual project entry point is in the repository root README.

Public-protocol Home/Play, Forge, and Assist shell for the current Platform Foundation Alpha surface.

- `Home/Play` is the launcher-first surface for package-discovered experiences.
- `Forge` is the creation and inspection surface for sessions, events, proposals, capabilities, surfaces, assets, projections, and package labs.
- `Assist` is a drawer that bridges lightweight play edits and deeper Forge work through approval-gated proposals.
- **Text Surface Proof (Phase T1)** is a client-side UI proof inside the Assistant Drawer. It demonstrates progressive mock-streaming text with live line/height estimates using a lightweight text-layout adapter (`src/text-layout`). This is not a kernel feature and does not depend on real model/agent calls.
- **Optional Text Engine (Phase T2)** adds a `TextEngine` interface, engine registry, and fallback engine implementation. The Assistant Drawer now shows the active engine name, version, and state. A generic stream-frame-to-buffer adapter (`stream-adapter.ts`) is available for wiring future stream sources.

This client uses only public host APIs:

- `POST /rpc`
- `GET /kernel/event.subscribe/:session_id`

Run the host first:

```bash
cargo run -p ygg-cli -- host serve --http 127.0.0.1:8787 --profile profiles/forge-alpha.yaml
```

Then serve `clients/web` with any static web server. This is intentionally not a final visual design or a content runtime.

## Text Surface Proof

The `src/text-layout` module provides a Pretext-aligned API shape with a browser-only canvas fallback:

- `prepareText(text, font, options?)` → opaque handle
- `layoutPreparedText(prepared, maxWidth, lineHeight)` → `{ lineCount, height }`
- `createStreamingBuffer(font, lineHeight, maxWidth)` → streaming accumulator with `append`, `end`, `reset`, `measure`, `layoutLines`

It compiles and runs without installing `@chenglou/pretext`. If Pretext is adopted later, swap the internal implementations while keeping the same types and callers.

The proof draws from `src/text-layout/mock.ts` (inert chunks, no network) and renders inside the Assistant Drawer with state badges (`idle`, `streaming`, `ended`, `reset`) and live metadata.

## Optional Text Engine (Phase T2)

Phase T2 introduces the engine abstraction layer:

### TextEngine interface (`engine.ts`)

Defines `TextEngine` — an abstract interface for text layout engines with methods mirroring the fallback adapter API. Includes configuration types (`EngineConfig`, `TextEngineConfig`, `TextEngineName`, `TextEngineState`, `TextEngineDiagnostics`).

### FallbackTextEngine (`fallback-engine.ts`)

Implements `TextEngine` using the browser canvas API. Wraps the original adapter with backward-compatible exported functions (`prepareText`, `layoutPreparedText`, `createStreamingBuffer`, etc.). Cache is now bounded (default 4096 entries) with FIFO eviction.

### Engine registry (`registry.ts`)

- `registerTextEngine(engine)` — register a new engine
- `activateTextEngine(name)` — switch the active engine at runtime
- `getActiveTextEngine()` — get the current TextEngine instance
- `selectTextEngine(name)` — activate by name and return the engine
- `getTextEngineState(name)` / `getTextEngineDiagnostics()` — inspect engines
- `resolveEnginePreference()` — reads from URL params, localStorage, or `globalThis.__YGG_TEXT_ENGINE__` (T3 will wire to Pretext feature flags)

### Stream adapter (`stream-adapter.ts`)

- `feedStreamFrame(buffer, frame)` — generic frame→buffer adapter supporting `start`, `chunk`, `progress`, `end`, `error`, `cancelled`, `timeout` frames. No model/agent semantics.
- Convenience constructors: `frameStart()`, `frameChunk()`, `frameProgress()`, `frameEnd()`, `frameError()`, `frameCancelled()`, `frameTimeout()`
