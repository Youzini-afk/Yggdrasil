/**
 * Optional Text Engine Alpha — Bridge between Ygg text-layout types and Pretext shapes.
 *
 * Provides isolated mapping functions so that the PretextTextEngine does not
 * directly couple to either Ygg internals or Pretext internals. If the real
 * @chenglou/pretext module is not available, the type skeleton and adapter
 * functions here still compile and allow graceful fallback.
 */

import type {
  FontDescriptor,
  LayoutLine,
  LayoutLineRange,
  LayoutLinesResult,
  LayoutResult,
  LineStats,
  PreparedText,
  PreparedTextWithSegments,
  PrepareOptions,
} from "./types.js";

import type {
  PretextPrepared,
  PretextPreparedWithSegments,
  PretextLayoutResult,
  PretextLayoutLinesResult,
  PretextLineStats,
  PretextLayoutLineRange,
  PretextOptions,
} from "./pretext-shim.js";

// --- Option mapping (Ygg → Pretext) ---

/**
 * Convert Ygg PrepareOptions to Pretext PretextOptions.
 * The shapes are identical by design; this function provides an explicit
 * mapping boundary so either side can diverge independently.
 */
export function toPretextOptions(options?: PrepareOptions): PretextOptions | undefined {
  if (!options) return undefined;
  return {
    whiteSpace: options.whiteSpace,
    wordBreak: options.wordBreak,
    letterSpacing: options.letterSpacing,
  };
}

// --- Result mapping (Pretext → Ygg) ---

/**
 * Map a Pretext PretextLayoutResult to Ygg LayoutResult.
 */
export function fromPretextLayoutResult(result: PretextLayoutResult): LayoutResult {
  return {
    lineCount: result.lineCount,
    height: result.height,
  };
}

/**
 * Map a Pretext PretextLayoutLinesResult to Ygg LayoutLinesResult.
 * This requires converting PretextLayoutLine (no cursors) to Ygg LayoutLine
 * (with cursors). Since Pretext lines don't expose cursors, we synthesize
 * sequential cursors from the line data.
 */
export function fromPretextLayoutLinesResult(
  result: PretextLayoutLinesResult,
  prepared: PretextPreparedWithSegments,
): LayoutLinesResult {
  const lines: LayoutLine[] = [];
  let segIdx = 0;
  let graphIdx = 0;

  for (const line of result.lines) {
    const start = { segmentIndex: segIdx, graphemeIndex: graphIdx };
    // Advance cursor through segments/characters for this line
    const graphemes = Array.from(line.text);
    let remaining = graphemes.length;

    // Walk segments to find how far we advance
    while (remaining > 0 && segIdx < prepared.segments.length) {
      const segGraphemes = Array.from(prepared.segments[segIdx]!).length - graphIdx;
      if (segGraphemes <= remaining) {
        remaining -= segGraphemes;
        segIdx++;
        graphIdx = 0;
      } else {
        graphIdx += remaining;
        remaining = 0;
      }
    }

    const end = { segmentIndex: segIdx, graphemeIndex: graphIdx };
    lines.push({
      text: line.text,
      width: line.width,
      start,
      end,
    });
  }

  return {
    lineCount: result.lineCount,
    height: result.height,
    lines,
  };
}

/**
 * Map Pretext PretextLineStats to Ygg LineStats.
 */
export function fromPretextLineStats(stats: PretextLineStats): LineStats {
  return {
    lineCount: stats.lineCount,
    maxLineWidth: stats.maxLineWidth,
  };
}

/**
 * Map a Pretext PretextLayoutLineRange to a Ygg LayoutLineRange.
 */
export function fromPretextLineRange(range: PretextLayoutLineRange): LayoutLineRange {
  return {
    width: range.width,
    start: { segmentIndex: range.start.segmentIndex, graphemeIndex: range.start.graphemeIndex },
    end: { segmentIndex: range.end.segmentIndex, graphemeIndex: range.end.graphemeIndex },
  };
}

// --- Opaque handle bridging ---

/**
 * Wrap a Pretext prepared handle so it satisfies Ygg's PreparedText brand.
 * Uses an internal symbol to distinguish from fallback prepared handles.
 */
const PRETEXT_BRAND = Symbol("PretextPrepared");

export type BridgedPretextPrepared = PreparedText & {
  readonly __pretextBrand: typeof PRETEXT_BRAND;
  readonly _pretextHandle: PretextPrepared;
};

/**
 * Bridge a PretextPrepared to Ygg's PreparedText opaque type.
 */
export function bridgePrepared(handle: PretextPrepared): BridgedPretextPrepared {
  return {
    __brand: "PreparedText",
    __pretextBrand: PRETEXT_BRAND,
    _pretextHandle: handle,
  } as unknown as BridgedPretextPrepared;
}

/**
 * Bridge a PretextPreparedWithSegments to Ygg's PreparedTextWithSegments.
 */
export type BridgedPretextPreparedWithSegments = PreparedTextWithSegments & {
  readonly __pretextBrand: typeof PRETEXT_BRAND;
  readonly _pretextHandle: PretextPreparedWithSegments;
};

export function bridgePreparedWithSegments(handle: PretextPreparedWithSegments): BridgedPretextPreparedWithSegments {
  return {
    __brand: "PreparedText",
    __pretextBrand: PRETEXT_BRAND,
    _pretextHandle: handle,
    segments: handle.segments,
    widths: handle.widths,
    letterSpacing: handle.letterSpacing,
  } as unknown as BridgedPretextPreparedWithSegments;
}

/**
 * Check if a Ygg PreparedText is a bridged Pretext handle.
 */
export function isBridgedPretextPrepared(prepared: PreparedText): prepared is BridgedPretextPrepared {
  return PRETEXT_BRAND in (prepared as any);
}

/**
 * Check if a Ygg PreparedTextWithSegments is a bridged Pretext handle.
 */
export function isBridgedPretextPreparedWithSegments(prepared: PreparedTextWithSegments): prepared is BridgedPretextPreparedWithSegments {
  return PRETEXT_BRAND in (prepared as any);
}

/**
 * Extract the underlying Pretext handle from a bridged prepared.
 * Throws if the handle is not a bridged Pretext handle.
 */
export function unbridgePrepared(prepared: PreparedText): PretextPrepared {
  if (isBridgedPretextPrepared(prepared)) {
    return prepared._pretextHandle;
  }
  throw new Error("PretextBridge: expected a bridged PretextPrepared handle, got a fallback handle");
}

/**
 * Extract the underlying Pretext handle from a bridged prepared-with-segments.
 * Throws if the handle is not a bridged Pretext handle.
 */
export function unbridgePreparedWithSegments(prepared: PreparedTextWithSegments): PretextPreparedWithSegments {
  if (isBridgedPretextPreparedWithSegments(prepared)) {
    return prepared._pretextHandle;
  }
  throw new Error("PretextBridge: expected a bridged PretextPreparedWithSegments handle, got a fallback handle");
}
