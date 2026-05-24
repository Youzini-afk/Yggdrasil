import { useEffect, useState, type FormEvent, type ReactNode } from "react";
import { motion, AnimatePresence } from "motion/react";
import {
  ArrowRight,
  CheckCircle,
  Download,
  Folder as PIconFolder,
  GitBranch as PIconGit,
  GithubLogo,
  Globe as PIconGlobe,
  Info,
  Key as PIconKey,
  Lightning,
  LinkSimple,
  Question,
  Warning,
  XCircle,
} from "@/components/icons";
import { Button } from "@/components/ui/button";
import { Field, InputGroup, Checkbox } from "@/components/ui/input";
import { Modal, ModalHeader, ModalFooter } from "@/components/ui/modal";
import { StatusPill } from "@/components/ui/status-pill";
import { EyebrowSm } from "@/components/ui/typography";
import { useToast } from "@/components/ui/toast";
import { useKernel } from "@/lib/kernel-client";
import { cn } from "@/lib/cn";
import type {
  InstallConsent,
  InstallDetectedKind,
  InstallExecuteResult,
  InstallPlan,
} from "@/protocol/client";

type InstallStep = "url" | "plan" | "progress" | "external";

type InstallPhase = "resolving" | "detecting" | "reviewed" | "executing" | "completed" | "failed";

interface ShortcutEntry {
  url: string;
  tag: string;
}

const SHORTCUTS: ShortcutEntry[] = [
  { url: "https://github.com/Youzini-afk/Yggdrasil-Tavern", tag: "native" },
  { url: "/absolute/path/to/local-project", tag: "local" },
];

