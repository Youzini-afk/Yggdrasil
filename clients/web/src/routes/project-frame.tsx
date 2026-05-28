import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { ArrowLeft, ArrowsClockwise, BookOpenText, StopCircle } from "@/components/icons";
import { Button } from "@/components/ui/button";
import { StatusPill, projectStateTone } from "@/components/ui/status-pill";
import { Tooltip } from "@/components/ui/tooltip";
import { useKernel } from "@/lib/kernel-client";
import { useRoute } from "@/lib/router";
import { useToast } from "@/components/ui/toast";
import { useT } from "@/lib/locale";
import { openProjectInTab } from "@/lib/project-launcher";
import { mountSurface, type SurfaceHostHandle } from "@/surfaces/surface-host";
import { resolveSurfaceBundle, type ResolvedSurfaceBundle } from "@/surfaces/bundle-resolver";
import { formatBytes } from "@/lib/format";
import type { KernelEvent, PackageRecord, ProjectRecord, SurfaceContributionRecord, UpdateCheckResult } from "@/protocol/client";

const FRAME_CONTAINER_ID = "ygg-project-frame";
const UPDATE_AVAILABLE_STATUSES = new Set(["available", "update_available", "repair_required"]);

interface ProjectDiagnostics {
  bundle?: ResolvedSurfaceBundle;
  packages: PackageRecord[];
  events: KernelEvent[];
  updates?: UpdateCheckResult;
  errors: string[];
  refreshedAt: string;
}

interface ConsoleSummary {
  packageTotal: number;
  packageHealthy: number;
  packageProblem: number;
  recentEvents: number;
  updateAvailable: number;
  updateChecked: boolean;
}

