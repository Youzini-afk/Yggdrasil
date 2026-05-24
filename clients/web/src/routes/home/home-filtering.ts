import type { FilterChip } from "@/components/home/utility-strip";
import type { ProjectRecord } from "@/protocol/client";
import { FILTER_OPTIONS } from "./home-constants";

export interface ProjectCounts {
  all: number;
  running: number;
  stopped: number;
  failed: number;
}

export function countsForProjects(projectList: ProjectRecord[]): ProjectCounts {
  const running = projectList.filter((p) => p.state === "running").length;
  const stopped = projectList.filter((p) => p.state === "stopped" || p.state === "installed").length;
  const failed = projectList.filter((p) => p.state === "failed").length;
  return { all: projectList.length, running, stopped, failed };
}

export function filtersWithCounts(counts: ProjectCounts): FilterChip[] {
  return FILTER_OPTIONS.map((option) => ({
    ...option,
    count: counts[option.id as keyof ProjectCounts] ?? 0,
  }));
}

export function filterProjects(projectList: ProjectRecord[], activeFilter: string, search: string): ProjectRecord[] {
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
}
