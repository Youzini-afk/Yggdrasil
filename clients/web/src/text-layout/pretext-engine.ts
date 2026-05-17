/**
 * Optional Text Engine Alpha — PretextTextEngine implementation.
 *
 * Implements the TextEngine interface using @chenglou/pretext via dynamic import.
 * If the Pretext module is not available (not installed, import fails), the engine
 * throws a diagnostic error that the registry can catch and use for fallback.
 *
 * This module never requires @chenglou/pretext at compile time or runtime.
 * TypeScript compiles without the package installed thanks to the local type
 * definitions in pretext-shim.ts.
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

import type {
  PretextModuleShape,
  PretextPreparedWithSegments,
} from "./pretext-shim.js";
import { PRETEXT_MODULE_SPECIFIER } from "./pretext-shim.js";

import {
  bridgePrepared,
  bridgePreparedWithSegments,
  fromPretextLayoutResult,
  fromPretextLayoutLinesResult,
  fromPretextLineStats,
  fromPretextLineRange,
  unbridgePrepared,
  unbridgePreparedWithSegments,
  toPretextOptions,
} from "./pretext-bridge.js";

// --- Lazy module loader ---

/** Cached Pretext module reference, or null if not loaded yet. */
let pretextModule: PretextModuleShape | null = null;

/** Whether we've already attempted to load Pretext (to avoid repeated failed imports). */
let loadAttempted = false;

/** Error from the last load attempt, if any. */
let loadError: string | null = null;

/**
 * Attempt to dynamically import @chenglou/pretext.
 * Returns the module if available, or throws a diagnostic error.
 * Caches the result so subsequent calls are cheap.
 */
async function loadPretextModule(): Promise<PretextModuleShape> {
  if (pretextModule) return pretextModule;
  if (loadAttempted) {
    throw new Error(
      `PretextTextEngine: @chenglou/pretext is not available. ` +
      `Previous load attempt failed: ${loadError ?? "unknown error"}. ` +
      `Install @chenglou/pretext or use the fallback engine.`
    );
  }

  loadAttempted = true;
  try {
    // Dynamic import with unknown-safe casting.
    // At runtime, if the package is not installed, this will throw.
    // We cast through unknown to PretextModuleShape for type safety.
    const mod: unknown = await import(/* webpackIgnore: true */ PRETEXT_MODULE_SPECIFIER);
    pretextModule = mod as PretextModuleShape;
    return pretextModule;
  } catch (err) {
    loadError = err instanceof Error ? err.message : String(err);
    throw new Error(
      `PretextTextEngine: Failed to load @chenglou/pretext. ` +
      `Error: ${loadError}. ` +
      `The fallback engine will be used instead. ` +
      `To use Pretext, install @chenglou/pretext as a dependency.`
    );
  }
}

/**
 * Synchronous check: has Pretext been loaded and cached?
 * Used by the engine to report availability without async.
 */
export function isPretextAvailable(): boolean {
  return pretextModule !== null;
}

/**
 * Get the cached load error, if any.
 * Returns null if no load has been attempted or if load succeeded.
 */
export function getPretextLoadError(): string | null {
  return loadError;
}

/**
 * Reset the load state (useful for retry after dynamic package loading).
 * After reset, the next call to the engine methods will attempt a fresh load.
 */
export function resetPretextLoadState(): void {
  pretextModule = null;
  loadAttempted = false;
  loadError = null;
}

// --- Streaming buffer wrapper ---

/**
 * Create a StreamingTextBuffer that delegates to Pretext for layout.
 * This wraps the fallback streaming buffer structure but uses Pretext
 * for prepare/layout operations.
 */
function createPretextStreamingBuffer(
  module: PretextModuleShape,
  font: FontDescriptor,
  lineHeight: number,
  maxWidth: number,
): StreamingTextBuffer {
  let text = "";
  let state: import("./types.js").StreamingBufferState = "idle";
  let pretextPrepared: PretextPreparedWithSegments | null = null;
  let bridgedPrepared: PreparedTextWithSegments | null = null;

  function ensurePrepared(): PreparedTextWithSegments {
    if (bridgedPrepared && pretextPrepared) return bridgedPrepared;
    pretextPrepared = module.prepareWithSegments(text, font, { whiteSpace: "pre-wrap" });
    bridgedPrepared = bridgePreparedWithSegments(pretextPrepared);
    return bridgedPrepared;
  }

  return {
    get state() { return state; },
    set state(value: import("./types.js").StreamingBufferState) { state = value; },
    get text() { return text; },
    get prepared() { return bridgedPrepared; },
    get maxWidth() { return maxWidth; },
    set maxWidth(value: number) { maxWidth = value; },
    get lineHeight() { return lineHeight; },
    get font() { return font; },
    append(chunk: string) {
      if (state === "ended" || state === "reset") state = "streaming";
      if (state === "idle") state = "streaming";
      text += chunk;
      pretextPrepared = null;
      bridgedPrepared = null;
    },
    end() { state = "ended"; },
    reset() {
      text = "";
      state = "reset";
      pretextPrepared = null;
      bridgedPrepared = null;
    },
    measure() {
      const p = ensurePrepared();
      const handle = unbridgePreparedWithSegments(p);
      const result = module.layout(handle, maxWidth, lineHeight);
      return fromPretextLayoutResult(result);
    },
    layoutLines() {
      const p = ensurePrepared();
      const handle = unbridgePreparedWithSegments(p);
      const result = module.layoutWithLines(handle, maxWidth, lineHeight);
      return fromPretextLayoutLinesResult(result, handle);
    },
    lineStats() {
      const p = ensurePrepared();
      const handle = unbridgePreparedWithSegments(p);
      const result = module.measureLineStats(handle, maxWidth);
      return fromPretextLineStats(result);
    },
  };
}

