import { useState } from "react";
import { CheckCircle, GithubLogo, Info, Question } from "@/components/icons";
import { Button } from "@/components/ui/button";
import { ModalFooter, ModalHeader } from "@/components/ui/modal";
import { StatusPill } from "@/components/ui/status-pill";
import { cn } from "@/lib/cn";
import { useT } from "@/lib/locale";
import type { InstallPlan } from "@/protocol/client";

export function ExternalWizardStep({
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
  const t = useT();
  const [choice, setChoice] = useState<"wrap" | "workspace">("wrap");

  return (
    <>
      <ModalHeader
        eyebrow={t("installExternalEyebrow")}
        title={t("installExternalTitle")}
        description={t("installExternalDescription")}
      />

      <div className="flex items-center gap-3 rounded-[12px] border border-whisper-border px-4 py-3">
        <GithubLogo size={18} className="text-charcoal-ink" />
        <span className="flex-1 truncate font-mono text-[13px] text-charcoal-ink">{url}</span>
        <StatusPill tone="neutral" label={t("installExternalStatus")} showDot={false} />
      </div>

      <div className="mt-5 rounded-[12px] border border-aged-brass-border bg-aged-brass-surface-soft px-4 py-3 text-[12px] text-charcoal-ink">
        <div className="flex items-start gap-2">
          <Info size={15} className="mt-0.5 shrink-0 text-aged-brass-deep" />
          <p className="leading-snug">{t("installExternalInfo")}</p>
        </div>
      </div>

      <div className="mt-4 rounded-[12px] border border-whisper-border bg-pure-surface px-4 py-3 text-[12px]">
        <p className="font-medium text-charcoal-ink">
          {plan ? t("installExternalPackagesResolved", plan.packages.length) : t("installExternalPlanUnavailable")}
        </p>
        {plan ? (
          <p className="mt-1 font-mono text-[11px] text-steel-secondary">{t("installRootPrefix")} {plan.root_id}</p>
        ) : planError ? (
          <p className="mt-1 font-mono text-[11px] leading-snug text-deep-rust">{planError}</p>
        ) : null}
      </div>

      <div className="mt-5 flex flex-col gap-3">
        <ExternalChoiceCard
          selected={choice === "wrap"}
          onSelect={() => setChoice("wrap")}
          title={t("installExternalChoiceWrapTitle")}
          recommended
          recommendedLabel={t("installRecommended")}
          description={t("installExternalChoiceWrapDescription")}
          chips={[
            { label: t("installExternalChipCliOnlyGeneration"), tone: "warn" },
            { label: t("installExternalChipNoWebExecution"), tone: "warn" },
          ]}
        />
        <ExternalChoiceCard
          selected={choice === "workspace"}
          onSelect={() => setChoice("workspace")}
          title={t("installExternalChoiceWorkspaceTitle")}
          description={t("installExternalChoiceWorkspaceDescription")}
          chips={[
            { label: t("installExternalChipCliOnlyDescriptor"), tone: "warn" },
            { label: t("installExternalChipInstallBlocked"), tone: "warn" },
          ]}
        />
      </div>

      <button
        type="button"
        className="mt-5 inline-flex items-center gap-1.5 text-[12px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
      >
        <Question size={12} />
        {t("installExternalHelp")}
      </button>

      <ModalFooter className="justify-end">
        <Button tone="tertiary" onClick={onBack}>
          {t("back")}
        </Button>
        <Button tone="secondary" onClick={onCancel}>
          {t("cancel")}
        </Button>
        <Button tone="primary" onClick={onContinue} disabled>
          {t("installContinueDisabled")}
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
  recommendedLabel,
}: {
  selected: boolean;
  onSelect: () => void;
  title: string;
  description: string;
  chips: Array<{ label: string; tone: "good" | "warn" }>;
  recommended?: boolean;
  recommendedLabel?: string;
}) {
  return (
    <button
      type="button"
      onClick={onSelect}
      className={cn(
        "group flex w-full items-start gap-4 rounded-[16px] border bg-pure-surface p-5 text-left transition",
        selected
          ? "border-l-4 border-l-aged-brass border-aged-brass-border bg-aged-brass-surface-soft"
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
          {recommended && recommendedLabel ? <StatusPill tone="accent" label={recommendedLabel} showDot={false} /> : null}
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
