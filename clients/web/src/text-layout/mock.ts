/**
 * Text Surface Proof Alpha — Mock streaming chunks and helpers.
 *
 * Provides inert, client-side mock text for the Assistant Drawer streaming
 * proof. No network calls, no model/agent coupling.
 */

export const MOCK_STREAM_CHUNKS: string[] = [
  "The ",
  "first ",
  "signs ",
  "of ",
  "spring ",
  "arrived ",
  "quietly. ",
  "\n",
  "Birds ",
  "began ",
  "to ",
  "trace \n",
  "new ",
  "patterns ",
  "across ",
  "the ",
  "sky, ",
  "and ",
  "the ",
  "air ",
  "carried ",
  "a ",
  "scent ",
  "of ",
  "blossom ",
  "and ",
  "rain. ",
  "\n",
  "In ",
  "the ",
  "distance, ",
  "a ",
  "mountain ",
  "ridge ",
  "slowly ",
  "emerged ",
  "from ",
  "the ",
  "morning ",
  "mist—",
  "a ",
  "reminder ",
  "that ",
  "every ",
  "landscape ",
  "is ",
  "also ",
  "a ",
  "text ",
  "waiting ",
  "to ",
  "be ",
  "measured. ",
];

/** Build a deterministic chunk schedule from the mock corpus. */
export function buildMockChunks(): string[] {
  return MOCK_STREAM_CHUNKS.slice();
}

/** Simple chunk producer with randomised pacing for visual variety. */
export function createMockChunkProducer(chunks?: string[]) {
  const source = chunks ?? buildMockChunks();
  let index = 0;
  return {
    next(): string | null {
      if (index >= source.length) return null;
      return source[index++]!;
    },
    get done() {
      return index >= source.length;
    },
    get total() {
      return source.length;
    },
    reset() {
      index = 0;
    },
  };
}