export function InstallModal({
  open,
  onClose,
  onInstalled,
}: {
  open: boolean;
  onClose: () => void;
  onInstalled?: () => void;
}) {
  const client = useKernel();
  const toast = useToast();
  const [step, setStep] = useState<InstallStep>("url");
  const [url, setUrl] = useState("");
  const [approvedPermissions, setApprovedPermissions] = useState(false);
  const [plan, setPlan] = useState<InstallPlan | null>(null);
  const [detectedKind, setDetectedKind] = useState<InstallDetectedKind | null>(null);
  const [resolveError, setResolveError] = useState<string | null>(null);
  const [externalPlanError, setExternalPlanError] = useState<string | null>(null);
  const [isResolving, setIsResolving] = useState(false);
  const [isExecuting, setIsExecuting] = useState(false);
  const [installResult, setInstallResult] = useState<InstallExecuteResult | null>(null);
  const [progressPhases, setProgressPhases] = useState<InstallPhase[]>([]);
  const [progressError, setProgressError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) reset();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  const reset = () => {
    setStep("url");
    setUrl("");
    setApprovedPermissions(false);
    setPlan(null);
    setDetectedKind(null);
    setResolveError(null);
    setExternalPlanError(null);
    setIsResolving(false);
    setIsExecuting(false);
    setInstallResult(null);
    setProgressPhases([]);
    setProgressError(null);
  };

  const handleClose = () => {
    onClose();
    setTimeout(reset, 250);
  };

  const onContinueFromUrl = async () => {
    if (!url.trim() || isResolving) return;
    const source = { root_url: url.trim(), root_ref: "HEAD" };
    setResolveError(null);
    setExternalPlanError(null);
    setIsResolving(true);
    setProgressPhases(["resolving", "detecting"]);
    try {
      const [planOutcome, kindOutcome] = await Promise.allSettled([
        client.resolveInstallPlan(source),
        client.detectInstallKind(source),
      ]);
      if (kindOutcome.status === "rejected") {
        throw kindOutcome.reason;
      }
      const kind = kindOutcome.value;
      setDetectedKind(kind);

      if (planOutcome.status === "fulfilled") {
        setPlan(planOutcome.value);
      } else {
        const planError = errorMessage(planOutcome.reason);
        setPlan(null);
        if (kind.kind !== "external") {
          throw new Error(planError);
        }
        setExternalPlanError(planError);
      }

      setApprovedPermissions(false);
      setProgressPhases(["resolving", "detecting", "reviewed"]);
      if (kind.kind === "external") {
        setStep("external");
      } else {
        setStep("plan");
      }
    } catch (err) {
      const message = errorMessage(err);
      setResolveError(message);
      setProgressPhases(["resolving", "failed"]);
      toast.push({ variant: "error", title: "Install plan failed", body: message });
    } finally {
      setIsResolving(false);
    }
  };

  const onConfirmPlan = async () => {
    if (!plan || !approvedPermissions || isExecuting || detectedKind?.kind === "external") return;
    const consent: InstallConsent = {
      approved_capabilities: plan.permissions_summary.new_capabilities,
      approved_network_hosts: plan.permissions_summary.new_network_hosts,
      approved_secret_refs: plan.permissions_summary.new_secret_refs,
    };
    setStep("progress");
    setIsExecuting(true);
    setProgressError(null);
    setProgressPhases(["resolving", "detecting", "reviewed", "executing"]);
    try {
      const result = await client.executeInstallPlan(plan, consent, "default");
      setInstallResult(result);
      setProgressPhases(["resolving", "detecting", "reviewed", "executing", "completed"]);
      toast.push({
        variant: "success",
        title: "Install complete",
        body: `${result.installed.length} package${result.installed.length === 1 ? "" : "s"} installed${
          result.project?.project_id ? ` · project ${result.project.project_id}` : ""
        }`,
      });
      onInstalled?.();
      window.setTimeout(handleClose, 700);
    } catch (err) {
      const message = errorMessage(err);
      setProgressError(message);
      setProgressPhases(["resolving", "detecting", "reviewed", "executing", "failed"]);
      toast.push({ variant: "error", title: "Install failed", body: message });
    } finally {
      setIsExecuting(false);
    }
  };

  const onContinueExternal = () => {
    if (detectedKind?.kind === "external") {
      setStep("external");
    } else {
      setStep("plan");
    }
  };

  return (
    <Modal open={open} onOpenChange={handleClose} size={step === "plan" ? "lg" : "md"} contentLabel="Install project">
      <AnimatePresence mode="wait">
        {step === "url" ? (
          <motion.div
            key="url"
            initial={{ opacity: 0, x: 8 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -8 }}
            transition={{ duration: 0.18 }}
          >
            <UrlStep
              url={url}
              onUrlChange={setUrl}
              shortcuts={SHORTCUTS}
              onSelectShortcut={(s) => setUrl(s.url)}
              onContinue={onContinueFromUrl}
              onCancel={handleClose}
              loading={isResolving}
              error={resolveError}
            />
          </motion.div>
        ) : null}
        {step === "plan" ? (
          <motion.div
            key="plan"
            initial={{ opacity: 0, x: 8 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -8 }}
            transition={{ duration: 0.18 }}
          >
            {plan ? (
              <PlanStep
                url={url}
                plan={plan}
                detectedKind={detectedKind}
                approvedPermissions={approvedPermissions}
                onApprovalChange={setApprovedPermissions}
                onBack={() => setStep("url")}
                onCancel={handleClose}
                onConfirm={onConfirmPlan}
                installing={isExecuting}
              />
            ) : null}
          </motion.div>
        ) : null}
        {step === "external" ? (
          <motion.div
            key="external"
            initial={{ opacity: 0, x: 8 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -8 }}
            transition={{ duration: 0.18 }}
          >
            <ExternalWizardStep
              url={url}
              plan={plan}
              planError={externalPlanError}
              onBack={() => setStep("url")}
              onCancel={handleClose}
              onContinue={onContinueExternal}
            />
          </motion.div>
        ) : null}
        {step === "progress" ? (
          <motion.div
            key="progress"
            initial={{ opacity: 0, x: 8 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -8 }}
            transition={{ duration: 0.18 }}
          >
            <ProgressStep
              url={url}
              plan={plan}
              phases={progressPhases}
              result={installResult}
              error={progressError}
              onCancel={handleClose}
            />
          </motion.div>
        ) : null}
      </AnimatePresence>
    </Modal>
  );
}

