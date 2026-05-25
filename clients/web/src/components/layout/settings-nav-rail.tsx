import { Folder, GitBranch, Info, Package, Plug } from "@/components/icons";
import { useRoute } from "@/lib/router";
import type { SettingsTab } from "@/lib/router";
import { cn } from "@/lib/cn";
import { useT } from "@/lib/locale";

const NAV_ITEMS: Array<{
  id: SettingsTab;
  Icon: typeof Plug;
}> = [
  { id: "api-connections", Icon: Plug },
  { id: "installed-packages", Icon: Package },
  { id: "profiles", Icon: GitBranch },
  { id: "storage", Icon: Folder },
  { id: "about", Icon: Info },
];

const navLabelKey: Record<SettingsTab, "settingsApiConnections" | "settingsInstalledPackages" | "settingsProfiles" | "settingsStorage" | "settingsAbout"> = {
  "api-connections": "settingsApiConnections",
  "installed-packages": "settingsInstalledPackages",
  profiles: "settingsProfiles",
  storage: "settingsStorage",
  about: "settingsAbout",
};

export function SettingsNavRail({ active }: { active: SettingsTab }) {
  const [, navigate] = useRoute();
  const t = useT();

  return (
    <aside className="shrink-0 border-whisper-border lg:w-[240px] lg:border-r lg:pr-6">
      <p className="eyebrow mb-3 px-1 lg:mb-6 lg:px-3">{t("settingsTitle")}</p>
      <nav className="flex gap-1 overflow-x-auto pb-1 lg:flex-col lg:overflow-visible lg:pb-0">
        {NAV_ITEMS.map(({ id, Icon }) => {
          const isActive = active === id;
          const label = t(navLabelKey[id]);
          return (
            <button
              key={id}
              type="button"
              onClick={() => navigate({ kind: "settings", tab: id })}
              aria-current={isActive ? "page" : undefined}
              className={cn(
                // Structural left border (transparent when inactive) keeps the
                // content offset stable so hovering doesn't shift label
                // position by 2px.
                "flex shrink-0 items-center gap-2 rounded-[8px] py-2 pr-3 pl-[10px] text-left text-[13px] font-medium transition border-l-2",
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
      <div className="mt-4 px-1 text-[11px] text-muted-tone lg:mt-8 lg:px-3">
        {t("settingsHelper")}
      </div>
    </aside>
  );
}
