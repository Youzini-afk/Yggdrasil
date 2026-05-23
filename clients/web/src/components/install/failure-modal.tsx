import { ArrowsClockwise, Aperture, Copy, GithubLogo } from "@/components/icons";
import { Button } from "@/components/ui/button";
import { Field, Checkbox } from "@/components/ui/input";
import { Modal, ModalHeader, ModalFooter } from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { useState } from "react";

const STDERR_LOG = [
  "[14:32:11] loading model: stable-diffusion-xl-1.0",
  "[14:32:14] checkpoint loaded · 6.8 GB allocated",
  "[14:32:15] applying LoRA: forest-photography-v2",
  "[14:32:16] LoRA bake complete · 7.12 GB allocated",
  "[14:32:17] starting inference batch · 4 prompts queued",
  "[14:32:18] memory pressure: 7.42 GB / 8.00 GB available",
  "[14:32:18] kernel: image-studio invoked oom-killer",
  "[14:32:18] subprocess exit · code 137",
];

export interface FailureModalProps {
  open: boolean;
  onClose: () => void;
  onRestart?: () => void;
  onUninstall?: () => void;
  /** Optional override of the demo data. */
  detail?: {
    projectName?: string;
    icon?: React.ReactNode;
    title?: string;
    summary?: string;
    log?: string[];
  };
}

export function FailureModal({ open, onClose, onRestart, onUninstall, detail }: FailureModalProps) {
  const toast = useToast();
  const [sendReport, setSendReport] = useState(false);

  const projectName = detail?.projectName ?? "Image Studio";
  const log = detail?.log ?? STDERR_LOG;

  const copy = () => {
    navigator.clipboard?.writeText(log.join("\n"));
    toast.push({ variant: "success", title: "Log copied", duration: 2400 });
  };

  return (
    <Modal open={open} onOpenChange={onClose} accent="rust" size="lg" contentLabel={`${projectName} crash details`}>
      <ModalHeader
        eyebrow={`Failure — ${projectName.toUpperCase()}`}
        title={detail?.title ?? "Subprocess crashed with exit 137"}
        description={
          detail?.summary ??
          "Out of memory while loading model. Project state preserved — last checkpoint 8 minutes ago."
        }
      />

      {/* Identity row */}
      <div className="flex items-center gap-3 rounded-[12px] border border-whisper-border px-4 py-3">
        {detail?.icon ?? <Aperture size={20} className="text-deep-rust" />}
        <span className="font-display text-[14px] font-bold text-charcoal-ink">{projectName}</span>
        <span className="font-mono text-[12px] text-muted-tone">v0.0.3</span>
        <GithubLogo size={14} className="ml-2 text-steel-secondary" />
        <span className="ml-auto font-mono text-[11px] text-muted-tone">
          Crashed 14:32:18 · 8m ago
        </span>
      </div>

      {/* Diagnosis & impact */}
      <div className="mt-6 grid grid-cols-2 gap-6">
        <DiagnosisColumn
          eyebrow="Diagnosis"
          rows={[
            ["Exit code", "137 (SIGKILL)"],
            ["Cause", "OOM killer"],
            ["Memory peak", "7.42 GB"],
            ["Uptime", "47 min"],
          ]}
        />
        <DiagnosisColumn
          eyebrow="Impact"
          rows={[
            ["Sessions lost", "0 (auto-saved)"],
            ["Last checkpoint", "8m ago"],
            ["Active proposals", "2 preserved"],
            ["Asset writes", "none in flight"],
          ]}
        />
      </div>

      <div className="my-6 h-px bg-whisper-border" />

      {/* Log */}
      <section>
        <div className="flex items-center justify-between">
          <p className="font-mono text-[10px] uppercase tracking-[0.12em] text-muted-tone">
            Stderr · last {log.length} lines
          </p>
          <div className="flex items-center gap-3">
            <button
              type="button"
              onClick={copy}
              className="inline-flex items-center gap-1 text-[11px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
            >
              <Copy size={12} />
              Copy log
            </button>
            <button
              type="button"
              className="text-[11px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
            >
              Open full log →
            </button>
          </div>
        </div>
        <div className="mt-2 space-y-0.5 rounded-[10px] bg-warm-bone p-3 font-mono text-[11px] leading-relaxed text-charcoal-ink">
          {log.map((line, idx) => (
            <p key={idx} className={idx === log.length - 1 ? "text-deep-rust" : ""}>
              {line}
            </p>
          ))}
        </div>
      </section>

      <ModalFooter className="justify-between">
        <Checkbox
          checked={sendReport}
          onCheckedChange={setSendReport}
          label="Send anonymous crash report"
        />
        <div className="flex items-center gap-3">
          <Button tone="tertiary" onClick={onClose}>
            Close
          </Button>
          <Button
            tone="destructive"
            onClick={() => {
              onUninstall?.();
              onClose();
            }}
          >
            Stop and uninstall
          </Button>
          <Button
            tone="primary"
            onClick={() => {
              onRestart?.();
              onClose();
            }}
          >
            <ArrowsClockwise size={14} />
            Restart project
          </Button>
        </div>
      </ModalFooter>
    </Modal>
  );
}

function DiagnosisColumn({
  eyebrow,
  rows,
}: {
  eyebrow: string;
  rows: Array<[string, string]>;
}) {
  return (
    <div>
      <p className="font-mono text-[10px] uppercase tracking-[0.12em] text-muted-tone">{eyebrow}</p>
      <dl className="mt-3 space-y-2 text-[12px]">
        {rows.map(([label, value]) => (
          <div key={label} className="flex justify-between gap-4">
            <dt className="font-medium text-steel-secondary">{label}</dt>
            <dd className="font-mono text-charcoal-ink">{value}</dd>
          </div>
        ))}
      </dl>
    </div>
  );
}
