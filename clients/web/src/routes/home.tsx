import { Suspense, lazy, useCallback, useState } from "react";
import { Hero } from "@/components/home/hero";
import { UtilityStrip } from "@/components/home/utility-strip";
import { ProjectCard } from "@/components/home/project-card";
import { InstallCard } from "@/components/home/install-card";
import { ActivityTimeline } from "@/components/home/activity-timeline";
import { WorkshopUtilities, QUICK_ACTION_ICONS } from "@/components/home/workshop-utilities";
import { Eyebrow } from "@/components/ui/typography";
import { Skeleton } from "@/components/ui/skeleton";
import { EmptyState } from "@/components/ui/empty-state";
import { Plus, Warning } from "@/components/icons";
import { useKernel } from "@/lib/kernel-client";
import { useRoute, type Route } from "@/lib/router";
import { useToast } from "@/components/ui/toast";
import { formatGreetingTime } from "@/lib/format";
import type { FailureDetail } from "@/components/install/failure-modal";
import { useHomeProjects } from "./home/use-home-projects";
import { useProjectActions } from "./home/use-project-actions";

const InstallModal = lazy(() =>
  import("@/components/install/install-modal").then((module) => ({ default: module.InstallModal })),
);
const FailureModal = lazy(() =>
  import("@/components/install/failure-modal").then((module) => ({ default: module.FailureModal })),
);

export function HomePage() {
  const client = useKernel();
  const toast = useToast();
  const [, navigate] = useRoute();

  const [search, setSearch] = useState("");
  const [activeFilter, setActiveFilter] = useState("all");
  const [showInstall, setShowInstall] = useState(false);
  const [failureProjectId, setFailureProjectId] = useState<string | null>(null);
  const [failureDetail, setFailureDetail] = useState<FailureDetail>();
  const navigateTo = useCallback((route: Route) => navigate(route), [navigate]);
  const launchProject = useCallback(
    (projectId: string) => {
      navigateTo({ kind: "project", projectId });
    },
    [navigateTo],
  );

  const {
    projects,
    lifecycleEvents,
    projectList,
    counts,
    filters,
    filtered,
    diskSegments,
    totalDisk,
    diskCapacity,
    recentActivity,
    timelineRows,
  } = useHomeProjects({ client, search, activeFilter, onLaunch: launchProject });

  const { onStop, onUninstall, onInstallClick, onShowFailure, onCardLaunch } = useProjectActions({
    client,
    onLaunch: launchProject,
    pushToast: toast.push,
    refreshProjects: () => projects.refresh(),
    setShowInstall,
    setFailureProjectId,
    setFailureDetail,
  });

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
              body="Project inventory is unavailable. Try again from the local UI."
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
                    onConfigure: () => navigateTo({ kind: "settings", tab: "installed-packages" }),
                    onViewLogs:
                      project.state === "failed" ? () => void onShowFailure(project) : undefined,
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
            onViewAll={() => navigateTo({ kind: "settings", tab: "installed-packages" })}
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
                label: "Data folder",
                shortcut: "⌘O",
                icon: QUICK_ACTION_ICONS.Folder,
                onClick: () => toast.push({ variant: "info", title: "Use the CLI to open the local platform data directory." }),
              },
              {
                id: "settings",
                label: "Settings",
                shortcut: "⌘,",
                icon: QUICK_ACTION_ICONS.GearSix,
                onClick: () => navigateTo({ kind: "settings", tab: "api-connections" }),
              },
              {
                id: "switch-profile",
                label: "Switch profile",
                shortcut: "⌘P",
                icon: QUICK_ACTION_ICONS.Terminal,
                onClick: () => navigateTo({ kind: "settings", tab: "profiles" }),
              },
            ]}
          />
        </div>
      </div>

      {showInstall ? (
        <Suspense fallback={null}>
          <InstallModal open={showInstall} onClose={() => setShowInstall(false)} onInstalled={projects.refresh} />
        </Suspense>
      ) : null}
      {failureProjectId !== null ? (
        <Suspense fallback={null}>
          <FailureModal
            open={failureProjectId !== null}
            onClose={() => setFailureProjectId(null)}
            onRestart={() => {
              if (failureProjectId) navigateTo({ kind: "project", projectId: failureProjectId });
            }}
            onUninstall={() => {
              if (failureProjectId) onUninstall(failureProjectId);
            }}
            detail={failureDetail}
          />
        </Suspense>
      ) : null}
    </div>
  );
}