export function ProjectFrame({ projectId, chrome = "shell" }: { projectId: string; chrome?: "shell" | "none" }) {
  const client = useKernel();
  const toast = useToast();
  const t = useT();
  const [, navigate] = useRoute();
  const [project, setProject] = useState<(ProjectRecord & { running_session_id?: string; entry_surface_id?: string }) | null>(null);
  const [stopping, setStopping] = useState(false);
  const [refreshingDiagnostics, setRefreshingDiagnostics] = useState(false);
  const [updatingProject, setUpdatingProject] = useState(false);
  const [diagnostics, setDiagnostics] = useState<ProjectDiagnostics | null>(null);
  const [frameState, setFrameState] = useState<"loading" | "mounted" | "start_failed" | "mount_failed" | "stopped">("loading");
  const handleRef = useRef<SurfaceHostHandle | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const projectRef = useRef(project);
  const [mountAttempt, setMountAttempt] = useState(0);

  useEffect(() => {
    projectRef.current = project;
  }, [project]);

  useEffect(() => {
    if (typeof document === "undefined") return;
    const previousTitle = document.title;
    document.title = chrome === "none"
      ? `${project?.title ?? projectId} — Yggdrasil`
      : `${project?.title ?? projectId} console — Yggdrasil`;
    return () => {
      document.title = previousTitle;
    };
  }, [chrome, project?.title, projectId]);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        setFrameState("loading");
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

        if (!detail.entry_surface_id) {
          if (chrome !== "none") {
            setFrameState("mounted");
            return;
          }
          setFrameState("start_failed");
          return;
        }
        const bundle = await resolveSurfaceBundle(client, detail.entry_surface_id);
        if (chrome !== "none") {
          setDiagnostics((current) => ({
            bundle,
            packages: current?.packages ?? [],
            events: current?.events ?? [],
            updates: current?.updates,
            errors: current?.errors ?? [],
            refreshedAt: current?.refreshedAt ?? new Date().toISOString(),
          }));
          setFrameState("mounted");
          return;
        }

        // Installed native projects can resolve to a copied project bundle even
        // when the contributing surface package has not been autoloaded into the
        // current host process yet (for example immediately after a web install).
        // Treat contribution metadata as best-effort so an available project UI
        // does not fail before the iframe has a chance to mount.
        const contribution = await client
          .describeSurface(detail.entry_surface_id)
          .catch((): SurfaceContributionRecord | null => null);
        if (cancelled) return;
        const allowedCapabilityIds = allowedSurfaceCapabilityIdsForTest(contribution);
        let handle: SurfaceHostHandle;
        try {
          handle = await mountSurface({
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
        } catch (err) {
          setFrameState("mount_failed");
          toast.push({
            variant: "error",
            title: t("projectFrameMountFailedTitle"),
            body: t("projectFrameMountFailedBody"),
          });
          return;
        }
        if (cancelled) {
          await handle.unmount();
          return;
        }
        handleRef.current = handle;
        setFrameState("mounted");
      } catch (err) {
        setFrameState("start_failed");
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
  }, [chrome, client, mountAttempt, projectId, toast, t]);

  const onRetry = useCallback(() => {
    void handleRef.current?.unmount().catch(() => {});
    handleRef.current = null;
    setMountAttempt((attempt) => attempt + 1);
  }, []);

  const onStop = useCallback(async () => {
    if (stopping) return;
    if (typeof window !== "undefined" && !window.confirm(t("projectFrameStopConfirm"))) return;
    setStopping(true);
    try {
      await client.stopProject(projectId);
      toast.push({ variant: "success", title: t("projectFrameStopped", project?.title ?? projectId) });
      await handleRef.current?.unmount().catch(() => {});
      handleRef.current = null;
      setProject((current) => current ? { ...current, state: "stopped", running_session_id: undefined } : current);
      setFrameState("stopped");
    } catch (err) {
      toast.push({
        variant: "error",
        title: t("projectFrameStopFailedTitle"),
        body: t("projectFrameStopFailedBody"),
      });
    } finally {
      setStopping(false);
    }
  }, [client, project?.title, projectId, stopping, t, toast]);

  const onOpenProjectTab = useCallback(() => {
    const opened = openProjectInTab(projectId);
    if (!opened) {
      toast.push({
        variant: "warning",
        title: t("projectFrameProjectTabBlockedTitle"),
        body: t("projectFrameProjectTabBlockedBody"),
      });
    }
  }, [projectId, t, toast]);

  useEffect(() => {
    if (chrome !== "none") return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (!event.isTrusted) return;
      if (!(event.metaKey || event.ctrlKey)) return;
      if (event.key !== "." && event.code !== "Period") return;
      const target = event.target as Element | null;
      if (target?.closest('input, textarea, select, [contenteditable="true"]')) return;
      event.preventDefault();
      void onStop();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [chrome, onStop]);

  const loadDiagnostics = useCallback(async () => {
    setRefreshingDiagnostics(true);
    const errors: string[] = [];
    let latestProject = projectRef.current;
    try {
      const fetchedProject = await client.getProject(projectId);
      latestProject = fetchedProject;
      setProject((current) => ({
        ...(fetchedProject as ProjectRecord & { running_session_id?: string; entry_surface_id?: string }),
        state: (current?.state === "running" && !fetchedProject.running_session_id ? current.state : fetchedProject.state),
        running_session_id: fetchedProject.running_session_id ?? current?.running_session_id,
      }));
    } catch (err) {
      errors.push(errorMessage(err));
    }

    const currentProject = projectRef.current;
    const entrySurfaceId = latestProject?.entry_surface_id ?? currentProject?.entry_surface_id;
    const sessionId = latestProject?.running_session_id ?? currentProject?.running_session_id;
    const declaredPackageRefs = latestProject?.packages ?? currentProject?.packages ?? [];

    const [bundleResult, packageListResult, eventResult, updateResult] = await Promise.allSettled([
      entrySurfaceId ? resolveSurfaceBundle(client, entrySurfaceId) : Promise.resolve(undefined),
      client.packages(),
      sessionId ? client.listEvents(sessionId) : Promise.resolve([]),
      client.checkProjectUpdates(projectId),
    ]);

    const bundle = unwrapSettled(bundleResult, errors);
    const packageList = unwrapSettled(packageListResult, errors) ?? [];
    const events = unwrapSettled(eventResult, errors) ?? [];
    const updates = unwrapSettled(updateResult, errors);
    const projectPackages = filterProjectPackages(packageList, declaredPackageRefs, projectId, updates?.results ?? []);

    setDiagnostics({
      bundle,
      packages: projectPackages,
      events: [...events].slice(-8).reverse(),
      updates,
      errors,
      refreshedAt: new Date().toISOString(),
    });
    setRefreshingDiagnostics(false);
  }, [client, projectId]);

  useEffect(() => {
    if (chrome === "none") return;
    void loadDiagnostics();
  }, [chrome, loadDiagnostics]);

  const onRefreshDiagnostics = useCallback(() => {
    void loadDiagnostics();
  }, [loadDiagnostics]);

  const onUpdateProject = useCallback(async () => {
    if (updatingProject) return;
    setUpdatingProject(true);
    try {
      const result = await client.updateProject(projectId);
      const updated = result.updated_packages?.length ?? 0;
      toast.push({
        variant: result.updated ? "success" : "info",
        title: result.updated ? t("projectFrameUpdateCompleteTitle") : t("projectFrameUpdateCurrentTitle"),
        body: result.updated
          ? t("projectFrameUpdateCompleteBody", updated)
          : result.reason ?? result.status ?? t("projectFrameUpdateCurrentBody"),
      });
      await loadDiagnostics();
    } catch (err) {
      toast.push({
        variant: "error",
        title: t("projectFrameUpdateFailedTitle"),
        body: errorMessage(err),
      });
    } finally {
      setUpdatingProject(false);
    }
  }, [client, loadDiagnostics, projectId, t, toast, updatingProject]);

  const consoleSummary = useMemo(() => summarizeConsoleDiagnostics(diagnostics), [diagnostics]);
  const isStandalone = chrome === "none";

  return (
    <div className={isStandalone ? "flex h-[100dvh] flex-col overflow-hidden bg-warm-bone" : "flex min-h-[calc(100dvh-60px)] flex-col"}>
      {isStandalone ? null : (
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
          <Tooltip label={t("projectFrameRefreshDiagnostics")}>
            <Button tone="tertiary" size="sm" onClick={onRefreshDiagnostics} disabled={refreshingDiagnostics}>
              <ArrowsClockwise size={14} />
              {refreshingDiagnostics ? t("projectFrameRefreshing") : t("projectFrameRefresh")}
            </Button>
          </Tooltip>
          <Button tone="tertiary" size="sm" onClick={onUpdateProject} disabled={updatingProject}>
            {updatingProject ? t("projectFrameUpdating") : t("projectFrameUpdateProject")}
          </Button>
          <span className="mx-2 hidden h-4 w-px bg-whisper-border sm:inline-block" aria-hidden />
          {project?.state === "running" ? (
            <Tooltip label={t("projectFrameStopProject")}>
              <Button tone="icon" size="icon-sm" onClick={onStop} disabled={stopping} aria-label={t("projectFrameStop")}>
                <StopCircle size={16} className="text-deep-rust" />
              </Button>
            </Tooltip>
          ) : null}
        </div>
      </div>
      )}

      <div className="relative min-h-0 flex-1 overflow-hidden">
        {isStandalone ? (
          <div
            ref={containerRef}
            id={FRAME_CONTAINER_ID}
            className="h-full w-full"
            style={{ background: "var(--color-warm-bone)" }}
          />
        ) : (
          <div className="h-full overflow-auto bg-warm-bone p-4 sm:p-6">
            <ProjectConsole
              projectId={projectId}
              project={project}
              diagnostics={diagnostics}
              summary={consoleSummary}
              loading={refreshingDiagnostics && !diagnostics}
              stopping={stopping}
              updating={updatingProject}
              onOpenProjectTab={onOpenProjectTab}
              onRefresh={onRefreshDiagnostics}
              onUpdate={onUpdateProject}
              onStop={onStop}
            />
          </div>
        )}
        {frameState === "loading" ? (
          <div className="pointer-events-none absolute inset-0 grid place-items-center bg-warm-bone">
            <div className="min-w-[280px] max-w-[420px] rounded-[24px] border border-whisper-border bg-pure-surface p-5 shadow-card">
              <p className="font-display text-[18px] font-bold text-charcoal-ink">{project?.title ?? projectId}</p>
              <p className="mt-2 text-[13px] text-steel-secondary">{t("projectFrameLoadingSurface")}</p>
            </div>
          </div>
        ) : null}
        {frameState === "start_failed" || frameState === "mount_failed" || frameState === "stopped" ? (
          <div className="absolute inset-0 grid place-items-center bg-warm-bone p-6">
            <div className="max-w-[460px] rounded-[24px] border border-whisper-border bg-pure-surface p-6 text-center shadow-card">
              <p className="font-display text-[20px] font-bold text-charcoal-ink">
                {frameState === "stopped"
                  ? t("projectFrameStoppedTitle")
                  : frameState === "start_failed"
                    ? t("projectFrameStartFailedTitle")
                    : t("projectFrameMountFailedTitle")}
              </p>
              <p className="mt-2 text-[13px] leading-relaxed text-steel-secondary">
                {frameState === "stopped"
                  ? t("projectFrameStoppedBody")
                  : frameState === "start_failed"
                    ? t("projectFrameStartFailedBody")
                    : t("projectFrameMountFailedBody")}
              </p>
              <div className="mt-5 flex flex-col justify-center gap-2 sm:flex-row">
                {frameState !== "stopped" ? (
                  <Button tone="primary" size="sm" onClick={onRetry}>
                    {t("retry")}
                  </Button>
                ) : null}
                {frameState === "mount_failed" && project?.state === "running" ? (
                  <Button tone="secondary" size="sm" onClick={onStop} disabled={stopping}>
                    {t("projectFrameStopProject")}
                  </Button>
                ) : null}
                <Button tone="tertiary" size="sm" onClick={() => navigate({ kind: "home" })}>
                  {t("projectFrameBackHome")}
                </Button>
              </div>
            </div>
          </div>
        ) : null}
      </div>
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

export function summarizeConsoleDiagnostics(diagnostics: ProjectDiagnostics | null): ConsoleSummary {
  const packages = diagnostics?.packages ?? [];
  const updateResults = diagnostics?.updates?.results ?? [];
  return {
    packageTotal: packages.length,
    packageHealthy: packages.filter((pkg) => pkg.state === "ready" || pkg.state === "running").length,
    packageProblem: packages.filter((pkg) => pkg.state === "degraded" || pkg.state === "failed" || pkg.last_failure).length,
    recentEvents: diagnostics?.events.length ?? 0,
    updateAvailable: updateResults.filter((record) => record.available || UPDATE_AVAILABLE_STATUSES.has(record.status ?? "")).length,
    updateChecked: Boolean(diagnostics?.updates),
  };
}

function ProjectConsole({
  projectId,
  project,
  diagnostics,
  summary,
  loading,
  stopping,
  updating,
  onOpenProjectTab,
  onRefresh,
  onUpdate,
  onStop,
}: {
  projectId: string;
  project: (ProjectRecord & { running_session_id?: string; entry_surface_id?: string }) | null;
  diagnostics: ProjectDiagnostics | null;
  summary: ConsoleSummary;
  loading: boolean;
  stopping: boolean;
  updating: boolean;
  onOpenProjectTab: () => void;
  onRefresh: () => void;
  onUpdate: () => void;
  onStop: () => void;
}) {
  const t = useT();
  const bundle = diagnostics?.bundle;
  const updateResults = diagnostics?.updates?.results ?? [];
  const updateStatus = loading
    ? t("projectFrameDiagnosticsLoading")
    : !summary.updateChecked
      ? t("projectFrameUpdateUnavailable")
      : summary.updateAvailable > 0
        ? t("projectFrameUpdatesAvailable", summary.updateAvailable)
        : t("projectFrameUpdatesCurrent");
  const projectStatusItems = [
    [t("projectFrameProjectId"), project?.id ?? projectId],
    [t("projectFrameProjectType"), formatProjectType(project?.type)],
    [t("projectFrameSession"), project?.running_session_id ? t("projectFrameActiveSession") : "—"],
    [t("projectFrameStorage"), project?.storage_summary ? formatBytes(project.storage_summary.total_bytes ?? undefined) : "—"],
  ];

  if (loading && !diagnostics) return <ProjectConsoleSkeleton />;

  return (
    <div className="mx-auto max-w-6xl space-y-5">
      <section className="rounded-[24px] border border-whisper-border bg-pure-surface p-5 shadow-card sm:p-6">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <p className="font-display text-[22px] font-bold text-charcoal-ink">{t("projectFrameConsoleTitle")}</p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Button tone="primary" size="sm" onClick={onOpenProjectTab}>{t("projectFrameOpenProjectTab")}</Button>
            <Button tone="secondary" size="sm" onClick={onRefresh} disabled={loading}>
              <ArrowsClockwise size={14} />
              {loading ? t("projectFrameRefreshing") : t("projectFrameRefresh")}
            </Button>
            <Button tone="secondary" size="sm" onClick={onUpdate} disabled={updating}>{updating ? t("projectFrameUpdating") : t("projectFrameUpdateProject")}</Button>
            {project?.state === "running" ? <Button tone="destructive" size="sm" onClick={onStop} disabled={stopping}>{t("projectFrameStopProject")}</Button> : null}
          </div>
        </div>

        <div className="mt-5 grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
          <MetricCard label={t("projectFramePackages") } value={t("projectFramePackageHealth", summary.packageHealthy, summary.packageTotal)} warn={summary.packageProblem > 0} />
          <MetricCard label={t("projectFrameUpdates") } value={updateStatus} warn={summary.updateAvailable > 0} />
          <MetricCard label={t("projectFrameActivity") } value={t("projectFrameRecentEvents", summary.recentEvents)} />
        </div>

        <dl className="mt-5 grid gap-3 text-[12px] sm:grid-cols-2 xl:grid-cols-4">
          {projectStatusItems.map(([label, value]) => <KeyValue key={label} label={label} value={value} />)}
        </dl>
      </section>

      <div className="grid gap-5 xl:grid-cols-[1.05fr_0.95fr]">
        <ConsoleSection title={t("projectFrameInterfaceSection")} description={t("projectFrameInterfaceDescription")}>
          <div className="space-y-3">
            <KeyValue label={t("projectFrameEntrySurface")} value={project?.entry_surface_id ?? "—"} />
            <KeyValue label={t("projectFrameBundleUrl")} value={bundle?.bundleUrl ?? t("projectFrameBundleUnavailable")} />
            <KeyValue label={t("projectFrameBundleFingerprint")} value={bundle?.bundleFingerprint ?? fingerprintFromUrl(bundle?.bundleUrl) ?? "—"} />
            <KeyValue label={t("projectFrameLastResolved")} value={diagnostics?.refreshedAt ? new Date(diagnostics.refreshedAt).toLocaleString() : "—"} />
          </div>
        </ConsoleSection>

        <ConsoleSection title={t("projectFrameUpdatesSection")} description={t("projectFrameUpdatesDescription")}>
          <div className="space-y-3">
            <StatusPill tone={summary.updateAvailable > 0 ? "update" : "neutral"} label={updateStatus} />
            {updateResults.length === 0 ? <p className="text-[12px] text-steel-secondary">{summary.updateChecked ? t("projectFrameNoUpdateRecords") : t("projectFrameUpdateUnavailable")}</p> : null}
            {updateResults.slice(0, 4).map((record) => (
              <div key={record.package_id ?? record.id ?? record.reason} className="rounded-[12px] border border-whisper-border bg-warm-bone p-3">
                <p className="truncate font-mono text-[12px] text-charcoal-ink">{record.package_id ?? record.id ?? "—"}</p>
                <p className="mt-1 text-[12px] text-steel-secondary">{record.reason ?? record.status ?? "—"}</p>
              </div>
            ))}
          </div>
        </ConsoleSection>
      </div>

      <ConsoleSection title={t("projectFramePackagesSection")} description={t("projectFramePackagesDescription")}>
        {diagnostics?.packages.length ? (
          <div className="grid gap-3 lg:grid-cols-2">
            {diagnostics.packages.map((pkg) => <PackageDiagnosticCard key={pkg.id} pkg={pkg} />)}
          </div>
        ) : <p className="text-[13px] text-steel-secondary">{loading ? t("projectFrameDiagnosticsLoading") : t("projectFrameNoPackages")}</p>}
      </ConsoleSection>

      <ConsoleSection title={t("projectFrameActivitySection")} description={t("projectFrameActivityDescription")}>
        {diagnostics?.events.length ? (
          <div className="space-y-2">
            {diagnostics.events.map((event) => (
              <div key={event.id} className="flex gap-3 rounded-[12px] border border-whisper-border bg-warm-bone p-3 text-[12px]">
                <span className="font-mono text-muted-tone">#{event.sequence}</span>
                <div className="min-w-0 flex-1">
                  <p className="truncate text-charcoal-ink">{humanEventKind(event.kind)}</p>
                  <p className="truncate text-steel-secondary">{event.writer_package_id} · {formatEventTime(event.created_at)}</p>
                </div>
              </div>
            ))}
          </div>
        ) : <p className="text-[13px] text-steel-secondary">{project?.running_session_id ? t("projectFrameNoEvents") : t("projectFrameNoSession")}</p>}
      </ConsoleSection>

      {diagnostics?.errors.length ? (
        <ConsoleSection title={t("projectFrameDiagnosticsWarnings")}>
          <ul className="space-y-2 text-[12px] text-deep-rust">
            {diagnostics.errors.slice(0, 4).map((error) => <li key={error}>{error}</li>)}
          </ul>
        </ConsoleSection>
      ) : null}
    </div>
  );
}

function MetricCard({ label, value, warn = false }: { label: string; value: string; warn?: boolean }) {
  return <div className="rounded-[16px] border border-whisper-border bg-warm-bone p-4"><p className="font-mono text-[10px] uppercase tracking-[0.12em] text-muted-tone">{label}</p><p className={warn ? "mt-2 text-[14px] font-semibold text-deep-rust" : "mt-2 text-[14px] font-semibold text-charcoal-ink"}>{value}</p></div>;
}

function ProjectConsoleSkeleton() {
  return (
    <div className="mx-auto max-w-6xl space-y-5">
      <section className="rounded-[24px] border border-whisper-border bg-pure-surface p-5 shadow-card sm:p-6">
        <div className="h-6 w-48 rounded-full bg-whisper-border" />
        <div className="mt-5 grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
          {Array.from({ length: 3 }).map((_, index) => <div key={index} className="h-20 rounded-[16px] bg-warm-bone" />)}
        </div>
      </section>
      <div className="grid gap-5 xl:grid-cols-2">
        <div className="h-56 rounded-[20px] border border-whisper-border bg-pure-surface shadow-card" />
        <div className="h-56 rounded-[20px] border border-whisper-border bg-pure-surface shadow-card" />
      </div>
      <div className="h-64 rounded-[20px] border border-whisper-border bg-pure-surface shadow-card" />
    </div>
  );
}

function ConsoleSection({ title, description, children }: { title: string; description?: string; children: ReactNode }) {
  return <section className="rounded-[20px] border border-whisper-border bg-pure-surface p-5 shadow-card"><p className="font-display text-[18px] font-bold text-charcoal-ink">{title}</p>{description ? <p className="mt-1 text-[12px] leading-relaxed text-steel-secondary">{description}</p> : null}<div className="mt-4">{children}</div></section>;
}

function KeyValue({ label, value }: { label: string; value: string }) {
  return <div className="min-w-0 rounded-[12px] border border-whisper-border bg-warm-bone p-3"><dt className="font-mono text-[10px] uppercase tracking-[0.12em] text-muted-tone">{label}</dt><dd className="mt-1 truncate font-mono text-[12px] text-charcoal-ink" title={value}>{value}</dd></div>;
}

function PackageDiagnosticCard({ pkg }: { pkg: PackageRecord }) {
  const t = useT();
  const failure = pkg.last_failure;
  return <div className="rounded-[14px] border border-whisper-border bg-warm-bone p-4"><div className="flex items-start justify-between gap-3"><div className="min-w-0"><p className="truncate font-mono text-[12px] text-charcoal-ink">{pkg.id}</p><p className="mt-1 font-mono text-[11px] text-muted-tone">{pkg.version} · {pkg.entry_kind}</p></div><StatusPill tone={projectStateTone(pkg.state)} label={pkg.state} /></div><p className="mt-3 text-[12px] text-steel-secondary">{t("projectFramePackageCounts", pkg.capability_count, pkg.hook_count)}</p>{failure ? <p className="mt-2 rounded-[10px] bg-deep-rust-surface p-2 text-[12px] text-deep-rust">{failure.reason}</p> : null}{failure?.log_tail_redacted?.length ? <pre className="mt-2 max-h-24 overflow-auto rounded-[10px] bg-pure-surface p-2 font-mono text-[11px] text-steel-secondary">{failure.log_tail_redacted.slice(-3).map((line) => line.line).join("\n")}</pre> : null}</div>;
}

function filterProjectPackages(packages: PackageRecord[], refs: string[], projectId: string, updates: Array<{ package_id?: string; id?: string }>): PackageRecord[] {
  const refTokens = refs.flatMap((ref) => [ref, ref.split("/").at(-2), ref.split("/").at(-1)?.replace(/\.ya?ml$/, "")]).filter(Boolean) as string[];
  const updatePackageIds = new Set(
    updates
      .flatMap((record) => [record.package_id, record.id])
      .filter((value): value is string => Boolean(value)),
  );
  const filtered = packages.filter(
    (pkg) =>
      pkg.id === projectId ||
      updatePackageIds.has(pkg.id) ||
      refTokens.some((token) => pkg.id === token || pkg.id.endsWith(`/${token}`) || pkg.id.endsWith(`__${token}`)),
  );
  return filtered.length > 0 ? filtered : packages;
}

function unwrapSettled<T>(result: PromiseSettledResult<T>, errors: string[]): T | undefined {
  if (result.status === "fulfilled") return result.value;
  errors.push(errorMessage(result.reason));
  return undefined;
}

function errorMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

function fingerprintFromUrl(bundleUrl?: string): string | undefined {
  if (!bundleUrl) return undefined;
  try {
    const url = new URL(bundleUrl, typeof window === "undefined" ? "http://localhost" : window.location.origin);
    return url.searchParams.get("v") ?? undefined;
  } catch {
    return undefined;
  }
}

function formatProjectType(type?: string): string {
  if (!type) return "—";
  return type
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function humanEventKind(kind: string): string {
  return kind
    .split(/[./_-]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function formatEventTime(value?: string): string {
  if (!value) return "—";
  const timestamp = Date.parse(value);
  if (!Number.isFinite(timestamp)) return value;
  return new Date(timestamp).toLocaleString();
}
