# Yggdrasil web shell

> English only for now. The bilingual project entry point is in the repository root README.

Public-protocol Home/Play, Forge, and Assist shell for the current Platform Foundation Alpha surface.

This client is a plain TypeScript SPA built by Vite. It does not use React or another frontend framework for the shell itself.

- `Home/Play` is the launcher-first surface for package-discovered experiences.
- `Forge` is the creation and inspection surface for sessions, events, proposals, capabilities, surfaces, assets, projections, and package labs.
- `Assist` is a drawer that bridges lightweight play edits and deeper Forge work through approval-gated proposals.
- **Text Surface Proof (Phase T1)** is a client-side UI proof inside the Assistant Drawer. It demonstrates progressive mock-streaming text with live line/height estimates using a lightweight text-layout adapter (`src/text-layout`). This is not a kernel feature and does not depend on real model/agent calls.
- **Optional Text Engine (Phase T2)** adds a `TextEngine` interface, engine registry, and fallback engine implementation. The Assistant Drawer now shows the active engine name, version, and state. A generic stream-frame-to-buffer adapter (`stream-adapter.ts`) is available for wiring future stream sources.
- **Optional Pretext Engine (Phase T3)** adds an optional `PretextTextEngine` behind dynamic import, runtime engine selection with feature flags, and graceful fallback. The repo builds without installing `@chenglou/pretext`. The Assistant Drawer shows engine preference, Pretext availability, and fallback reason.
- **Forge Text Preview (Phase T4)** adds a text preview helper that extracts safe plain-text previews from event payloads, stream frames, and proposal-like objects in the Forge surface. Events and proposals show an optional `<details>` with preview text, line/height estimates, and engine name — without replacing the existing JSON inspectors. No model/agent semantics; Play unchanged.

This client uses only public host APIs:

- `POST /rpc`
- `GET /kernel/v1/event.subscribe/:session_id`

Run the host first:

```bash
cargo run -p ygg-cli -- host serve --http 127.0.0.1:8787 --profile profiles/forge-alpha.yaml
```

## Vite scripts

Run commands from the repository root with `--prefix clients/web`, or from this directory without the prefix.

```bash
npm run dev --prefix clients/web
npm run build --prefix clients/web
npm run check --prefix clients/web
npm run preview --prefix clients/web
```

- `npm run dev` starts the Vite dev server on `127.0.0.1:1420`.
- `npm run build` runs `tsc --noEmit` and writes the production bundle to `clients/web/dist/`.
- `npm run check` runs TypeScript without emitting files.
- `npm run preview` serves the built `dist/` bundle for local inspection.

For production, serve `clients/web/dist/` with a static web server or embed it through `clients/desktop`. This is intentionally not a final visual design or a content runtime.

## SurfaceHost

`src/surfaces/surface-host.ts` mounts third-party surface bundles in sandboxed iframes using `/surface-frame.html`. Surface bundles are ESM modules with a named export that is either callable as `(root, props) => void` or exposes `{ mount(root, props) }`.

The iframe uses `sandbox="allow-scripts"`; host access is opt-in through the explicit postMessage RPC bridge. See [`../../docs/guides/SURFACE_HOSTING.md`](../../docs/guides/SURFACE_HOSTING.md) for the full contract.

## ST URL layout (for ST extension compatibility)

YdlTavern surfaces serve SillyTavern-compatible ESM modules at standard ST URLs:

- `/script.js` — ST core globals shim
- `/scripts/extensions.js` — Extension manager shim
- `/scripts/events.js`, `/scripts/st-context.js`, `/scripts/group-chats.js`,
  `/scripts/secrets.js`, `/scripts/power-user.js`

These are served by the `ydltavern-st-compat-server` Vite plugin during dev,
reading from `../../YdlTavern/packages/ydltavern-surface/dist/st-compat/`.

Production hosting needs a static fileserver route (TODO Round 9).

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

- **Stream events**: `kernel/v1/stream.chunk`, `kernel/v1/stream.progress`, `kernel/v1/stream.error`, `kernel/v1/stream.cancelled`, `kernel/v1/stream.timeout` — extracts `text`, `message`, `summary`, `reason`, or `content` fields depending on event kind.
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