/* ────────────────────────────────────────────────────────────────
   Step 1 — URL input
   ──────────────────────────────────────────────────────────────── */

function UrlStep({
  url,
  onUrlChange,
  shortcuts,
  onSelectShortcut,
  onContinue,
  onCancel,
  loading,
  error,
}: {
  url: string;
  onUrlChange: (value: string) => void;
  shortcuts: ShortcutEntry[];
  onSelectShortcut: (s: ShortcutEntry) => void;
  onContinue: () => void;
  onCancel: () => void;
  loading: boolean;
  error: string | null;
}) {
  const onSubmit = (event: FormEvent) => {
    event.preventDefault();
    onContinue();
  };

  return (
    <form onSubmit={onSubmit}>
      <ModalHeader
        eyebrow="Install — Step 1 of 3"
        title="Where is the project?"
        description="Yggdrasil installs from public Git repositories or local folders. We'll review the project before anything runs."
      />

      <Field label="Source URL or path" required>
        <InputGroup
          leftIcon={<LinkSimple size={16} />}
          value={url}
          onChange={(e) => onUrlChange(e.target.value)}
          placeholder="github.com/user/repo  or  /absolute/path/to/local-folder"
          spellCheck={false}
          disabled={loading}
          autoFocus
        />
      </Field>
      <p className="mt-1 text-[12px] text-steel-secondary">
        HTTPS Git only · absolute local paths
      </p>

      <section className="mt-5 flex flex-col gap-2">
        <EyebrowSm>Shortcuts</EyebrowSm>
        <div className="flex flex-wrap gap-2">
          {shortcuts.map((entry) => (
            <button
              key={entry.url}
              type="button"
              onClick={() => onSelectShortcut(entry)}
              disabled={loading}
              className="inline-flex items-center gap-2 rounded-full border border-whisper-border px-3 py-1 text-[12px] text-charcoal-ink transition hover:bg-whisper-border-strong/30"
            >
              <GithubLogo size={12} className="text-steel-secondary" />
              <span className="font-mono">{entry.url}</span>
              <span className="font-mono text-[11px] text-muted-tone">{entry.tag}</span>
            </button>
          ))}
        </div>
      </section>

      {error ? (
        <div className="mt-5 rounded-[12px] border border-deep-rust bg-deep-rust-surface px-4 py-3 text-[12px] text-deep-rust">
          <div className="flex items-start gap-2">
            <XCircle size={15} className="mt-0.5 shrink-0" />
            <div>
              <p className="font-medium">Could not resolve install plan</p>
              <p className="mt-1 font-mono text-[11px] leading-snug">{error}</p>
            </div>
          </div>
        </div>
      ) : null}

      <ModalFooter className="justify-between">
        <p className="flex items-center gap-1.5 text-[11px] text-muted-tone">
          <Lightning size={11} className="text-muted-tone" />
          Press ⌘V to paste · ↵ to continue · Esc to cancel
        </p>
        <div className="flex items-center gap-3">
          <Button type="button" tone="secondary" onClick={onCancel} disabled={loading}>
            Cancel
          </Button>
          <Button type="submit" tone="primary" disabled={!url.trim() || loading}>
            {loading ? "Resolving…" : "Continue"}
            {loading ? null : <ArrowRight size={14} />}
          </Button>
        </div>
      </ModalFooter>
    </form>
  );
}

/* ────────────────────────────────────────────────────────────────
   Step 2 — Plan review
   ──────────────────────────────────────────────────────────────── */

