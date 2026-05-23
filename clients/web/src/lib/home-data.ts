/**
 * Mock data for the Home view used when the backend is offline or empty.
 *
 * Real data comes from kernel.v1.project.list / kernel.v1.event.list. This
 * module exists so the layout always demonstrates the editorial workshop
 * aesthetic during dev. The home route prefers real data and falls back to
 * the mocks below if the protocol throws.
 */

import type { ProjectRecord } from "@/protocol/client";

export const MOCK_PROJECTS: Array<ProjectRecord & { description?: string; size_mb?: number; metrics?: string }> = [
  {
    id: "ydltavern",
    title: "YdlTavern",
    description: "SillyTavern-compatible roleplay surface. Catches existing community resources.",
    type: "yggdrasil_native",
    state: "running",
    icon: "BookOpenText",
    entry_surface_id: "ydltavern/play",
    size_mb: 84.3,
    metrics: "12 sessions · 4 ext · last 2h",
  } as never,
  {
    id: "coding-workshop",
    title: "Coding Workshop",
    description: "Agent-driven coding companion with branching scratchpads.",
    type: "yggdrasil_native",
    state: "stopped",
    size_mb: 142.1,
    metrics: "3 sessions · 1 ext · last yesterday",
  } as never,
  {
    id: "image-studio",
    title: "Image Studio",
    description: "Generative image bench wired through the platform's outbound layer.",
    type: "yggdrasil_native",
    state: "failed",
    size_mb: 47.2,
    metrics: "Crashed 8m ago · exit 137 (oom)",
  } as never,
];

export interface MockTimelineRow {
  id: string;
  projectName: string;
  age: string;
  message: string;
  iconKind: "outbound" | "secret" | "package" | "crash" | "checkpoint" | "retry" | "default";
  tone: "running" | "stopped" | "failed";
  action?: { label: string };
}

export const MOCK_TIMELINE: MockTimelineRow[] = [
  {
    id: "tl-1",
    projectName: "YdlTavern",
    age: "4m ago",
    message: "User session resumed · 142 turns",
    iconKind: "default",
    tone: "running",
    action: { label: "View" },
  },
  {
    id: "tl-2",
    projectName: "Coding Workshop",
    age: "1h ago",
    message: "Project stopped by user",
    iconKind: "default",
    tone: "stopped",
  },
  {
    id: "tl-3",
    projectName: "Image Studio",
    age: "1h ago",
    message: "Subprocess crashed · exit 137 (oom)",
    iconKind: "crash",
    tone: "failed",
    action: { label: "Retry" },
  },
  {
    id: "tl-4",
    projectName: "YdlTavern",
    age: "1h ago",
    message: "Outbound to api.openai.com · 2.3 MB",
    iconKind: "outbound",
    tone: "running",
    action: { label: "Audit" },
  },
  {
    id: "tl-5",
    projectName: "YdlTavern",
    age: "3h ago",
    message: "Secret OPENAI_API_KEY accessed · platform scope",
    iconKind: "secret",
    tone: "running",
    action: { label: "Audit" },
  },
  {
    id: "tl-6",
    projectName: "Image Studio",
    age: "5h ago",
    message: "Package install · official/image-tools-lab v0.0.3",
    iconKind: "install",
    tone: "failed",
    action: { label: "View" },
  } as never,
  {
    id: "tl-7",
    projectName: "Coding Workshop",
    age: "8h ago",
    message: "Session checkpoint saved · 4 branches",
    iconKind: "checkpoint",
    tone: "stopped",
  },
];