## SDK Extraction, Tests, and Hardening (Phase T5)

Phase T5 extracts reusable text-surface helpers into `sdk/typescript/text-surface`, adds font-loading helpers, cache diagnostics, and a lightweight self-test harness.

### text-surface SDK (`sdk/typescript/text-surface`)

A pure TypeScript frontend SDK for third-party UIs. No dependency on `clients/web` private modules. Types are self-contained (stable minimal shapes).

- `createTextSurfaceBuffer(font, lineHeight, maxWidth)` — streaming text accumulator with `append`, `end`, `reset` and lifecycle state tracking
- `applyStreamFrame(buffer, frame)` — generic frame→buffer adapter (mirrors `feedStreamFrame` with SDK-native types)
- `extractTextChunk(payload)` — safe plain-text extraction from arbitrary record objects
- `createScrollAnchor(buffer, options?)` — scroll-position anchor for streaming views (tracks offset and whether at tail)
- Frame constructors: `frameStart`, `frameChunk`, `frameProgress`, `frameEnd`, `frameError`, `frameCancelled`, `frameTimeout`

### Font loading helper (`font-helper.ts`)

Non-blocking font readiness checks using the browser Font Loading API:

- `ensureTextSurfaceFontLoaded(family, testText?)` — async: triggers font load and resolves when ready (non-fatal on failure)
- `describeFontLoadState(family)` — returns `FontLoadState` snapshot (`loaded`/`loading`/`unloaded`/`unsupported`)
- `ensureFontsLoaded(families)` — batch parallel font loading
- `describeFontLoadStates(families)` — batch state snapshots

In non-browser contexts, all fonts report `"unsupported"` so callers can skip font-dependent layout gracefully.

### Cache diagnostics

The `BoundedWidthCache` in `fallback-engine.ts` now exposes:

- `fontCount` — number of distinct font descriptors cached
- `maxEntries` — configured maximum cache entries
- `estimatedBytes` — rough memory estimate

Public function `getCacheDiagnostics()` returns a `CacheDiagnostics` snapshot (`totalEntries`, `fontCount`, `maxEntries`, `estimatedBytes`) for monitoring cache pressure.

### Self-test harness (`self-test.ts`)

Lightweight test harness that exercises the fallback engine, registry, stream adapter, and text preview with pure TS assertions. No external test framework required.

- `runTextLayoutSelfTest()` — runs all tests and returns `SelfTestResult[]`
- Call from browser console: `import { runTextLayoutSelfTest } from "./text-layout/self-test"; const results = runTextLayoutSelfTest(); console.table(results);`
- Tests cover: FallbackTextEngine construction/activation/layout, BoundedWidthCache diagnostics, Registry defaults/activation/selection, Stream adapter frame dispatch, Text preview extraction, Async initialization result

## Agent Observability (Phase J5)

Phase J5 adds a purely client-side observability layer for agent-like packages, surfaces, events, and proposals. No kernel/v1/protocol changes; no real model or network calls.

### Agent observability helper (`src/agent/observability.ts`)

Extracts agent-like observability using generic string heuristics (no hardcoded official package):

- **Package detection**: package ids containing `agent`, `pi-agent`, `tool-bridge`, `trace`, `run`, or `assistant`
- **Surface detection**: surface ids/slots/titles containing the same hints
- **Signal detection**:
  - `runSignals`: trace-like events (kind/payload containing `trace`, `tool`, `run`, `proposal`, or payload fields `trace_events`, `tool_calls`, `stream_frames`, `proposal_draft`)
  - `toolSignals`: tool bridge events (kind containing `tool_bridge`, `tool-bridge`, or payload method `kernel.v1.capability.invoke/stream`, or payload fields `tool_calls` / `tool_bridge_plan`)
  - `streamSignals`: stream lifecycle events (`kernel/v1/stream.*`)
  - `proposalSignals`: proposals from agent-like packages or with trace-like `expected_effects`
- **Safety badges**: inferred from payloads — `ambiguous provider`, `rejected`, `missing provider`, `permission denied`, `audit/redaction`

Functions:

- `buildAgentObservability(packages, allSurfaces, events, proposals, capabilities)` → `AgentObservabilityModel`
- `renderAgentObservabilitySection(model, events, proposals)` → HTML string for Forge surface
- `renderAgentReadinessPanel(agentSurfaces, agentCapabilities)` → HTML string for Assistant Drawer
- `filterAgentLikeCapabilities(capabilities)` → agent-like capability subset

### Forge surface (J5 additions)

An "Agent Observability" section is rendered after Proposals and before Events:

- **Cards/summary**: counts for agent packages, agent surfaces, run/tool/proposal/stream signals
- **Safety badges row**: colored badges for ambiguous/rejected/provider/permission/redaction states
- **Trace timeline**: latest package-owned trace/tool/run signals with kind, sequence, writer, and JSON payload preview + optional T4 text preview
- **Proposal explanations**: agent-like proposals rendered with T4 text preview (expected_effects/operations), alongside the existing JSON inspector

### Assistant Drawer (J5 additions)

A lightweight "Agent Readiness" panel is shown above Host diagnostics:

- **Readiness indicator**: `●` when agent-like surfaces/capabilities detected, `○` otherwise
- **Counters**: agent-like surfaces and capabilities counts
- **Note**: emphasizes no real model, no network, proposal-gated, plan-only behavior
- **Placeholder buttons**: "Start agent" and "Run tool plan" are disabled (template only — no real agent loop or model call)

### CSS additions (J5)

- `.agent-observability-section` — Forge section spacing
- `.agent-metric` — metric card accent color
- `.safety-badge-row` / `.safety-badge` / `.severity-{ok,warn,error,info}` — diagnostics badges
- `.agent-timeline` / `.timeline-list` / `.timeline-row` / `.timeline-meta` / `.timeline-badge` / `.timeline-seq` / `.timeline-kind` / `.timeline-writer` / `.timeline-payload` — trace timeline
- `.agent-proposal-list` — proposal explanations list
- `.agent-readiness-panel` / `.agent-readiness-header` / `.agent-readiness-badge` / `.agent-readiness-title` / `.agent-readiness-body` / `.agent-readiness-note` / `.agent-readiness-actions` — Assistant Drawer readiness panel

## Forge Agent Workspace / Observability UI Shell (Phase E)

Phase E upgrades the Forge surface from "agent observability proof" to the first version of an Agentic Forge control room. The UI is product-architecture scaffolding — not final visual design — structured, Ygg-native, and deliberately not chat-first.

### Agentic Forge Workspace sections

Six new workspace panels appear in a dedicated `.forge-workspace-section` within the Forge surface:

1. **Run Timeline** — Detects run lifecycle events (`run.*`, `lifecycle_state`, `working_state`) from package-owned events. Shows run status, package ID, node/edge counts, and working state fields.
2. **Plan Graph (read-only)** — Detects plan graph node events (`plan_node`, `plan_graph`, node kinds like `observe`, `infer`, `tool_call`, `inspect`, `branch_op`, `compare`, `propose`, `wait`). Shows node kind badges, input/output refs, and approval policy.
3. **Branch Diff / Lineage Panel** — Detects branch policy events (`scratch_branch`, `target_branch`, `branch_policy`, `fork`, `stale`). Shows branch type (scratch/target/fork/lineage), revision, intent, and stale/promote-requires-proposal badges.
4. **Candidate Compare / Promote Panel** — Detects candidate-like proposals and events (`candidate_id`, `candidate_seed`, `create_candidate`, `compare_candidate`). Shows candidate status, target/scratch branches, diff summary, confidence/uncertainty, changed asset refs, and inspection refs.
5. **Tool / Inference Trace Panel** — Two sub-panels: tool traces (`tool_call`, `tool_observation`, `tool_risk`, `tool_bridge`) and inference traces (`inference`, `provider_kind`, `model_performed`, `network_performed`, `replay`). Shows plan/observation/risk chips and replay match/mismatch badges.
6. **Controls** — Approval/reject/cancel/promote/fork/archive affordances. Live actions for current state; disabled-safe affordances with public-protocol payload previews explaining what payload shape a third-party agentic-forge package would expect.