function PlanStep({
  url,
  plan,
  detectedKind,
  approvedPermissions,
  onApprovalChange,
  onBack,
  onCancel,
  onConfirm,
  installing,
}: {
  url: string;
  plan: InstallPlan;
  detectedKind: InstallDetectedKind | null;
  approvedPermissions: boolean;
  onApprovalChange: (v: boolean) => void;
  onBack: () => void;
  onCancel: () => void;
  onConfirm: () => void;
  installing: boolean;
}) {
  const rootPackage = plan.packages.find((pkg) => pkg.id === plan.root_id) ?? plan.packages[0];
  const permissionGroups = [
    {
      label: "Capabilities",
      detail: formatList(plan.permissions_summary.new_capabilities, "No new capability invokes"),
      count: plan.permissions_summary.new_capabilities.length,
    },
    {
      label: "Network",
      detail: formatList(plan.permissions_summary.new_network_hosts, "No new network hosts"),
      count: plan.permissions_summary.new_network_hosts.length,
    },
    {
      label: "Secrets",
      detail: formatList(plan.permissions_summary.new_secret_refs, "No new secret refs"),
      count: plan.permissions_summary.new_secret_refs.length,
    },
  ];
  const conformance = summarizeConformance(plan);
  const isExternal = detectedKind?.kind === "external";

  return (
    <>
      <ModalHeader
        eyebrow="Install — Step 2 of 3"
        title="Review the install plan"
        description="Install Lab resolved this plan. Approve requested permissions to begin installation."
      />

      {/* Project identity */}
      <div className="flex items-center gap-3 rounded-[12px] border border-whisper-border bg-aged-brass-surface-soft px-4 py-3">
        <GithubLogo size={20} className="text-charcoal-ink shrink-0" />
        <span className="flex-1 truncate font-mono text-[13px] text-charcoal-ink">{url}</span>
        <StatusPill tone="accent" label="RESOLVED" />
        <span className="font-mono text-[11px] text-muted-tone">root: {plan.root_id}</span>
      </div>

      {isExternal ? (
        <div className="mt-4 rounded-[12px] border border-aged-brass-border bg-aged-brass-surface-soft px-4 py-3 text-[12px] text-charcoal-ink">
          <div className="flex items-start gap-2">
            <Warning size={15} className="mt-0.5 shrink-0 text-aged-brass-deep" />
            <div>
              <p className="font-medium">External adapter generation is CLI-only in this build.</p>
              <p className="mt-1 text-steel-secondary">
                The package plan is real, but the web UI will not execute it without a project descriptor.
              </p>
            </div>
          </div>
        </div>
      ) : null}

      {/* Project metadata */}
      <section className="mt-6">
        <EyebrowSm>Project</EyebrowSm>
        <dl className="mt-3 grid grid-cols-2 gap-x-8 gap-y-2 text-[12px]">
          {[
            ["Kind", formatDetectedKind(detectedKind), "accent"],
            ["Root package", plan.root_id, "mono"],
            ["Version", rootPackage?.version ?? "—", "mono"],
            ["Source", rootPackage?.source ?? "—"],
            ["Commit", rootPackage?.commit_sha ? shortHash(rootPackage.commit_sha) : "—", "mono"],
            ["Signed", plan.signature_summary.all_signed ? "All signed" : "Unsigned packages", plan.signature_summary.all_signed ? "accent" : "neutral"],
          ].map(([label, value, hint]) => (
            <div key={label as string} className="flex justify-between">
              <dt className="font-medium text-steel-secondary">{label}</dt>
              <dd className={cn("text-charcoal-ink", hint === "mono" && "font-mono")}>
                {hint === "accent" ? (
                  <span className="inline-flex items-center gap-1.5">
                    <span className="size-1.5 rounded-full bg-aged-brass" aria-hidden />
                    {value}
                  </span>
                ) : hint === "neutral" ? (
                  <span className="inline-flex items-center gap-1.5">
                    <span className="size-1.5 rounded-full bg-steel-secondary" aria-hidden />
                    {value}
                  </span>
                ) : (
                  value
                )}
              </dd>
            </div>
          ))}
        </dl>
      </section>

      <div className="my-6 h-px bg-whisper-border" />

      {/* Dependencies */}
      <section>
        <div className="flex items-center justify-between">
          <EyebrowSm>Packages</EyebrowSm>
          <span className="text-[11px] text-steel-secondary">
            {plan.packages.length} package{plan.packages.length === 1 ? "" : "s"} will be installed
          </span>
        </div>
        <ul className="mt-3 divide-y divide-whisper-border">
          {plan.packages.map((pkg) => (
            <li key={`${pkg.id}:${pkg.tree_hash}`} className="flex items-center gap-3 py-2.5">
              <PackageMarker signed={pkg.signed} />
              <span className="flex-1 truncate font-mono text-[12px] text-charcoal-ink">{pkg.id}</span>
              <span className="font-mono text-[11px] text-muted-tone">{pkg.version}</span>
              <StatusPill tone={pkg.id === plan.root_id ? "accent" : "neutral"} label={pkg.source.toUpperCase()} showDot={false} />
            </li>
          ))}
        </ul>
      </section>

      <div className="my-6 h-px bg-whisper-border" />

      {/* Permissions */}
      <section>
        <div className="flex items-center justify-between">
          <EyebrowSm>Permissions requested</EyebrowSm>
          <span className="text-[11px] text-steel-secondary">
            {permissionGroups.reduce((sum, group) => sum + group.count, 0)} total entries
          </span>
        </div>
        <ul className="mt-3 divide-y divide-whisper-border">
          {permissionGroups.map((p) => (
            <li key={p.label} className="flex gap-3 py-3">
              <span className="rounded-full bg-aged-brass-surface-soft p-2 text-aged-brass shrink-0">
                <PermissionIcon label={p.label} />
              </span>
              <div className="min-w-0 flex-1">
                <p className="text-[12px] font-medium text-charcoal-ink">{p.label}</p>
                <p className="mt-0.5 truncate text-[11px] text-steel-secondary">{p.detail}</p>
              </div>
            </li>
          ))}
        </ul>
      </section>

      <div className="my-6 h-px bg-whisper-border" />

      <section className="grid gap-3 text-[12px] md:grid-cols-3">
        <SummaryCard
          icon={<CheckCircle size={16} className="text-aged-brass" />}
          title="Signatures"
          body={
            plan.signature_summary.all_signed
              ? "All packages signed"
              : `Unsigned: ${formatList(plan.signature_summary.unsigned_packages, "none")}`
          }
        />
        <SummaryCard
          icon={plan.integrity_summary.manifest_hashes_match_lockfile ? <CheckCircle size={16} className="text-aged-brass" /> : <Warning size={16} className="text-aged-brass-deep" />}
          title="Integrity"
          body={
            plan.integrity_summary.manifest_hashes_match_lockfile
              ? "No lockfile drift detected"
              : `${plan.integrity_summary.drift_detected.length} drift item${plan.integrity_summary.drift_detected.length === 1 ? "" : "s"}`
          }
        />
        <SummaryCard
          icon={conformance.hasFailures ? <Warning size={16} className="text-aged-brass-deep" /> : <CheckCircle size={16} className="text-aged-brass" />}
          title="Conformance"
          body={conformance.label}
        />
      </section>

      <ModalFooter className="justify-between">
        <Checkbox
          checked={approvedPermissions}
          onCheckedChange={onApprovalChange}
          label="Approve requested permissions"
        />
        <div className="flex items-center gap-3">
          <Button tone="tertiary" onClick={onBack} disabled={installing}>
            Back
          </Button>
          <Button tone="secondary" onClick={onCancel} disabled={installing}>
            Cancel
          </Button>
          <Button tone="primary" onClick={onConfirm} disabled={!approvedPermissions || installing || isExternal}>
            <Download size={14} />
            {installing ? "Installing…" : "Install"}
          </Button>
        </div>
      </ModalFooter>
    </>
  );
}

