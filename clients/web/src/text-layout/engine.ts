/**
 * Text Surface Proof Alpha — TextEngine interface and configuration types.
 *
 * This module defines the abstract TextEngine interface that concrete engines
 * (FallbackTextEngine, future PretextEngine, etc.) must implement. It also
 * provides configuration types for engine registration and selection.
 */

import type {
  FontDescriptor,
  LayoutLineRange,
  LayoutLinesResult,
  LayoutResult,
  LineStats,
  PreparedText,
  PreparedTextWithSegments,
  PrepareOptions,
  StreamingTextBuffer,
} from "./types.js";

// --- Engine name type ---

/** Well-known engine names. Extend with custom string for third-party engines. */
export type TextEngineName = "fallback" | (string & {});

// --- Engine config types ---

/** Base configuration shared by all engine registrations. */
export type EngineConfig = {
  /** Human-readable engine name for diagnostics. */
  readonly name: TextEngineName;
  /** Semantic version of the engine implementation. */
  readonly version: string;
  /** Optional description for diagnostics / UI. */
  readonly description?: string;
};

/** Configuration specific to TextEngine registration. */
export type TextEngineConfig = EngineConfig & {
  /** Whether this engine should be the default active engine (first registered wins if none specified). */
  readonly default?: boolean;
  /** Maximum width-cache entries this engine may hold (0 = unlimited). */
  readonly maxCacheEntries?: number;
};

/** Runtime state of a registered engine. */
export type TextEngineState = "ready" | "active" | "error" | "unavailable";

/** Diagnostic snapshot of a registered engine. */
export type TextEngineDiagnostics = {
  readonly name: TextEngineName;
  readonly version: string;
  readonly state: TextEngineState;
  readonly isFallback: boolean;
  readonly description?: string;
  readonly error?: string;
};

// --- TextEngine interface ---

/**
 * Abstract text layout engine interface.
 *
 * All concrete engines (fallback, Pretext, etc.) must implement this interface.
 * The registry selects the active engine at runtime; consumers call through the
 * active engine instance without knowing which implementation they use.
 */
export interface TextEngine {
  /** Engine identity and config. */
  readonly config: TextEngineConfig;

  /** Current runtime state. */
  readonly state: TextEngineState;

  /**
   * Prepare text for layout measurement.
   * Returns an opaque handle that can be reused across multiple layout calls.
   */
  prepareText(text: string, font: FontDescriptor, options?: PrepareOptions): PreparedText;

  /**
   * Prepare text with visible segment data for manual line layout.
   */
  prepareTextWithSegments(text: string, font: FontDescriptor, options?: PrepareOptions): PreparedTextWithSegments;

  /**
   * Compute line count and total height from a prepared handle.
   */
  layoutPreparedText(prepared: PreparedText, maxWidth: number, lineHeight: number): LayoutResult;

  /**
   * Compute line count, total height, and materialized line strings.
   */
  layoutPreparedTextWithLines(
    prepared: PreparedTextWithSegments,
    maxWidth: number,
    lineHeight: number,
  ): LayoutLinesResult;

  /**
   * Compute line count and max line width without allocating line strings.
   */
  measureLineStats(prepared: PreparedTextWithSegments, maxWidth: number): LineStats;

  /**
   * Walk line ranges without materializing line strings.
   * Returns total line count.
   */
  walkLineRanges(
    prepared: PreparedTextWithSegments,
    maxWidth: number,
    onLine: (line: LayoutLineRange) => void,
  ): number;

  /**
   * Create a streaming text buffer that accumulates chunks and can re-layout.
   */
  createStreamingBuffer(
    font: FontDescriptor,
    lineHeight: number,
    maxWidth: number,
  ): StreamingTextBuffer;

  /**
   * Clear internal caches (e.g. width cache on font changes).
   */
  clearCache(): void;
}
