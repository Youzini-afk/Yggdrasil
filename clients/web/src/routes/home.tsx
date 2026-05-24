import { useEffect, useMemo, useState } from "react";
import { Hero } from "@/components/home/hero";
import { UtilityStrip, type FilterChip } from "@/components/home/utility-strip";
import { ProjectCard } from "@/components/home/project-card";
import { InstallCard } from "@/components/home/install-card";
import { ActivityTimeline, type TimelineRow } from "@/components/home/activity-timeline";
import {
  WorkshopUtilities,
  QUICK_ACTION_ICONS,
  type DiskSegment,
} from "@/components/home/workshop-utilities";
import type { ActivityRow } from "@/components/home/activity-micro-card";
import { Eyebrow } from "@/components/ui/typography";
import { Skeleton } from "@/components/ui/skeleton";
import { EmptyState } from "@/components/ui/empty-state";
import { Plus, Warning } from "@/components/icons";
import { Button } from "@/components/ui/button";
import { useAsync, useKernel } from "@/lib/kernel-client";
import { useRoute } from "@/lib/router";
import { useToast } from "@/components/ui/toast";
import { formatGreetingTime, formatRelativeAge } from "@/lib/format";
import { InstallModal } from "@/components/install/install-modal";
import { FailureModal } from "@/components/install/failure-modal";
import { projectStateTone, type StatusTone } from "@/components/ui/status-pill";
import type { KernelEvent, ProjectRecord } from "@/protocol/client";

const FILTER_OPTIONS: FilterChip[] = [
  { id: "all", label: "All", count: 0 },
  { id: "running", label: "Running", count: 0, toneDot: "running" },
  { id: "stopped", label: "Stopped", count: 0, toneDot: "stopped" },
  { id: "failed", label: "Failed", count: 0, toneDot: "failed" },
];

const TONE_TO_DISK_CLASS: Record<string, string> = {
  running: "bg-aged-brass",
  stopped: "bg-steel-secondary",
  failed: "bg-deep-rust",
};

const TIMELINE_SESSION = "kernel_project_lifecycle";

/**
 * Map an event payload's structural hints to a timeline icon. Heuristic only —
 * we deliberately do not parse package-internal payloads.
 */
function iconKindFor(event: KernelEvent): TimelineRow["iconKind"] {
  const kind = event.kind.toLowerCase();
  if (kind.includes("crash") || kind.includes("fail")) return "crash";
  if (kind.includes("install")) return "package";
  if (kind.includes("checkpoint")) return "checkpoint";
  if (kind.includes("retry")) return "retry";
  if (kind.includes("outbound")) return "outbound";
  if (kind.includes("secret")) return "secret";
  return "default";
}

