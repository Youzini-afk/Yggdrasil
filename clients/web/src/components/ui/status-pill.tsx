import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/cn";

const pillVariants = cva(
  "inline-flex items-center gap-1.5 font-mono text-[10px] font-medium uppercase tracking-[0.06em] rounded-full px-2.5 py-1",
  {
    variants: {
      tone: {
        running: "bg-aged-brass-surface text-aged-brass-deep",
        stopped: "bg-whisper-border-strong/40 text-steel-secondary",
        starting: "bg-whisper-border-strong/40 text-muted-tone",
        failed: "bg-deep-rust-surface text-deep-rust",
        update: "bg-aged-brass-surface text-aged-brass-deep",
        neutral: "bg-whisper-border-strong/40 text-steel-secondary",
        accent: "bg-aged-brass-surface text-aged-brass-deep",
      },
    },
    defaultVariants: { tone: "neutral" },
  },
);

const dotVariants = cva("inline-block size-1.5 rounded-full", {
  variants: {
    tone: {
      running: "bg-aged-brass animate-[pulse-dot_2.4s_ease-in-out_infinite]",
      stopped: "bg-steel-secondary",
      starting: "bg-muted-tone",
      failed: "bg-deep-rust",
      update: "bg-aged-brass",
      neutral: "bg-steel-secondary",
      accent: "bg-aged-brass",
    },
  },
  defaultVariants: { tone: "neutral" },
});

export interface StatusPillProps extends VariantProps<typeof pillVariants> {
  label: string;
  className?: string;
  showDot?: boolean;
}

export function StatusPill({ tone, label, className, showDot = true }: StatusPillProps) {
  return (
    <span className={cn(pillVariants({ tone }), className)}>
      {showDot ? <span className={dotVariants({ tone })} aria-hidden /> : null}
      {label}
    </span>
  );
}

export type StatusTone = NonNullable<VariantProps<typeof pillVariants>["tone"]>;

const STATE_TO_TONE: Record<string, StatusTone> = {
  installed: "stopped",
  stopped: "stopped",
  starting: "starting",
  stopping: "starting",
  running: "running",
  failed: "failed",
  archived: "stopped",
};

export function projectStateTone(state: string): StatusTone {
  return STATE_TO_TONE[state] ?? "neutral";
}
