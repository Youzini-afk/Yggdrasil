// Filled in Phase 4.
import { SettingsNavRail } from "@/components/layout/settings-nav-rail";
import type { SettingsTab } from "@/lib/router";
import { EmptyState } from "@/components/ui/empty-state";
import { GearSix } from "@/components/icons";

export function SettingsPage({ tab }: { tab: SettingsTab }) {
  return (
    <div className="mx-auto flex w-full max-w-[1400px] flex-1 gap-8 px-8 py-10">
      <SettingsNavRail active={tab} />
      <section className="flex-1">
        <EmptyState icon={<GearSix />} title={`Settings — ${tab}`} body="Phase 4 will implement this panel." />
      </section>
    </div>
  );
}
