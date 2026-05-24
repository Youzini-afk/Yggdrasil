import { useEffect, useMemo, useState } from "react";
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
import { Skeleton } from "@/components/ui/skeleton";
import { EmptyState } from "@/components/ui/empty-state";
import { useToast } from "@/components/ui/toast";
import { useAsync, useKernel } from "@/lib/kernel-client";
import { classifyPackageKind } from "@/lib/format";
import { cn } from "@/lib/cn";
import type { PackageRecord, ProjectRecord, SubprocessLogLine } from "@/protocol/client";

interface RowEntry {
  id: string;
  name: string;
  packageId: string;
  version: string;
  kind: "PROJECT" | "OFFICIAL" | "THIRD-PARTY";
  state: PackageRecord["state"];
  capabilityCount: number;
  hookCount: number;
  entryKind: string;
  isProject: boolean;
}

const FILTER_TABS = [
  { id: "all", label: "All" },
  { id: "PROJECT", label: "Projects" },
  { id: "OFFICIAL", label: "Official" },
  { id: "THIRD-PARTY", label: "Third-party" },
] as const;

type FilterId = (typeof FILTER_TABS)[number]["id"];

function pillTone(state: string): StatusTone {
  if (state === "running" || state === "ready") return "running";
  if (state === "failed" || state === "degraded") return "failed";
  if (state === "starting" || state === "loading" || state === "stopping") return "starting";
  return "stopped";
}