### Key design decisions

- **No chat-first UI**: The Forge workspace object model is run/plan/candidate/diff/proposal/trace — not chat history or prompt boxes. The existing Assistant Drawer remains as a lightweight entry point.
- **Public protocol only**: All data is heuristically extracted from public protocol events, proposals, surfaces, capabilities, packages, assets, and projections. No kernel internals, no real model/network calls, no direct SQLite access.
- **Third-party replaceable**: All panels carry protocol-shape documentation (collapsible `<details>` with JSON examples) and text emphasizing that any third-party agentic-forge-lab package can drive these panels. No official package hardcoding.
- **Disabled-safe affordances**: Control actions that have no current target are shown as disabled affordances with their expected protocol payload, making the protocol contract visible even when no agent package is loaded.

### Forge Agent Workspace view model (`src/agent/observability.ts`)

New types and functions:

- `ForgeAgentWorkspaceModel` — top-level view model containing `runs`, `planNodes`, `branchEntries`, `candidates`, `toolTraces`, `inferenceTraces`, `controlActions`
- `RunTimelineEntry`, `PlanGraphNode`, `BranchLineageEntry`, `CandidateCard`, `ToolTraceEntry`, `InferenceTraceEntry`, `ControlAction` — typed view models derived from heuristics on public protocol data
- `buildForgeAgentWorkspace(events, proposals, capabilities, packages, assets, projections)` → `ForgeAgentWorkspaceModel`
- `renderForgeAgentWorkspaceSections(model)` → HTML string for the Forge surface

### Forge surface (Phase E additions)

The Forge surface now includes:

- A `.forge-workspace-section` block rendered after the existing Agent Observability section, before Events
- Six workspace panels (details/summary), each with collapsible protocol-shape documentation
- A controls section with live and disabled-safe action affordances

### CSS additions (Phase E)

- `.forge-workspace-section` — dedicated section with purple-tinted border and dark background
- `.workspace-note` — explanatory text for third-party replaceability
- `.forge-workspace-grid` / `.workspace-panel` / `.workspace-panel-header` / `.workspace-panel-body` — collapsible panel layout
- `.run-entry` / `.plan-node-entry` / `.branch-entry` / `.candidate-entry` / `.trace-entry` / `.control-action-entry` — entry card styles
- `.run-status-dot.status-{ok,error,warn,info}` — colored status indicators
- `.run-label`, `.run-meta-item`, `.run-meta-label` — typography helpers
- `.plan-node-kind-badge` — node kind pill badge
- `.branch-type-icon` — branch type visual indicator
- `.candidate-diff` / `.candidate-stats` / `.candidate-refs` — candidate detail sections
- `.trace-chip` — tool/inference capability indicator chips
- `.control-action-icon` / `.control-action-reason` — control affordance styling
- `.button-disabled-safe` — dashed-border disabled button for protocol preview affordances
- `.protocol-preview-details` / `.protocol-preview-summary` / `.protocol-preview-code` — collapsible protocol shape documentation
- `.workspace-controls-panel` — controls panel with top margin

## Experience Observability (Beta 3)

Phase Beta 3 adds a set of public-protocol-only observability panels to the Forge surface. These panels give users and creators insight into session health, package health, agent run health, proposal causal chains, failure breadcrumbs, cost/latency summaries, asset provenance, and guardrail/audit summaries — all derived without kernel internals, model/network calls, or SQLite access.

### Experience observability helper (`src/agent/experience-observability.ts`)

New view model types:

- `ExperienceObservabilityModel` — top-level view model containing all sub-panels
- `SessionHealth` — session id, status (active/forked/closed/unknown), event count/range, duration, fork count
- `PackageHealth` — per-package state, entry kind, capability/event/surface counts, last active sequence
- `AgentRunHealth` — run id, label, status, package, node/edge counts, duration, start/end sequence, failure flag/reason
- `ProposalCausalChain` — proposal id, status, parent/child proposal ids, operation count, event sequence, derivation source
- `FailureBreadcrumb` — event/proposal/inference failures with severity, kind, package, reason, related run/proposal
- `CostLatencySummary` — total events, symbolic estimated cost, latency, inference/tool/stream/proposal breakdown
- `AssetProvenanceSummary` — asset id, origin package, mime, size, proposal/approval trail, run references, hash
- `GuardrailAuditSummary` — total guardrail checks, blocked/warning/passed/redaction/ambiguous/rejection/permission counts, detailed guardrail log

