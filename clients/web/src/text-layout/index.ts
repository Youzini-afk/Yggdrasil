// Text Surface Proof Alpha — Public exports for the lightweight text-layout adapter.
//
// This module re-exports types, fallback implementations, engine abstraction,
// registry, stream-adapter, T3 optional Pretext engine support, and T4 text preview.

// --- Types (unchanged) ---
export type {
  FontDescriptor,
  LayoutCursor,
  LayoutLine,
  LayoutLineRange,
  LayoutLinesResult,
  LayoutResult,
  LineStats,
  PreparedText,
  PreparedTextWithSegments,
  PrepareOptions,
  StreamingBufferState,
  StreamingTextBuffer,
} from "./types.js";

// --- Engine types (T2) ---
export type {
  TextEngine,
  TextEngineName,
  EngineConfig,
  TextEngineConfig,
  TextEngineState,
  TextEngineDiagnostics,
} from "./engine.js";

// --- Stream adapter types (T2) ---
export type {
  StreamFrameKind,
  StreamFrame,
  FeedResult,
} from "./stream-adapter.js";

// --- T3: Config and preference types ---
export type {
  TextEnginePreference,
  TextEngineInitializationResult,
} from "./config.js";
export {
  resolveTextEnginePreference,
  parseTextEnginePreference,
  preferenceToEngineName,
  persistTextEnginePreference,
  ENGINE_PREFERENCE_VALUES,
} from "./config.js";

// --- T3: Pretext shim types (for downstream type use) ---
export type {
  PretextPrepared,
  PretextPreparedWithSegments,
  PretextLayoutResult,
  PretextLayoutLinesResult,
  PretextLayoutLine,
  PretextLineStats,
  PretextLayoutLineRange,
  PretextOptions,
  PretextModuleShape,
} from "./pretext-shim.js";

// --- T3: Pretext bridge ---
export {
  bridgePrepared,
  bridgePreparedWithSegments,
  isBridgedPretextPrepared,
  isBridgedPretextPreparedWithSegments,
  unbridgePrepared,
  unbridgePreparedWithSegments,
  toPretextOptions,
  fromPretextLayoutResult,
  fromPretextLayoutLinesResult,
  fromPretextLineStats,
  fromPretextLineRange,
} from "./pretext-bridge.js";
export type {
  BridgedPretextPrepared,
  BridgedPretextPreparedWithSegments,
} from "./pretext-bridge.js";

// --- T3: Pretext engine ---
export {
  PretextTextEngine,
  PRETEXT_ENGINE_CONFIG,
  isPretextAvailable,
  getPretextLoadError,
  resetPretextLoadState,
} from "./pretext-engine.js";

// --- Backward-compatible functions (re-exported from fallback-engine) ---
export {
  clearAdapterCache,
  createStreamingBuffer,
  layoutPreparedText,
  layoutPreparedTextWithLines,
  measureLineStats,
  prepareText,
  prepareTextWithSegments,
  walkLineRanges,
} from "./fallback-engine.js";

// --- Fallback engine class (T2) ---
export { FallbackTextEngine, FALLBACK_ENGINE_CONFIG } from "./fallback-engine.js";

// --- Registry (T2 + T3) ---
export {
  registerTextEngine,
  activateTextEngine,
  getActiveTextEngine,
  getActiveTextEngineName,
  getTextEngineState,
  selectTextEngine,
  getTextEngineDiagnostics,
  unregisterTextEngine,
  resolveEnginePreference,
  // T3 additions:
  initializeTextEnginePreference,
  getInitializationResult,
  isPretextAvailable as isPretextEngineAvailable,
  getPretextAvailabilityError,
} from "./registry.js";

// --- Stream adapter (T2) ---
export {
  feedStreamFrame,
  frameStart,
  frameChunk,
  frameProgress,
  frameEnd,
  frameError,
  frameCancelled,
  frameTimeout,
} from "./stream-adapter.js";

// --- T4: Text preview helper ---
export type {
  TextPreviewKind,
  TextPreviewResult,
} from "./text-preview.js";
export {
  extractEventPreview,
  extractProposalPreview,
  kindBadgeLabel,
} from "./text-preview.js";

// --- Mock (unchanged) ---
export { buildMockChunks, createMockChunkProducer, MOCK_STREAM_CHUNKS } from "./mock.js";
