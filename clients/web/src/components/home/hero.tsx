import { Eyebrow, HeroTitle } from "@/components/ui/typography";
import { ActivityMicroCard, type ActivityRow } from "@/components/home/activity-micro-card";

export interface HeroProps {
  greeting: string;
  summary: string;
  meta: string;
  recentActivity: ActivityRow[];
}

export function Hero({ greeting, summary, meta, recentActivity }: HeroProps) {
  return (
    <section className="grid grid-cols-1 gap-8 lg:grid-cols-[1fr_auto] lg:gap-12">
      <div className="flex flex-col gap-3">
        <Eyebrow>{meta}</Eyebrow>
        <HeroTitle>{greeting}</HeroTitle>
        <p className="max-w-[80ch] text-[15px] leading-relaxed text-steel-secondary">{summary}</p>
      </div>
      <div className="lg:flex lg:justify-end">
        <ActivityMicroCard rows={recentActivity} />
      </div>
    </section>
  );
}
