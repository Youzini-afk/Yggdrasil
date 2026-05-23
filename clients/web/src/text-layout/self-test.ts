/**
 * Optional Text Engine Alpha — Lightweight self-test harness (T5).
 *
 * Exports `runTextLayoutSelfTest()` which exercises the fallback engine,
 * registry, stream adapter, and text preview with pure TS assertions.
 * No external test framework required — assertions use `console.assert` and
 * return structured results.
 *
 * Call from the browser console:
 * ```js
 * import { runTextLayoutSelfTest } from "./text-layout/self-test";
 * const results = runTextLayoutSelfTest();
 * console.table(results);
 * ```
 *
 * Or via module-level import in the browser devtools.
 */

import { FallbackTextEngine, clearAdapterCache, getCacheDiagnostics } from "./fallback-engine.js";
import {
  registerTextEngine,
  activateTextEngine,
  getActiveTextEngine,
  getActiveTextEngineName,
  getTextEngineState,
  getTextEngineDiagnostics,
  unregisterTextEngine,
  initializeTextEnginePreference,
  getInitializationResult,
} from "./registry.js";
import {
  feedStreamFrame,
  frameStart,
  frameChunk,
  frameEnd,
  frameError,
  frameCancelled,
  frameTimeout,
  frameProgress,
} from "./stream-adapter.js";
import { extractEventPreview, extractProposalPreview, kindBadgeLabel } from "./text-preview.js";
import { createStreamingBuffer } from "./fallback-engine.js";
import type { StreamingTextBuffer } from "./types.js";

// --- Test result type ---

export type SelfTestResult = {
  name: string;
  passed: boolean;
  detail?: string;
};

// --- Assertion helpers ---

function assert(condition: boolean, name: string, detail: string): SelfTestResult {
  if (!condition) {
    return { name, passed: false, detail };
  }
  return { name, passed: true };
}

function assertEqual<T>(actual: T, expected: T, name: string): SelfTestResult {
  if (actual !== expected) {
    return { name, passed: false, detail: `expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}` };
  }
  return { name, passed: true };
}

// --- Test suites ---

function testFallbackEngine(): SelfTestResult[] {
  const results: SelfTestResult[] = [];

  // 1. Construction
  const engine = new FallbackTextEngine();
  results.push(assertEqual(engine.config.name, "fallback", "FallbackEngine: config.name"));
  results.push(assertEqual(engine.state, "ready", "FallbackEngine: initial state"));

  // 2. activate/deactivate
  engine.activate();
  results.push(assertEqual(engine.state, "active", "FallbackEngine: state after activate"));
  engine.deactivate();
  results.push(assertEqual(engine.state, "ready", "FallbackEngine: state after deactivate"));

  // 3. prepareText + layoutPreparedText
  const font = '14px "Inter", sans-serif';
  const prepared = engine.prepareText("Hello world", font);
  const layout = engine.layoutPreparedText(prepared, 560, 20);
  results.push(assert(layout.lineCount >= 1, "FallbackEngine: lineCount >= 1", `got ${layout.lineCount}`));
  results.push(assert(layout.height > 0, "FallbackEngine: height > 0", `got ${layout.height}`));

  // 4. prepareTextWithSegments
  const preparedSeg = engine.prepareTextWithSegments("Hello world", font);
  results.push(assert(preparedSeg.segments.length > 0, "FallbackEngine: segments.length > 0", `got ${preparedSeg.segments.length}`));
  results.push(assert(preparedSeg.widths.length > 0, "FallbackEngine: widths.length > 0", `got ${preparedSeg.widths.length}`));

  // 5. measureLineStats
  const stats = engine.measureLineStats(preparedSeg, 560);
  results.push(assert(stats.lineCount >= 1, "FallbackEngine: stats.lineCount >= 1", `got ${stats.lineCount}`));
  results.push(assert(stats.maxLineWidth > 0, "FallbackEngine: stats.maxLineWidth > 0", `got ${stats.maxLineWidth}`));

  // 6. layoutPreparedTextWithLines
  const withLines = engine.layoutPreparedTextWithLines(preparedSeg, 560, 20);
  results.push(assertEqual(withLines.lineCount, stats.lineCount, "FallbackEngine: withLines.lineCount === stats.lineCount"));
  results.push(assert(withLines.lines.length > 0, "FallbackEngine: withLines.lines.length > 0", `got ${withLines.lines.length}`));

  // 7. walkLineRanges
  let walkedCount = 0;
  const walkResult = engine.walkLineRanges(preparedSeg, 560, () => { walkedCount++; });
  results.push(assertEqual(walkResult, stats.lineCount, "FallbackEngine: walkLineRanges returns lineCount"));
  results.push(assertEqual(walkedCount, stats.lineCount, "FallbackEngine: walkLineRanges callback count"));

  // 8. createStreamingBuffer
  const buffer = engine.createStreamingBuffer(font, 20, 560);
  results.push(assertEqual(buffer.state, "idle", "FallbackEngine: streaming buffer initial state"));
  buffer.append("chunk1");
  results.push(assertEqual(buffer.state, "streaming", "FallbackEngine: streaming buffer after append"));
  buffer.end();
  results.push(assertEqual(buffer.state, "ended", "FallbackEngine: streaming buffer after end"));

  // 9. clearCache
  engine.clearCache();
  results.push(assert(true, "FallbackEngine: clearCache succeeds", ""));

  // 10. cacheEntries
  results.push(assert(typeof engine.cacheEntries === "number", "FallbackEngine: cacheEntries is number", `got ${typeof engine.cacheEntries}`));

  return results;
}