export function HomePage() {
  const client = useKernel();
  const toast = useToast();
  const [, navigate] = useRoute();

  const projects = useAsync(() => client.listProjects(), [client]);
  const lifecycleEvents = useAsync(
    () => client.listEvents(TIMELINE_SESSION).catch<KernelEvent[]>(() => []),
    [client],
  );

  const [search, setSearch] = useState("");
  const [activeFilter, setActiveFilter] = useState("all");
  const [showInstall, setShowInstall] = useState(false);
  const [failureProjectId, setFailureProjectId] = useState<string | null>(null);

  // Cmd/Ctrl + N opens the install modal.
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "n") {
        event.preventDefault();
        setShowInstall(true);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  const projectList = projects.data ?? [];
  const counts = useMemo(() => {
    const running = projectList.filter((p) => p.state === "running").length;
    const stopped = projectList.filter((p) => p.state === "stopped" || p.state === "installed").length;
    const failed = projectList.filter((p) => p.state === "failed").length;
    return { all: projectList.length, running, stopped, failed };
  }, [projectList]);

  const filters: FilterChip[] = FILTER_OPTIONS.map((option) => ({
    ...option,
    count: counts[option.id as keyof typeof counts] ?? 0,
  }));

  const filtered = useMemo(() => {
    return projectList.filter((p) => {
      const matchesFilter =
        activeFilter === "all" ||
        (activeFilter === "stopped" && (p.state === "stopped" || p.state === "installed")) ||
        p.state === activeFilter;
      const matchesSearch =
        !search ||
        p.title.toLowerCase().includes(search.toLowerCase()) ||
        (p.description ?? "").toLowerCase().includes(search.toLowerCase());
      return matchesFilter && matchesSearch;
    });
  }, [projectList, activeFilter, search]);

  // Disk usage from project storage summaries supplied by the runtime.
  const diskSegments: DiskSegment[] = useMemo(() => {
    return projectList.map((project) => ({
      id: project.id,
      label: project.title,
      bytes: project.storage_summary?.total_bytes ?? null,
      measurementState: project.storage_summary?.measurement_state ?? "unknown",
      toneClass: TONE_TO_DISK_CLASS[project.state] ?? "bg-steel-secondary",
    }));
  }, [projectList]);

  const totalDisk = useMemo(
    () => diskSegments.reduce((sum, segment) => sum + (segment.bytes ?? 0), 0),
    [diskSegments],
  );
  const diskCapacity = Math.max(totalDisk, 1);

  const recentActivity: ActivityRow[] = useMemo(
    () =>
      projectList
        .filter((p) => p.state === "running" || p.state === "stopped")
        .slice(0, 2)
        .map((project) => ({
          id: project.id,
          projectName: project.title,
          toneDot: projectStateTone(project.state),
          age: project.state === "running" ? "now" : "—",
          action: {
            label: project.state === "running" ? "Resume" : "Open",
            onClick: () => onLaunch(project.id),
          },
        })),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [projectList],
  );

  // Build timeline from real lifecycle events. Empty when there are none.
  const timelineRows: TimelineRow[] = useMemo(() => {
    const events = lifecycleEvents.data ?? [];
    return events
      .slice(-8)
      .reverse()
      .map((event) => {
        const project = projectList.find((p) => p.id === (event.metadata as { project_id?: string })?.project_id);
        return {
          id: event.id,
          projectName: project?.title ?? event.writer_package_id ?? "kernel",
          toneDot: project ? projectStateTone(project.state) : ("neutral" as StatusTone),
          age: formatRelativeAge(event.created_at),
          message: event.kind.replace(/^kernel\/v1\//, ""),
          iconKind: iconKindFor(event),
        } satisfies TimelineRow;
      });
  }, [lifecycleEvents.data, projectList]);

  const onLaunch = (projectId: string) => {
    navigate({ kind: "project", projectId });
  };

  const onStop = async (projectId: string, title: string) => {
    try {
      await client.stopProject(projectId);
      toast.push({ variant: "success", title: `Stopped ${title}` });
      projects.refresh();
    } catch (err) {
      toast.push({
        variant: "error",
        title: "Stop failed",
        body: err instanceof Error ? err.message : String(err),
      });
    }
  };

  const onUninstall = (title: string) => {
    toast.push({
      variant: "info",
      title: `Uninstall ${title}`,
      body: `Confirm in CLI: yg uninstall ${title}`,
    });
  };

  const onInstallClick = () => setShowInstall(true);

  const onShowFailure = (projectId: string) => setFailureProjectId(projectId);

  const onCardLaunch = (project: ProjectRecord) => {
    if (project.state === "failed") {
      onShowFailure(project.id);
      return;
    }
    onLaunch(project.id);
  };

  return (
    <div className="mx-auto flex w-full max-w-[1400px] flex-col gap-8 px-8 py-10">
      <Hero
        meta={formatGreetingTime()}
        greeting="Welcome back"
        summary={
          projects.loading
            ? "Reading your workshop…"
            : counts.all > 0
              ? `${counts.all} projects on the shelf. ${counts.running} running, ${counts.stopped} idle. ${
                  counts.failed > 0 ? `${counts.failed} need attention.` : "No pending updates."
                }`
              : "Your workshop is empty. Install a project to begin."
        }
        recentActivity={recentActivity}
      />

      <div className="grid grid-cols-1 gap-8 lg:grid-cols-[60fr_40fr]">
        <div className="flex flex-col gap-5">
          <UtilityStrip
            search={search}
            onSearchChange={setSearch}
            filters={filters}
            activeFilter={activeFilter}
            onFilterChange={setActiveFilter}
          />

          <div className="flex items-center justify-between">
            <Eyebrow>Projects — {counts.all.toString().padStart(2, "0")} installed</Eyebrow>
          </div>

          {projects.error ? (
            <EmptyState
              icon={<Warning />}
              title="Couldn't reach the host"
              body={projects.error.message}
              action={{ label: "Retry", onClick: () => projects.refresh() }}
            />
          ) : projects.loading ? (
            <div
              className="grid gap-5"
              style={{ gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))" }}
            >
              {Array.from({ length: 4 }).map((_, idx) => (
                <Skeleton key={idx} className="h-[220px] rounded-[20px]" />
              ))}
            </div>
          ) : projectList.length === 0 ? (
            <EmptyState
              icon={<Plus />}
              title="No projects installed yet"
              body="Yggdrasil is your workshop. Install a project to begin — projects can be a Yggdrasil-native source like YdlTavern, or any external git/local repo."
              action={{ label: "Install a project", onClick: onInstallClick }}
            />
          ) : (
            <div
              className="grid gap-5"
              style={{ gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))" }}
            >
              {filtered.map((project, idx) => (
                <ProjectCard
                  key={project.id}
                  index={idx}
                  data={{
                    id: project.id,
                    title: project.title,
                    description: project.description,
                    state: project.state,
                    type: project.type,
                    source: project.type === "yggdrasil_native" ? "github" : "local",
                  }}
                  actions={{
                    onLaunch: () => onCardLaunch(project),
                    onStop: () => onStop(project.id, project.title),
                    onRestart: () => onCardLaunch(project),
                    onUninstall: () => onUninstall(project.title),
                    onConfigure: () => navigate({ kind: "settings", tab: "installed-packages" }),
                    onViewLogs:
                      project.state === "failed" ? () => onShowFailure(project.id) : undefined,
                  }}
                />
              ))}
              <InstallCard onClick={onInstallClick} index={filtered.length} />
            </div>
          )}
        </div>

        <div className="flex flex-col gap-6">
          <ActivityTimeline
            rows={timelineRows.slice(0, 6)}
            loading={lifecycleEvents.loading && timelineRows.length === 0}
            onViewAll={() => navigate({ kind: "settings", tab: "installed-packages" })}
          />
          <WorkshopUtilities
            updates={[]}
            totalDisk={totalDisk}
            diskCapacity={diskCapacity}
            diskSegments={diskSegments}
            quickActions={[
              {
                id: "install",
                label: "Install URL",
                shortcut: "⌘N",
                icon: QUICK_ACTION_ICONS.Plus,
                onClick: onInstallClick,
              },
              {
                id: "open-folder",
                label: "Open folder",
                shortcut: "⌘O",
                icon: QUICK_ACTION_ICONS.Folder,
                onClick: () => toast.push({ variant: "info", title: "Open ~/.yggdrasil" }),
              },
              {
                id: "settings",
                label: "Settings",
                shortcut: "⌘,",
                icon: QUICK_ACTION_ICONS.GearSix,
                onClick: () => navigate({ kind: "settings", tab: "api-connections" }),
              },
              {
                id: "switch-profile",
                label: "Switch profile",
                shortcut: "⌘P",
                icon: QUICK_ACTION_ICONS.Terminal,
                onClick: () => navigate({ kind: "settings", tab: "profiles" }),
              },
            ]}
          />
        </div>
      </div>

      <InstallModal open={showInstall} onClose={() => setShowInstall(false)} onInstalled={projects.refresh} />
      <FailureModal
        open={failureProjectId !== null}
        onClose={() => setFailureProjectId(null)}
        onRestart={() => {
          if (failureProjectId) navigate({ kind: "project", projectId: failureProjectId });
        }}
        onUninstall={() => {
          if (failureProjectId) onUninstall(failureProjectId);
        }}
        detail={
          failureProjectId
            ? {
                projectName:
                  projectList.find((p) => p.id === failureProjectId)?.title ?? failureProjectId,
              }
            : undefined
        }
      />
    </div>
  );
}
