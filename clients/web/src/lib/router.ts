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
 *   #/project/<projectId>             Project frame (mounted iframe)
 */

import { useEffect, useState } from "react";

export type SettingsTab =
  | "api-connections"
  | "installed-packages"
  | "profiles"
  | "storage"
  | "about";

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

function parseHash(hash: string): Route {
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
      return { kind: "project", projectId: decodeURIComponent(rest[0]) };
    } catch {
      return { kind: "home" };
    }
  }
  return { kind: "home" };
}

function serializeRoute(route: Route): string {
  switch (route.kind) {
    case "home":
      return "#/";
    case "settings":
      return `#/settings/${route.tab}`;
    case "project":
      return `#/project/${encodeURIComponent(route.projectId)}`;
  }
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