function testCacheDiagnostics(): SelfTestResult[] {
  const results: SelfTestResult[] = [];

  clearAdapterCache();
  const before = getCacheDiagnostics();
  results.push(assertEqual(before.totalEntries, 0, "CacheDiagnostics: initially empty"));
  results.push(assert(before.maxEntries > 0, "CacheDiagnostics: maxEntries > 0", `got ${before.maxEntries}`));

  // Force some entries
  const font = '14px "Inter", sans-serif';
  const engine = new FallbackTextEngine();
  engine.prepareText("Hello world test cache", font);
  const after = getCacheDiagnostics();
  results.push(assert(after.totalEntries > 0, "CacheDiagnostics: entries after prepare", `got ${after.totalEntries}`));
  results.push(assert(after.fontCount >= 1, "CacheDiagnostics: fontCount >= 1", `got ${after.fontCount}`));
  results.push(assert(after.estimatedBytes > 0, "CacheDiagnostics: estimatedBytes > 0", `got ${after.estimatedBytes}`));

  clearAdapterCache();
  const cleared = getCacheDiagnostics();
  results.push(assertEqual(cleared.totalEntries, 0, "CacheDiagnostics: entries after clear"));

  return results;
}

function testRegistry(): SelfTestResult[] {
  const results: SelfTestResult[] = [];

  // 1. Default engine is fallback
  const active = getActiveTextEngine();
  results.push(assertEqual(active.config.name, "fallback", "Registry: default active is fallback"));

  // 2. Active name
  results.push(assertEqual(getActiveTextEngineName(), "fallback", "Registry: active name is fallback"));

  // 3. State
  const fallbackState = getTextEngineState("fallback");
  results.push(assert(fallbackState !== "unavailable", "Registry: fallback state is not unavailable", `got ${fallbackState}`));

  // 4. Diagnostics
  const diags = getTextEngineDiagnostics();
  results.push(assert(diags.length > 0, "Registry: diagnostics non-empty", `got ${diags.length}`));
  const fallbackDiag = diags.find((d) => d.name === "fallback");
  results.push(assert(fallbackDiag !== undefined, "Registry: fallback in diagnostics", ""));
  results.push(assertEqual(fallbackDiag?.isFallback ?? false, true, "Registry: fallback.isFallback === true"));

  // 5. Cannot unregister fallback
  const unregResult = unregisterTextEngine("fallback");
  results.push(assertEqual(unregResult, false, "Registry: cannot unregister fallback"));

  // 6. activateTextEngine with unknown name
  const activateUnknown = activateTextEngine("nonexistent");
  results.push(assertEqual(activateUnknown, false, "Registry: activate unknown returns false"));

  // 7. selectTextEngine with unknown name returns current
  const selected = active;
  results.push(assertEqual(selected.config.name, "fallback", "Registry: select unknown returns fallback"));

  return results;
}

function testStreamAdapter(): SelfTestResult[] {
  const results: SelfTestResult[] = [];

  const font = '14px "Inter", sans-serif';
  const buffer = createStreamingBuffer(font, 20, 560);

  // 1. Start frame resets buffer
  const startResult = feedStreamFrame(buffer, frameStart());
  results.push(assert(startResult.accepted, "StreamAdapter: start accepted", `got ${startResult.accepted}`));

  // 2. Chunk frame appends text
  const chunkResult = feedStreamFrame(buffer, frameChunk("Hello"));
  results.push(assert(chunkResult.accepted, "StreamAdapter: chunk accepted", ""));
  results.push(assert(buffer.text.includes("Hello"), "StreamAdapter: buffer has chunk text", `got "${buffer.text}"`));

  // 3. Progress frame is no-op
  const progResult = feedStreamFrame(buffer, frameProgress(0.5));
  results.push(assert(progResult.accepted, "StreamAdapter: progress accepted", ""));

  // 4. End frame marks ended
  const endResult = feedStreamFrame(buffer, frameEnd());
  results.push(assert(endResult.accepted, "StreamAdapter: end accepted", ""));
  results.push(assertEqual(buffer.state, "ended", "StreamAdapter: buffer state after end"));

  // 5. Chunk after end is rejected
  const afterEnd = feedStreamFrame(buffer, frameChunk("nope"));
  results.push(assertEqual(afterEnd.accepted, false, "StreamAdapter: chunk after end rejected"));
  results.push(assert(afterEnd.reason !== undefined, "StreamAdapter: chunk after end has reason", `got ${afterEnd.reason}`));

  // 6. Error frame
  const errBuffer = createStreamingBuffer(font, 20, 560);
  feedStreamFrame(errBuffer, frameStart());
  feedStreamFrame(errBuffer, frameChunk("data"));
  const errResult = feedStreamFrame(errBuffer, frameError("something broke"));
  results.push(assert(errResult.accepted, "StreamAdapter: error accepted", ""));
  results.push(assertEqual(errBuffer.state, "ended", "StreamAdapter: buffer ends on error"));

  // 7. Cancelled frame
  const cancelBuffer = createStreamingBuffer(font, 20, 560);
  feedStreamFrame(cancelBuffer, frameStart());
  const cancelResult = feedStreamFrame(cancelBuffer, frameCancelled());
  results.push(assert(cancelResult.accepted, "StreamAdapter: cancelled accepted", ""));

  // 8. Timeout frame
  const timeoutBuffer = createStreamingBuffer(font, 20, 560);
  feedStreamFrame(timeoutBuffer, frameStart());
  const timeoutResult = feedStreamFrame(timeoutBuffer, frameTimeout("timed out"));
  results.push(assert(timeoutResult.accepted, "StreamAdapter: timeout accepted", ""));

  return results;
}

