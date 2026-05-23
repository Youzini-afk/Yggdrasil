import { SettingsNavRail } from "@/components/layout/settings-nav-rail";
import type { SettingsTab } from "@/lib/router";
import { ApiConnectionsPanel } from "./api-connections";
import { InstalledPackagesPanel } from "./installed-packages";
import { ProfilesPanel } from "./profiles";
import { StoragePanel } from "./storage";
import { AboutPanel } from "./about";

export function SettingsPage({ tab }: { tab: SettingsTab }) {
  return (
    <div className="mx-auto flex w-full max-w-[1400px] flex-1 gap-8 px-8 py-10">
      <SettingsNavRail active={tab} />
      <section className="flex-1 min-w-0">{renderTab(tab)}</section>
    </div>
  );
}

function renderTab(tab: SettingsTab) {
  switch (tab) {
    case "api-connections":
      return <ApiConnectionsPanel />;
    case "installed-packages":
      return <InstalledPackagesPanel />;
    case "profiles":
      return <ProfilesPanel />;
    case "storage":
      return <StoragePanel />;
    case "about":
      return <AboutPanel />;
  }
}
