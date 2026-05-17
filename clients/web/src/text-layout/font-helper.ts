/**
 * Optional Text Engine Alpha — Font loading helper (T5).
 *
 * Provides non-blocking font readiness checks for the text surface.
 * Works with the browser Font Loading API (`document.fonts`). In non-browser
 * contexts, every font is reported as "unsupported" so callers can skip
 * font-dependent layout gracefully.
 *
 * This module does NOT force-blocking — it returns a promise that resolves
 * when the font is ready, or immediately if the Font Loading API is absent.
 */

// --- Font load state ---

/** The load state of a given font family. */
export type FontLoadState = {
  /** The font family name that was queried. */
  readonly family: string;
  /** Current status: loaded, loading, unloaded, or unsupported (non-browser). */
  readonly status: "loaded" | "loading" | "unloaded" | "unsupported";
  /** If status is "loading", whether we are currently awaiting the font. */
  readonly pending?: boolean;
};

// --- Browser detection ---

function hasFontLoadingAPI(): boolean {
  return (
    typeof document !== "undefined" &&
    typeof document.fonts !== "undefined" &&
    typeof document.fonts.load === "function"
  );
}

// --- Public API ---

/**
 * Ensure a font family is loaded (non-blocking).
 *
 * Uses the browser Font Loading API to trigger a load for the given family
 * and returns a promise that resolves when the font is ready. If the API is
 * not available (SSR, non-browser), resolves immediately.
 *
 * This function never throws — if the font cannot be loaded, it still resolves
 * (the browser will fall back to a system font).
 *
 * @param family — the font family name (e.g. `"Inter"`)
 * @param testText — optional test string to force glyph loading (default: `"AaBbCc"`)
 * @returns Promise that resolves when the font is loaded or unavailable
 */
export async function ensureTextSurfaceFontLoaded(
  family: string,
  testText: string = "AaBbCc",
): Promise<void> {
  if (!hasFontLoadingAPI()) return;

  try {
    // Use the FontFaceSet.load() API — this triggers font download if not
    // yet loaded, and returns immediately if already available.
    await document.fonts.load(`16px "${family}"`, testText);
  } catch {
    // Font load failure is non-fatal. The browser will use a fallback.
  }
}

/**
 * Describe the current font loading state for a given family.
 *
 * Returns a `FontLoadState` snapshot. In non-browser contexts, always returns
 * `{ family, status: "unsupported" }`.
 *
 * @param family — the font family name (e.g. `"Inter"`)
 * @returns FontLoadState snapshot
 */
export function describeFontLoadState(family: string): FontLoadState {
  if (!hasFontLoadingAPI()) {
    return { family, status: "unsupported" };
  }

  try {
    // Check if any FontFace for this family is already loaded
    const faces = [...document.fonts];
    const matching = faces.filter((f) => f.family === family);

    if (matching.length === 0) {
      return { family, status: "unloaded" };
    }

    const allLoaded = matching.every((f) => f.status === "loaded");
    const anyLoading = matching.some((f) => f.status === "loading");

    if (allLoaded) {
      return { family, status: "loaded" };
    }
    if (anyLoading) {
      return { family, status: "loading", pending: true };
    }
    // Some faces may be in "unloaded" state (not yet triggered)
    return { family, status: "unloaded" };
  } catch {
    return { family, status: "unsupported" };
  }
}

// --- Batch font loading ---

/**
 * Ensure multiple font families are loaded in parallel.
 *
 * @param families — list of font family names
 * @returns Promise that resolves when all fonts are loaded or unavailable
 */
export async function ensureFontsLoaded(families: string[]): Promise<void> {
  await Promise.all(families.map((f) => ensureTextSurfaceFontLoaded(f)));
}

/**
 * Describe font loading states for multiple families.
 *
 * @param families — list of font family names
 * @returns array of FontLoadState snapshots
 */
export function describeFontLoadStates(families: string[]): FontLoadState[] {
  return families.map((f) => describeFontLoadState(f));
}
