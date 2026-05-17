# Yggdrasil web shell

> English only for now. The bilingual project entry point is in the repository root README.

Public-protocol Home/Play, Forge, and Assist shell for the current Platform Foundation Alpha surface.

- `Home/Play` is the launcher-first surface for package-discovered experiences.
- `Forge` is the creation and inspection surface for sessions, events, proposals, capabilities, surfaces, assets, projections, and package labs.
- `Assist` is a drawer that bridges lightweight play edits and deeper Forge work through approval-gated proposals.
- **Text Surface Proof (Phase T1)** is a client-side UI proof inside the Assistant Drawer. It demonstrates progressive mock-streaming text with live line/height estimates using a lightweight text-layout adapter (`src/text-layout`). This is not a kernel feature and does not depend on real model/agent calls.
- **Optional Text Engine (Phase T2)** adds a `TextEngine` interface, engine registry, and fallback engine implementation. The Assistant Drawer now shows the active engine name, version, and state. A generic stream-frame-to-buffer adapter (`stream-adapter.ts`) is available for wiring future stream sources.
- **Optional Pretext Engine (Phase T3)** adds an optional `PretextTextEngine` behind dynamic import, runtime engine selection with feature flags, and graceful fallback. The repo builds without installing `@chenglou/pretext`. The Assistant Drawer shows engine preference, Pretext availability, and fallback reason.
- **Forge Text Preview (Phase T4)** adds a text preview helper that extracts safe plain-text previews from event payloads, stream frames, and proposal-like objects in the Forge surface. Events and proposals show an optional `<details>` with preview text, line/height estimates, and engine name — without replacing the existing JSON inspectors. No model/agent semantics; Play unchanged.

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
- `getActiveTextEngine()` — get the current TextEngine instance (synchronous)
- `selectTextEngine(name)` — activate by name and return the engine
- `getTextEngineState(name)` / `getTextEngineDiagnostics()` — inspect engines
- `resolveEnginePreference()` — reads from URL params, localStorage, or `globalThis.__YGG_TEXT_ENGINE__`

### Stream adapter (`stream-adapter.ts`)

- `feedStreamFrame(buffer, frame)` — generic frame→buffer adapter supporting `start`, `chunk`, `progress`, `end`, `error`, `cancelled`, `timeout` frames. No model/agent semantics.
- Convenience constructors: `frameStart()`, `frameChunk()`, `frameProgress()`, `frameEnd()`, `frameError()`, `frameCancelled()`, `frameTimeout()`

## Optional Pretext Engine (Phase T3)

Phase T3 adds the optional Pretext engine with dynamic import and runtime feature flags:

### PretextTextEngine (`pretext-engine.ts`)

Implements `TextEngine` using `@chenglou/pretext` via dynamic import. Requires an async `initialize()` call before layout methods can be used. If the Pretext module is not installed, `initialize()` throws a diagnostic error that the registry catches for graceful fallback. The synchronous `getActiveTextEngine()` always returns a valid engine (fallback if Pretext is unavailable).

- `PretextTextEngine` class with async `initialize()`, sync layout methods
- `isPretextAvailable()` — check if the Pretext module was loaded
- `resetPretextLoadState()` — reset cached load state for retry

### Pretext bridge (`pretext-bridge.ts`)

Isolates the mapping between Ygg text-layout types and Pretext shapes. Provides `toPretextOptions()`, `fromPretextLayoutResult()`, `fromPretextLayoutLinesResult()`, `fromPretextLineStats()`, `fromPretextLineRange()`, and opaque handle bridging (`bridgePrepared`, `unbridgePrepared`, etc.). If the real module is unavailable, the type skeleton and adapter functions still compile.

### Pretext shim (`pretext-shim.ts`)

Local type definitions that mirror the `@chenglou/pretext` API surface (v0.0.7). Allows TypeScript compilation without the package installed. Defines `PretextModuleShape` interface for safe dynamic import casting.

### Engine configuration (`config.ts`)

Resolves the preferred text engine from multiple sources (priority order):

1. **URL param**: `?text-engine=fallback|pretext|auto` (highest, for testing)
2. **localStorage**: `ygg_text_engine`
3. **Global/env**: `globalThis.__YGG_TEXT_ENGINE__`
4. **Default**: `auto` (use Pretext if available, otherwise fallback)

Types: `TextEnginePreference` (`"auto" | "fallback" | "pretext"`), `TextEngineInitializationResult`.

### Async initialization (`registry.ts` T3 additions)

- `initializeTextEnginePreference()` — async: resolves preference, tries to load and activate Pretext, falls back gracefully. Records the result for diagnostics.
- `getInitializationResult()` — returns the last init result (preference, active engine, fallback reason, Pretext availability).
- `isPretextEngineAvailable()` — check if Pretext module was loaded.

### Assistant Drawer (T3 additions)

The Text Surface Proof now shows additional diagnostic badges:

- **Engine badge**: active engine name, version, state
- **Preference badge**: user preference (`auto`, `fallback`, `pretext`)
- **Pretext availability badge**: whether Pretext module is loaded
- **Fallback reason badge**: reason if preferred engine was not activated (tooltip shows full reason)

## Forge Text Preview (Phase T4)

Phase T4 adds optional text previews to the Forge Events and Proposals sections. These previews extract human-readable text from event payloads and proposal fields, showing it alongside (not replacing) the existing JSON inspectors.

### Text preview helper (`text-preview.ts`)

Extracts safe plain-text previews from:

- **Stream events**: `kernel/stream.chunk`, `kernel/stream.progress`, `kernel/stream.error`, `kernel/stream.cancelled`, `kernel/stream.timeout` — extracts `text`, `message`, `summary`, `reason`, or `content` fields depending on event kind.
- **Arbitrary payloads**: any event payload with common text fields (`text`, `message`, `summary`, `reason`, `content`) — shown when text is ≥ 40 characters.
- **Proposal fields**: `expected_effects` and `operations` — extracts long string fields (≥ 60 characters) from proposal data.

Functions:

- `extractEventPreview(eventKind, payload)` — returns `TextPreviewResult` with `hasPreview`, `text`, `kind`, `lineEstimate`, `heightEstimate`, `engineName`
- `extractProposalPreview(proposal)` — same shape, for proposal expected_effects/operations
- `kindBadgeLabel(kind)` — human-readable label for the preview source kind

The helper uses the active text engine (or fallback) for layout estimation with sensible defaults (560px max width, 20px line height, 14px Inter font).

### Forge Events (T4 additions)

Each event row now includes an optional `<details class="text-preview-details">` below the existing JSON `<code>`. When a stream payload or long text field is detected:

- **Preview text**: escaped plain text in a scrollable `<pre>` container
- **Line/height estimate**: badge showing `~N lines`, `~Npx`
- **Engine badge**: which engine was used for estimation
- **Kind badge**: the preview source (e.g. `stream:chunk`, `stream:error`, `text`)

The original JSON `<code>` is always preserved.

### Forge Proposals (T4 additions)

Each proposal row includes an optional `<details class="text-preview-details">` below the existing "Inspect proposal" JSON details. When proposal `expected_effects` or `operations` contain long string fields:

- Same preview layout as events (text, estimates, badges)
- Kind badge shows `effects` or `operations`

The original JSON inspector is always preserved.

### CSS additions (T4)

- `.text-preview-details` — collapsible details container, compact styling
- `.text-preview-panel` — inner panel with subtle border and dark background
- `.text-preview-meta` — flex row for badges (reuses `.text-proof-badge` from Assistant Drawer)
- `.text-preview-stage` — scrollable pre container for preview text (max-height 180px)