export function InstalledPackagesPanel() {
  const client = useKernel();
  const toast = useToast();
  const [filter, setFilter] = useState<FilterId>("all");
  const [search, setSearch] = useState("");

  const packages = useAsync(() => client.packages(), [client]);
  const projects = useAsync(
    () => client.listProjects().catch<ProjectRecord[]>(() => []),
    [client],
  );

  const rows = useMemo<RowEntry[]>(() => {
    const projectIds = new Set((projects.data ?? []).map((p) => p.id));
    return (packages.data ?? []).map((p) => {
      const isProject = projectIds.has(p.id);
      const kind: RowEntry["kind"] = isProject ? "PROJECT" : classifyPackageKind(p.id);
      return {
        id: p.id,
        name: deriveName(p.id),
        packageId: p.id,
        version: p.version,
        kind,
        state: p.state,
        capabilityCount: p.capability_count,
        hookCount: p.hook_count,
        entryKind: p.entry_kind,
        isProject,
      };
    });
  }, [packages.data, projects.data]);

  const filtered = useMemo(() => {
    return rows.filter((p) => {
      const matchKind = filter === "all" || p.kind === filter;
      const matchSearch =
        !search ||
        p.name.toLowerCase().includes(search.toLowerCase()) ||
        p.packageId.toLowerCase().includes(search.toLowerCase());
      return matchKind && matchSearch;
    });
  }, [rows, filter, search]);

  const counts = useMemo(() => {
    const acc: Record<string, number> = { PROJECT: 0, OFFICIAL: 0, "THIRD-PARTY": 0 };
    for (const p of rows) acc[p.kind] = (acc[p.kind] ?? 0) + 1;
    return acc;
  }, [rows]);

  // Cmd/Ctrl + F focuses the search input.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f") {
        const input = document.querySelector<HTMLInputElement>("[data-pkg-search]");
        if (input) {
          e.preventDefault();
          input.focus();
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const total = rows.length;

  const onViewLogs = async (packageId: string) => {
    try {
      const [status, logs] = await Promise.all([
        client.packageStatus(packageId).catch<PackageRecord | null>(() => null),
        client.packageLogs(packageId).catch<SubprocessLogLine[]>(() => []),
      ]);
      const failure = status?.last_failure;
      const lines = failure?.stderr_tail_redacted.length
        ? failure.stderr_tail_redacted
        : logs.map((log) => `[${log.stream}] ${log.line}`).slice(-20);
      toast.push({
        variant: lines.length > 0 ? "info" : "warning",
        title: lines.length > 0 ? `Redacted logs for ${packageId}` : "No logs available",
        body: lines.length > 0 ? lines.slice(-3).join("\n") : "The kernel did not return a bounded redacted log tail for this package.",
        duration: lines.length > 0 ? 0 : undefined,
        action: lines.length > 0 ? { label: "Copy", onClick: () => navigator.clipboard?.writeText(lines.join("\n")) } : undefined,
      });
    } catch (err) {
      toast.push({
        variant: "error",
        title: "Couldn't load logs",
        body: err instanceof Error ? err.message : String(err),
      });
    }
  };

  return (
    <>
      <header className="mb-8">
        <Eyebrow>
          {packages.loading
            ? "Installed packages · loading…"
            : `Installed packages · ${total} packages`}
        </Eyebrow>
        <PageTitle className="mt-2">Workshop inventory</PageTitle>
        <p className="mt-2 max-w-[60ch] text-[13px] leading-relaxed text-steel-secondary">
          Projects, official packages, and dependencies installed in this workshop. Refresh checks
          upstream sources.
        </p>
      </header>

      <div className="mb-5 flex flex-wrap items-center gap-3">
        <div className="w-[280px]">
          <InputGroup
            data-pkg-search
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
          {FILTER_TABS.map((tab) => {
            const count = tab.id === "all" ? total : counts[tab.id] ?? 0;
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
          <Button
            tone="secondary"
            size="sm"
            onClick={() => {
              packages.refresh();
              projects.refresh();
              toast.push({ variant: "info", title: "Refreshing inventory…", duration: 2400 });
            }}
            disabled={packages.loading}
          >
            <ArrowsClockwise size={14} />
            Refresh
          </Button>
        </div>
      </div>

      {packages.error ? (
        <EmptyState
          icon={<Package />}
          title="Couldn't load packages"
          body={packages.error.message}
          action={{ label: "Retry", onClick: () => packages.refresh() }}
        />
      ) : packages.loading ? (
        <Card>
          <ul className="divide-y divide-whisper-border">
            {Array.from({ length: 6 }).map((_, idx) => (
              <li key={idx} className="flex items-center gap-4 px-5 py-4">
                <Skeleton className="size-5 rounded-full" />
                <div className="flex-1 space-y-1.5">
                  <Skeleton className="h-3 w-44" />
                  <Skeleton className="h-2.5 w-72" />
                </div>
                <Skeleton className="h-3 w-12" />
                <Skeleton className="h-3 w-20" />
                <Skeleton className="h-5 w-20 rounded-full" />
              </li>
            ))}
          </ul>
        </Card>
      ) : filtered.length === 0 ? (
        <Card>
          <EmptyState
            icon={<Package />}
            title={total === 0 ? "No packages installed yet" : "No packages match this filter"}
            body={
              total === 0
                ? "Install a project from Home or run yg install on the CLI."
                : "Try a different filter or clear the search."
            }
          />
        </Card>
      ) : (
        <Card>
          <div
            className="grid items-center gap-4 border-b border-whisper-border bg-aged-brass-surface-soft px-5 py-2.5 font-mono text-[10px] uppercase tracking-[0.12em] text-steel-secondary"
            style={{ gridTemplateColumns: "32px 2.4fr 0.8fr 0.9fr 1fr 1fr 32px" }}
          >
            <span />
            <span>Package</span>
            <span>Version</span>
            <span>Kind</span>
            <span>Capabilities</span>
            <span>State</span>
            <span />
          </div>
          <ul className="divide-y divide-whisper-border">
            {filtered.map((entry) => (
              <li
                key={entry.id}
                className="grid items-center gap-4 px-5 py-3"
                style={{ gridTemplateColumns: "32px 2.4fr 0.8fr 0.9fr 1fr 1fr 32px" }}
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
                  <p className="truncate font-mono text-[11px] text-muted-tone">
                    {entry.packageId}
                  </p>
                </div>
                <span className="font-mono text-[12px] text-charcoal-ink">{entry.version}</span>
                <StatusPill
                  tone={entry.kind === "PROJECT" ? "accent" : "neutral"}
                  label={entry.kind}
                  showDot={false}
                />
                <span className="font-mono text-[11px] text-steel-secondary">
                  {entry.capabilityCount} cap · {entry.hookCount} hook · {entry.entryKind}
                </span>
                <StatusPill tone={pillTone(entry.state)} label={entry.state.toUpperCase()} />
                <Dropdown>
                  <DropdownTrigger asChild>
                    <Button tone="icon" size="icon-sm" aria-label="More">
                      <DotsThree size={16} />
                    </Button>
                  </DropdownTrigger>
                  <DropdownMenu>
                    <DropdownItem onSelect={() => copyId(entry.packageId, toast)}>
                      Copy package id
                    </DropdownItem>
                    <DropdownItem>View permissions</DropdownItem>
                    <DropdownItem onSelect={() => void onViewLogs(entry.packageId)}>View logs</DropdownItem>
                    <DropdownSeparator />
                    <DropdownItem destructive>Uninstall…</DropdownItem>
                  </DropdownMenu>
                </Dropdown>
              </li>
            ))}
          </ul>
          {filtered.length < total ? (
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
      )}
    </>
  );
}

function deriveName(packageId: string): string {
  if (packageId.includes("/")) return packageId.split("/").slice(-1)[0];
  if (packageId.includes("__")) return packageId.split("__").slice(-1)[0];
  return packageId;
}

function copyId(id: string, toast: ReturnType<typeof useToast>) {
  navigator.clipboard?.writeText(id);
  toast.push({ variant: "success", title: "Package id copied", duration: 2000 });
}
