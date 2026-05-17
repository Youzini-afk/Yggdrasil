// Text Surface Proof Alpha — Public exports for the lightweight text-layout adapter.
//
// This module re-exports types and fallback implementations aligned with the
// Pretext API shape. If Pretext is installed later, swap the implementations
// here without changing consumers.

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

export {
  clearAdapterCache,
  createStreamingBuffer,
  layoutPreparedText,
  layoutPreparedTextWithLines,
  measureLineStats,
  prepareText,
  prepareTextWithSegments,
  walkLineRanges,
} from "./adapter.js";

export { buildMockChunks, createMockChunkProducer, MOCK_STREAM_CHUNKS } from "./mock.js";
