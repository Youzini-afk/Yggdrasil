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
import { useLocale } from "@/lib/locale";
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
  const { locale, t } = useLocale();

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
  } = useHomeProjects({
    client,
    search,
    activeFilter,
    onLaunch: launchProject,
    labels: {
      all: t("homeFilterAll"),
      running: t("homeFilterRunning"),
      stopped: t("homeFilterStopped"),
      failed: t("homeFilterFailed"),
      now: t("homeNow"),
      resume: t("homeActionResume"),
      open: t("homeActionOpen"),
    },
  });

  const { onStop, onUninstall, onInstallClick, onShowFailure, onCardLaunch } = useProjectActions({
    client,
    onLaunch: launchProject,
    pushToast: toast.push,
    refreshProjects: () => projects.refresh(),
    setShowInstall,
    setFailureProjectId,
    setFailureDetail,
    labels: {
      stoppedToast: (title) => t("homeStoppedToast", title),
      stopFailedTitle: t("homeStopFailedTitle"),
      stopFailedBody: t("homeStopFailedBody"),
      uninstallTitle: (title) => t("homeUninstallTitle", title),
      uninstallBody: (title) => t("homeUninstallBody", title),
      loadingDiagnostics: t("homeLoadingDiagnostics"),
      loadingDiagnosticsSummary: t("homeLoadingDiagnosticsSummary"),
      descriptorNoPackages: t("homeDescriptorNoPackages"),
      noPackageStatus: t("homeNoPackageStatus"),
      diagnosticsUnavailable: t("homeDiagnosticsUnavailable"),
    },
  });

  return (
    <div className="mx-auto flex w-full max-w-[1920px] flex-col gap-8 px-4 py-6 sm:px-6 lg:px-8 lg:py-10 2xl:px-12">
      <Hero
        meta={formatGreetingTime(locale)}
        greeting={t("homeGreeting")}
        summary={
          projects.loading
            ? t("homeReading")
            : counts.all > 0
              ? t("homeShelfSummary", counts.all, counts.running, counts.stopped, counts.failed)
              : t("homeEmptyWorkshop")
        }
        recentActivity={recentActivity}
        activityLabels={{ title: t("homeActivityRecent"), empty: t("homeActivityEmpty") }}
      />

      <div className="grid grid-cols-1 gap-8 lg:grid-cols-[1fr_380px] xl:grid-cols-[1fr_420px] 2xl:grid-cols-[1fr_460px]">
        <div className="flex flex-col gap-5">
          <UtilityStrip
            search={search}
            onSearchChange={setSearch}
            filters={filters}
            activeFilter={activeFilter}
            onFilterChange={setActiveFilter}
            sortLabel={t("homeSortRecent")}
            sortPrefix={t("homeSortPrefix")}
            searchPlaceholder={t("homeSearchPlaceholder")}
          />

          <div className="flex items-center justify-between">
            <Eyebrow>{t("homeInstalledEyebrow", counts.all)}</Eyebrow>
          </div>

          {projects.error ? (
            <EmptyState
              icon={<Warning />}
              title={t("homeErrorTitle")}
              body={t("homeErrorBody")}
              action={{ label: t("retry"), onClick: () => projects.refresh() }}
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
              title={t("homeEmptyTitle")}
              body={t("homeEmptyBody")}
              action={{ label: t("homeInstallLabel"), onClick: onInstallClick }}
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
                  labels={{
                    restart: t("homeActionRestart"),
                    resume: t("homeActionResume"),
                    play: t("homeActionPlay"),
                    more: t("homeMore"),
                    actionsAria: (title) => t("homeActionsAria", title),
                    stop: t("homeActionStop"),
                    open: t("homeActionOpen"),
                    configure: t("homeActionConfigure"),
                    viewLogs: t("homeActionViewLogs"),
                    uninstall: t("homeActionUninstall"),
                  }}
                />
              ))}
              <InstallCard
                onClick={onInstallClick}
                index={filtered.length}
                title={t("homeInstallLabel")}
                hint={t("homeInstallHint")}
              />
            </div>
          )}
        </div>

        <div className="flex flex-col gap-6">
          <ActivityTimeline
            rows={timelineRows.slice(0, 6)}
            loading={lifecycleEvents.loading && timelineRows.length === 0}
            onViewAll={() => navigateTo({ kind: "settings", tab: "installed-packages" })}
            title={t("homeActivityLast24h")}
            emptyLabel={t("homeActivityNo24h")}
            viewAllLabel={t("homeViewFullAuditLog")}
          />
          <WorkshopUtilities
            updates={[]}
            totalDisk={totalDisk}
            diskCapacity={diskCapacity}
            diskSegments={diskSegments}
            labels={{
              workshop: t("homeWorkshop"),
              updates: t("homeUpdates"),
              updatesAvailable: (count) => t("homeUpdatesAvailable", count),
              everythingUpToDate: t("homeEverythingUpToDate"),
              update: t("homeUpdate"),
              updateAll: t("homeUpdateAll"),
              diskUsage: t("homeDiskUsage"),
              diskUsed: (value) => t("homeDiskUsed", value),
              unknown: t("homeUnknown"),
              measuring: t("homeMeasuring"),
              noStorageMeasured: t("homeNoStorageMeasured"),
              manageStorage: t("homeManageStorage"),
              quickActions: t("homeQuickActions"),
            }}
            quickActions={[
              {
                id: "install",
                label: t("homeQuickInstallUrl"),
                shortcut: "⌘N",
                icon: QUICK_ACTION_ICONS.Plus,
                onClick: onInstallClick,
              },
              {
                id: "open-folder",
                label: t("homeQuickDataFolder"),
                shortcut: "⌘O",
                icon: QUICK_ACTION_ICONS.Folder,
                onClick: () => toast.push({ variant: "info", title: t("homeOpenDataFolderToast") }),
              },
              {
                id: "settings",
                label: t("homeQuickSettings"),
                shortcut: "⌘,",
                icon: QUICK_ACTION_ICONS.GearSix,
                onClick: () => navigateTo({ kind: "settings", tab: "api-connections" }),
              },
              {
                id: "switch-profile",
                label: t("homeQuickSwitchProfile"),
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
