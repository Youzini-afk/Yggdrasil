import { cn } from "@/lib/cn";
import { EyebrowSm } from "@/components/ui/typography";
import type { StatusTone } from "@/components/ui/status-pill";

export interface ActivityRow {
  id: string;
  projectName: string;
  toneDot: StatusTone;
  age: string;
  action: { label: string; onClick: () => void };
}

const dotClass: Record<StatusTone, string> = {
  running: "bg-aged-brass animate-[pulse-dot_2.4s_ease-in-out_infinite]",
  stopped: "bg-steel-secondary",
  starting: "bg-muted-tone",
  failed: "bg-deep-rust",
  update: "bg-aged-brass",
  neutral: "bg-steel-secondary",
  accent: "bg-aged-brass",
};

export function ActivityMicroCard({ rows }: { rows: ActivityRow[] }) {
  if (rows.length === 0) {
    return (
      <div className="flex max-w-[360px] flex-col gap-2 rounded-[16px] border border-whisper-border bg-pure-surface p-4">
        <EyebrowSm>Recent activity</EyebrowSm>
        <p className="text-[12px] text-muted-tone">Nothing yet — open a project to start.</p>
      </div>
    );
  }
  return (
    <div className="flex w-full max-w-[360px] flex-col gap-3 rounded-[16px] border border-whisper-border bg-pure-surface p-4">
      <EyebrowSm>Recent activity</EyebrowSm>
      <ul className="divide-y divide-whisper-border">
        {rows.map((row) => (
          <li key={row.id} className="flex items-center gap-2 py-2 text-[13px] first:pt-0 last:pb-0">
            <span className={cn("size-1.5 shrink-0 rounded-full", dotClass[row.toneDot])} aria-hidden />
            <span className="font-medium text-charcoal-ink">{row.projectName}</span>
            <span className="ml-auto font-mono text-[11px] text-muted-tone">{row.age}</span>
            <button
              type="button"
              onClick={row.action.onClick}
              className="text-[12px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
            >
              {row.action.label}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