function PermissionIcon({ label }: { label: string }) {
  switch (label) {
    case "Network":
      return <PIconGlobe size={16} />;
    case "Secrets":
      return <PIconKey size={16} />;
    case "Filesystem":
      return <PIconFolder size={16} />;
    default:
      return <PIconGit size={16} />;
  }
}

function PackageMarker({ signed }: { signed: boolean }) {
  return signed ? (
    <CheckCircle size={14} className="shrink-0 text-aged-brass" weight="fill" />
  ) : (
    <span className="flex size-3.5 shrink-0 items-center justify-center rounded-full border border-whisper-border-strong/70">
      <span className="size-1.5 rounded-full bg-muted-tone" aria-hidden />
    </span>
  );
}

function SummaryCard({ icon, title, body }: { icon: ReactNode; title: string; body: string }) {
  return (
    <div className="rounded-[12px] border border-whisper-border bg-pure-surface p-3">
      <div className="flex items-center gap-2">
        {icon}
        <p className="font-medium text-charcoal-ink">{title}</p>
      </div>
      <p className="mt-2 line-clamp-2 text-[11px] leading-snug text-steel-secondary">{body}</p>
    </div>
  );
}

function formatList(values: string[] | undefined, empty: string, limit = 3) {
  if (!values || values.length === 0) return empty;
  const shown = values.slice(0, limit).join(" · ");
  const remaining = values.length - limit;
  return remaining > 0 ? `${shown} · +${remaining} more` : shown;
}

