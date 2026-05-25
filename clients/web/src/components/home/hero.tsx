import { Eyebrow, HeroTitle } from "@/components/ui/typography";
import { ContinueCard, type ContinueCardLabels, type ContinueCardEntry } from "@/components/home/continue-card";

export interface HeroProps {
  greeting: string;
  summary: string;
  meta: string;
  continueEntry: ContinueCardEntry | null;
  continueLabels: ContinueCardLabels;
  hasInstalledProjects: boolean;
  onContinue: (projectId: string) => void;
  onInstall: () => void;
  onBrowseProjects?: () => void;
}

export function Hero({
  greeting,
  summary,
  meta,
  continueEntry,
  continueLabels,
  hasInstalledProjects,
  onContinue,
  onInstall,
  onBrowseProjects,
}: HeroProps) {
  return (
    <section className="grid grid-cols-1 gap-8 lg:grid-cols-[1fr_auto] lg:gap-12">
      <div className="flex flex-col gap-3">
        <Eyebrow>{meta}</Eyebrow>
        <HeroTitle>{greeting}</HeroTitle>
        <p className="max-w-[80ch] text-[15px] leading-relaxed text-steel-secondary">{summary}</p>
      </div>
      <div className="lg:flex lg:justify-end">
        <ContinueCard
          entry={continueEntry}
          labels={continueLabels}
          onContinue={onContinue}
          onInstall={onInstall}
          onBrowseProjects={onBrowseProjects}
          hasInstalledProjects={hasInstalledProjects}
        />
      </div>
    </section>
  );
}
