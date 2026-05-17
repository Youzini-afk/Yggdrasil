/**
 * Optional Text Engine Alpha — Text Preview helper (T4).
 *
 * Extracts safe plain-text previews from arbitrary event payloads, stream
 * frames, and proposal-like objects. No model/agent semantics — this is a
 * pure extraction + layout-estimation layer for the Forge surface.
 *
 * Supported sources:
 *   - kernel/stream.chunk, kernel/stream.progress, kernel/stream.error,
 *     kernel/stream.cancelled, kernel/stream.timeout event payloads
 *   - Common payload fields: text, message, summary, reason, content
 *   - Proposal expected_effects / operations (long string fields)
 */

import type { TextEngineName } from "./engine.js";
import { getActiveTextEngine, getActiveTextEngineName } from "./registry.js";

// --- Types ---

/** The kind of source from which the preview text was extracted. */
export type TextPreviewKind =
  | "stream-chunk"
  | "stream-progress"
  | "stream-error"
  | "stream-cancelled"
  | "stream-timeout"
  | "payload-field"
  | "proposal-effects"
  | "proposal-operations"
  | "none";

/** Result of extracting a text preview. */
export type TextPreviewResult = {
  /** Whether any preview text was extracted. */
  readonly hasPreview: boolean;
  /** The extracted plain text (empty string if none). */
  readonly text: string;
  /** The kind of source that produced this preview. */
  readonly kind: TextPreviewKind;
  /** Estimated line count (0 if no text). */
  readonly lineEstimate: number;
  /** Estimated height in px (0 if no text). */
  readonly heightEstimate: number;
  /** Name of the engine used for estimation. */
  readonly engineName: TextEngineName;
};

/** Default dimensions used for layout estimation. */
const PREVIEW_MAX_WIDTH = 560;
const PREVIEW_LINE_HEIGHT = 20;
const PREVIEW_FONT = '14px "Inter", "Helvetica Neue", Arial, sans-serif';
const PREVIEW_MIN_LENGTH = 40;

// --- Stream event kind detection ---

const STREAM_KINDS = new Set([
  "kernel/stream.chunk",
  "kernel/stream.progress",
  "kernel/stream.error",
  "kernel/stream.cancelled",
  "kernel/stream.timeout",
]);

function mapStreamKind(eventKind: string): TextPreviewKind {
  switch (eventKind) {
    case "kernel/stream.chunk": return "stream-chunk";
    case "kernel/stream.progress": return "stream-progress";
    case "kernel/stream.error": return "stream-error";
    case "kernel/stream.cancelled": return "stream-cancelled";
    case "kernel/stream.timeout": return "stream-timeout";
    default: return "none";
  }
}

// --- Common payload field extraction ---

/** Common field names that typically carry human-readable text. */
const COMMON_TEXT_FIELDS = [
  "text",
  "message",
  "summary",
  "reason",
  "content",
] as const;

/**
 * Extract a string value from a record using common field names.
 * Returns the first non-empty string found, or undefined.
 */
function extractCommonField(payload: Record<string, unknown>): string | undefined {
  for (const field of COMMON_TEXT_FIELDS) {
    const value = payload[field];
    if (typeof value === "string" && value.length > 0) {
      return value;
    }
  }
  return undefined;
}

// --- Stream payload extraction ---

/**
 * Extract preview text from a stream event payload.
 * For stream.chunk: prefer payload.text
 * For stream.error / stream.timeout: prefer payload.message or payload.reason
 * For stream.cancelled: prefer payload.reason or payload.message
 * For stream.progress: prefer payload.message or payload.summary
 */