// --- PretextTextEngine class ---

export const PRETEXT_ENGINE_CONFIG: TextEngineConfig = Object.freeze({
  name: "pretext",
  version: "0.0.7",
  description: "Optional Pretext text layout engine. Requires @chenglou/pretext installed.",
  default: false,
  maxCacheEntries: 0, // Pretext manages its own cache
});

/**
 * Error thrown when PretextTextEngine methods are called before
 * the Pretext module has been loaded via initialize().
 */
class PretextNotLoadedError extends Error {
  constructor() {
    super(
      "PretextTextEngine: not initialized. Call initialize() first, or use the fallback engine."
    );
    this.name = "PretextNotLoadedError";
  }
}

/**
 * PretextTextEngine implements TextEngine using @chenglou/pretext.
 *
 * This engine requires an async `initialize()` call to load the Pretext module
 * before any layout methods can be used. If Pretext is not installed, initialize()
 * will throw a diagnostic error that the registry can catch for fallback.
 *
 * Usage:
 *   const engine = new PretextTextEngine();
 *   try {
 *     await engine.initialize();
 *     registry.registerTextEngine(engine);
 *   } catch (e) {
 *     // Fall back to fallback engine
 *   }
 */
export class PretextTextEngine implements TextEngine {
  readonly config: TextEngineConfig;
  private _state: TextEngineState = "ready";
  private module: PretextModuleShape | null = null;
  private _loadError: string | null = null;

  constructor(config?: Partial<TextEngineConfig>) {
    this.config = {
      ...PRETEXT_ENGINE_CONFIG,
      ...config,
      name: "pretext", // name is always "pretext"
    };
  }

  get state(): TextEngineState {
    return this._state;
  }

  /** The error from the last load attempt, if any. */
  get loadError(): string | null {
    return this._loadError;
  }

  /** Whether this engine has been initialized and Pretext is available. */
  get isAvailable(): boolean {
    return this.module !== null;
  }

  /**
   * Asynchronously load the Pretext module.
   * Must be called before any layout methods.
   * Throws a diagnostic error if Pretext is not available.
   */
  async initialize(): Promise<void> {
    if (this.module) return; // already initialized
    try {
      this.module = await loadPretextModule();
      this._state = "ready";
      this._loadError = null;
    } catch (err) {
      this._state = "error";
      this._loadError = err instanceof Error ? err.message : String(err);
      throw err;
    }
  }

  /** Mark this engine as active. Called by the registry. */
  activate(): void {
    if (this.module) {
      this._state = "active";
    }
    // If not initialized, keep current state (likely "error" or "ready")
  }

  /** Mark this engine as no longer active. Called by the registry. */
  deactivate(): void {
    if (this._state === "active") {
      this._state = this.module ? "ready" : "error";
    }
  }

  private requireModule(): PretextModuleShape {
    if (!this.module) {
      throw new PretextNotLoadedError();
    }
    return this.module;
  }

  prepareText(text: string, font: FontDescriptor, options?: PrepareOptions): PreparedText {
    const mod = this.requireModule();
    const handle = mod.prepare(text, font, toPretextOptions(options));
    return bridgePrepared(handle);
  }

  prepareTextWithSegments(text: string, font: FontDescriptor, options?: PrepareOptions): PreparedTextWithSegments {
    const mod = this.requireModule();
    const handle = mod.prepareWithSegments(text, font, toPretextOptions(options));
    return bridgePreparedWithSegments(handle);
  }

  layoutPreparedText(prepared: PreparedText, maxWidth: number, lineHeight: number): LayoutResult {
    const mod = this.requireModule();
    const handle = unbridgePrepared(prepared);
    const result = mod.layout(handle, maxWidth, lineHeight);
    return fromPretextLayoutResult(result);
  }

  layoutPreparedTextWithLines(
    prepared: PreparedTextWithSegments,
    maxWidth: number,
    lineHeight: number,
  ): LayoutLinesResult {
    const mod = this.requireModule();
    const handle = unbridgePreparedWithSegments(prepared);
    const result = mod.layoutWithLines(handle, maxWidth, lineHeight);
    return fromPretextLayoutLinesResult(result, handle);
  }

  measureLineStats(prepared: PreparedTextWithSegments, maxWidth: number): LineStats {
    const mod = this.requireModule();
    const handle = unbridgePreparedWithSegments(prepared);
    const result = mod.measureLineStats(handle, maxWidth);
    return fromPretextLineStats(result);
  }

  walkLineRanges(
    prepared: PreparedTextWithSegments,
    maxWidth: number,
    onLine: (line: LayoutLineRange) => void,
  ): number {
    const mod = this.requireModule();
    const handle = unbridgePreparedWithSegments(prepared);
    return mod.walkLineRanges(handle, maxWidth, (range: import("./pretext-shim.js").PretextLayoutLineRange) => {
      onLine(fromPretextLineRange(range));
    });
  }

  createStreamingBuffer(
    font: FontDescriptor,
    lineHeight: number,
    maxWidth: number,
  ): StreamingTextBuffer {
    const mod = this.requireModule();
    return createPretextStreamingBuffer(mod, font, lineHeight, maxWidth);
  }

  clearCache(): void {
    // Pretext manages its own internal cache; no-op from our side.
  }
}