Functions:

- `buildExperienceObservability(events, proposals, packages, capabilities, allSurfaces, assets, sessionId?)` → `ExperienceObservabilityModel`
- `renderExperienceObservabilitySection(model)` → HTML string for the Forge surface

### Forge surface (Beta 3 additions)

The Forge surface now includes an "Experience Observability (Beta 3)" section rendered after the Agentic Forge Workspace and before Events. It contains:

1. **Session Health** — session status indicator, event range, duration, fork count. Public protocol shape documented in collapsible details.
2. **Package Health** — per-package cards showing state, entry kind, capability/event/surface counts, last active event sequence.
3. **Agent Run Health** — per-run cards with status dot, node/edge counts, duration, failure reason highlight when detected.
4. **Proposal Causal Chain** — proposals linked by parent/child relationships showing causal flow through the session. Root proposals and child links are visually distinguished.
5. **Failure Breadcrumbs** — chronologically sorted failure events from event payloads and proposal rejections. Each entry shows severity badge, source (event/proposal/inference), kind, package, and reason code.
6. **Cost / Latency Summary** — metric grid with total events, inference/tool/stream/proposal counts, estimated latency, and symbolic cost estimate. Breakdown by category.
7. **Asset Provenance** — per-asset cards showing origin, mime, size, proposal/approval trail, run references, and hash. Assets without proposal trails are flagged.
8. **Guardrail / Audit Summary** — aggregated guardrail metrics (blocked/warning/passed/redaction/ambiguous/rejection/permission) plus detailed guardrail log with verdict badges.

### Key design decisions (Beta 3)

- **No chat-first UI**: Panels show observability data (session state, run health, causal chains, breadcrumbs, cost, provenance, guardrails) — not chat history, assistant messages, or prompt boxes.
- **Public protocol only**: All data is heuristically extracted from public protocol events, proposals, packages, surfaces, and assets. No kernel internals, no model/network calls, no direct SQLite access, no runtime private modules.
- **Mock/protocol-shaped data by default**: When no packages emit relevant events, panels show explanatory empty states describing what protocol shapes would populate them. No hardcoded official package ids.
- **Symbolic cost estimation**: Cost/latency values are estimated from observable event counts and timestamp spreads. No real model calls are made — all inference is proposal-gated and plan-only.
- **Proposal-driven causality**: Causal chains are built from proposal target_session_id groupings and event payload references to parent/child proposal ids. This reflects the Yggdrasil architecture where proposals are the authoritative record of change.
- **No runtime internals**: The builder functions accept only public protocol types (`KernelEvent`, `ProposalRecord`, `PackageRecord`, etc.) and never import from runtime crates or access SQLite.

### CSS additions (Beta 3)

- `.experience-observability-section` — section with blue-tinted border and dark background
- `.phase-badge` — small badge for "Beta 3" phase label
- `.exp-obs-grid` / `.exp-obs-panel` / `.exp-obs-panel-wide` / `.exp-obs-panel-header` / `.exp-obs-panel-body` — collapsible panel layout
- `.exp-obs-metrics` / `.exp-obs-metric` — metric grid with centered values
- `.exp-obs-list` / `.exp-obs-entry` / `.exp-obs-entry-header` / `.exp-obs-entry-meta` — list entry cards
- `.exp-obs-entry-error` — error-highlighted entry border/background
- `.exp-obs-cost-note` — cost estimate note with amber code color
- `.exp-obs-breakdown` / `.exp-obs-breakdown-list` / `.exp-obs-breakdown-item` — cost breakdown rows
- `.exp-obs-asset-runs` — asset run reference chips
- `.exp-obs-failure-reason` — failure reason block with red tint
- `.exp-obs-breadcrumb-list` / `.exp-obs-breadcrumb-entry` / `.exp-obs-breadcrumb-header` / `.exp-obs-breadcrumb-reason` / `.exp-obs-breadcrumb-meta` — breadcrumb entry layout
- `.exp-obs-causal-chain` / `.exp-obs-chain-entry` / `.exp-obs-chain-header` / `.exp-obs-chain-links` / `.exp-obs-chain-link` — causal chain layout
- `.exp-obs-guardrail-entries` — guardrail log container
- `.verdict-block` / `.verdict-warn` / `.verdict-pass` — guardrail verdict border accents