function shortHash(hash: string) {
  return hash.length > 12 ? `${hash.slice(0, 12)}…` : hash;
}

function formatDetectedKind(kind: InstallDetectedKind | null) {
  switch (kind?.kind) {
    case "native":
      return "Native project";
    case "declared_external":
      return "Declared external";
    case "external":
      return "External";
    default:
      return "Detected";
  }
}

function errorMessage(err: unknown) {
  return err instanceof Error ? err.message : String(err);
}

function summarizeConformance(plan: InstallPlan) {
  let checks = 0;
  let failures = 0;
  let warnings = 0;
  for (const pkg of plan.packages) {
    const report = pkg.conformance;
    if (!report) continue;
    if (Array.isArray(report.checks)) {
      checks += report.checks.length;
      failures += report.checks.filter((check) => check.passed === false || check.status === "failed").length;
    }
    if (Array.isArray(report.failures)) failures += report.failures.length;
    if (Array.isArray(report.warnings)) warnings += report.warnings.length;
    if (report.passed === false && failures === 0) failures += 1;
  }
  if (checks === 0 && failures === 0 && warnings === 0) {
    return { hasFailures: false, label: "No conformance details returned" };
  }
  return {
    hasFailures: failures > 0,
    label: `${checks} check${checks === 1 ? "" : "s"}, ${failures} failure${failures === 1 ? "" : "s"}, ${warnings} warning${warnings === 1 ? "" : "s"}`,
  };
}

/* ────────────────────────────────────────────────────────────────
   Step 3 — Progress
   ──────────────────────────────────────────────────────────────── */

