import { ArrowsClockwise, Copy, GithubLogo, Warning } from "@/components/icons";
import { Button } from "@/components/ui/button";
import { Modal, ModalHeader, ModalFooter } from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";

export interface FailureDetail {
  projectName: string;
  icon?: React.ReactNode;
  title?: string;
  summary?: string;
  log?: string[];
  /** Real failure metadata from kernel events, when available. */
  exitCode?: string;
  cause?: string;
  uptime?: string;
  lastCheckpoint?: string;
  failedAt?: string;
  redactionState?: string;
  logRedacted?: boolean;
}

export interface FailureModalProps {
  open: boolean;
  onClose: () => void;
  onRestart?: () => void;
  onUninstall?: () => void;
  detail?: FailureDetail;
}

export function FailureModal({ open, onClose, onRestart, onUninstall, detail }: FailureModalProps) {
  const toast = useToast();

  const projectName = detail?.projectName ?? "Project";
  const log = detail?.log ?? [];
  const canCopyLog = detail?.logRedacted === true && log.length > 0;

  const copy = () => {
    if (!canCopyLog) return;
    navigator.clipboard?.writeText(log.join("\n"));
    toast.push({ variant: "success", title: "Log copied", duration: 2400 });
  };

  return (
    <Modal open={open} onOpenChange={onClose} accent="rust" size="lg" contentLabel={`${projectName} failure details`}>
      <ModalHeader
        eyebrow={`Failure — ${projectName.toUpperCase()}`}
        title={detail?.title ?? "Project failed"}
        description={
          detail?.summary ?? "Project state is preserved. See the log below for the failure."
        }
      />

      {/* Identity row */}
      <div className="flex items-center gap-3 rounded-[12px] border border-whisper-border px-4 py-3">
        {detail?.icon ?? <Warning size={20} className="text-deep-rust" />}
        <span className="font-display text-[14px] font-bold text-charcoal-ink">{projectName}</span>
        <GithubLogo size={14} className="ml-2 text-steel-secondary" />
        {detail?.failedAt ? (
          <span className="ml-auto font-mono text-[11px] text-muted-tone">{detail.failedAt}</span>
        ) : null}
      </div>

      {/* Diagnosis & impact */}
      <div className="mt-6 grid grid-cols-2 gap-6">
        <DiagnosisColumn
          eyebrow="Diagnosis"
          rows={[
            ["Exit code", detail?.exitCode ?? "—"],
            ["Cause", detail?.cause ?? "—"],
            ["Uptime", detail?.uptime ?? "—"],
          ]}
        />
        <DiagnosisColumn
          eyebrow="Impact"
          rows={[
            ["Last checkpoint", detail?.lastCheckpoint ?? "—"],
            ["Sessions", "preserved"],
          ]}
        />
      </div>

      <div className="my-6 h-px bg-whisper-border" />

      {/* Log */}
      <section>
        <div className="flex items-center justify-between">
          <p className="font-mono text-[10px] uppercase tracking-[0.12em] text-muted-tone">
            Redacted stderr · last {log.length} lines
          </p>
          <div className="flex items-center gap-3">
            <button
              type="button"
              onClick={copy}
              disabled={!canCopyLog}
              className="inline-flex items-center gap-1 text-[11px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
            >
              <Copy size={12} />
              {canCopyLog ? "Copy log" : "No redacted log"}
            </button>
          </div>
        </div>
        <div
          className="mt-2 space-y-0.5 rounded-[10px] p-3 font-mono text-[11px] leading-relaxed text-charcoal-ink"
          style={{ background: "var(--color-inset-surface)" }}
        >
          {log.length > 0 ? (
            log.map((line, idx) => (
              <p key={idx} className={idx === log.length - 1 ? "text-deep-rust" : ""}>
                {line}
              </p>
            ))
          ) : (
            <p className="text-muted-tone">No diagnostic log tail is available for this package.</p>
          )}
        </div>
      </section>

      <ModalFooter className="justify-end">
        {/* No telemetry promise on About page; do not offer report opt-in
            until a real, audited reporting capability lands. */}
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
