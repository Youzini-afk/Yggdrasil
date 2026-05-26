import { useEffect, useRef, useState } from "react";
import { ArrowLeft, BookOpenText, DotsThree, ListBullets, StopCircle } from "@/components/icons";
import { Button } from "@/components/ui/button";
import { StatusPill, projectStateTone } from "@/components/ui/status-pill";
import { Tooltip } from "@/components/ui/tooltip";
import { useKernel } from "@/lib/kernel-client";
import { useRoute } from "@/lib/router";
import { useToast } from "@/components/ui/toast";
import { useT } from "@/lib/locale";
import { mountSurface, type SurfaceHostHandle } from "@/surfaces/surface-host";
import { resolveSurfaceBundle } from "@/surfaces/bundle-resolver";
import type { ProjectRecord, SurfaceContributionRecord } from "@/protocol/client";

const FRAME_CONTAINER_ID = "ygg-project-frame";

export function ProjectFrame({ projectId }: { projectId: string }) {
  const client = useKernel();
  const toast = useToast();
  const t = useT();
  const [, navigate] = useRoute();
  const [project, setProject] = useState<(ProjectRecord & { running_session_id?: string }) | null>(null);
  const [stopping, setStopping] = useState(false);
  const handleRef = useRef<SurfaceHostHandle | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const detail = await client.getProject(projectId);
        if (cancelled) return;

        // Capture the session id from start_project — the project record from
        // get_project is a stale snapshot, and ignoring the new id would mount
        // the surface with sessionId=undefined.
        let sessionId = detail.running_session_id;
        let runtimeState = detail.state;
        if (!sessionId) {
          const started = await client.startProject(projectId);
          if (cancelled) return;
          sessionId = started.session_id;
          runtimeState = (started.new_state as typeof runtimeState) ?? "running";
        }

        // Reflect the live state in the project frame topbar.
        setProject({ ...detail, state: runtimeState, running_session_id: sessionId });

        if (!detail.entry_surface_id) return;
        const [bundle, contribution] = await Promise.all([
          resolveSurfaceBundle(client, detail.entry_surface_id),
          client.describeSurface(detail.entry_surface_id).catch<SurfaceContributionRecord | null>(() => null),
        ]);
        if (cancelled) return;
        const allowedCapabilityIds = allowedSurfaceCapabilityIdsForTest(contribution);
        const handle = await mountSurface({
          containerId: FRAME_CONTAINER_ID,
          surfaceId: bundle.surfaceId,
          bundleUrl: bundle.bundleUrl,
          exportName: bundle.exportName,
          stylesheets: bundle.stylesheets,
          wrapperClass: bundle.wrapperClass,
          initialProps: { projectId },
          hostBridge: {
            currentSessionId: sessionId,
            allowedCapabilityIds,
            callRpc: (method, params) => client.invokeWithSession(method, params, sessionId),
            subscribeEvents: (cb) =>
              client.subscribeEvents(sessionId, (event) => cb(event)),
          },
        });
        if (cancelled) {
          await handle.unmount();
          return;
        }
        handleRef.current = handle;
      } catch (err) {
        toast.push({
          variant: "error",
          title: t("projectFrameStartFailedTitle"),
          body: t("projectFrameStartFailedBody"),
        });
      }
    })();

    return () => {
      cancelled = true;
      handleRef.current?.unmount().catch(() => {});
      handleRef.current = null;
    };
  }, [client, projectId, toast, t]);

  const onStop = async () => {
    setStopping(true);
    try {
      await client.stopProject(projectId);
      toast.push({ variant: "success", title: t("projectFrameStopped", project?.title ?? projectId) });
      navigate({ kind: "home" });
    } catch (err) {
      toast.push({
        variant: "error",
        title: t("projectFrameStopFailedTitle"),
        body: t("projectFrameStopFailedBody"),
      });
    } finally {
      setStopping(false);
    }
  };

  return (
    <div className="flex min-h-[calc(100dvh-60px)] flex-col">
      {/* Project frame topbar */}
      <div className="flex h-10 items-center justify-between border-b border-whisper-border bg-pure-surface px-3 sm:px-4">
        <div className="flex min-w-0 items-center gap-2 sm:gap-3">
          <Tooltip label={t("projectFrameBackHome")}>
            <Button tone="icon" size="icon-sm" onClick={() => navigate({ kind: "home" })} aria-label={t("projectFrameBackHome")}>
              <ArrowLeft size={16} />
            </Button>
          </Tooltip>
          <BookOpenText size={16} className="hidden text-aged-brass sm:inline" />
          <span className="truncate font-display text-[14px] font-bold leading-none text-charcoal-ink">
            {project?.title ?? projectId}
          </span>
          <StatusPill
            tone={projectStateTone(project?.state ?? "starting")}
            label={t("projectFrameState", project?.state ?? "starting")}
          />
        </div>
        <div className="flex items-center gap-1">
          <Button tone="tertiary" size="sm" className="hidden sm:inline-flex">
            <ListBullets size={14} />
            {t("projectFrameAuditLog")}
          </Button>
          <span className="mx-2 hidden h-4 w-px bg-whisper-border sm:inline-block" aria-hidden />
          {project?.state === "running" ? (
            <Tooltip label={t("projectFrameStopProject")}>
              <Button tone="icon" size="icon-sm" onClick={onStop} disabled={stopping} aria-label={t("projectFrameStop")}>
                <StopCircle size={16} className="text-deep-rust" />
              </Button>
            </Tooltip>
          ) : null}
          <Tooltip label={t("projectFrameMore")}>
            <Button tone="icon" size="icon-sm" aria-label={t("projectFrameMore")}>
              <DotsThree size={16} />
            </Button>
          </Tooltip>
        </div>
      </div>

      {/* Iframe surface — neutral platform-bone background until the project
          paints. The project owns its territory once mounted. */}
      <div
        ref={containerRef}
        id={FRAME_CONTAINER_ID}
        className="flex-1"
        style={{ background: "var(--color-warm-bone)" }}
      />
    </div>
  );
}

export function allowedSurfaceCapabilityIdsForTest(record: SurfaceContributionRecord | null): Set<string> {
  const ids = new Set<string>();
  const surface = record?.surface;
  if (surface?.capability_id) ids.add(surface.capability_id);
  if (surface?.activation.launch_capability_id) ids.add(surface.activation.launch_capability_id);
  for (const capabilityId of surface?.allowed_capability_ids ?? []) ids.add(capabilityId);
  return ids;
}
