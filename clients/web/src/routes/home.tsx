import { useMemo, useState } from "react";
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
import { useAsync, useKernel } from "@/lib/kernel-client";
import { useRoute } from "@/lib/router";
import { useToast } from "@/components/ui/toast";
import { formatGreetingTime } from "@/lib/format";
import { MOCK_PROJECTS, MOCK_TIMELINE } from "@/lib/home-data";
import type { StatusTone } from "@/components/ui/status-pill";
import type { ProjectRecord } from "@/protocol/client";

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

export function HomePage() {
  const client = useKernel();
  const toast = useToast();
  const [, navigate] = useRoute();

  const projects = useAsync(() => client.listProjects().catch(() => MOCK_PROJECTS as ProjectRecord[]), [client]);

  const [search, setSearch] = useState("");
  const [activeFilter, setActiveFilter] = useState("all");

  const projectList = projects.data ?? MOCK_PROJECTS;
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

  const totalSize = projectList.reduce((sum, p) => sum + ((p as { size_mb?: number }).size_mb ?? 0), 0);
  const diskSegments: DiskSegment[] = projectList
    .filter((p) => (p as { size_mb?: number }).size_mb)
    .map((p) => ({
      id: p.id,
      label: p.title,
      bytes: ((p as { size_mb?: number }).size_mb ?? 0) * 1_048_576,
      toneClass: TONE_TO_DISK_CLASS[p.state] ?? "bg-steel-secondary",
    }));

  const recentActivity: ActivityRow[] = projectList
    .filter((p) => p.state === "running" || p.state === "stopped")
    .slice(0, 2)
    .map((project, idx) => ({
      id: project.id,
      projectName: project.title,
      toneDot: project.state === "running" ? "running" : "stopped",
      age: idx === 0 ? "2h ago" : "yesterday",
      action: {
        label: project.state === "running" ? "Resume" : "Open",
        onClick: () => onLaunch(project.id),
      },
    }));

  const timelineRows: TimelineRow[] = MOCK_TIMELINE.map((row) => ({
    id: row.id,
    projectName: row.projectName,
    toneDot: row.tone as StatusTone,
    age: row.age,
    message: row.message,
    iconKind: row.iconKind,
    action: row.action ? { ...row.action, onClick: () => {} } : undefined,
  }));

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
      body: "Confirmation modal arrives in Phase 5.",
    });
  };

  const onInstallClick = () => {
    toast.push({
      variant: "info",
      title: "Install flow",
      body: "Install modal arrives in Phase 5. Use yg install <url> on the CLI for now.",
    });
  };

  return (
    <div className="mx-auto flex w-full max-w-[1400px] flex-col gap-8 px-8 py-10">
      <Hero
        meta={formatGreetingTime()}
        greeting="Welcome back"
        summary={
          counts.all > 0
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
            <Eyebrow>
              Projects — {counts.all.toString().padStart(2, "0")} installed
            </Eyebrow>
          </div>

          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
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
                  version: (project as { size_mb?: number }).size_mb ? "v0.1.0" : undefined,
                  source: project.type === "yggdrasil_native" ? "github" : "local",
                  sizeMB: (project as { size_mb?: number }).size_mb,
                  metricsLine: (project as { metrics?: string }).metrics,
                  failureLine:
                    project.state === "failed" ? (project as { metrics?: string }).metrics : undefined,
                }}
                actions={{
                  onLaunch: () => onLaunch(project.id),
                  onStop: () => onStop(project.id, project.title),
                  onRestart: () => onLaunch(project.id),
                  onUninstall: () => onUninstall(project.title),
                  onConfigure: () => navigate({ kind: "settings", tab: "installed-packages" }),
                }}
              />
            ))}
            <InstallCard onClick={onInstallClick} index={filtered.length} />
          </div>
        </div>

        <div className="flex flex-col gap-6">
          <ActivityTimeline
            rows={timelineRows.slice(0, 6)}
            onViewAll={() => navigate({ kind: "settings", tab: "installed-packages" })}
          />
          <WorkshopUtilities
            updates={[
              {
                id: "u1",
                packageId: "official/model-provider-lab",
                fromVersion: "v0.4.1",
                toVersion: "v0.5.0",
                onUpdate: () =>
                  toast.push({ variant: "info", title: "Update queued (mock)" }),
              },
              {
                id: "u2",
                packageId: "official/storage-lab",
                fromVersion: "v0.2.2",
                toVersion: "v0.2.4",
                onUpdate: () =>
                  toast.push({ variant: "info", title: "Update queued (mock)" }),
              },
            ]}
            totalDisk={totalSize * 1_048_576}
            diskCapacity={1024 * 1_048_576}
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
    </div>
  );
}
