/**
 * Text Surface Proof Alpha — TextEngine registry and selection.
 *
 * Provides registerTextEngine / activateTextEngine / getActiveTextEngine /
 * getTextEngineState / selectTextEngine. Default is the fallback engine.
 *
 * Supports parsing engine preference from localStorage, URL params, or
 * environment strings — but T3 will wire these to Pretext selection.
 */

import type { TextEngine, TextEngineConfig, TextEngineDiagnostics, TextEngineName, TextEngineState } from "./engine.js";
import { FallbackTextEngine } from "./fallback-engine.js";

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
  // Try to resolve initial preference
  const preferred = resolveEnginePreference();
  if (preferred && preferred !== "fallback") {
    // Will be resolved when the preferred engine is registered later (T3)
    activeEngineName = "fallback";
  } else {
    activeEngineName = "fallback";
  }
  // Activate the fallback
  fallback.activate();
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
  const preferred = resolveEnginePreference();
  if (preferred === name) {
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

// --- Preference resolution (localStorage / URL / env string) ---

const STORAGE_KEY = "ygg_text_engine";
const URL_PARAM = "text-engine";

/**
 * Resolve the preferred engine name from localStorage, URL search params,
 * or environment strings. Returns undefined if no preference is set.
 *
 * T3 will extend this to support Pretext feature flags.
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
