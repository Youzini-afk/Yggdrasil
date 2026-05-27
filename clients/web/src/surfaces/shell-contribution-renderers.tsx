import { Card } from "@/components/ui/card";
import { cn } from "@/lib/cn";
import { resolveShellIcon } from "./shell-icons";
import type {
  HomeCardContribution,
  QuickActionContribution,
  WorkshopCardCategory,
  WorkshopCardContribution,
} from "./shell-contributions";

export interface QuickActionListProps {
  items: QuickActionContribution[];
  onActionClick?: (item: QuickActionContribution) => void;
  isActionDisabled?: (item: QuickActionContribution) => boolean;
  ariaLabel?: string;
  className?: string;
}

export function QuickActionList({
  items,
  onActionClick,
  isActionDisabled,
  ariaLabel,
  className,
}: QuickActionListProps) {
  if (items.length === 0) return null;

  return (
    <nav aria-label={ariaLabel} className={className}>
      <ul className="grid grid-cols-1 gap-1.5 sm:grid-cols-2">
        {items.map((item) => {
          const Icon = resolveShellIcon(item.iconHint);
          const disabled = isActionDisabled?.(item) ?? false;
          return (
            <li key={`${item.packageId}:${item.id}`}>
              <button
                type="button"
                onClick={() => onActionClick?.(item)}
                disabled={disabled}
                aria-label={item.description ? `${item.title}. ${item.description}` : item.title}
                className="flex w-full items-center gap-2 rounded-[8px] px-2 py-2 text-left text-[12px] text-charcoal-ink transition hover:bg-whisper-border-strong/30 disabled:cursor-not-allowed disabled:opacity-50"
              >
                <Icon size={14} className="shrink-0 text-steel-secondary" aria-hidden />
                <span className="min-w-0 flex-1">
                  <span className="block truncate font-medium">{item.title}</span>
                  {item.description ? <span className="block truncate text-[10px] text-muted-tone">{item.description}</span> : null}
                </span>
              </button>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}

export interface WorkshopCardListProps {
  items: WorkshopCardContribution[];
  onCardClick?: (item: WorkshopCardContribution) => void;
  isCardDisabled?: (item: WorkshopCardContribution) => boolean;
  ariaLabel?: string;
  categoryLabels?: Partial<Record<WorkshopCardCategory, string>>;
  className?: string;
}

const CATEGORY_LABELS: Record<WorkshopCardCategory, string> = {
  example: "Example",
  template: "Template",
  tool: "Tool",
};

export function WorkshopCardList({
  items,
  onCardClick,
  isCardDisabled,
  ariaLabel,
  categoryLabels,
  className,
}: WorkshopCardListProps) {
  if (items.length === 0) return null;

  return (
    <section aria-label={ariaLabel} className={className}>
      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
        {items.map((item) => {
          const Icon = resolveShellIcon(item.iconHint);
          const disabled = isCardDisabled?.(item) ?? false;
          return (
            <button
              key={`${item.packageId}:${item.id}`}
              type="button"
              onClick={() => onCardClick?.(item)}
              disabled={disabled}
              aria-label={item.description ? `${item.title}. ${item.description}` : item.title}
              className="group rounded-[20px] text-left outline-none focus-visible:ring-2 focus-visible:ring-aged-brass focus-visible:ring-offset-2 focus-visible:ring-offset-warm-bone disabled:cursor-not-allowed disabled:opacity-50"
            >
              <Card className="flex h-full flex-col gap-3 p-5 transition group-hover:border-aged-brass/50 group-hover:shadow-card-hover">
                <div className="flex items-start justify-between gap-3">
                  <span className="grid size-9 place-items-center rounded-[12px] bg-aged-brass/10 text-aged-brass">
                    <Icon size={18} aria-hidden />
                  </span>
                  {item.category ? (
                    <span className="rounded-full border border-whisper-border px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.16em] text-muted-tone">
                      {categoryLabels?.[item.category] ?? CATEGORY_LABELS[item.category]}
                    </span>
                  ) : null}
                </div>
                <div className="space-y-1">
                  <h3 className="text-[14px] font-semibold text-charcoal-ink">{item.title}</h3>
                  {item.description ? <p className="text-[12px] leading-5 text-muted-tone">{item.description}</p> : null}
                </div>
              </Card>
            </button>
          );
        })}
      </div>
    </section>
  );
}

export interface HomeCapabilityCardsProps {
  items: HomeCardContribution[];
  onCardClick?: (item: HomeCardContribution) => void;
  isCardDisabled?: (item: HomeCardContribution) => boolean;
  ariaLabel?: string;
  className?: string;
  maxItems?: number;
}

export function HomeCapabilityCards({
  items,
  onCardClick,
  isCardDisabled,
  ariaLabel,
  className,
  maxItems = 3,
}: HomeCapabilityCardsProps) {
  const visibleItems = items.slice(0, maxItems);
  if (visibleItems.length === 0) return null;

  return (
    <section aria-label={ariaLabel} className={cn("grid gap-3 md:grid-cols-3", className)}>
      {visibleItems.map((item) => {
        const Icon = resolveShellIcon(item.iconHint);
        const disabled = isCardDisabled?.(item) ?? false;
        return (
          <button
            key={`${item.packageId}:${item.id}`}
            type="button"
            onClick={() => onCardClick?.(item)}
            disabled={disabled}
            aria-label={item.description ? `${item.title}. ${item.description}` : item.title}
            className="group rounded-[20px] text-left outline-none focus-visible:ring-2 focus-visible:ring-aged-brass focus-visible:ring-offset-2 focus-visible:ring-offset-warm-bone disabled:cursor-not-allowed disabled:opacity-50"
          >
            <Card className="flex h-full flex-col gap-3 p-5 transition group-hover:border-aged-brass/50 group-hover:shadow-card-hover">
              <div className="flex items-center gap-3">
                <span className="grid size-9 place-items-center rounded-[12px] bg-whisper-border-strong/30 text-charcoal-ink">
                  <Icon size={18} aria-hidden />
                </span>
              </div>
              <div className="space-y-1">
                <h3 className="text-[14px] font-semibold text-charcoal-ink">{item.title}</h3>
                {item.description ? <p className="text-[12px] leading-5 text-muted-tone">{item.description}</p> : null}
              </div>
            </Card>
          </button>
        );
      })}
    </section>
  );
}
