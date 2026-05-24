import { SettingsNavRail } from "@/components/layout/settings-nav-rail";
import type { SettingsTab } from "@/lib/router";
import { ApiConnectionsPanel } from "./api-connections";
import { InstalledPackagesPanel } from "./installed-packages";
import { ProfilesPanel } from "./profiles";
import { StoragePanel } from "./storage";
import { AboutPanel } from "./about";

export function SettingsPage({ tab }: { tab: SettingsTab }) {
  return (
    <div className="mx-auto flex w-full max-w-[1400px] flex-1 flex-col gap-6 px-4 py-6 sm:px-6 lg:flex-row lg:gap-8 lg:px-8 lg:py-10">
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
