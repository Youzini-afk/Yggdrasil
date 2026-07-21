import type { YggProtocolClient } from "../protocol/client";

export interface ResolvedSurfaceBundle {
  surfaceId: string;
  bundleUrl: string;
  bundleFingerprint?: string;
  exportName: string;
  stylesheets: string[];
  wrapperClass?: string;
  projectId?: string;
  source: "installed_project" | "dev_path";
}

export async function resolveSurfaceBundle(
  client: YggProtocolClient,
  surfaceId: string,
): Promise<ResolvedSurfaceBundle> {
  const result = await client.invoke("host.surface.bundle.resolve", { surface_id: surfaceId });
  const r = result as {
    surface_id: string;
    bundle_url: string;
    bundle_fingerprint?: string;
    export_name: string;
    stylesheets?: string[];
    wrapper_class?: string;
    project_id?: string;
    source: "installed_project" | "dev_path";
  };
  return {
    surfaceId: r.surface_id,
    bundleUrl: r.bundle_url,
    bundleFingerprint: r.bundle_fingerprint,
    exportName: r.export_name,
    stylesheets: r.stylesheets ?? [],
    wrapperClass: r.wrapper_class,
    projectId: r.project_id,
    source: r.source,
  };
}