## Experience Beta 5 — Forge Creator Loop UI

Phase Beta 5 adds a set of public-protocol-only creator loop panels to the Forge surface. These panels scaffold the authoring workflow: creator readiness assessment, template-to-playable guidance, package diagnostics explainability, composition readiness verification, fixture/reload controls with payload previews, and a replacement & permission checklist — all derived without runtime internals, privileged Studio, marketplace/monetization UI, or SQLite access.

### Creator loop view model (`src/agent/creator-loop.ts`)

New view model types:

- `CreatorLoopModel` — top-level view model containing all sub-panels
- `CreatorReadiness` — overall readiness state (ready/almost_ready/needs_work/unknown) with package/capability/surface/experience counts, missing pieces, and recommendations
- `TemplateRecommendation` — template card with label, description, template type, suggested-for context, CLI commands, and prerequisites
- `PackageDiagnosticCard` — per-package diagnostics showing state, entry kind, issues (error/warn/info with codes), strengths, and a human-readable summary
- `DiagnosticIssue` — individual issue with severity, message, and optional diagnostic code
- `CompositionReadiness` — composition state (valid/incomplete/invalid/unknown), package list, missing packages, surface slots, uncovered slots, and check result text
- `FixtureControl` — fixture/reload control with action type (check/conformance/run-fixture/reload), disabled state, disabled reason, and public-protocol payload preview
- `ReplacementChecklistItem` — checklist item with category, label, status (ok/warn/error/info), detail, and optional action-needed guidance
- `WalkthroughStep` — walkthrough step with order, title, description, status (complete/in_progress/pending/skipped), optional action, and optional CLI command

Functions:

- `buildCreatorLoopModel(packages, capabilities, allSurfaces, events, proposals, assets, projections, sessionId?)` → `CreatorLoopModel`
- `renderCreatorLoopSection(model)` → HTML string for the Forge surface

### Forge surface (Beta 5 additions)

The Forge surface now includes a "Creator Loop (Beta 5)" section rendered after Experience Observability (Beta 3) and before Events. It contains:

1. **Creator Readiness** — Overall readiness indicator with package/capability/surface/experience entry counts, session status dot, missing pieces list, and recommendations. Shows green "ready" when all pieces are present and a session is active; amber/red states guide the creator toward what's missing.

2. **Template to Playable** — An 8-step walkthrough from "Initialize Package" through "Launch Experience," with each step's status (complete/in_progress/pending) detected from current workspace state. Includes a "Recommended Templates" section that suggests template types not yet present (e.g., suggests "Experience Package" when no `experience_entry` surface is detected, "Forge Panel" when no `forge_panel` slot is covered). Each recommendation shows description, CLI commands, and prerequisites.

3. **Package Diagnostics Explainability** — Per-package cards that explain why each package is in its current state. Diagnostics include state-based errors (loaded/activated/error), missing capabilities, missing surfaces, high-risk permission warnings, and entry-kind concerns. Strengths are listed alongside issues for balanced assessment. Each card has a human-readable summary.

4. **Composition Readiness** — Evaluates the structural completeness of the loaded workspace as a "composition." Checks for required surface slots (`experience_entry`), missing package references, uncovered recommended slots (`forge_panel`, `assistant_action`), and overall validity. Shows per-composition status badge and detailed slot/package breakdown.

5. **Fixture / Reload Controls** — Per-package control affordances for check, conformance, run-fixture, and reload actions. Each control shows a public-protocol payload preview (expected method, params, and response shape) — no actual runtime invocation. Disabled-safe stubs are provided when no packages are loaded, with explanatory reasons. Controls document what a third-party fixture/lab package would consume.

