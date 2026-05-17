/**
 * Text Surface Proof Alpha — Lightweight fallback adapter skeleton.
 *
 * This module implements a browser-only, canvas-based fallback for the Pretext
 * API shape defined in `types.ts`. It does not depend on `@chenglou/pretext`;
 * the implementations can be swapped later by importing Pretext and re-exporting
 * its functions under the same names.
 */

import type {
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

// --- Shared canvas measurement cache ---

let sharedCanvas: HTMLCanvasElement | OffscreenCanvas | null = null;
let sharedCtx: CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D | null = null;

function getSharedContext(): CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D {
  if (sharedCtx) return sharedCtx;
  if (typeof OffscreenCanvas !== "undefined") {
    sharedCanvas = new OffscreenCanvas(256, 256);
  } else {
    sharedCanvas = document.createElement("canvas");
    sharedCanvas.width = 256;
    sharedCanvas.height = 256;
  }
  const ctx = sharedCanvas.getContext("2d");
  if (!ctx) throw new Error("TextLayoutAdapter: Canvas 2D context not available");
  sharedCtx = ctx;
  return sharedCtx;
}

const widthCache = new Map<string, Map<string, number>>();

function measureSegment(segment: string, font: FontDescriptor): number {
  let fontCache = widthCache.get(font);
  if (!fontCache) {
    fontCache = new Map<string, number>();
    widthCache.set(font, fontCache);
  }
  const cached = fontCache.get(segment);
  if (cached !== undefined) return cached;
  const ctx = getSharedContext();
  ctx.font = font;
  const w = ctx.measureText(segment).width;
  fontCache.set(segment, w);
  return w;
}

function clearMeasurementCache(): void {
  widthCache.clear();
}

// --- Internal prepared representation ---

type InternalPrepared = {
  readonly __brand: "PreparedText";
  segments: string[];
  widths: number[];
  kinds: SegmentKind[];
  letterSpacing: number;
  font: FontDescriptor;
};

type SegmentKind = "text" | "space" | "hard-break" | "tab";

function isCJKChar(ch: string): boolean {
  const cp = ch.codePointAt(0) ?? 0;
  return (
    (cp >= 0x4e00 && cp <= 0x9fff) ||
    (cp >= 0x3400 && cp <= 0x4dbf) ||
    (cp >= 0x2e80 && cp <= 0x2eff) ||
    (cp >= 0x3000 && cp <= 0x303f) ||
    (cp >= 0x3040 && cp <= 0x309f) ||
    (cp >= 0x30a0 && cp <= 0x30ff) ||
    (cp >= 0xac00 && cp <= 0xd7af) ||
    (cp >= 0xf900 && cp <= 0xfaff) ||
    (cp >= 0x20000 && cp <= 0x2a6df)
  );
}

function splitIntoSegments(text: string, whiteSpace: PrepareOptions["whiteSpace"], wordBreak: PrepareOptions["wordBreak"]): { segments: string[]; kinds: SegmentKind[] } {
  const segments: string[] = [];
  const kinds: SegmentKind[] = [];

  if (whiteSpace === "pre-wrap") {
    // Preserve spaces, tabs, and hard breaks as separate segments
    let current = "";
    for (const ch of text) {
      if (ch === "\n") {
        if (current) { segments.push(current); kinds.push("text"); current = ""; }
        segments.push("\n"); kinds.push("hard-break");
      } else if (ch === "\t") {
        if (current) { segments.push(current); kinds.push("text"); current = ""; }
        segments.push("\t"); kinds.push("tab");
      } else if (ch === " ") {
        if (current) { segments.push(current); kinds.push("text"); current = ""; }
        segments.push(" "); kinds.push("space");
      } else {
        current += ch;
      }
    }
    if (current) { segments.push(current); kinds.push("text"); }
    return { segments, kinds };
  }

  // whiteSpace: normal — collapse spaces
  let current = "";
  let inSpace = false;
  for (const ch of text) {
    if (ch === "\n" || ch === "\t" || ch === " ") {
      if (current) { segments.push(current); kinds.push("text"); current = ""; }
      if (!inSpace) { segments.push(" "); kinds.push("space"); inSpace = true; }
    } else {
      inSpace = false;
      current += ch;
    }
  }
  if (current) { segments.push(current); kinds.push("text"); }

  // For CJK or wordBreak: keep-all, further split text segments
  if (wordBreak === "keep-all") {
    // keep-all: do not split words; but for CJK we still split per character
    const newSegments: string[] = [];
    const newKinds: SegmentKind[] = [];
    for (let i = 0; i < segments.length; i++) {
      const seg = segments[i]!;
      const kind = kinds[i]!;
      if (kind !== "text") {
        newSegments.push(seg); newKinds.push(kind);
        continue;
      }
      // Split only on CJK boundaries
      let run = "";
      for (const ch of seg) {
        if (isCJKChar(ch)) {
          if (run) { newSegments.push(run); newKinds.push("text"); run = ""; }
          newSegments.push(ch); newKinds.push("text");
        } else {
          run += ch;
        }
      }
      if (run) { newSegments.push(run); newKinds.push("text"); }
    }
    return { segments: newSegments, kinds: newKinds };
  }

  // Normal mode: split on CJK per-character for line breaking, keep Latin words intact
  const newSegments: string[] = [];
  const newKinds: SegmentKind[] = [];
  for (let i = 0; i < segments.length; i++) {
    const seg = segments[i]!;
    const kind = kinds[i]!;
    if (kind !== "text") {
      newSegments.push(seg); newKinds.push(kind);
      continue;
    }
    let run = "";
    let runIsCJK: boolean | null = null;
    for (const ch of seg) {
      const cjk = isCJKChar(ch);
      if (runIsCJK === null) {
        runIsCJK = cjk;
        run = ch;
      } else if (runIsCJK === cjk) {
        run += ch;
      } else {
        newSegments.push(run); newKinds.push("text");
        run = ch;
        runIsCJK = cjk;
      }
    }
    if (run) { newSegments.push(run); newKinds.push("text"); }
  }
  return { segments: newSegments, kinds: newKinds };
}

function buildPrepared(text: string, font: FontDescriptor, options?: PrepareOptions): InternalPrepared {
  const { segments, kinds } = splitIntoSegments(text, options?.whiteSpace, options?.wordBreak);
  const widths = segments.map((s, i) => {
    const w = measureSegment(s, font);
    const spacing = (options?.letterSpacing ?? 0) * Math.max(0, countGraphemes(s) - (kinds[i] === "space" || kinds[i] === "hard-break" || kinds[i] === "tab" ? 0 : 1));
    return w + spacing;
  });
  return {
    __brand: "PreparedText",
    segments,
    widths,
    kinds,
    letterSpacing: options?.letterSpacing ?? 0,
    font,
  };
}

function countGraphemes(text: string): number {
  if (typeof Intl !== "undefined" && "Segmenter" in Intl) {
    const seg = new Intl.Segmenter(undefined, { granularity: "grapheme" });
    let n = 0;
    for (const _ of seg.segment(text)) n++;
    return n;
  }
  // Fallback: count code points
  return Array.from(text).length;
}

// --- Public API ---

export function prepareText(text: string, font: FontDescriptor, options?: PrepareOptions): PreparedText {
  return buildPrepared(text, font, options) as unknown as PreparedText;
}

export function prepareTextWithSegments(text: string, font: FontDescriptor, options?: PrepareOptions): PreparedTextWithSegments {
  return buildPrepared(text, font, options) as unknown as PreparedTextWithSegments;
}

function getInternal(prepared: PreparedText): InternalPrepared {
  return prepared as unknown as InternalPrepared;
}

function layoutCore(prepared: InternalPrepared, maxWidth: number): { lines: LayoutLineRange[]; lineCount: number } {
  const { segments, widths, kinds } = prepared;
  const lines: LayoutLineRange[] = [];
  let currentWidth = 0;
  let startSeg = 0;
  let startGrapheme = 0;
  let segStart = 0;
  let graphemeStart = 0;

  const pushLine = (endSeg: number, endGrapheme: number, width: number) => {
    lines.push({
      width,
      start: { segmentIndex: startSeg, graphemeIndex: startGrapheme },
      end: { segmentIndex: endSeg, graphemeIndex: endGrapheme },
    });
    startSeg = endSeg;
    startGrapheme = endGrapheme;
  };

  for (let i = 0; i < segments.length; i++) {
    const seg = segments[i]!;
    const w = widths[i]!;
    const kind = kinds[i]!;

    if (kind === "hard-break") {
      pushLine(i, 0, currentWidth);
      currentWidth = 0;
      segStart = i + 1;
      graphemeStart = 0;
      continue;
    }

    if (kind === "space") {
      // Trailing space hangs past the line edge (CSS behavior)
      currentWidth += w;
      continue;
    }

    if (kind === "tab") {
      const tabAdvance = measureSegment(" ", prepared.font) * 8;
      currentWidth += tabAdvance;
      continue;
    }

    if (currentWidth + w <= maxWidth || currentWidth === 0) {
      currentWidth += w;
      continue;
    }

    // Need to break before this segment. But if the segment itself is wider
    // than maxWidth, we must break inside it (grapheme boundaries).
    if (w > maxWidth) {
      // Break before this segment if there is already content on the line
      if (currentWidth > 0) {
        pushLine(segStart, graphemeStart, currentWidth);
        currentWidth = 0;
        segStart = i;
        graphemeStart = 0;
      }
      // Now split the oversized segment at grapheme boundaries
      const graphemes = Array.from(seg);
      let subWidth = 0;
      let gi = 0;
      for (; gi < graphemes.length; gi++) {
        const gw = measureSegment(graphemes[gi]!, prepared.font) + prepared.letterSpacing;
        if (subWidth + gw > maxWidth && subWidth > 0) {
          pushLine(i, gi, subWidth);
          subWidth = 0;
          segStart = i;
          graphemeStart = gi;
        }
        subWidth += gw;
      }
      currentWidth = subWidth;
      segStart = i;
      graphemeStart = graphemes.length;
    } else {
      pushLine(segStart, graphemeStart, currentWidth);
      currentWidth = w;
      segStart = i;
      graphemeStart = seg.length;
    }
  }

  if (currentWidth > 0 || lines.length === 0) {
    pushLine(segments.length, 0, currentWidth);
  }

  return { lines, lineCount: lines.length };
}

export function layoutPreparedText(prepared: PreparedText, maxWidth: number, lineHeight: number): LayoutResult {
  const { lineCount } = layoutCore(getInternal(prepared), maxWidth);
  return { lineCount, height: lineCount * lineHeight };
}

export function layoutPreparedTextWithLines(prepared: PreparedTextWithSegments, maxWidth: number, lineHeight: number): LayoutLinesResult {
  const internal = getInternal(prepared) as InternalPrepared;
  const { lines: ranges, lineCount } = layoutCore(internal, maxWidth);
  const lines: LayoutLine[] = ranges.map((range) => {
    const { segmentIndex: s, graphemeIndex: g } = range.start;
    const { segmentIndex: e, graphemeIndex: eg } = range.end;
    let text = "";
    if (s === e) {
      text = internal.segments[s]!.slice(g, eg);
    } else {
      text = internal.segments[s]!.slice(g);
      for (let i = s + 1; i < e; i++) {
        text += internal.segments[i];
      }
      if (e < internal.segments.length) {
        text += internal.segments[e]!.slice(0, eg);
      }
    }
    return {
      text,
      width: range.width,
      start: range.start,
      end: range.end,
    };
  });
  return { lineCount, height: lineCount * lineHeight, lines };
}

export function measureLineStats(prepared: PreparedTextWithSegments, maxWidth: number): LineStats {
  const internal = getInternal(prepared) as InternalPrepared;
  const { lines } = layoutCore(internal, maxWidth);
  let maxLineWidth = 0;
  for (const line of lines) {
    if (line.width > maxLineWidth) maxLineWidth = line.width;
  }
  return { lineCount: lines.length, maxLineWidth };
}

export function walkLineRanges(
  prepared: PreparedTextWithSegments,
  maxWidth: number,
  onLine: (line: LayoutLineRange) => void,
): number {
  const internal = getInternal(prepared) as InternalPrepared;
  const { lines, lineCount } = layoutCore(internal, maxWidth);
  for (const line of lines) onLine(line);
  return lineCount;
}

// --- Streaming buffer ---

export function createStreamingBuffer(
  font: FontDescriptor,
  lineHeight: number,
  maxWidth: number,
): StreamingTextBuffer {
  let text = "";
  let state: StreamingBufferState = "idle";
  let prepared: PreparedTextWithSegments | null = null;

  function ensurePrepared(): PreparedTextWithSegments {
    if (!prepared) {
      prepared = prepareTextWithSegments(text, font, { whiteSpace: "pre-wrap" });
    }
    return prepared;
  }

  return {
    get state() {
      return state;
    },
    get text() {
      return text;
    },
    get prepared() {
      return prepared;
    },
    get maxWidth() {
      return maxWidth;
    },
    set maxWidth(value: number) {
      maxWidth = value;
    },
    get lineHeight() {
      return lineHeight;
    },
    get font() {
      return font;
    },
    append(chunk: string) {
      if (state === "ended" || state === "reset") state = "streaming";
      if (state === "idle") state = "streaming";
      text += chunk;
      // Invalidate prepared cache so next measure re-prepares
      prepared = null;
    },
    end() {
      state = "ended";
    },
    reset() {
      text = "";
      state = "reset";
      prepared = null;
    },
    measure() {
      const p = ensurePrepared();
      return layoutPreparedText(p, maxWidth, lineHeight);
    },
    layoutLines() {
      const p = ensurePrepared();
      return layoutPreparedTextWithLines(p, maxWidth, lineHeight);
    },
    lineStats() {
      const p = ensurePrepared();
      return measureLineStats(p, maxWidth);
    },
  };
}

/** Clear the shared segment width cache. Useful on font changes. */
export function clearAdapterCache(): void {
  clearMeasurementCache();
}
