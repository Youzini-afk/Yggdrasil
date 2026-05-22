// Bundle URL resolver.
//
// V0: hardcoded demo mapping. Production should read bundle/export metadata
// from manifest metadata and serve package assets through a host static route.

export interface ResolvedBundle {
  bundleUrl: string;
  exportName: string;
  wrapperClass?: string;
  stylesheets?: string[];
}

const YDLTAVERN_STYLESHEETS = [
  "/surface-bundles/ydltavern/styles/surface.css",
  "/surface-bundles/ydltavern/styles/mobile.css",
];

const DEMO_BUNDLES: Record<string, Record<string, ResolvedBundle>> = {
  "ydltavern/surface": {
    "ydltavern/play": {
      bundleUrl: "/surface-bundles/ydltavern/bundle.mjs",
      exportName: "mountTavernPlaySurface",
      wrapperClass: "ydltavern-surface",
      stylesheets: YDLTAVERN_STYLESHEETS,
    },
    "ydltavern/settings": {
      bundleUrl: "/surface-bundles/ydltavern/bundle.mjs",
      exportName: "mountTavernSettingsSurface",
      wrapperClass: "ydltavern-surface",
      stylesheets: YDLTAVERN_STYLESHEETS,
    },
    "ydltavern/extensions": {
      bundleUrl: "/surface-bundles/ydltavern/bundle.mjs",
      exportName: "mountTavernExtensionsSurface",
      wrapperClass: "ydltavern-surface",
      stylesheets: YDLTAVERN_STYLESHEETS,
    },
    "ydltavern/character": {
      bundleUrl: "/surface-bundles/ydltavern/bundle.mjs",
      exportName: "mountTavernCharactersSurface",
      wrapperClass: "ydltavern-surface",
      stylesheets: YDLTAVERN_STYLESHEETS,
    },
    "ydltavern/world-info": {
      bundleUrl: "/surface-bundles/ydltavern/bundle.mjs",
      exportName: "mountTavernWorldInfoSurface",
      wrapperClass: "ydltavern-surface",
      stylesheets: YDLTAVERN_STYLESHEETS,
    },
    "ydltavern/persona": {
      bundleUrl: "/surface-bundles/ydltavern/bundle.mjs",
      exportName: "mountTavernPersonaSurface",
      wrapperClass: "ydltavern-surface",
      stylesheets: YDLTAVERN_STYLESHEETS,
    },
    "ydltavern/ai-response-config": {
      bundleUrl: "/surface-bundles/ydltavern/bundle.mjs",
      exportName: "mountTavernAIResponseConfigSurface",
      wrapperClass: "ydltavern-surface",
      stylesheets: YDLTAVERN_STYLESHEETS,
    },
    "ydltavern/user-settings": {
      bundleUrl: "/surface-bundles/ydltavern/bundle.mjs",
      exportName: "mountTavernUserSettingsSurface",
      wrapperClass: "ydltavern-surface",
      stylesheets: YDLTAVERN_STYLESHEETS,
    },
    "ydltavern/backgrounds": {
      bundleUrl: "/surface-bundles/ydltavern/bundle.mjs",
      exportName: "mountTavernBackgroundsSurface",
      wrapperClass: "ydltavern-surface",
      stylesheets: YDLTAVERN_STYLESHEETS,
    },
  },
};

export function resolveSurfaceBundle(
  packageId: string,
  surfaceId: string,
  manifestMetadata?: Record<string, unknown>,
): ResolvedBundle | null {
  const pkgBundles = DEMO_BUNDLES[packageId];
  if (pkgBundles?.[surfaceId]) return pkgBundles[surfaceId];

  // TODO(production): read bundle/export metadata from manifestMetadata and
  // resolve it through a host-owned same-origin static package asset route.
  void manifestMetadata;
  return null;
}