function extractStreamPayload(payload: Record<string, unknown>, kind: TextPreviewKind): string | undefined {
  switch (kind) {
    case "stream-chunk": {
      const text = payload["text"];
      if (typeof text === "string" && text.length > 0) return text;
      return extractCommonField(payload);
    }
    case "stream-error":
    case "stream-timeout": {
      // Prefer message/reason for error states
      for (const field of ["message", "reason", "text", "summary", "content"] as const) {
        const value = payload[field];
        if (typeof value === "string" && value.length > 0) return value;
      }
      return undefined;
    }
    case "stream-cancelled": {
      for (const field of ["reason", "message", "text", "summary", "content"] as const) {
        const value = payload[field];
        if (typeof value === "string" && value.length > 0) return value;
      }
      return undefined;
    }
    case "stream-progress": {
      for (const field of ["message", "summary", "text", "content", "reason"] as const) {
        const value = payload[field];
        if (typeof value === "string" && value.length > 0) return value;
      }
      return undefined;
    }
    default:
      return undefined;
  }
}

// --- Proposal extraction ---

/** Minimum string length for a proposal field to be considered "long enough" for preview. */
const PROPOSAL_FIELD_MIN_LENGTH = 60;

/**
 * Extract preview text from proposal expected_effects.
 * Looks for long string fields in the expected_effects object.
 */
function extractProposalEffects(expectedEffects: unknown): { text: string; kind: TextPreviewKind } | null {
  if (typeof expectedEffects !== "object" || expectedEffects === null) return null;
  const record = expectedEffects as Record<string, unknown>;
  const parts: string[] = [];
  for (const value of Object.values(record)) {
    if (typeof value === "string" && value.length >= PROPOSAL_FIELD_MIN_LENGTH) {
      parts.push(value);
    }
  }
  if (parts.length === 0) return null;
  return { text: parts.join("\n"), kind: "proposal-effects" };
}

/**
 * Extract preview text from proposal operations array.
 * Looks for long string fields within operation objects.
 */
function extractProposalOperations(operations: unknown[]): { text: string; kind: TextPreviewKind } | null {
  const parts: string[] = [];
  for (const op of operations) {
    if (typeof op !== "object" || op === null) continue;
    const record = op as Record<string, unknown>;
    for (const value of Object.values(record)) {
      if (typeof value === "string" && value.length >= PROPOSAL_FIELD_MIN_LENGTH) {
        parts.push(value);
      }
    }
  }
  if (parts.length === 0) return null;
  return { text: parts.join("\n"), kind: "proposal-operations" };
}

// --- Layout estimation ---

/**
 * Estimate line count and height for the given text using the active text engine.
 * Falls back to a simple newline-count heuristic if the engine is unavailable.
 */
function estimateLayout(text: string): { lineCount: number; height: number; engineName: TextEngineName } {
  if (!text || text.length === 0) {
    return { lineCount: 0, height: 0, engineName: getActiveTextEngineName() };
  }

  try {
    const engine = getActiveTextEngine();
    const prepared = engine.prepareTextWithSegments(text, PREVIEW_FONT, { whiteSpace: "pre-wrap" });
    const result = engine.layoutPreparedText(prepared as any, PREVIEW_MAX_WIDTH, PREVIEW_LINE_HEIGHT);
    return {
      lineCount: result.lineCount,
      height: result.height,
      engineName: engine.config.name,
    };
  } catch {
    // Fallback: simple newline heuristic
    const newlineCount = text.split("\n").length;
    const estimatedLines = Math.max(newlineCount, Math.ceil(text.length / 80));
    return {
      lineCount: estimatedLines,
      height: estimatedLines * PREVIEW_LINE_HEIGHT,
      engineName: "fallback",
    };
  }
}

// --- Public API ---

/**
 * Extract a text preview from an event payload and kind.
 *
 * This function examines the event kind and payload to extract human-readable
 * text suitable for display in the Forge Events section. It supports stream
 * events and payloads with common text fields.
 *
 * @param eventKind - The event kind string (e.g. "kernel/stream.chunk")
 * @param payload   - The event payload (unknown, will be inspected safely)
 * @returns A TextPreviewResult with extracted text and layout estimates
 */
