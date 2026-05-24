import { Folder, GearSix, Plus, Terminal } from "@/components/icons";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Eyebrow, EyebrowSm } from "@/components/ui/typography";
import { cn } from "@/lib/cn";
import { formatBytes } from "@/lib/format";

export interface UpdateEntry {
  id: string;
  packageId: string;
  fromVersion: string;
  toVersion: string;
  onUpdate: () => void;
}

export interface DiskSegment {
  id: string;
  label: string;
  bytes: number | null;
  measurementState?: string;
  toneClass: string;
}

export interface QuickAction {
  id: string;
  label: string;
  shortcut: string;
  icon: typeof Plus;
  onClick: () => void;
}

export interface WorkshopUtilitiesProps {
  updates: UpdateEntry[];
  totalDisk: number; // bytes
  diskSegments: DiskSegment[];
  diskCapacity: number; // bytes
  quickActions: QuickAction[];
  onUpdateAll?: () => void;
  onManageStorage?: () => void;
}

function isMeasured(segment: DiskSegment): segment is DiskSegment & { bytes: number } {
  return typeof segment.bytes === "number" && Number.isFinite(segment.bytes);
}

function formatSegmentBytes(segment: DiskSegment): string {
  if (isMeasured(segment)) return formatBytes(segment.bytes);
  return segment.measurementState === "measuring" ? "Measuring" : "Unknown";
}

export function WorkshopUtilities({
  updates,
  totalDisk,
  diskSegments,
  diskCapacity,
  quickActions,
  onUpdateAll,
  onManageStorage,
}: WorkshopUtilitiesProps) {
  const measuredSegments = diskSegments.filter(isMeasured);
  const hasMeasuredStorage = measuredSegments.length > 0;
  const hasPositiveStorage = totalDisk > 0 && measuredSegments.some((segment) => segment.bytes > 0);

  return (
    <section className="flex flex-col gap-3">
      <Eyebrow>Workshop</Eyebrow>
      <Card className="divide-y divide-whisper-border">
        {/* Updates */}
        <div className="flex flex-col gap-3 p-5">
          <div className="flex items-center justify-between">
            <EyebrowSm>Updates</EyebrowSm>
            <span className="font-mono text-[10px] text-aged-brass">
              {updates.length} available
            </span>
          </div>
          {updates.length === 0 ? (
            <p className="text-[12px] text-muted-tone">Everything is up to date.</p>
          ) : (
            <ul className="space-y-2">
              {updates.map((update) => (
                <li key={update.id} className="flex items-center justify-between gap-2">
                  <div className="min-w-0 flex-1">
                    <p className="truncate font-mono text-[12px] text-charcoal-ink">{update.packageId}</p>
                    <p className="font-mono text-[10px] text-steel-secondary">
                      {update.fromVersion} → {update.toVersion}
                    </p>
                  </div>
                  <Button tone="secondary" size="sm" onClick={update.onUpdate} className="h-7 px-2 text-[11px]">
                    Update
                  </Button>
                </li>
              ))}
            </ul>
          )}
          {updates.length > 0 ? (
            <button
              type="button"
              onClick={onUpdateAll}
              className="self-start text-[12px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
            >
              Update all →
            </button>
          ) : null}
        </div>

        {/* Disk */}
        <div className="flex flex-col gap-2 p-5">
          <div className="flex items-center justify-between">
            <EyebrowSm>Disk usage</EyebrowSm>
            <span className="font-mono text-[10px] text-charcoal-ink">
              {hasMeasuredStorage ? `${formatBytes(totalDisk)} used` : "Unknown"}
            </span>
          </div>
          <div className="flex h-1.5 w-full overflow-hidden rounded-full bg-whisper-border-strong/40">
            {hasPositiveStorage
              ? measuredSegments
                  .filter((segment) => segment.bytes > 0)
                  .map((segment) => (
                    <span
                      key={segment.id}
                      className={cn("h-full", segment.toneClass)}
                      style={{ width: `${Math.min(100, (segment.bytes / Math.max(diskCapacity, 1)) * 100)}%` }}
                      title={`${segment.label} · ${formatBytes(segment.bytes)}`}
                    />
                  ))
              : null}
          </div>
          {diskSegments.length === 0 ? (
            <p className="text-[12px] text-muted-tone">No project storage measured.</p>
          ) : hasMeasuredStorage ? (
            <ul className="flex flex-wrap gap-x-3 gap-y-1 text-[10px] text-muted-tone">
              {diskSegments.map((segment) => (
                <li key={segment.id} className="flex items-center gap-1">
                  <span className={cn("size-1.5 rounded-full", segment.toneClass)} aria-hidden />
                  {segment.label} · {formatSegmentBytes(segment)}
                </li>
              ))}
            </ul>
          ) : (
            <p className="text-[12px] text-muted-tone">No project storage measured.</p>
          )}
          <button
            type="button"
            onClick={onManageStorage}
            className="self-start text-[12px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
          >
            Manage storage →
          </button>
        </div>

        {/* Quick actions */}
        <div className="flex flex-col gap-2 p-5">
          <EyebrowSm>Quick actions</EyebrowSm>
          <ul className="grid grid-cols-2 gap-1.5">
            {quickActions.map((action) => {
              const Icon = action.icon;
              return (
                <li key={action.id}>
                  <button
                    type="button"
                    onClick={action.onClick}
                    className="flex w-full items-center gap-2 rounded-[8px] px-2 py-2 text-left text-[12px] text-charcoal-ink transition hover:bg-whisper-border-strong/30"
                  >
                    <Icon size={14} className="text-steel-secondary" />
                    <span className="flex-1 font-medium">{action.label}</span>
                    <span className="font-mono text-[10px] text-muted-tone">{action.shortcut}</span>
                  </button>
                </li>
              );
            })}
          </ul>
        </div>
      </Card>
    </section>
  );
}

export const QUICK_ACTION_ICONS = { Plus, Folder, GearSix, Terminal };
