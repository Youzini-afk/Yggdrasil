import {
  Folder,
  GearSix,
  Info,
  Package,
  Play,
  Plus,
  Stack,
  Terminal,
  User,
  Warning,
  Wrench,
} from "@/components/icons";

export type ShellIconComponent = typeof Info;

const SHELL_ICON_REGISTRY: Record<string, ShellIconComponent> = {
  add: Plus,
  folder: Folder,
  gear: GearSix,
  info: Info,
  package: Package,
  play: Play,
  plus: Plus,
  settings: GearSix,
  stack: Stack,
  terminal: Terminal,
  tool: Wrench,
  user: User,
  warning: Warning,
  wrench: Wrench,
};

export function resolveShellIcon(iconHint: string | undefined): ShellIconComponent {
  if (!iconHint) return Info;
  return SHELL_ICON_REGISTRY[iconHint] ?? Info;
}
