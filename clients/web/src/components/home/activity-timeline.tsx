import {
  ArrowsClockwise,
  CheckCircle,
  Cloud,
  Globe,
  Key,
  Package,
  Stack,
  Warning,
} from "@/components/icons";
import { Card } from "@/components/ui/card";
import { Eyebrow } from "@/components/ui/typography";
import { cn } from "@/lib/cn";
import type { StatusTone } from "@/components/ui/status-pill";

const dotClass: Record<StatusTone, string> = {
  running: "bg-aged-brass",
  stopped: "bg-steel-secondary",
  starting: "bg-muted-tone",
  failed: "bg-deep-rust",
  update: "bg-aged-brass",
  neutral: "bg-steel-secondary",
  accent: "bg-aged-brass",
};

const eventIconMap: Record<string, typeof Globe> = {
  default: Stack,
  outbound: Globe,
  secret: Key,
  package: Package,
  crash: Warning,
  checkpoint: CheckCircle,
  retry: ArrowsClockwise,
  install: Cloud,
};

export interface TimelineRow {
  id: string;
  projectName: string;
  toneDot: StatusTone;
  age: string;
  message: string;
  iconKind?: keyof typeof eventIconMap;
  action?: { label: string; onClick: () => void };
}

export function ActivityTimeline({
  rows,
  onViewAll,
}: {
  rows: TimelineRow[];
  onViewAll?: () => void;
}) {
  return (
    <section className="flex flex-col gap-3">
      <Eyebrow>Activity — last 24h</Eyebrow>
      <Card>
        {rows.length === 0 ? (
          <div className="px-6 py-12 text-center text-[13px] text-muted-tone">
            No activity in the last 24 hours.
          </div>
        ) : (
          <ul className="divide-y divide-whisper-border">
            {rows.map((row) => {
              const Icon = eventIconMap[row.iconKind ?? "default"] ?? eventIconMap.default;
              return (
                <li key={row.id} className="flex gap-3 px-5 py-3">
                  <div className="flex flex-col items-center pt-0.5">
                    <span className={cn("size-1.5 rounded-full", dotClass[row.toneDot])} aria-hidden />
                  </div>
                  <div className="min-w-0 flex-1">
                    <div className="flex items-baseline justify-between gap-2">
                      <p className="text-[12px] font-medium text-charcoal-ink">{row.projectName}</p>
                      <span className="font-mono text-[10px] text-muted-tone">{row.age}</span>
                    </div>
                    <div className="mt-0.5 flex items-center gap-2 text-[11px] text-charcoal-ink">
                      <Icon size={12} className="shrink-0 text-steel-secondary" />
                      <span className="min-w-0 flex-1 truncate">{row.message}</span>
                      {row.action ? (
                        <button
                          type="button"
                          onClick={row.action.onClick}
                          className="shrink-0 font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
                        >
                          {row.action.label}
                        </button>
                      ) : null}
                    </div>
                  </div>
                </li>
              );
            })}
          </ul>
        )}
        {rows.length > 0 ? (
          <div className="border-t border-whisper-border px-5 py-3 text-right">
            <button
              type="button"
              onClick={onViewAll}
              className="text-[12px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
            >
              View full audit log →
            </button>
          </div>
        ) : null}
      </Card>
    </section>
  );
}
