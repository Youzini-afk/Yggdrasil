// Text Surface Proof Alpha — Public exports for the lightweight text-layout adapter.
//
// This module re-exports types, fallback implementations, engine abstraction,
// registry, and stream-adapter. If Pretext is installed later (T3), swap the
// active engine via the registry without changing consumers.

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

// --- Engine types (new in T2) ---
export type {
  TextEngine,
  TextEngineName,
  EngineConfig,
  TextEngineConfig,
  TextEngineState,
  TextEngineDiagnostics,
} from "./engine.js";

// --- Stream adapter types (new in T2) ---
export type {
  StreamFrameKind,
  StreamFrame,
  FeedResult,
} from "./stream-adapter.js";

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

// --- Fallback engine class (new in T2) ---
export { FallbackTextEngine, FALLBACK_ENGINE_CONFIG } from "./fallback-engine.js";

// --- Registry (new in T2) ---
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
} from "./registry.js";

// --- Stream adapter (new in T2) ---
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

// --- Mock (unchanged) ---
export { buildMockChunks, createMockChunkProducer, MOCK_STREAM_CHUNKS } from "./mock.js";
