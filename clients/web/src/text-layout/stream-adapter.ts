/**
 * Text Surface Proof Alpha — Generic stream-frame-to-buffer adapter.
 *
 * Provides `feedStreamFrame(buffer, frame)` — a generic adapter that translates
 * stream lifecycle frames into StreamingTextBuffer operations.
 *
 * Supported frame kinds: start, chunk, progress, end, error, cancelled, timeout.
 * No model/agent semantics — this is a pure frame→buffer translation layer.
 */

import type { StreamingTextBuffer } from "./types.js";

// --- Stream frame types ---

/** Generic stream frame kinds. */
export type StreamFrameKind =
  | "start"
  | "chunk"
  | "progress"
  | "end"
  | "error"
  | "cancelled"
  | "timeout";

/** A generic stream frame. */
export type StreamFrame = {
  /** The kind of frame. */
  kind: StreamFrameKind;
  /** Text payload for chunk frames. Ignored for other frame kinds. */
  text?: string;
  /** Progress value (0–1) for progress frames. */
  progress?: number;
  /** Error message for error/timeout frames. */
  error?: string;
  /** Optional metadata. */
  metadata?: Record<string, unknown>;
};

/** Result of feeding a frame into a buffer. */
export type FeedResult = {
  /** Whether the frame was accepted. */
  accepted: boolean;
  /** Current buffer state after feeding. */
  state: import("./types.js").StreamingBufferState;
  /** Optional reason if the frame was rejected. */
  reason?: string;
};

// --- feedStreamFrame ---

/**
 * Feed a stream frame into a streaming text buffer.
 *
 * Frame semantics:
 * - `start`:   resets the buffer and begins a new stream
 * - `chunk`:   appends text to the buffer (if streaming)
 * - `progress`: no-op on buffer, but confirms stream is alive
 * - `end`:     marks the buffer as ended
 * - `error`:   marks the buffer as ended (stream failed)
 * - `cancelled`: marks the buffer as ended (stream cancelled)
 * - `timeout`:  marks the buffer as ended (stream timed out)
 *
 * @param buffer - The StreamingTextBuffer to feed into
 * @param frame  - The stream frame to process
 * @returns FeedResult indicating acceptance and current state
 */
export function feedStreamFrame(
  buffer: StreamingTextBuffer,
  frame: StreamFrame,
): FeedResult {
  switch (frame.kind) {
    case "start": {
      buffer.reset();
      // Transition from reset → streaming by appending an empty starter
      // The buffer state will become "streaming" on the first chunk
      return { accepted: true, state: buffer.state };
    }

    case "chunk": {
      if (buffer.state === "ended") {
        return { accepted: false, state: buffer.state, reason: "stream already ended" };
      }
      const text = frame.text ?? "";
      if (text.length > 0) {
        buffer.append(text);
      }
      return { accepted: true, state: buffer.state };
    }

    case "progress": {
      // Progress frames are acknowledged but don't modify buffer content.
      // They confirm the stream is alive.
      if (buffer.state !== "streaming") {
        return { accepted: false, state: buffer.state, reason: "not in streaming state" };
      }
      return { accepted: true, state: buffer.state };
    }

    case "end": {
      if (buffer.state === "idle") {
        return { accepted: false, state: buffer.state, reason: "stream not started" };
      }
      buffer.end();
      return { accepted: true, state: buffer.state };
    }

    case "error": {
      // On error, end the stream so UI can show final state
      if (buffer.state === "streaming" || buffer.state === "idle") {
        buffer.end();
      }
      return {
        accepted: true,
        state: buffer.state,
        reason: frame.error ?? "stream error",
      };
    }

    case "cancelled": {
      // On cancellation, end the stream
      if (buffer.state === "streaming" || buffer.state === "idle") {
        buffer.end();
      }
      return {
        accepted: true,
        state: buffer.state,
        reason: "stream cancelled",
      };
    }

    case "timeout": {
      // On timeout, end the stream
      if (buffer.state === "streaming" || buffer.state === "idle") {
        buffer.end();
      }
      return {
        accepted: true,
        state: buffer.state,
        reason: frame.error ?? "stream timeout",
      };
    }

    default: {
      return {
        accepted: false,
        state: buffer.state,
        reason: `unknown frame kind: ${(frame as any).kind}`,
      };
    }
  }
}

// --- Frame constructors (convenience helpers) ---

/** Create a start frame. */
export function frameStart(metadata?: Record<string, unknown>): StreamFrame {
  return { kind: "start", metadata };
}

/** Create a chunk frame. */
export function frameChunk(text: string, metadata?: Record<string, unknown>): StreamFrame {
  return { kind: "chunk", text, metadata };
}

/** Create a progress frame. */
export function frameProgress(progress: number, metadata?: Record<string, unknown>): StreamFrame {
  return { kind: "progress", progress, metadata };
}

/** Create an end frame. */
export function frameEnd(metadata?: Record<string, unknown>): StreamFrame {
  return { kind: "end", metadata };
}

/** Create an error frame. */
export function frameError(error: string, metadata?: Record<string, unknown>): StreamFrame {
  return { kind: "error", error, metadata };
}

/** Create a cancelled frame. */
export function frameCancelled(metadata?: Record<string, unknown>): StreamFrame {
  return { kind: "cancelled", metadata };
}

/** Create a timeout frame. */
export function frameTimeout(error?: string, metadata?: Record<string, unknown>): StreamFrame {
  return { kind: "timeout", error, metadata };
}
