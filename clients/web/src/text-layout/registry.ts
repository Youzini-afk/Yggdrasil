/**
 * Text Surface Proof Alpha — TextEngine registry and selection.
 *
 * Provides registerTextEngine / activateTextEngine / getActiveTextEngine /
 * getTextEngineState / selectTextEngine. Default is the fallback engine.
 *
 * T3 adds:
 * - `initializeTextEnginePreference()`: async init that tries to activate
 *   the preferred engine (from URL/localStorage/env), falls back gracefully.
 * - `getInitializationResult()`: returns the last init result for diagnostics.
 * - `isPretextAvailable()`: checks Pretext module availability.
 * - Enhanced `resolveEnginePreference()` now returns "auto" by default and
 *   supports the TextEnginePreference type from config.ts.
 */

import type { TextEngine, TextEngineConfig, TextEngineDiagnostics, TextEngineName, TextEngineState } from "./engine.js";
import { FallbackTextEngine } from "./fallback-engine.js";
import { PretextTextEngine, isPretextAvailable as checkPretextAvailable, getPretextLoadError } from "./pretext-engine.js";
import {
  resolveTextEnginePreference,
  preferenceToEngineName,
  persistTextEnginePreference,
  type TextEnginePreference,
  type TextEngineInitializationResult,
} from "./config.js";

// --- Registry internals ---

type RegisteredEngine = {
  engine: TextEngine;
  config: TextEngineConfig;
  isFallback: boolean;
  error?: string;
};

const registry = new Map<TextEngineName, RegisteredEngine>();
let activeEngineName: TextEngineName = "fallback";
let initialised = false;

/** Track initialization result for diagnostics. */
let lastInitResult: TextEngineInitializationResult | null = null;

/** Ensure the fallback engine is always registered. */
function ensureInit(): void {
  if (initialised) return;
  initialised = true;
  const fallback = new FallbackTextEngine();
  registry.set("fallback", {
    engine: fallback,
    config: fallback.config,
    isFallback: true,
  });
  // Activate the fallback
  fallback.activate();
  activeEngineName = "fallback";
}

// --- Public API ---

/**
 * Register a TextEngine instance. If this is the first non-fallback engine
 * and matches the stored preference, it becomes active automatically.
 * Returns true if registration succeeded.
 */
export function registerTextEngine(engine: TextEngine): boolean {
  ensureInit();
  const name = engine.config.name;
  if (registry.has(name)) {
    return false; // already registered
  }
  registry.set(name, {
    engine,
    config: engine.config,
    isFallback: name === "fallback",
  });

  // If user preference matches this engine, activate it
  const preferred = resolveTextEnginePreference();
  const preferredName = preferenceToEngineName(preferred);
  if (preferredName !== undefined && preferredName === name) {
    activateTextEngine(name);
  }
  return true;
}

/**
 * Activate a registered engine by name. Deactivates the previous engine.
 * Returns true if the engine was found and activated.
 */
export function activateTextEngine(name: TextEngineName): boolean {
  ensureInit();
  const entry = registry.get(name);
  if (!entry) return false;

  // Deactivate previous
  const prev = registry.get(activeEngineName);
  if (prev && "deactivate" in prev.engine && typeof (prev.engine as any).deactivate === "function") {
    (prev.engine as any).deactivate();
  }

  // Activate new
  if ("activate" in entry.engine && typeof (entry.engine as any).activate === "function") {
    (entry.engine as any).activate();
  }
  activeEngineName = name;
  persistEnginePreference(name);
  return true;
}

/**
 * Get the currently active TextEngine. Always returns a valid engine
 * (falls back to the built-in fallback if the active engine is removed).
 *
 * This is a synchronous API — it returns whatever engine is currently active.
 * For async initialization, use `initializeTextEnginePreference()`.
 */
export function getActiveTextEngine(): TextEngine {
  ensureInit();
  const entry = registry.get(activeEngineName);
  if (entry) return entry.engine;
  // Safety: always return fallback
  const fallback = registry.get("fallback");
  if (fallback) return fallback.engine;
  // Should never happen, but create a new one as last resort
  return new FallbackTextEngine();
}

/**
 * Get the state of a specific registered engine.
 */
export function getTextEngineState(name: TextEngineName): TextEngineState | "unavailable" {
  ensureInit();
  const entry = registry.get(name);
  if (!entry) return "unavailable";
  return entry.engine.state;
}

/**
 * Select an engine by name (alias for activateTextEngine, returns the engine).
 * If the name is not registered, returns the current active engine unchanged.
 */
export function selectTextEngine(name: TextEngineName): TextEngine {
  ensureInit();
  if (registry.has(name)) {
    activateTextEngine(name);
  }
  return getActiveTextEngine();
}

/**
 * Get diagnostics for all registered engines.
 */
export function getTextEngineDiagnostics(): TextEngineDiagnostics[] {
  ensureInit();
  const result: TextEngineDiagnostics[] = [];
  for (const [name, entry] of registry) {
    result.push({
      name,
      version: entry.config.version,
      state: entry.engine.state,
      isFallback: entry.isFallback,
      description: entry.config.description,
      error: entry.error,
    });
  }
  return result;
}

/**
 * Get the name of the currently active engine.
 */
export function getActiveTextEngineName(): TextEngineName {
  ensureInit();
  return activeEngineName;
}

/**
 * Unregister a non-fallback engine. Returns true if removed.
 * If the removed engine was active, falls back to "fallback".
 */
