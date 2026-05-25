import { CaretDown, MagnifyingGlass } from "@/components/icons";
import { InputGroup } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/cn";
import { STATUS_DOT_CLASS, type StatusTone } from "@/components/ui/status-pill";

export interface FilterChip {
  id: string;
  label: string;
  count: number;
  toneDot?: StatusTone;
}

export interface UtilityStripProps {
  search: string;
  onSearchChange: (value: string) => void;
  filters: FilterChip[];
  activeFilter: string;
  onFilterChange: (id: string) => void;
  sortLabel?: string;
  sortPrefix?: string;
  searchPlaceholder?: string;
  onSortClick?: () => void;
}

export function UtilityStrip({
  search,
  onSearchChange,
  filters,
  activeFilter,
  onFilterChange,
  sortLabel = "Recent",
  sortPrefix = "Sort",
  searchPlaceholder = "Search projects, packages...",
  onSortClick,
}: UtilityStripProps) {
  return (
    <div className="flex flex-wrap items-center gap-3">
      <div className="min-w-0 flex-1 sm:flex-initial">
        <InputGroup
          leftIcon={<MagnifyingGlass size={16} />}
          rightSlot={
            <span className="hidden rounded-[4px] border border-whisper-border bg-pure-surface px-1.5 py-0.5 font-mono text-[10px] text-muted-tone sm:inline">
              ⌘K
            </span>
          }
          placeholder={searchPlaceholder}
          value={search}
          onChange={(event) => onSearchChange(event.target.value)}
          className="w-full sm:w-[260px] lg:w-[300px]"
        />
      </div>
      <div className="flex flex-wrap items-center gap-1.5">
        {filters.map((filter) => {
          const isActive = filter.id === activeFilter;
          return (
            <button
              key={filter.id}
              type="button"
              onClick={() => onFilterChange(filter.id)}
              className={cn(
                "inline-flex h-8 items-center gap-1.5 rounded-full border px-3 text-[12px] font-medium transition",
                isActive
                  ? "border-aged-brass-border bg-aged-brass-surface text-charcoal-ink"
                  : "border-whisper-border bg-transparent text-charcoal-ink hover:bg-whisper-border-strong/30",
              )}
            >
              {filter.toneDot ? (
                <span className={cn("size-1.5 rounded-full", STATUS_DOT_CLASS[filter.toneDot])} aria-hidden />
              ) : null}
              <span>{filter.label}</span>
              <span className="font-mono text-[10px] text-muted-tone">·{String(filter.count).padStart(2, "0")}</span>
            </button>
          );
        })}
      </div>
      <div className="ml-auto">
        <Button tone="secondary" size="sm" onClick={onSortClick}>
          {sortPrefix}: {sortLabel}
          <CaretDown size={12} className="text-muted-tone" />
        </Button>
      </div>
    </div>
  );
}