export function extractEventPreview(
  eventKind: string,
  payload: unknown,
): TextPreviewResult {
  // Try stream event extraction first
  if (STREAM_KINDS.has(eventKind)) {
    const streamKind = mapStreamKind(eventKind);
    if (typeof payload === "object" && payload !== null) {
      const text = extractStreamPayload(payload as Record<string, unknown>, streamKind);
      if (text && text.length >= PREVIEW_MIN_LENGTH) {
        const layout = estimateLayout(text);
        return {
          hasPreview: true,
          text,
          kind: streamKind,
          lineEstimate: layout.lineCount,
          heightEstimate: layout.height,
          engineName: layout.engineName,
        };
      }
      // Even for shorter text from error/cancelled/timeout events, show preview
      if (text && (streamKind === "stream-error" || streamKind === "stream-cancelled" || streamKind === "stream-timeout")) {
        const layout = estimateLayout(text);
        return {
          hasPreview: true,
          text,
          kind: streamKind,
          lineEstimate: layout.lineCount,
          heightEstimate: layout.height,
          engineName: layout.engineName,
        };
      }
    }
    // Stream event but no text found
    return emptyResult();
  }

  // Try common field extraction for any event
  if (typeof payload === "object" && payload !== null) {
    const text = extractCommonField(payload as Record<string, unknown>);
    if (text && text.length >= PREVIEW_MIN_LENGTH) {
      const layout = estimateLayout(text);
      return {
        hasPreview: true,
        text,
        kind: "payload-field",
        lineEstimate: layout.lineCount,
        heightEstimate: layout.height,
        engineName: layout.engineName,
      };
    }
  }

  return emptyResult();
}

/**
 * Extract a text preview from a proposal record.
 *
 * Examines expected_effects and operations for long string fields
 * that are useful for a Forge text preview.
 *
 * @param proposal - A proposal-like object with expected_effects and operations
 * @returns A TextPreviewResult with extracted text and layout estimates
 */
export function extractProposalPreview(proposal: {
  expected_effects?: unknown;
  operations?: unknown[];
}): TextPreviewResult {
  // Try expected_effects first
  const effects = proposal.expected_effects;
  if (effects !== undefined) {
    const extracted = extractProposalEffects(effects);
    if (extracted) {
      const layout = estimateLayout(extracted.text);
      return {
        hasPreview: true,
        text: extracted.text,
        kind: extracted.kind,
        lineEstimate: layout.lineCount,
        heightEstimate: layout.height,
        engineName: layout.engineName,
      };
    }
  }

  // Try operations
  const ops = proposal.operations;
  if (Array.isArray(ops) && ops.length > 0) {
    const extracted = extractProposalOperations(ops);
    if (extracted) {
      const layout = estimateLayout(extracted.text);
      return {
        hasPreview: true,
        text: extracted.text,
        kind: extracted.kind,
        lineEstimate: layout.lineCount,
        heightEstimate: layout.height,
        engineName: layout.engineName,
      };
    }
  }

  return emptyResult();
}

/** Return an empty (no preview) result. */
function emptyResult(): TextPreviewResult {
  return {
    hasPreview: false,
    text: "",
    kind: "none",
    lineEstimate: 0,
    heightEstimate: 0,
    engineName: getActiveTextEngineName(),
  };
}

/**
 * Get a human-readable badge label for a TextPreviewKind.
 */
export function kindBadgeLabel(kind: TextPreviewKind): string {
  switch (kind) {
    case "stream-chunk": return "stream:chunk";
    case "stream-progress": return "stream:progress";
    case "stream-error": return "stream:error";
    case "stream-cancelled": return "stream:cancelled";
    case "stream-timeout": return "stream:timeout";
    case "payload-field": return "text";
    case "proposal-effects": return "effects";
    case "proposal-operations": return "operations";
    case "none": return "";
  }
}
