/**
 * Optional Text Engine Alpha — Engine selection configuration and feature flags.
 *
 * Resolves the preferred text engine from multiple sources:
 *   1. URL search param: ?text-engine=fallback|pretext|auto
 *   2. localStorage key:  ygg_text_engine
 *   3. Global/env hint:    globalThis.__YGG_TEXT_ENGINE__
 *
 * Default is "auto" — which means: use Pretext if available, otherwise fallback.
 *
 * This module also provides the TextEnginePreference type and utilities for
 * the async initialization flow.
 */

import type { TextEngineName } from "./engine.js";

// --- Preference type ---

/**
 * User-preference values for text engine selection.
 * - "auto": use Pretext if available, otherwise fallback
 * - "fallback": always use the fallback engine
 * - "pretext": require Pretext (fail with diagnostic error if unavailable)
 */
export type TextEnginePreference = "auto" | "fallback" | "pretext";

/** Well-known engine names that are also valid preference values. */
export const ENGINE_PREFERENCE_VALUES: readonly TextEnginePreference[] = [
  "auto",
  "fallback",
  "pretext",
] as const;

// --- Config keys ---

const STORAGE_KEY = "ygg_text_engine";
const URL_PARAM = "text-engine";

// --- Preference resolution ---

/**
 * Parse and validate a text engine preference string.
 * Returns undefined if the value is not a recognized preference.
 */
export function parseTextEnginePreference(value: string | null | undefined): TextEnginePreference | undefined {
  if (!value) return undefined;
  const normalized = value.trim().toLowerCase();
  if (ENGINE_PREFERENCE_VALUES.includes(normalized as TextEnginePreference)) {
    return normalized as TextEnginePreference;
  }
  return undefined;
}

/**
 * Resolve the preferred text engine from URL, localStorage, or global env.
 * Returns the raw preference value, or "auto" if no preference is set.
 *
 * Priority order:
 *   1. URL param `?text-engine=` (highest, for testing)
 *   2. localStorage `ygg_text_engine`
 *   3. globalThis.__YGG_TEXT_ENGINE__ (SSR/build-time embed)
 *   4. Default: "auto"
 */
export function resolveTextEnginePreference(): TextEnginePreference {
  // 1. URL param (highest priority, for testing)
  if (typeof URLSearchParams !== "undefined" && typeof location !== "undefined") {
    const params = new URLSearchParams(location.search);
    const urlPref = parseTextEnginePreference(params.get(URL_PARAM));
    if (urlPref) return urlPref;
  }

  // 2. localStorage
  if (typeof localStorage !== "undefined") {
    const stored = localStorage.getItem(STORAGE_KEY);
    const storedPref = parseTextEnginePreference(stored);
    if (storedPref) return storedPref;
  }

  // 3. Environment string (window.__YGG_TEXT_ENGINE__ for SSR/build-time embed)
  if (typeof globalThis !== "undefined") {
    const envEngine = (globalThis as any).__YGG_TEXT_ENGINE__;
    if (typeof envEngine === "string") {
      const envPref = parseTextEnginePreference(envEngine);
      if (envPref) return envPref;
    }
  }

  // 4. Default
  return "auto";
}

/**
 * Convert a TextEnginePreference to a concrete TextEngineName.
 * For "auto", returns undefined — the caller must resolve it based on availability.
 * For "fallback" or "pretext", returns the corresponding engine name.
 */
export function preferenceToEngineName(preference: TextEnginePreference): TextEngineName | undefined {
  switch (preference) {
    case "fallback":
      return "fallback";
    case "pretext":
      return "pretext";
    case "auto":
      return undefined; // resolve based on availability
  }
}

/**
 * Persist the engine preference to localStorage.
 */
export function persistTextEnginePreference(preference: TextEnginePreference): void {
  if (typeof localStorage !== "undefined") {
    try {
      localStorage.setItem(STORAGE_KEY, preference);
    } catch {
      // localStorage may be unavailable (SSR, privacy mode)
    }
  }
}

// --- Initialization result type ---

/**
 * Result of the async engine initialization flow.
 * Describes which engine was preferred, which became active, and why.
 */
export type TextEngineInitializationResult = {
  /** The preference that was resolved (from URL/localStorage/env). */
  readonly preference: TextEnginePreference;
  /** The engine name that was initially preferred (before availability check). */
  readonly preferredEngine: TextEngineName;
  /** The engine name that became active after initialization. */
  readonly activeEngine: TextEngineName;
  /** Whether the preferred engine was successfully activated. */
  readonly success: boolean;
  /** Reason for fallback, if the preferred engine was not activated. */
  readonly fallbackReason?: string;
  /** Whether Pretext module was found and loaded. */
  readonly pretextAvailable: boolean;
  /** Error from Pretext load attempt, if any. */
  readonly pretextLoadError?: string;
};
