import { useMemo, useState } from "react";
import { ArrowsClockwise, DotsThree, MagnifyingGlass, Package } from "@/components/icons";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Eyebrow, PageTitle } from "@/components/ui/typography";
import { StatusPill, type StatusTone } from "@/components/ui/status-pill";
import { InputGroup } from "@/components/ui/input";
import {
  Dropdown,
  DropdownTrigger,
  DropdownMenu,
  DropdownItem,
  DropdownSeparator,
} from "@/components/ui/dropdown";
import { useToast } from "@/components/ui/toast";
import { cn } from "@/lib/cn";

interface PackageEntry {
  id: string;
  name: string;
  packageId: string;
  version: string;
  kind: "PROJECT" | "OFFICIAL" | "THIRD-PARTY";
  sizeMB: number;
  state: "running" | "installed" | "stopped" | "failed" | "starting";
  updateAvailable?: string;
  updatedAge: string;
}

const MOCK_PACKAGES: PackageEntry[] = [
  {
    id: "p1",
    name: "YdlTavern",
    packageId: "youzini-afk__YdlTavern__2a47e5c",
    version: "v0.1.0",
    kind: "PROJECT",
    sizeMB: 84.3,
    state: "running",
    updatedAge: "2h ago",
  },
  {
    id: "p2",
    name: "Coding Workshop",
    packageId: "local__coding-workshop",
    version: "v0.2.4",
    kind: "PROJECT",
    sizeMB: 142.1,
    state: "stopped",
    updatedAge: "yesterday",
  },
  {
    id: "p3",
    name: "Image Studio",
    packageId: "github__example__image-studio__c2e84f1",
    version: "v0.0.3",
    kind: "PROJECT",
    sizeMB: 47.2,
    state: "failed",
    updatedAge: "5h ago",
  },
  {
    id: "p4",
    name: "ydltavern-engine",
    packageId: "official__ydltavern-engine",
    version: "v0.1.0",
    kind: "OFFICIAL",
    sizeMB: 8.4,
    state: "installed",
    updatedAge: "2d ago",
  },
  {
    id: "p5",
    name: "ydltavern-surface",
    packageId: "official__ydltavern-surface",
    version: "v0.1.0",
    kind: "OFFICIAL",
    sizeMB: 1.2,
    state: "installed",
    updatedAge: "2d ago",
  },
  {
    id: "p6",
    name: "model-provider-lab",
    packageId: "official__model-provider-lab",
    version: "v0.4.1",
    kind: "OFFICIAL",
    sizeMB: 0.8,
    state: "installed",
    updateAvailable: "v0.5.0",
    updatedAge: "1w ago",
  },
  {
    id: "p7",
    name: "secret-store-lab",
    packageId: "official__secret-store-lab",
    version: "v0.3.0",
    kind: "OFFICIAL",
    sizeMB: 0.4,
    state: "installed",
    updatedAge: "3d ago",
  },
  {
    id: "p8",
    name: "image-tools-lab",
    packageId: "github__example__image-tools-lab",
    version: "v0.0.3",
    kind: "THIRD-PARTY",
    sizeMB: 12.6,
    state: "installed",
    updatedAge: "5h ago",
  },
];

const filterTabs = [
  { id: "all", label: "All" },
  { id: "PROJECT", label: "Projects" },
  { id: "OFFICIAL", label: "Official" },
  { id: "THIRD-PARTY", label: "Third-party" },
] as const;

type FilterId = (typeof filterTabs)[number]["id"];

function pillTone(state: PackageEntry["state"]): StatusTone {
  if (state === "running") return "running";
  if (state === "failed") return "failed";
  if (state === "starting") return "starting";
  return "stopped";
}