export function unregisterTextEngine(name: TextEngineName): boolean {
  if (name === "fallback") return false; // cannot remove fallback
  const removed = registry.delete(name);
  if (removed && activeEngineName === name) {
    activeEngineName = "fallback";
    const fallback = registry.get("fallback");
    if (fallback && "activate" in fallback.engine && typeof (fallback.engine as any).activate === "function") {
      (fallback.engine as any).activate();
    }
  }
  return removed;
}

// --- T3: Async engine initialization ---

/**
 * Asynchronously initialize the text engine based on user preference.
 *
 * This function:
 * 1. Resolves the user's preferred engine from URL/localStorage/env.
 * 2. If preference is "auto" or "pretext", attempts to load and register
 *    the Pretext engine.
 * 3. If Pretext loads successfully and preference allows, activates it.
 * 4. If Pretext fails or preference is "fallback", uses the fallback engine.
 * 5. Records the result for diagnostics.
 *
 * After calling this, the synchronous `getActiveTextEngine()` returns the
 * resolved engine.
 *
 * This function is idempotent — calling it multiple times is safe (it will
 * only attempt initialization once unless reset).
 */
export async function initializeTextEnginePreference(): Promise<TextEngineInitializationResult> {
  ensureInit();

  const preference = resolveTextEnginePreference();
  const preferredName = preferenceToEngineName(preference) ?? "pretext"; // "auto" → try pretext first

  let pretextAvailable = false;
  let pretextLoadError: string | undefined;

  // If preference is "fallback", skip Pretext entirely
  if (preference === "fallback") {
    const result: TextEngineInitializationResult = {
      preference,
      preferredEngine: "fallback",
      activeEngine: "fallback",
      success: true,
      fallbackReason: undefined,
      pretextAvailable: false,
    };
    lastInitResult = result;
    return result;
  }

  // Try to load Pretext
  const pretextEngine = new PretextTextEngine();
  try {
    await pretextEngine.initialize();
    // Register the engine
    const registered = registerTextEngine(pretextEngine);
    if (registered) {
      // Set error field on registry entry to null (no error)
      const entry = registry.get("pretext");
      if (entry) entry.error = undefined;
    }
    pretextAvailable = true;
  } catch (err) {
    pretextLoadError = err instanceof Error ? err.message : String(err);
    // Register the engine in error state so diagnostics can report it
    const registered = registerTextEngine(pretextEngine);
    if (registered) {
      const entry = registry.get("pretext");
      if (entry) entry.error = pretextLoadError;
    }
    pretextAvailable = false;
  }

  // Decide which engine to activate
  if (pretextAvailable && (preference === "auto" || preference === "pretext")) {
    activateTextEngine("pretext");
    const result: TextEngineInitializationResult = {
      preference,
      preferredEngine: preferredName,
      activeEngine: "pretext",
      success: true,
      fallbackReason: undefined,
      pretextAvailable: true,
    };
    lastInitResult = result;
    return result;
  }

  // Fallback
  const reason = pretextLoadError
    ? `Pretext unavailable: ${pretextLoadError}`
    : (preference === "pretext"
      ? "Pretext explicitly requested but unavailable"
      : "No Pretext module found; using fallback");

  // Ensure fallback is active
  activateTextEngine("fallback");

  const result: TextEngineInitializationResult = {
    preference,
    preferredEngine: preferredName,
    activeEngine: "fallback",
    success: preference !== "pretext", // success if auto and fell back, failure if pretext was required
    fallbackReason: reason,
    pretextAvailable,
    pretextLoadError,
  };
  lastInitResult = result;
  return result;
}

/**
 * Get the result of the last `initializeTextEnginePreference()` call.
 * Returns null if initialization has not been called yet.
 */
export function getInitializationResult(): TextEngineInitializationResult | null {
  return lastInitResult;
}

/**
 * Check if Pretext is available (module loaded successfully).
 * Returns true only if the Pretext module has been loaded and cached.
 */
export function isPretextAvailable(): boolean {
  return checkPretextAvailable();
}

/**
 * Get the Pretext load error, if any.
 * Returns null if no load has been attempted or if load succeeded.
 */
export function getPretextAvailabilityError(): string | null {
  return getPretextLoadError();
}

// --- Preference resolution (localStorage / URL / env string) ---

const STORAGE_KEY = "ygg_text_engine";
const URL_PARAM = "text-engine";

/**
 * Resolve the preferred engine name from localStorage, URL search params,
 * or environment strings. Returns the engine name string or undefined.
 *
 * This is the legacy sync resolver that returns TextEngineName.
 * For the richer preference resolution, use `resolveTextEnginePreference()`
 * from config.ts.
 */
export function resolveEnginePreference(): TextEngineName | undefined {
  // 1. URL param (highest priority, for testing)
  if (typeof URLSearchParams !== "undefined" && typeof location !== "undefined") {
    const params = new URLSearchParams(location.search);
    const urlEngine = params.get(URL_PARAM);
    if (urlEngine) return urlEngine;
  }

  // 2. localStorage
  if (typeof localStorage !== "undefined") {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) return stored;
  }

  // 3. Environment string (window.__YGG_TEXT_ENGINE__ for SSR/build-time embed)
  if (typeof globalThis !== "undefined") {
    const envEngine = (globalThis as any).__YGG_TEXT_ENGINE__;
    if (typeof envEngine === "string" && envEngine) return envEngine;
  }

  return undefined;
}

/**
 * Persist the active engine name to localStorage.
 */
function persistEnginePreference(name: TextEngineName): void {
  if (typeof localStorage !== "undefined") {
    try {
      localStorage.setItem(STORAGE_KEY, name);
    } catch {
      // localStorage may be unavailable (SSR, privacy mode)
    }
  }
}
