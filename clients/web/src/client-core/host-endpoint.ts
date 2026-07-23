const DEFAULT_LOCAL_HOST = "http://127.0.0.1:8787";

export interface HostEndpointProvider {
  readonly baseUrl: string;
}

export interface YggRuntimeBootstrap {
  hostBaseUrl?: string;
  platform?: "web" | "desktop" | "pwa";
}

declare global {
  interface Window {
    __YGG_RUNTIME__?: YggRuntimeBootstrap;
  }
}

export function normalizeHostBaseUrl(value: string): string {
  return value.replace(/\/+$/, "");
}

/** Resolve the control-plane endpoint once, independently of React. */
export function resolveHostBaseUrl(explicit?: string): string {
  if (explicit) return normalizeHostBaseUrl(explicit);

  if (typeof window !== "undefined") {
    const injected = window.__YGG_RUNTIME__?.hostBaseUrl;
    if (injected) return normalizeHostBaseUrl(injected);
  }

  if (typeof location !== "undefined" && location.origin && location.origin !== "null") {
    return normalizeHostBaseUrl(location.origin);
  }

  return DEFAULT_LOCAL_HOST;
}

export function createHostEndpointProvider(explicit?: string): HostEndpointProvider {
  return Object.freeze({ baseUrl: resolveHostBaseUrl(explicit) });
}
