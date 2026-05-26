import { isValidProjectId, projectPath } from "@/lib/router";

export interface ProjectTabWindow {
  open(url: string, target: string, features?: string): Window | null;
  location: Pick<Location, "assign">;
}

const PROJECT_TAB_FEATURES = "noopener,noreferrer";

export function projectTabTargetName(projectId: string): string {
  return `ygg-project-${fnv1a(projectId).toString(36).padStart(7, "0").slice(0, 16)}`;
}

export function openProjectInTab(projectId: string, hostWindow: ProjectTabWindow = window): boolean {
  if (!isValidProjectId(projectId)) return false;
  const url = projectPath(projectId);
  const target = projectTabTargetName(projectId);
  const opened = hostWindow.open(url, target, PROJECT_TAB_FEATURES);
  if (opened) {
    try {
      opened.opener = null;
    } catch {
      // Some browsers expose opener as read-only when noopener is already in force.
    }
    return true;
  }
  return false;
}

function fnv1a(value: string): number {
  let hash = 0x811c9dc5;
  for (let i = 0; i < value.length; i += 1) {
    hash ^= value.charCodeAt(i);
    hash = Math.imul(hash, 0x01000193);
  }
  return hash >>> 0;
}
