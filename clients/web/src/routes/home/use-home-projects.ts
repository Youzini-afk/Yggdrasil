import { useMemo } from "react";
import type { TimelineRow } from "@/components/home/activity-timeline";
import type { DiskSegment } from "@/components/home/workshop-utilities";
import { projectStateTone, type StatusTone } from "@/components/ui/status-pill";
import { formatRelativeAge } from "@/lib/format";
import { useAsync } from "@/lib/kernel-client";
import type { KernelEvent, ProjectRecord, YggProtocolClient } from "@/protocol/client";
import { TIMELINE_SESSION, TONE_TO_DISK_CLASS } from "./home-constants";
import { countsForProjects, filterProjects, filtersWithCounts } from "./home-filtering";
import { iconKindFor } from "./timeline";
import { useRecentlyOpened } from "./use-recently-opened";

export function useHomeProjects({
  client,
  search,
  activeFilter,
  labels,
}: {
  client: YggProtocolClient;
  search: string;
  activeFilter: string;
  labels: {
    all: string;
    running: string;
    stopped: string;
    failed: string;
  };
}) {
  const projects = useAsync(() => client.listProjects(), [client]);
  const lifecycleEvents = useAsync(
    () => client.listEvents(TIMELINE_SESSION).catch<KernelEvent[]>(() => []),
    [client],
  );
  const { list: recentList } = useRecentlyOpened();

  const projectList = projects.data ?? [];
  const counts = useMemo(() => countsForProjects(projectList), [projectList]);
  const filters = useMemo(() => filtersWithCounts(counts, labels), [counts, labels]);
  const filtered = useMemo(
    () => filterProjects(projectList, activeFilter, search),
    [projectList, activeFilter, search],
  );

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

  const continueEntry = useMemo(() => {
    for (const entry of recentList) {
      const project = projectList.find((p) => p.id === entry.projectId);
      if (!project) continue;
      return {
        projectId: project.id,
        title: project.title,
        state: project.state,
        openedAt: entry.openedAt,
      };
    }
    return null;
  }, [recentList, projectList]);

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

  return {
    projects,
    lifecycleEvents,
    projectList,
    counts,
    filters,
    filtered,
    diskSegments,
    totalDisk,
    diskCapacity,
    continueEntry,
    timelineRows,
  };
}
