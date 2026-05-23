import { useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "motion/react";
import {
  ArrowRight,
  CheckCircle,
  Cloud,
  Download,
  GithubLogo,
  Lightning,
  LinkSimple,
  Question,
} from "@/components/icons";
import { Button } from "@/components/ui/button";
import { Field, InputGroup, Checkbox } from "@/components/ui/input";
import { Modal, ModalHeader, ModalFooter } from "@/components/ui/modal";
import { StatusPill } from "@/components/ui/status-pill";
import { EyebrowSm } from "@/components/ui/typography";
import { useToast } from "@/components/ui/toast";
import { cn } from "@/lib/cn";

type InstallStep = "url" | "plan" | "progress" | "external";

interface SuggestionEntry {
  id: string;
  url: string;
  kind: "native" | "external";
  language?: string;
}

const MOCK_SUGGESTIONS: SuggestionEntry[] = [
  { id: "s1", url: "github.com/Youzini-afk/Yggdrasil-Tavern", kind: "native" },
  { id: "s2", url: "github.com/Youzini-afk/Ygg-Coding", kind: "native" },
  { id: "s3", url: "github.com/Youzini-afk/Yggdrasil-CLI", kind: "external", language: "Rust" },
];

const RECENT: Array<{ url: string; tag: string }> = [
  { url: "github.com/example/some-tool", tag: "v1.2.0" },
  { url: "~/projects/scratch", tag: "local" },
];

/**
 * The install flow is a UX prototype on top of the eventual install protocol.
 * The host does not yet expose `kernel.v1.install.*` plan / dependency / progress
 * methods, so the wizard runs against deterministic mock data and a simulated
 * progress interval. The plan step shows a `PROTOTYPE` chip so users know to
 * use `yg install` on the CLI for real installs until the protocol lands.
 */
export function InstallModal({ open, onClose }: { open: boolean; onClose: () => void }) {
  const toast = useToast();
  const [step, setStep] = useState<InstallStep>("url");
  const [url, setUrl] = useState("");
  const [trustPublisher, setTrustPublisher] = useState(false);
  const [progressFraction, setProgressFraction] = useState(0);
  // Stable ref so cancel/unmount can clear the simulation interval and a stale
  // tick can't fire `Installation complete` after the user dismisses.
  const intervalRef = useRef<number | null>(null);

  const clearInterval = () => {
    if (intervalRef.current !== null) {
      window.clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  };

  // Always clear any in-flight interval when the modal closes / unmounts.
  useEffect(() => clearInterval, []);
  useEffect(() => {
    if (!open) clearInterval();
  }, [open]);

  const reset = () => {
    clearInterval();
    setStep("url");
    setUrl("");
    setTrustPublisher(false);
    setProgressFraction(0);
  };

  const handleClose = () => {
    clearInterval();
    onClose();
    setTimeout(reset, 250);
  };

  const filteredSuggestions = MOCK_SUGGESTIONS.filter((s) =>
    url.length === 0 ? false : s.url.toLowerCase().startsWith(url.toLowerCase()),
  );

  const onContinueFromUrl = () => {
    if (!url) return;
    const suggestion = MOCK_SUGGESTIONS.find((s) => s.url === url);
    if (suggestion?.kind === "external") {
      setStep("external");
    } else {
      setStep("plan");
    }
  };

  const onConfirmPlan = () => {
    setStep("progress");
    // Simulated progress until the install protocol is wired.
    let frac = 0;
    intervalRef.current = window.setInterval(() => {
      frac = Math.min(frac + 0.18, 1);
      setProgressFraction(frac);
      if (frac >= 1) {
        clearInterval();
        window.setTimeout(() => {
          handleClose();
          toast.push({
            variant: "success",
            title: "Install simulated",
            body: "When the install protocol lands, this completes the real install.",
          });
        }, 600);
      }
    }, 480);
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
              suggestions={filteredSuggestions}
              onSelectSuggestion={(s) => setUrl(s.url)}
              onContinue={onContinueFromUrl}
              onCancel={handleClose}
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
            <PlanStep
              url={url}
              trustPublisher={trustPublisher}
              onTrustChange={setTrustPublisher}
              onBack={() => setStep("url")}
              onCancel={handleClose}
              onConfirm={onConfirmPlan}
            />
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
              onBack={() => setStep("url")}
              onCancel={handleClose}
              onContinue={() => setStep("plan")}
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
            <ProgressStep url={url} progress={progressFraction} onCancel={handleClose} />
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
  suggestions,
  onSelectSuggestion,
  onContinue,
  onCancel,
}: {
  url: string;
  onUrlChange: (value: string) => void;
  suggestions: SuggestionEntry[];
  onSelectSuggestion: (s: SuggestionEntry) => void;
  onContinue: () => void;
  onCancel: () => void;
}) {
  return (
    <>
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
          placeholder="github.com/user/repo  or  ~/projects/local-folder"
          spellCheck={false}
          autoFocus
        />
      </Field>
      <p className="mt-1 text-[12px] text-steel-secondary">
        HTTPS Git only · ~/path or absolute local paths
      </p>

      {suggestions.length > 0 ? (
        <section className="mt-5 flex flex-col gap-2">
          <EyebrowSm>Matches</EyebrowSm>
          <ul className="divide-y divide-whisper-border rounded-[12px] border border-whisper-border bg-pure-surface">
            {suggestions.map((suggestion, idx) => (
              <li key={suggestion.id}>
                <button
                  type="button"
                  onClick={() => onSelectSuggestion(suggestion)}
                  className={cn(
                    "flex w-full items-center gap-3 px-3 py-2.5 text-left transition",
                    idx === 0 ? "bg-aged-brass-surface-soft" : "hover:bg-whisper-border-strong/20",
                  )}
                >
                  <GithubLogo size={16} className="text-charcoal-ink" />
                  <span className="flex-1 truncate font-mono text-[13px] text-charcoal-ink">
                    {suggestion.url}
                  </span>
                  <StatusPill
                    tone={suggestion.kind === "native" ? "accent" : "neutral"}
                    label={
                      suggestion.kind === "native"
                        ? "NATIVE PROJECT"
                        : `EXTERNAL (${suggestion.language ?? "?"})`
                    }
                    showDot={false}
                  />
                  {idx === 0 ? (
                    <span className="font-mono text-[11px] text-muted-tone">↵</span>
                  ) : null}
                </button>
              </li>
            ))}
          </ul>
        </section>
      ) : null}

      <section className="mt-5 flex flex-col gap-2">
        <EyebrowSm>Recent</EyebrowSm>
        <div className="flex flex-wrap gap-2">
          {RECENT.map((entry) => (
            <button
              key={entry.url}
              type="button"
              onClick={() => onUrlChange(entry.url)}
              className="inline-flex items-center gap-2 rounded-full border border-whisper-border px-3 py-1 text-[12px] text-charcoal-ink transition hover:bg-whisper-border-strong/30"
            >
              <GithubLogo size={12} className="text-steel-secondary" />
              <span className="font-mono">{entry.url}</span>
              <span className="font-mono text-[11px] text-muted-tone">{entry.tag}</span>
            </button>
          ))}
        </div>
      </section>

      <ModalFooter className="justify-between">
        <p className="flex items-center gap-1.5 text-[11px] text-muted-tone">
          <Lightning size={11} className="text-muted-tone" />
          Press ⌘V to paste · ↵ to install · Esc to cancel
        </p>
        <div className="flex items-center gap-3">
          <Button tone="secondary" onClick={onCancel}>
            Cancel
          </Button>
          <Button tone="primary" onClick={onContinue} disabled={!url}>
            Continue
            <ArrowRight size={14} />
          </Button>
        </div>
      </ModalFooter>
    </>
  );
}

/* ────────────────────────────────────────────────────────────────
   Step 2 — Plan review
   ──────────────────────────────────────────────────────────────── */

function PlanStep({
  url,
  trustPublisher,
  onTrustChange,
  onBack,
  onCancel,
  onConfirm,
}: {
  url: string;
  trustPublisher: boolean;
  onTrustChange: (v: boolean) => void;
  onBack: () => void;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const dependencies = [
    { id: "ydltavern-engine", version: "v0.1.0", official: true },
    { id: "ydltavern-surface", version: "v0.1.0", official: true },
  ];
  const permissions = [
    {
      label: "Network",
      detail: "api.openai.com · api.anthropic.com · generativelanguage.googleapis.com",
    },
    {
      label: "Secrets",
      detail: "secret_ref:store:OPENAI_API_KEY · secret_ref:store:ANTHROPIC_API_KEY · 3 more",
    },
    { label: "Filesystem", detail: "Read: ~/.yggdrasil/projects/ydltavern · Write: same" },
  ];

  return (
    <>
      <ModalHeader
        eyebrow="Install — Step 2 of 3"
        title="Review the install plan"
        description="Yggdrasil resolved the project source. Approve below to begin installation."
      />

      {/* Project identity */}
      <div className="flex items-center gap-3 rounded-[12px] border border-whisper-border bg-aged-brass-surface-soft px-4 py-3">
        <GithubLogo size={20} className="text-charcoal-ink shrink-0" />
        <span className="flex-1 truncate font-mono text-[13px] text-charcoal-ink">{url}</span>
        <StatusPill tone="accent" label="RESOLVED" />
        <span className="font-mono text-[11px] text-muted-tone">ref: main · 2a47e5c</span>
      </div>

      {/* Project metadata */}
      <section className="mt-6">
        <EyebrowSm>Project</EyebrowSm>
        <dl className="mt-3 grid grid-cols-2 gap-x-8 gap-y-2 text-[12px]">
          {[
            ["Type", "Native (project.yaml)", "accent"],
            ["Title", "YdlTavern"],
            ["Version", "v0.1.0", "mono"],
            ["Signed", "Unsigned", "neutral"],
            ["Size", "84.3 MB", "mono"],
            ["License", "AGPL-3.0"],
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
          <EyebrowSm>Dependencies</EyebrowSm>
          <span className="text-[11px] text-steel-secondary">
            {dependencies.length} packages will be installed
          </span>
        </div>
        <ul className="mt-3 divide-y divide-whisper-border">
          {dependencies.map((dep) => (
            <li key={dep.id} className="flex items-center gap-3 py-2.5">
              <Cloud size={14} className="text-steel-secondary" />
              <span className="flex-1 font-mono text-[12px] text-charcoal-ink">{dep.id}</span>
              <span className="font-mono text-[11px] text-muted-tone">{dep.version}</span>
              {dep.official ? <StatusPill tone="accent" label="OFFICIAL" showDot={false} /> : null}
            </li>
          ))}
        </ul>
      </section>

      <div className="my-6 h-px bg-whisper-border" />

      {/* Permissions */}
      <section>
        <div className="flex items-center justify-between">
          <EyebrowSm>Permissions requested</EyebrowSm>
          <span className="text-[11px] text-steel-secondary">{permissions.length} categories</span>
        </div>
        <ul className="mt-3 divide-y divide-whisper-border">
          {permissions.map((p) => (
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

      {/* Conformance */}
      <div className="flex items-center gap-2">
        <CheckCircle size={16} className="text-aged-brass" />
        <span className="text-[12px] font-medium text-charcoal-ink">All 8 checks passed</span>
        <button
          type="button"
          className="ml-auto text-[11px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
        >
          View report
        </button>
      </div>

      <ModalFooter className="justify-between">
        <Checkbox
          checked={trustPublisher}
          onCheckedChange={onTrustChange}
          label="Trust this publisher for future installs"
        />
        <div className="flex items-center gap-3">
          <Button tone="tertiary" onClick={onBack}>
            Back
          </Button>
          <Button tone="secondary" onClick={onCancel}>
            Cancel
          </Button>
          <Button tone="primary" onClick={onConfirm}>
            <Download size={14} />
            Install
          </Button>
        </div>
      </ModalFooter>
    </>
  );
}

import { GitBranch as PIconGit, Globe as PIconGlobe, Key as PIconKey, Folder as PIconFolder } from "@/components/icons";
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

/* ────────────────────────────────────────────────────────────────
   Step 3 — Progress
   ──────────────────────────────────────────────────────────────── */

function ProgressStep({
  url,
  progress,
  onCancel,
}: {
  url: string;
  progress: number;
  onCancel: () => void;
}) {
  const totalSteps = 6;
  const completedSteps = Math.min(Math.floor(progress * totalSteps), totalSteps);
  const steps = [
    { label: "Resolved source from Git", duration: "0.4s" },
    { label: "Conformance check (8/8 passed)", duration: "1.2s" },
    { label: "Permission plan approved", duration: "—" },
    { label: "Fetching package contents", duration: "in progress" },
    { label: "Verifying integrity hash", duration: "—" },
    { label: "Registering project", duration: "—" },
  ];

  return (
    <>
      <ModalHeader
        eyebrow="Install — Step 3 of 3"
        title="Installing YdlTavern"
        description={
          <span className="font-mono text-[11px] text-muted-tone">{url} · main · 2a47e5c</span>
        }
      />

      {/* Progress bar */}
      <div className="space-y-2">
        <div className="h-1.5 overflow-hidden rounded-full bg-whisper-border-strong/40">
          <motion.div
            className="h-full bg-aged-brass"
            initial={{ width: 0 }}
            animate={{ width: `${progress * 100}%` }}
            transition={{ duration: 0.4 }}
          />
        </div>
        <div className="flex items-center justify-between text-[11px]">
          <span className="font-medium text-charcoal-ink">
            Step {completedSteps + 1} of {totalSteps} · {Math.round(progress * 100)}%
          </span>
          <span className="font-mono text-steel-secondary">
            {(progress * 84.3).toFixed(1)} MB / 84.3 MB · 1.8 MB/s
          </span>
        </div>
      </div>

      {/* Step list */}
      <ul className="mt-6 divide-y divide-whisper-border rounded-[12px] border border-whisper-border bg-pure-surface">
        {steps.map((step, idx) => {
          const isComplete = idx < completedSteps;
          const isCurrent = idx === completedSteps && progress < 1;
          return (
            <li key={step.label} className="flex items-center gap-3 px-4 py-2.5">
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
                {step.label}
              </span>
              <span
                className={cn(
                  "font-mono text-[11px]",
                  isCurrent ? "text-aged-brass-deep" : "text-muted-tone",
                )}
              >
                {step.duration}
              </span>
            </li>
          );
        })}
      </ul>

      {/* Activity log */}
      <section className="mt-6">
        <EyebrowSm>Activity</EyebrowSm>
        <div className="mt-2 space-y-0.5 rounded-[10px] bg-warm-bone p-3 font-mono text-[11px] text-steel-secondary">
          <p>[14:32:18] git fetch refs/heads/main · 6 commits ahead of cache</p>
          <p>[14:32:21] received pack · 2.3 MB</p>
          <p>[14:32:22] applied delta · 84.3 MB total</p>
          <p className="border-l-2 border-aged-brass pl-2 text-charcoal-ink">
            [14:32:24] verifying sha256:2a47e5c…
          </p>
        </div>
      </section>

      <ModalFooter className="justify-between">
        <button
          type="button"
          className="text-[12px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
        >
          Run in background
        </button>
        <Button tone="destructive" onClick={onCancel} disabled={progress >= 1}>
          Cancel install…
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
  onBack,
  onCancel,
  onContinue,
}: {
  url: string;
  onBack: () => void;
  onCancel: () => void;
  onContinue: () => void;
}) {
  const [choice, setChoice] = useState<"wrap" | "workspace">("wrap");

  return (
    <>
      <ModalHeader
        eyebrow="Install — External project"
        title="How do you want to use it?"
        description="This repository doesn't declare itself as a Yggdrasil project. Pick how Yggdrasil should treat it."
      />

      <div className="flex items-center gap-3 rounded-[12px] border border-whisper-border px-4 py-3">
        <GithubLogo size={18} className="text-charcoal-ink" />
        <span className="flex-1 truncate font-mono text-[13px] text-charcoal-ink">{url}</span>
        <StatusPill tone="neutral" label="EXTERNAL" showDot={false} />
        <span className="font-mono text-[11px] text-muted-tone">Python · 12.4 MB</span>
      </div>

      <div className="mt-5 flex flex-col gap-3">
        <ExternalChoiceCard
          selected={choice === "wrap"}
          onSelect={() => setChoice("wrap")}
          title="Wrap with adapter"
          recommended
          description="Yggdrasil generates a wrapper package so this tool becomes a regular Yggdrasil project with capabilities, surfaces, and permissions. Best when you want it to feel like other projects on Home."
          chips={[
            { label: "Auto-detected entrypoint", tone: "good" },
            { label: "Subprocess sandbox", tone: "good" },
            { label: "Adapter may need editing", tone: "warn" },
          ]}
        />
        <ExternalChoiceCard
          selected={choice === "workspace"}
          onSelect={() => setChoice("workspace")}
          title="Open as workspace"
          description="Yggdrasil exposes the cloned repo as a managed workspace. You explore it with the agent — no wrapping, no Home card. Best for one-off use or unfamiliar projects."
          chips={[
            { label: "No code generation", tone: "good" },
            { label: "Agent assist available", tone: "good" },
            { label: "Won't appear on Home", tone: "warn" },
          ]}
        />
      </div>

      <button
        type="button"
        className="mt-5 inline-flex items-center gap-1.5 text-[12px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
      >
        <Question size={12} />
        What happens if I change my mind?
      </button>

      <ModalFooter className="justify-end">
        <Button tone="tertiary" onClick={onBack}>
          Back
        </Button>
        <Button tone="secondary" onClick={onCancel}>
          Cancel
        </Button>
        <Button tone="primary" onClick={onContinue}>
          Continue
          <ArrowRight size={14} />
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