function ProgressStep({
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
          {result ? <p>wrote profile {result.profile_path}</p> : null}
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

/* ────────────────────────────────────────────────────────────────
   External project wizard
   ──────────────────────────────────────────────────────────────── */

function ExternalWizardStep({
  url,
  plan,
  planError,
  onBack,
  onCancel,
  onContinue,
}: {
  url: string;
  plan: InstallPlan | null;
  planError: string | null;
  onBack: () => void;
  onCancel: () => void;
  onContinue: () => void;
}) {
  const [choice, setChoice] = useState<"wrap" | "workspace">("wrap");

  return (
    <>
      <ModalHeader
        eyebrow="Install — External project"
        title="External adapter generation is CLI-only"
        description="This source does not declare a Yggdrasil project descriptor. The web UI will not execute the package install without one."
      />

      <div className="flex items-center gap-3 rounded-[12px] border border-whisper-border px-4 py-3">
        <GithubLogo size={18} className="text-charcoal-ink" />
        <span className="flex-1 truncate font-mono text-[13px] text-charcoal-ink">{url}</span>
        <StatusPill tone="neutral" label="EXTERNAL" showDot={false} />
      </div>

      <div className="mt-5 rounded-[12px] border border-aged-brass-border bg-aged-brass-surface-soft px-4 py-3 text-[12px] text-charcoal-ink">
        <div className="flex items-start gap-2">
          <Info size={15} className="mt-0.5 shrink-0 text-aged-brass-deep" />
          <p className="leading-snug">
            Use the CLI to generate a descriptor for wrap/workspace mode, then install the declared project from web.
          </p>
        </div>
      </div>

      <div className="mt-4 rounded-[12px] border border-whisper-border bg-pure-surface px-4 py-3 text-[12px]">
        <p className="font-medium text-charcoal-ink">
          {plan ? `${plan.packages.length} package${plan.packages.length === 1 ? "" : "s"} resolved` : "Package plan not available"}
        </p>
        {plan ? (
          <p className="mt-1 font-mono text-[11px] text-steel-secondary">root: {plan.root_id}</p>
        ) : planError ? (
          <p className="mt-1 font-mono text-[11px] leading-snug text-deep-rust">{planError}</p>
        ) : null}
      </div>

      <div className="mt-5 flex flex-col gap-3">
        <ExternalChoiceCard
          selected={choice === "wrap"}
          onSelect={() => setChoice("wrap")}
          title="Wrap with adapter"
          recommended
          description="Requires CLI descriptor generation in this build before web install can execute."
          chips={[
            { label: "CLI-only generation", tone: "warn" },
            { label: "No web execution", tone: "warn" },
          ]}
        />
        <ExternalChoiceCard
          selected={choice === "workspace"}
          onSelect={() => setChoice("workspace")}
          title="Open as workspace"
          description="Also requires a CLI-generated workspace descriptor before this web install path can continue."
          chips={[
            { label: "CLI-only descriptor", tone: "warn" },
            { label: "Install blocked here", tone: "warn" },
          ]}
        />
      </div>

      <button
        type="button"
        className="mt-5 inline-flex items-center gap-1.5 text-[12px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
      >
        <Question size={12} />
        Generate a project descriptor with the CLI, then return here.
      </button>

      <ModalFooter className="justify-end">
        <Button tone="tertiary" onClick={onBack}>
          Back
        </Button>
        <Button tone="secondary" onClick={onCancel}>
          Cancel
        </Button>
        <Button tone="primary" onClick={onContinue} disabled>
          Continue disabled
        </Button>
      </ModalFooter>
    </>
  );
}

function ExternalChoiceCard({
  selected,
  onSelect,
  title,
  description,
  chips,
  recommended,
}: {
  selected: boolean;
  onSelect: () => void;
  title: string;
  description: string;
  chips: Array<{ label: string; tone: "good" | "warn" }>;
  recommended?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={onSelect}
      className={cn(
        "group flex w-full items-start gap-4 rounded-[16px] border bg-pure-surface p-5 text-left transition",
        selected
          ? "border-l-[3px] border-l-aged-brass border-aged-brass-border bg-aged-brass-surface-soft"
          : "border-whisper-border hover:bg-whisper-border-strong/20",
      )}
    >
      <span
        className={cn(
          "mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-full border transition",
          selected ? "border-aged-brass" : "border-whisper-border-strong",
        )}
      >
        {selected ? <span className="size-2 rounded-full bg-aged-brass" /> : null}
      </span>
      <div className="flex-1">
        <div className="flex items-center gap-2">
          <h3 className="font-display text-[17px] font-bold text-charcoal-ink">{title}</h3>
          {recommended ? <StatusPill tone="accent" label="RECOMMENDED" showDot={false} /> : null}
        </div>
        <p className="mt-1 text-[13px] leading-snug text-steel-secondary">{description}</p>
        <ul className="mt-3 flex flex-wrap gap-2">
          {chips.map((chip) => (
            <li
              key={chip.label}
              className={cn(
                "inline-flex items-center gap-1.5 rounded-full border border-whisper-border px-2.5 py-0.5 text-[11px]",
                chip.tone === "good" ? "text-charcoal-ink" : "text-muted-tone",
              )}
            >
              {chip.tone === "good" ? (
                <CheckCircle size={11} className="text-aged-brass" weight="fill" />
              ) : (
                <span className="size-1.5 rounded-full bg-muted-tone" aria-hidden />
              )}
              {chip.label}
            </li>
          ))}
        </ul>
      </div>
    </button>
  );
}
