/**
 * Optional Text Engine Alpha — Type-safe module shape for @chenglou/pretext.
 *
 * Defines the expected Pretext module shape as local types so that TypeScript
 * compiles without the package installed. At runtime, the module is loaded via
 * dynamic import and cast to the PretextModuleShape interface.
 *
 * This is NOT a declare module augmentation — it's a standalone type definition
 * that mirrors the Pretext API surface at version 0.0.7.
 */

// --- Pretext type definitions (mirrors @chenglou/pretext 0.0.7) ---

/** Opaque prepared handle returned by Pretext's prepare(). */
export type PretextPrepared = {
  readonly __brand: "PretextPrepared";
};

/** Prepared handle with segment data. */
export type PretextPreparedWithSegments = PretextPrepared & {
  readonly segments: string[];
  readonly widths: number[];
  readonly letterSpacing: number;
};

/** Layout result from Pretext. */
export type PretextLayoutResult = {
  readonly lineCount: number;
  readonly height: number;
};

/** Layout with materialized lines. */
export type PretextLayoutLinesResult = PretextLayoutResult & {
  readonly lines: PretextLayoutLine[];
};

/** A single layout line from Pretext. */
export type PretextLayoutLine = {
  readonly text: string;
  readonly width: number;
};

/** Line stats (non-allocating). */
export type PretextLineStats = {
  readonly lineCount: number;
  readonly maxLineWidth: number;
};

/** Line range for walking. */
export type PretextLayoutLineRange = {
  readonly width: number;
  readonly start: { readonly segmentIndex: number; readonly graphemeIndex: number };
  readonly end: { readonly segmentIndex: number; readonly graphemeIndex: number };
};

/** Options for prepare/prepareWithSegments. */
export type PretextOptions = {
  readonly whiteSpace?: "normal" | "pre-wrap";
  readonly wordBreak?: "normal" | "keep-all";
  readonly letterSpacing?: number;
};

// --- Module shape interface (for dynamic import casting) ---

/**
 * The expected shape of the @chenglou/pretext module.
 * Used to type the result of dynamic import.
 */
export interface PretextModuleShape {
  prepare(text: string, font: string, options?: PretextOptions): PretextPrepared;
  layout(prepared: PretextPrepared, maxWidth: number, lineHeight: number): PretextLayoutResult;
  prepareWithSegments(text: string, font: string, options?: PretextOptions): PretextPreparedWithSegments;
  layoutWithLines(prepared: PretextPreparedWithSegments, maxWidth: number, lineHeight: number): PretextLayoutLinesResult;
  measureLineStats(prepared: PretextPreparedWithSegments, maxWidth: number): PretextLineStats;
  walkLineRanges(prepared: PretextPreparedWithSegments, maxWidth: number, onLine: (line: PretextLayoutLineRange) => void): number;
}

/**
 * Dynamic import specifier for @chenglou/pretext.
 * Uses a string constant so the import path is easy to update.
 */
export const PRETEXT_MODULE_SPECIFIER = "@chenglou/pretext";
