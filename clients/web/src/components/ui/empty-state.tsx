import type { ReactNode } from "react";
import { cn } from "@/lib/cn";
import { Button } from "@/components/ui/button";

export interface EmptyStateProps {
  icon?: ReactNode;
  title: string;
  body?: string;
  action?: { label: string; onClick: () => void };
  className?: string;
}

export function EmptyState({ icon, title, body, action, className }: EmptyStateProps) {
  return (
    <div
      className={cn(
        "mx-auto flex max-w-[40ch] flex-col items-center gap-4 rounded-[20px] border border-dashed border-whisper-border-strong/70 bg-transparent px-8 py-14 text-center",
        className,
      )}
    >
      {icon ? <span className="text-steel-secondary [&>svg]:size-12">{icon}</span> : null}
      <h3 className="font-display text-[18px] font-bold leading-tight text-charcoal-ink">{title}</h3>
      {body ? <p className="text-[14px] leading-relaxed text-steel-secondary">{body}</p> : null}
      {action ? (
        <Button tone="primary" onClick={action.onClick}>
          {action.label}
        </Button>
      ) : null}
    </div>
  );
}