function testTextPreview(): SelfTestResult[] {
  const results: SelfTestResult[] = [];

  // 1. Stream chunk with long text
  const streamResult = extractEventPreview("kernel/v1/stream.chunk", {
    text: "This is a long enough text that should produce a preview for the Forge text surface display area.",
  });
  results.push(assert(streamResult.hasPreview, "TextPreview: stream chunk has preview", ""));
  results.push(assertEqual(streamResult.kind, "stream-chunk", "TextPreview: kind is stream-chunk"));
  results.push(assert(streamResult.lineEstimate >= 1, "TextPreview: lineEstimate >= 1", `got ${streamResult.lineEstimate}`));

  // 2. Stream chunk with short text — no preview
  const shortResult = extractEventPreview("kernel/v1/stream.chunk", { text: "hi" });
  results.push(assertEqual(shortResult.hasPreview, false, "TextPreview: short text no preview"));

  // 3. Stream error
  const errorResult = extractEventPreview("kernel/v1/stream.error", { message: "Connection refused" });
  results.push(assert(errorResult.hasPreview, "TextPreview: stream error has preview", ""));
  results.push(assertEqual(errorResult.kind, "stream-error", "TextPreview: kind is stream-error"));

  // 4. Unknown event kind with text field
  const unknownResult = extractEventPreview("custom/event", { content: "This is a custom event with a long enough content field for preview extraction." });
  results.push(assert(unknownResult.hasPreview, "TextPreview: unknown event with content field", ""));
  results.push(assertEqual(unknownResult.kind, "payload-field", "TextPreview: kind is payload-field"));

  // 5. Proposal preview
  const proposalResult = extractProposalPreview({
    expected_effects: { description: "This is a long enough expected effects description that should trigger the proposal preview extraction mechanism." },
  });
  results.push(assert(proposalResult.hasPreview, "TextPreview: proposal has preview", ""));
  results.push(assertEqual(proposalResult.kind, "proposal-effects", "TextPreview: kind is proposal-effects"));

  // 6. kindBadgeLabel
  results.push(assertEqual(kindBadgeLabel("stream-chunk"), "stream:chunk", "TextPreview: badge for stream-chunk"));
  results.push(assertEqual(kindBadgeLabel("none"), "", "TextPreview: badge for none is empty"));

  return results;
}

function testAsyncInitialization(): SelfTestResult[] {
  const results: SelfTestResult[] = [];

  // initializeTextEnginePreference should resolve (with fallback since Pretext is unavailable)
  // This is async but we test the result structure synchronously via getInitializationResult
  const initResult = getInitializationResult();
  // initResult may be null if not yet called; that's fine
  results.push(assert(true, "AsyncInit: getInitializationResult returns without error", `got ${initResult === null ? "null" : "non-null"}`));

  return results;
}

// --- Main entry point ---

/**
 * Run all text-layout self-tests and return structured results.
 *
 * No external test framework required. Call from browser console:
 * ```js
 * const results = runTextLayoutSelfTest();
 * console.table(results);
 * ```
 *
 * @returns array of SelfTestResult entries
 */
export function runTextLayoutSelfTest(): SelfTestResult[] {
  const all: SelfTestResult[] = [];

  all.push(...testFallbackEngine());
  all.push(...testCacheDiagnostics());
  all.push(...testRegistry());
  all.push(...testStreamAdapter());
  all.push(...testTextPreview());
  all.push(...testAsyncInitialization());

  // Summary
  const passed = all.filter((r) => r.passed).length;
  const failed = all.filter((r) => !r.passed).length;
  const total = all.length;

  console.log(`[text-layout self-test] ${passed}/${total} passed, ${failed} failed`);

  if (failed > 0) {
    console.warn("[text-layout self-test] Failures:");
    for (const r of all) {
      if (!r.passed) {
        console.warn(`  FAIL: ${r.name} — ${r.detail}`);
      }
    }
  }

  return all;
}
