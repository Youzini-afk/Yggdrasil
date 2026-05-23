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
                // Structural left border (transparent when inactive) keeps the
                // content offset stable so hovering doesn't shift label
                // position by 2px.
                "flex items-center gap-2 rounded-[8px] py-2 pr-3 pl-[10px] text-left text-[13px] font-medium transition border-l-2",
                isActive
                  ? "border-l-aged-brass bg-aged-brass-surface text-charcoal-ink [&_svg]:text-aged-brass"
                  : "border-l-transparent text-charcoal-ink hover:bg-whisper-border-strong/30 [&_svg]:text-steel-secondary",
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
