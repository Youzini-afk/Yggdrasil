/**
 * Tiny in-app router — hash-based, three routes plus query/state params.
 *
 * We intentionally avoid React Router for a desktop shell with five static
 * pages. Hash routing avoids server config, works in Tauri WebView, and
 * survives reloads.
 *
 * Route grammar:
 *   #/                                Home
 *   #/settings/api-connections        Settings page
 *   #/settings/installed-packages     ...
 *   #/settings/profiles
 *   #/settings/storage
 *   #/settings/about
 *   /project/<projectId>              Chrome-free project tab host
 */

import { useEffect, useState } from "react";

export type SettingsTab =
  | "api-connections"
  | "installed-packages"
  | "profiles"
  | "storage"
  | "about";

const MAX_PROJECT_ID_LENGTH = 128;
const PROJECT_ID_PATTERN = /^[A-Za-z0-9][A-Za-z0-9._:@-]*$/;

export type Route =
  | { kind: "home" }
  | { kind: "settings"; tab: SettingsTab }
  | { kind: "project"; projectId: string };

export const SETTINGS_TABS: Array<{ id: SettingsTab; label: string }> = [
  { id: "api-connections", label: "API Connections" },
  { id: "installed-packages", label: "Installed Packages" },
  { id: "profiles", label: "Profiles" },
  { id: "storage", label: "Storage" },
  { id: "about", label: "About" },
];

export function parseHash(hash: string): Route {
  const path = hash.replace(/^#/, "").replace(/^\//, "");
  if (!path) return { kind: "home" };
  const [head, ...rest] = path.split("/");
  if (head === "settings") {
    const tab = (rest[0] ?? "api-connections") as SettingsTab;
    if (SETTINGS_TABS.some((t) => t.id === tab)) {
      return { kind: "settings", tab };
    }
    return { kind: "settings", tab: "api-connections" };
  }
  if (head === "project" && rest[0]) {
    // Malformed escapes (e.g., "%") would throw — fall back to Home.
    try {
      const projectId = decodeURIComponent(rest[0]);
      return isValidProjectId(projectId) ? { kind: "project", projectId } : { kind: "home" };
    } catch {
      return { kind: "home" };
    }
  }
  return { kind: "home" };
}

function encodeRouteProjectId(projectId: string): string {
  // Keep project ids with `/` out of hash routes too. Canonical project tabs use
  // `/project/<id>` and project ids are a single URL segment there.
  return encodeURIComponent(projectId);
}

export function serializeRoute(route: Route): string {
  switch (route.kind) {
    case "home":
      return "#/";
    case "settings":
      return `#/settings/${route.tab}`;
    case "project":
      return `#/project/${encodeRouteProjectId(route.projectId)}`;
  }
}

export function isValidProjectId(value: string): boolean {
  return value.length > 0
    && value.length <= MAX_PROJECT_ID_LENGTH
    && !value.includes("/")
    && !/[\u0000-\u001F\u007F]/.test(value)
    && PROJECT_ID_PATTERN.test(value);
}

export function projectPath(projectId: string): string {
  if (!isValidProjectId(projectId)) throw new Error("invalid project id");
  return `/project/${encodeURIComponent(projectId)}`;
}

export function parseProjectPath(pathname: string): { kind: "project"; projectId: string } | null {
  if (!pathname.startsWith("/project/")) return null;
  const suffix = pathname.slice("/project/".length);
  if (!suffix || suffix.includes("/")) return null;
  try {
    const projectId = decodeURIComponent(suffix);
    return isValidProjectId(projectId) ? { kind: "project", projectId } : null;
  } catch {
    return null;
  }
}

export function usePathProjectRoute(): { kind: "project"; projectId: string } | null {
  const [route, setRoute] = useState<{ kind: "project"; projectId: string } | null>(() =>
    typeof window === "undefined" ? null : parseProjectPath(window.location.pathname),
  );

  useEffect(() => {
    const onPopState = () => setRoute(parseProjectPath(window.location.pathname));
    window.addEventListener("popstate", onPopState);
    return () => window.removeEventListener("popstate", onPopState);
  }, []);

  return route;
}

export function useRoute(): [Route, (next: Route) => void] {
  const [route, setRoute] = useState<Route>(() =>
    typeof window === "undefined" ? { kind: "home" } : parseHash(window.location.hash),
  );

  useEffect(() => {
    const onHashChange = () => setRoute(parseHash(window.location.hash));
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, []);

  const navigate = (next: Route) => {
    const hash = serializeRoute(next);
    if (window.location.hash !== hash) {
      window.location.hash = hash;
    } else {
      setRoute(next);
    }
  };

  return [route, navigate];
}

export function routeLabel(route: Route): string {
  switch (route.kind) {
    case "home":
      return "Home";
    case "settings": {
      const tab = SETTINGS_TABS.find((t) => t.id === route.tab);
      return tab ? `Settings / ${tab.label}` : "Settings";
    }
    case "project":
      return `Projects / ${route.projectId}`;
  }
}
