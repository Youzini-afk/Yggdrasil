import { Folder, GearSix, GitBranch, Info, Package, Plug } from "@/components/icons";
import { useRoute } from "@/lib/router";
import type { SettingsTab } from "@/lib/router";
import { cn } from "@/lib/cn";

const NAV_ITEMS: Array<{
  id: SettingsTab;
  label: string;
  Icon: typeof Plug;
}> = [
  { id: "api-connections", label: "API Connections", Icon: Plug },
  { id: "installed-packages", label: "Installed Packages", Icon: Package },
  { id: "profiles", label: "Profiles", Icon: GitBranch },
  { id: "storage", label: "Storage", Icon: Folder },
  { id: "about", label: "About", Icon: Info },
];

export function SettingsNavRail({ active }: { active: SettingsTab }) {
  const [, navigate] = useRoute();

  return (
    <aside className="w-[240px] shrink-0 border-r border-whisper-border pr-6">
      <p className="eyebrow mb-6 px-3">Settings</p>
      <nav className="flex flex-col gap-1">
        {NAV_ITEMS.map(({ id, label, Icon }) => {
          const isActive = active === id;
          return (
            <button
              key={id}
              type="button"
              onClick={() => navigate({ kind: "settings", tab: id })}
              className={cn(
                "flex items-center gap-2 rounded-[8px] px-3 py-2 text-left text-[13px] font-medium transition",
                isActive
                  ? "bg-aged-brass-surface text-charcoal-ink border-l-2 border-l-aged-brass pl-[10px] [&_svg]:text-aged-brass"
                  : "text-charcoal-ink hover:bg-whisper-border-strong/30 [&_svg]:text-steel-secondary",
              )}
            >
              <Icon size={16} />
              <span>{label}</span>
            </button>
          );
        })}
      </nav>
      <div className="mt-8 px-3 text-[11px] text-muted-tone">
        Settings live on this machine. No SaaS sync.
      </div>
      <div className="mt-2 px-3 text-[11px] text-muted-tone flex items-center gap-1.5">
        <GearSix size={11} />
        <span className="font-mono">~/.yggdrasil</span>
      </div>
    </aside>
  );
}