export function InstalledPackagesPanel() {
  const toast = useToast();
  const [filter, setFilter] = useState<FilterId>("all");
  const [search, setSearch] = useState("");

  const total = MOCK_PACKAGES.length;
  const totalSize = MOCK_PACKAGES.reduce((sum, p) => sum + p.sizeMB, 0);

  const filtered = useMemo(() => {
    return MOCK_PACKAGES.filter((p) => {
      const matchKind = filter === "all" || p.kind === filter;
      const matchSearch =
        !search ||
        p.name.toLowerCase().includes(search.toLowerCase()) ||
        p.packageId.toLowerCase().includes(search.toLowerCase());
      return matchKind && matchSearch;
    });
  }, [filter, search]);

  const counts = MOCK_PACKAGES.reduce<Record<string, number>>((acc, p) => {
    acc[p.kind] = (acc[p.kind] ?? 0) + 1;
    return acc;
  }, {});

  return (
    <>
      <header className="mb-8">
        <Eyebrow>
          Installed packages · {total} packages · {totalSize.toFixed(1)} MB
        </Eyebrow>
        <PageTitle className="mt-2">Workshop inventory</PageTitle>
        <p className="mt-2 max-w-[60ch] text-[13px] leading-relaxed text-steel-secondary">
          Projects, official packages, and dependencies installed in this workshop. Updates check
          upstream when you click Refresh.
        </p>
      </header>

      <div className="mb-5 flex flex-wrap items-center gap-3">
        <div className="w-[280px]">
          <InputGroup
            leftIcon={<MagnifyingGlass size={16} />}
            placeholder="Filter packages…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            rightSlot={
              <span className="rounded-[4px] border border-whisper-border bg-warm-bone px-1.5 py-0.5 font-mono text-[10px] text-muted-tone">
                ⌘F
              </span>
            }
          />
        </div>
        <div className="flex flex-wrap items-center gap-1.5">
          {filterTabs.map((tab) => {
            const count =
              tab.id === "all" ? total : counts[tab.id] ?? 0;
            const isActive = filter === tab.id;
            return (
              <button
                key={tab.id}
                type="button"
                onClick={() => setFilter(tab.id)}
                className={cn(
                  "inline-flex h-8 items-center gap-1.5 rounded-full border px-3 text-[12px] font-medium transition",
                  isActive
                    ? "border-aged-brass-border bg-aged-brass-surface text-charcoal-ink"
                    : "border-whisper-border text-charcoal-ink hover:bg-whisper-border-strong/30",
                )}
              >
                {tab.label}
                <span className="font-mono text-[10px] text-muted-tone">
                  ·{String(count).padStart(2, "0")}
                </span>
              </button>
            );
          })}
        </div>
        <div className="ml-auto flex items-center gap-2">
          <Button tone="secondary" size="sm" onClick={() => toast.push({ variant: "info", title: "Checking upstream…" })}>
            <ArrowsClockwise size={14} />
            Refresh
          </Button>
          <Button tone="secondary" size="sm">
            Sort: Name <span aria-hidden>▾</span>
          </Button>
        </div>
      </div>

      <Card>
        {/* Header row */}
        <div
          className="grid items-center gap-4 border-b border-whisper-border bg-aged-brass-surface-soft px-5 py-2.5 font-mono text-[10px] uppercase tracking-[0.12em] text-steel-secondary"
          style={{ gridTemplateColumns: "32px 2.5fr 0.8fr 0.9fr 0.7fr 1.1fr 0.7fr 32px" }}
        >
          <span />
          <span>Package</span>
          <span>Version</span>
          <span>Kind</span>
          <span className="text-right">Size</span>
          <span>State</span>
          <span>Updated</span>
          <span />
        </div>
        <ul className="divide-y divide-whisper-border">
          {filtered.length === 0 ? (
            <li className="px-5 py-12 text-center text-[13px] text-muted-tone">
              No packages match this filter.
            </li>
          ) : (
            filtered.map((entry) => (
              <li
                key={entry.id}
                className="grid items-center gap-4 px-5 py-3"
                style={{ gridTemplateColumns: "32px 2.5fr 0.8fr 0.9fr 0.7fr 1.1fr 0.7fr 32px" }}
              >
                <Package
                  size={18}
                  className={
                    entry.kind === "PROJECT"
                      ? "text-aged-brass"
                      : entry.kind === "OFFICIAL"
                        ? "text-steel-secondary"
                        : "text-muted-tone"
                  }
                />
                <div className="min-w-0">
                  <p className="truncate font-display text-[13px] font-bold text-charcoal-ink">
                    {entry.name}
                  </p>
                  <p className="truncate font-mono text-[11px] text-muted-tone">{entry.packageId}</p>
                </div>
                <span className="font-mono text-[12px] text-charcoal-ink">{entry.version}</span>
                <StatusPill
                  tone={
                    entry.kind === "PROJECT" ? "accent" : entry.kind === "OFFICIAL" ? "neutral" : "neutral"
                  }
                  label={entry.kind}
                  showDot={false}
                />
                <span className="text-right font-mono text-[12px] text-charcoal-ink">
                  {entry.sizeMB.toFixed(1)} MB
                </span>
                {entry.updateAvailable ? (
                  <StatusPill tone="update" label={`UPDATE ${entry.updateAvailable}`} />
                ) : (
                  <StatusPill tone={pillTone(entry.state)} label={entry.state.toUpperCase()} />
                )}
                <span className="font-mono text-[11px] text-muted-tone">{entry.updatedAge}</span>
                <Dropdown>
                  <DropdownTrigger asChild>
                    <Button tone="icon" size="icon-sm" aria-label="More">
                      <DotsThree size={16} />
                    </Button>
                  </DropdownTrigger>
                  <DropdownMenu>
                    {entry.updateAvailable ? <DropdownItem>Update…</DropdownItem> : null}
                    <DropdownItem>View permissions</DropdownItem>
                    <DropdownItem>View logs</DropdownItem>
                    <DropdownSeparator />
                    <DropdownItem destructive>Uninstall…</DropdownItem>
                  </DropdownMenu>
                </Dropdown>
              </li>
            ))
          )}
        </ul>
        {filtered.length > 0 && filtered.length < total ? (
          <div className="flex items-center justify-between border-t border-whisper-border px-5 py-3">
            <span className="font-mono text-[11px] text-muted-tone">
              Showing {filtered.length} of {total}
            </span>
            <button
              type="button"
              onClick={() => {
                setFilter("all");
                setSearch("");
              }}
              className="text-[12px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
            >
              Show all →
            </button>
          </div>
        ) : null}
      </Card>
    </>
  );
}
