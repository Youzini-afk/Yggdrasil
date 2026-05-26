import { type FormEvent } from "react";
import { ArrowRight, GithubLogo, Lightning, LinkSimple, XCircle } from "@/components/icons";
import { Button } from "@/components/ui/button";
import { Field, InputGroup } from "@/components/ui/input";
import { ModalFooter, ModalHeader } from "@/components/ui/modal";
import { EyebrowSm } from "@/components/ui/typography";
import { useT } from "@/lib/locale";
import type { ShortcutEntry } from "./install-types";

export function UrlStep({
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
  const t = useT();
  const onSubmit = (event: FormEvent) => {
    event.preventDefault();
    onContinue();
  };

  return (
    <form onSubmit={onSubmit}>
      <ModalHeader
        eyebrow={t("installUrlEyebrow")}
        title={t("installUrlTitle")}
        description={t("installUrlDescription")}
      />

      <Field label={t("installSourceLabel")} required>
        <InputGroup
          leftIcon={<LinkSimple size={16} />}
          value={url}
          onChange={(e) => onUrlChange(e.target.value)}
          placeholder="github.com/user/repo"
          spellCheck={false}
          disabled={loading}
          autoFocus
        />
      </Field>
      <p className="mt-1 text-[12px] text-steel-secondary">
        {t("installSourceHelper")}
      </p>

      <section className="mt-5 flex flex-col gap-2">
        <EyebrowSm>{t("installShortcuts")}</EyebrowSm>
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
              <p className="font-medium">{t("installResolveErrorTitle")}</p>
              <p className="mt-1 font-mono text-[11px] leading-snug">{error}</p>
            </div>
          </div>
        </div>
      ) : null}

      <ModalFooter className="justify-between">
        <p className="flex items-center gap-1.5 text-[11px] text-muted-tone">
          <Lightning size={11} className="text-muted-tone" />
          {t("installKeyboardHint")}
        </p>
        <div className="flex items-center gap-3">
          <Button type="button" tone="secondary" onClick={onCancel} disabled={loading}>
            {t("cancel")}
          </Button>
          <Button type="submit" tone="primary" disabled={!url.trim() || loading}>
            {loading ? t("installResolving") : t("continue")}
            {loading ? null : <ArrowRight size={14} />}
          </Button>
        </div>
      </ModalFooter>
    </form>
  );
}