6. **Replacement & Permission Checklist** — Comprehensive checklist organized by category: Package Health, Surface Coverage, Permissions, Session & Events, and Assets & Projections. Each item shows status, detail, and actionable guidance. Covers package health (errors, missing capabilities, missing surfaces), surface coverage (experience entries, forge panels, assistant actions), permissions (high-risk review, approval policies), session state (events flowing, proposal tracking), and asset/projection coverage.

### Key design decisions (Beta 5)

- **No chat-first UI**: Panels show creator loop data (readiness, walkthrough, diagnostics, composition, fixtures, checklist) — not chat history, assistant messages, or prompt boxes.

- **Public protocol only**: All data is heuristically extracted from public protocol packages, capabilities, surfaces, events, proposals, assets, and projections. No kernel internals, no model/network calls, no direct SQLite access, no runtime private modules.

- **No marketplace/monetization UI**: No pricing, licensing, publishing, storefront, or distribution panels. The checklist explicitly focuses on technical readiness, not commercial readiness.

- **No privileged Studio**: All panels work with the same public protocol data available to any third-party package. No special access to runtime internals.

- **Example package IDs as public descriptor samples**: Template recommendations use example ids (`example/my-package`, `example/my-experience`) — never hardcoded official package ids or privileges.

- **Disabled-safe affordances with protocol documentation**: Fixture controls and checklist items carry public-protocol payload previews and explanatory reasons. When no target package is loaded, disabled stubs show the expected protocol shape.

- **Third-party replaceable walkthrough**: The walkthrough steps are derived from current workspace state — any package that contributes the right surfaces and capabilities drives the step completion detection. No official package hardcoding.

- **No chat interface**: The creator loop is a diagnostic and guidance surface, not a conversational assistant. The existing Assistant Drawer remains as a lightweight entry point for quick actions.

### CSS additions (Beta 5)

- `.creator-loop-section` — section with purple-tinted border and dark background
- `.cloop-grid` / `.cloop-panel` / `.cloop-panel-wide` / `.cloop-panel-header` / `.cloop-panel-body` — collapsible panel layout
- `.cloop-readiness-badge` — readiness indicator badge with severity color classes
- `.cloop-metrics` / `.cloop-metric` — metric grid with centered values
- `.cloop-missing-section` / `.cloop-issue-list` — missing pieces section
- `.cloop-recommendations` / `.cloop-rec-list` — recommendations list
- `.cloop-walkthrough` / `.cloop-step-list` / `.cloop-step-entry` / `.cloop-step-header` / `.cloop-step-number` / `.cloop-step-icon` / `.cloop-step-title` / `.cloop-step-badge` / `.cloop-step-desc` / `.cloop-step-command` — walkthrough step layout with status variants
- `.cloop-template-section` / `.cloop-template-grid` / `.cloop-template-card` / `.cloop-template-header` / `.cloop-template-desc` / `.cloop-template-commands` — template recommendation cards
- `.cloop-diagnostics-list` / `.cloop-diag-entry` / `.cloop-diag-header` / `.cloop-diag-summary` / `.cloop-diag-section` / `.cloop-strength-list` — package diagnostic entry layout
- `.cloop-checklist-item` / `.cloop-checklist-header` / `.cloop-checklist-icon` / `.cloop-checklist-detail` / `.cloop-checklist-action` — shared checklist item with severity colors
- `.cloop-checklist-categories` / `.cloop-checklist-category` / `.cloop-checklist-items` — checklist category grouping
- `.cloop-composition-list` / `.cloop-composition-entry` / `.cloop-composition-header` / `.cloop-composition-detail` / `.cloop-composition-meta` / `.cloop-composition-result` — composition readiness layout
- `.cloop-fixture-section` / `.cloop-fixture-list` / `.cloop-fixture-entry` / `.cloop-fixture-header` / `.cloop-fixture-icon` / `.cloop-fixture-desc` / `.cloop-fixture-reason` / `.cloop-fixture-action` — fixture control layout with disabled-safe variant
- `.cloop-disabled-safe` — faded opacity for disabled fixture controls
- Status modifier classes: `.status-complete`, `.status-in_progress` on step entries for border/background tinting
