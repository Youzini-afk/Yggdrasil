// Filled in Phase 3.
import { EmptyState } from "@/components/ui/empty-state";
import { Plus } from "@/components/icons";

export function HomePage() {
  return (
    <div className="mx-auto flex w-full max-w-[1400px] flex-col gap-8 px-8 py-12">
      <EmptyState icon={<Plus />} title="Home page placeholder" body="Phase 3 will implement this view." />
    </div>
  );
}
