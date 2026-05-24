import { motion } from "motion/react";
import { CheckCircle, XCircle } from "@/components/icons";
import { Button } from "@/components/ui/button";
import { ModalFooter, ModalHeader } from "@/components/ui/modal";
import { EyebrowSm } from "@/components/ui/typography";
import { cn } from "@/lib/cn";
import type { InstallExecuteResult, InstallPlan } from "@/protocol/client";
import type { InstallPhase } from "./install-types";

export function ProgressStep({
  url,
  plan,
  phases,
  result,
  error,
  onCancel,
}: {
  url: string;
  plan: InstallPlan | null;
  phases: InstallPhase[];
  result: InstallExecuteResult | null;
  error: string | null;
  onCancel: () => void;
}) {
  const phaseOrder: Array<{ id: InstallPhase; label: string; detail: string }> = [
    { id: "resolving", label: "Resolved install plan", detail: plan ? `${plan.packages.length} package${plan.packages.length === 1 ? "" : "s"}` : "complete" },
    { id: "detecting", label: "Detected project kind", detail: "complete" },
    { id: "reviewed", label: "Permissions approved", detail: "complete" },
    { id: "executing", label: "Executing install plan", detail: phases.includes("completed") ? "complete" : "in progress" },
    { id: "completed", label: "Install completed", detail: result ? `${result.installed.length} installed` : "waiting" },
  ];
  const failed = phases.includes("failed");
  const progress = phases.includes("completed") ? 1 : Math.min(phases.filter((p) => p !== "failed").length / phaseOrder.length, 0.92);

  return (
    <>
      <ModalHeader
        eyebrow="Install — Step 3 of 3"
        title={failed ? "Install failed" : phases.includes("completed") ? "Install complete" : "Installing project"}
        description={
          <span className="font-mono text-[11px] text-muted-tone">{url}{plan?.root_id ? ` · ${plan.root_id}` : ""}</span>
        }
      />

      <div className="space-y-2">
        <div className="h-1.5 overflow-hidden rounded-full bg-whisper-border-strong/40">
          <motion.div
            className={cn("h-full", failed ? "bg-deep-rust" : "bg-aged-brass")}
            initial={{ width: 0 }}
            animate={{ width: `${progress * 100}%` }}
            transition={{ duration: 0.4 }}
          />
        </div>
        <div className="flex items-center justify-between text-[11px]">
          <span className="font-medium text-charcoal-ink">
            {failed ? "Failed" : phases.includes("completed") ? "Completed" : "Executing"}
          </span>
          <span className="font-mono text-steel-secondary">{Math.round(progress * 100)}%</span>
        </div>
      </div>

      <ul className="mt-6 divide-y divide-whisper-border rounded-[12px] border border-whisper-border bg-pure-surface">
        {phaseOrder.map((phase) => {
          const isComplete = phases.includes(phase.id) && (phase.id !== "executing" || phases.includes("completed"));
          const isCurrent = phases.at(-1) === phase.id && !failed && !phases.includes("completed");
          return (
            <li key={phase.id} className="flex items-center gap-3 px-4 py-2.5">
              {isComplete ? (
                <CheckCircle size={14} className="text-aged-brass shrink-0" weight="fill" />
              ) : isCurrent ? (
                <span className="relative inline-flex size-3.5 items-center justify-center shrink-0">
                  <span className="absolute inset-0 rounded-full border border-aged-brass" />
                  <span className="size-1.5 rounded-full bg-aged-brass animate-[pulse-dot_1.4s_ease-in-out_infinite]" />
                </span>
              ) : (
                <span className="size-3.5 rounded-full border border-whisper-border-strong/70 shrink-0" />
              )}
              <span
                className={cn(
                  "flex-1 text-[13px]",
                  isComplete || isCurrent ? "font-medium text-charcoal-ink" : "text-muted-tone",
                )}
              >
                {phase.label}
              </span>
              <span
                className={cn(
                  "font-mono text-[11px]",
                  isCurrent ? "text-aged-brass-deep" : "text-muted-tone",
                )}
              >
                {phase.detail}
              </span>
            </li>
          );
        })}
        {failed ? (
          <li className="flex items-center gap-3 px-4 py-2.5">
            <XCircle size={14} className="shrink-0 text-deep-rust" weight="fill" />
            <span className="flex-1 text-[13px] font-medium text-deep-rust">Failed</span>
            <span className="font-mono text-[11px] text-muted-tone">see activity</span>
          </li>
        ) : null}
      </ul>

      <section className="mt-6">
        <EyebrowSm>Activity</EyebrowSm>
        <div className="mt-2 space-y-0.5 rounded-[10px] bg-warm-bone p-3 font-mono text-[11px] text-steel-secondary">
          {phases.includes("resolving") ? <p>resolve_plan completed for {plan?.root_id ?? url}</p> : null}
          {phases.includes("detecting") ? <p>detect_kind completed</p> : null}
          {phases.includes("reviewed") ? <p>requested permissions approved</p> : null}
          {phases.includes("executing") ? <p className={cn("border-l-2 pl-2", failed ? "border-deep-rust text-deep-rust" : "border-aged-brass text-charcoal-ink")}>execute_plan {failed ? "failed" : phases.includes("completed") ? "completed" : "running"}</p> : null}
          {result?.project?.project_id ? <p>registered project {result.project.project_id}</p> : null}
          {result ? <p>profile updated · lockfile refreshed</p> : null}
          {error ? <p className="whitespace-pre-wrap text-deep-rust">{error}</p> : null}
        </div>
      </section>

      <ModalFooter className="justify-end">
        <Button tone={failed ? "secondary" : "destructive"} onClick={onCancel} disabled={!failed && !phases.includes("completed")}>
          {failed || phases.includes("completed") ? "Close" : "Installing…"}
        </Button>
      </ModalFooter>
    </>
  );
}
