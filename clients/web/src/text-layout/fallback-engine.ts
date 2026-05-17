/**
 * Text Surface Proof Alpha — FallbackTextEngine.
 *
 * Wraps the existing canvas-based fallback adapter as a TextEngine implementation.
 * Also re-exports the original standalone functions for backward compatibility
 * so that existing callers (`prepareText`, `layoutPreparedText`, etc.) keep working.
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
import type { TextEngine, TextEngineConfig, TextEngineState } from "./engine.js";

// --- Shared canvas measurement cache with bounded size ---

const DEFAULT_MAX_CACHE_ENTRIES = 4096;

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
  if (!ctx) throw new Error("FallbackTextEngine: Canvas 2D context not available");
  sharedCtx = ctx;
  return sharedCtx;
}

/** LRU-bounded width cache keyed by (font → segment → width). */
class BoundedWidthCache {
  private readonly fontCaches = new Map<string, Map<string, number>>();
  private _totalEntries = 0;
  private readonly maxEntries: number;

  constructor(maxEntries = DEFAULT_MAX_CACHE_ENTRIES) {
    this.maxEntries = maxEntries;
  }

  get(font: FontDescriptor, segment: string): number | undefined {
    return this.fontCaches.get(font)?.get(segment);
  }

  set(font: FontDescriptor, segment: string, width: number): void {
    let fontCache = this.fontCaches.get(font);
    if (!fontCache) {
      fontCache = new Map<string, number>();
      this.fontCaches.set(font, fontCache);
    }
    if (!fontCache.has(segment)) {
      this._totalEntries++;
      this.evictIfNeeded();
    }
    fontCache.set(segment, width);
  }

  clear(): void {
    this.fontCaches.clear();
    this._totalEntries = 0;
  }

  get totalEntries(): number {
    return this._totalEntries;
  }

  private evictIfNeeded(): void {
    if (this.maxEntries <= 0 || this._totalEntries <= this.maxEntries) return;
    // Evict the oldest font-level cache entirely (simple FIFO at font level)
    const firstKey = this.fontCaches.keys().next().value;
    if (firstKey !== undefined) {
      const removed = this.fontCaches.get(firstKey)!;
      this._totalEntries -= removed.size;
      this.fontCaches.delete(firstKey);
    }
  }
}

const widthCache = new BoundedWidthCache();

function measureSegment(segment: string, font: FontDescriptor): number {
  const cached = widthCache.get(font, segment);
  if (cached !== undefined) return cached;
  const ctx = getSharedContext();
  ctx.font = font;
  const w = ctx.measureText(segment).width;
  widthCache.set(font, segment, w);
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

  if (wordBreak === "keep-all") {
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
  return Array.from(text).length;
}

// --- Layout core (shared by all layout functions) ---

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

    if (w > maxWidth) {
      if (currentWidth > 0) {
        pushLine(segStart, graphemeStart, currentWidth);
        currentWidth = 0;
        segStart = i;
        graphemeStart = 0;
      }
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

function getInternal(prepared: PreparedText): InternalPrepared {
  return prepared as unknown as InternalPrepared;
}

// --- Standalone exported functions (backward compat) ---

export function prepareText(text: string, font: FontDescriptor, options?: PrepareOptions): PreparedText {
  return buildPrepared(text, font, options) as unknown as PreparedText;
}

export function prepareTextWithSegments(text: string, font: FontDescriptor, options?: PrepareOptions): PreparedTextWithSegments {
  return buildPrepared(text, font, options) as unknown as PreparedTextWithSegments;
}

export function layoutPreparedText(prepared: PreparedText, maxWidth: number, lineHeight: number): LayoutResult {
  const { lineCount } = layoutCore(getInternal(prepared), maxWidth);
  return { lineCount, height: lineCount * lineHeight };
}

export function layoutPreparedTextWithLines(prepared: PreparedTextWithSegments, maxWidth: number, lineHeight: number): LayoutLinesResult {
  const internal = getInternal(prepared) as InternalPrepared;
  const { lines: ranges, lineCount } = layoutCore(internal, maxWidth);
  const lines = ranges.map((range) => {
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

export function createStreamingBuffer(
  font: FontDescriptor,
  lineHeight: number,
  maxWidth: number,
): StreamingTextBuffer {
  let text = "";
  let state: import("./types.js").StreamingBufferState = "idle";
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
    set state(value: import("./types.js").StreamingBufferState) {
      state = value;
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

// --- FallbackTextEngine class ---

export const FALLBACK_ENGINE_CONFIG: TextEngineConfig = Object.freeze({
  name: "fallback",
  version: "0.2.0",
  description: "Canvas-based fallback text layout engine. No external dependencies.",
  default: true,
  maxCacheEntries: DEFAULT_MAX_CACHE_ENTRIES,
});

/**
 * FallbackTextEngine implements TextEngine using the browser canvas API.
 * This is the default engine and is always available.
 */
export class FallbackTextEngine implements TextEngine {
  readonly config: TextEngineConfig;
  private _state: TextEngineState = "ready";

  constructor(config?: Partial<TextEngineConfig>) {
    this.config = {
      ...FALLBACK_ENGINE_CONFIG,
      ...config,
      name: "fallback", // name is always "fallback"
    };
  }

  get state(): TextEngineState {
    return this._state;
  }

  /** Mark this engine as active. Called by the registry. */
  activate(): void {
    this._state = "active";
  }

  /** Mark this engine as no longer active. Called by the registry. */
  deactivate(): void {
    this._state = "ready";
  }

  prepareText(text: string, font: FontDescriptor, options?: PrepareOptions): PreparedText {
    return prepareText(text, font, options);
  }

  prepareTextWithSegments(text: string, font: FontDescriptor, options?: PrepareOptions): PreparedTextWithSegments {
    return prepareTextWithSegments(text, font, options);
  }

  layoutPreparedText(prepared: PreparedText, maxWidth: number, lineHeight: number): LayoutResult {
    return layoutPreparedText(prepared, maxWidth, lineHeight);
  }

  layoutPreparedTextWithLines(
    prepared: PreparedTextWithSegments,
    maxWidth: number,
    lineHeight: number,
  ): LayoutLinesResult {
    return layoutPreparedTextWithLines(prepared, maxWidth, lineHeight);
  }

  measureLineStats(prepared: PreparedTextWithSegments, maxWidth: number): LineStats {
    return measureLineStats(prepared, maxWidth);
  }

  walkLineRanges(
    prepared: PreparedTextWithSegments,
    maxWidth: number,
    onLine: (line: LayoutLineRange) => void,
  ): number {
    return walkLineRanges(prepared, maxWidth, onLine);
  }

  createStreamingBuffer(
    font: FontDescriptor,
    lineHeight: number,
    maxWidth: number,
  ): StreamingTextBuffer {
    return createStreamingBuffer(font, lineHeight, maxWidth);
  }

  clearCache(): void {
    clearAdapterCache();
  }

  /** Get the number of cached width entries (diagnostics). */
  get cacheEntries(): number {
    return widthCache.totalEntries;
  }
}
