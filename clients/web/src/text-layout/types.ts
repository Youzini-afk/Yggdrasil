/**
 * Text Surface Proof Alpha — Type definitions aligned with Pretext API shape.
 *
 * This module defines the public types used by the lightweight adapter/fallback
 * in `clients/web/src/text-layout`. It mirrors Pretext’s surface types so that
 * the adapter can be swapped for real Pretext later without changing callers.
 */

/** Opaque handle returned by prepareText(). */
export type PreparedText = {
  readonly __brand: "PreparedText";
};

/** Richer handle that exposes segment data for manual layout. */
export type PreparedTextWithSegments = PreparedText & {
  readonly segments: string[];
  readonly widths: number[];
  readonly letterSpacing: number;
};

/** Cursor within prepared segments. */
export type LayoutCursor = {
  segmentIndex: number;
  graphemeIndex: number;
};

/** Height + line count result. */
export type LayoutResult = {
  lineCount: number;
  height: number;
};

/** Per-line text + width + cursors. */
export type LayoutLine = {
  text: string;
  width: number;
  start: LayoutCursor;
  end: LayoutCursor;
};

/** Non-materialized line range. */
export type LayoutLineRange = {
  width: number;
  start: LayoutCursor;
  end: LayoutCursor;
};

/** Result with materialized lines. */
export type LayoutLinesResult = LayoutResult & {
  lines: LayoutLine[];
};

/** Line stats without string allocation. */
export type LineStats = {
  lineCount: number;
  maxLineWidth: number;
};

/** Options for prepareText(). */
export type PrepareOptions = {
  whiteSpace?: "normal" | "pre-wrap";
  wordBreak?: "normal" | "keep-all";
  letterSpacing?: number;
};

/** Font style descriptor used for measurement. */
export type FontDescriptor = string;

/** Streaming buffer state. */
export type StreamingBufferState = "idle" | "streaming" | "ended" | "reset";

/** Streaming text buffer that accumulates chunks and can re-layout. */
export type StreamingTextBuffer = {
  state: StreamingBufferState;
  text: string;
  prepared: PreparedTextWithSegments | null;
  maxWidth: number;
  lineHeight: number;
  font: FontDescriptor;
  append(chunk: string): void;
  end(): void;
  reset(): void;
  measure(): LayoutResult;
  layoutLines(): LayoutLinesResult;
  lineStats(): LineStats;
};
