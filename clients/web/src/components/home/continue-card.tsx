import { useMemo } from "react";
import { Play, ArrowsClockwise, Warning, Signpost } from "@/components/icons";
import { Button } from "@/components/ui/button";
import { EyebrowSm } from "@/components/ui/typography";
import { cn } from "@/lib/cn";
import { StatusPill, projectStateTone, type StatusTone } from "@/components/ui/status-pill";

export interface ContinueCardEntry {
  projectId: string;
  title: string;
  state: string;
  openedAt: number;
}

export interface ContinueCardLabels {
  title: string;
  running: string;
  stopped: string;
  failed: string;
  continueAction: string;
  openAction: string;
  diagnoseAction: string;
  ageNow: string;
  emptyTitle: string;
  emptyBody: string;
  emptyInstall: string;
  emptyTryYdltavern: string;
  pickInstalled: string;
}

export interface ContinueCardProps {
  entry: ContinueCardEntry | null;
  labels: ContinueCardLabels;
  onContinue: (projectId: string) => void;
  onInstall: () => void;
  onBrowseProjects?: () => void;
  hasInstalledProjects: boolean;
}

const dotClassOverride: Record<StatusTone, string> = {
  running: "bg-aged-brass animate-[pulse-dot_2.4s_ease-in-out_infinite]",
  stopped: "bg-steel-secondary",
  starting: "bg-muted-tone",
  failed: "bg-deep-rust",
  update: "bg-muted-tone",
  neutral: "bg-steel-secondary",
  accent: "bg-aged-brass",
};

function ageLabel(openedAt: number, ageNow: string): string {
  const diff = Date.now() - openedAt;
  if (diff < 60_000) return ageNow;
  if (diff < 3_600_000) {
    const m = Math.floor(diff / 60_000);
    return `${m} min${m > 1 ? "s" : ""} ago`;
  }
  if (diff < 86_400_000) {
    const h = Math.floor(diff / 3_600_000);
    return `${h} hour${h > 1 ? "s" : ""} ago`;
  }
  const d = Math.floor(diff / 86_400_000);
  return `${d} day${d > 1 ? "s" : ""} ago`;
}

export function ContinueCard({
  entry,
  labels,
  onContinue,
  onInstall,
  onBrowseProjects,
  hasInstalledProjects,
}: ContinueCardProps) {
  const tone = useMemo(() => projectStateTone(entry?.state ?? "stopped"), [entry?.state]);

  const actionLabel = useMemo(() => {
    if (!entry) return "";
    if (entry.state === "running") return labels.continueAction;
    if (entry.state === "failed") return labels.diagnoseAction;
    return labels.openAction;
  }, [entry, labels]);

  const PrimaryIcon =
    entry?.state === "failed" ? Warning : entry?.state === "running" ? Play : Signpost;

  if (!entry) {
    return (
      <div className="flex w-full max-w-[420px] flex-col gap-3 rounded-[16px] border border-whisper-border bg-pure-surface p-5">
        <EyebrowSm>{labels.title}</EyebrowSm>
        <div className="flex flex-col items-center gap-3 py-6 text-center">
          <Signpost size={20} className="text-muted-tone" />
          <h4 className="font-display text-[16px] font-bold text-charcoal-ink">{labels.emptyTitle}</h4>
          <p className="max-w-[32ch] text-[13px] leading-relaxed text-steel-secondary">
            {labels.emptyBody}
          </p>
          <div className="flex flex-wrap items-center justify-center gap-2 pt-1">
            <Button tone="primary" size="sm" onClick={onInstall}>
              {labels.emptyInstall}
            </Button>
            {!hasInstalledProjects ? (
              <button
                type="button"
                onClick={onInstall}
                className="text-[12px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
              >
                {labels.emptyTryYdltavern}
              </button>
            ) : (
              <button
                type="button"
                onClick={onBrowseProjects}
                className="text-[12px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
              >
                {labels.pickInstalled}
              </button>
            )}
          </div>
        </div>
      </div>
    );
  }

  const stateLabel =
    entry.state === "running"
      ? labels.running
      : entry.state === "failed"
        ? labels.failed
        : labels.stopped;

  return (
    <div className="flex w-full max-w-[420px] flex-col gap-3 rounded-[16px] border border-whisper-border bg-pure-surface p-5">
      <EyebrowSm>{labels.title}</EyebrowSm>
      <div className="flex flex-col gap-2">
        <div className="flex items-center gap-2">
          <span
            className={cn("size-1.5 shrink-0 rounded-full", dotClassOverride[tone])}
            aria-hidden
          />
          <span className="font-display text-[18px] font-bold leading-tight text-charcoal-ink">
            {entry.title}
          </span>
        </div>

        <div className="flex items-center gap-2">
          <StatusPill tone={tone} label={stateLabel} />
          <span className="font-mono text-[11px] text-muted-tone">{ageLabel(entry.openedAt, labels.ageNow)}</span>
        </div>

        <Button
          tone="primary"
          size="sm"
          onClick={() => onContinue(entry.projectId)}
          className="mt-1 w-full justify-start"
        >
          <PrimaryIcon size={14} weight="fill" />
          {actionLabel}
        </Button>
      </div>
    </div>
  );
}
