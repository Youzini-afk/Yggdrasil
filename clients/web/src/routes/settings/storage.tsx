import { useMemo } from "react";
import { Folder } from "@/components/icons";
import { Card, CardSection } from "@/components/ui/card";
import { Eyebrow, EyebrowSm, PageTitle } from "@/components/ui/typography";
import { Skeleton } from "@/components/ui/skeleton";
import { useAsync, useKernel } from "@/lib/kernel-client";

interface PathEntry {
  label: string;
  path: string;
}

const FALLBACK_PATHS: PathEntry[] = [
  { label: "Root", path: "~/.yggdrasil" },
  { label: "Package store", path: "~/.yggdrasil/store" },
  { label: "Profiles", path: "~/.yggdrasil/profiles" },
  { label: "Trusted keys", path: "~/.yggdrasil/keys" },
  { label: "Cache", path: "~/.yggdrasil/cache" },
  { label: "Project secrets", path: "~/.yggdrasil/projects/<id>/secrets.dat" },
];

export function StoragePanel() {
  const client = useKernel();
  const diagnostics = useAsync(() => client.diagnostics().catch(() => null), [client]);

  const paths = useMemo<PathEntry[]>(() => {
    const d = diagnostics.data as {
      data_dir?: string;
      paths?: Record<string, string>;
    } | null;
    if (!d) return FALLBACK_PATHS;

    const root = d.data_dir ?? "~/.yggdrasil";
    const join = (sub: string) => (d.paths?.[sub] ?? `${root}/${sub}`);
    return [
      { label: "Root", path: root },
      { label: "Package store", path: join("store") },
      { label: "Profiles", path: join("profiles") },
      { label: "Trusted keys", path: join("keys") },
      { label: "Cache", path: join("cache") },
      { label: "Projects", path: join("projects") },
    ];
  }, [diagnostics.data]);

  const eventStoreKind =
    (diagnostics.data as { event_store?: { kind?: string } } | null)?.event_store?.kind ?? "sqlite";

  return (
    <>
      <header className="mb-8">
        <Eyebrow>Storage</Eyebrow>
        <PageTitle className="mt-2">Where your data lives</PageTitle>
        <p className="mt-2 max-w-[60ch] text-[13px] leading-relaxed text-steel-secondary">
          Yggdrasil keeps everything on this machine by default. Open these paths in your file
          manager to inspect, back up, or relocate.
        </p>
      </header>

      <Card>
        <CardSection>
          <EyebrowSm>Paths</EyebrowSm>
          {diagnostics.loading ? (
            <ul className="mt-3 space-y-2">
              {Array.from({ length: 6 }).map((_, idx) => (
                <li key={idx} className="flex items-center justify-between gap-4 py-2">
                  <Skeleton className="h-3 w-20" />
                  <Skeleton className="h-3 w-72" />
                </li>
              ))}
            </ul>
          ) : (
            <ul className="mt-3 divide-y divide-whisper-border">
              {paths.map((entry) => (
                <li
                  key={entry.label}
                  className="flex items-center justify-between gap-4 py-2.5 text-[13px]"
                >
                  <span className="text-steel-secondary">{entry.label}</span>
                  <span className="flex min-w-0 items-center gap-2 truncate font-mono text-charcoal-ink">
                    <Folder size={12} className="shrink-0 text-steel-secondary" />
                    <span className="truncate">{entry.path}</span>
                  </span>
                </li>
              ))}
            </ul>
          )}
        </CardSection>
        <CardSection divided>
          <EyebrowSm>Event store</EyebrowSm>
          <p className="mt-2 text-[13px] text-charcoal-ink">
            <span className="font-mono">{eventStoreKind}</span>
            <span className="mx-2 text-muted-tone">·</span>
            <span className="text-steel-secondary">
              {eventStoreKind === "sqlite"
                ? "Local file backend, default for single-host workshops."
                : eventStoreKind === "postgres"
                  ? "PostgreSQL backend, suitable for shared/team hosts."
                  : eventStoreKind === "memory"
                    ? "In-memory backend, no persistence between restarts."
                    : "Custom backend."}
            </span>
          </p>
        </CardSection>
        <CardSection divided>
          <EyebrowSm>Backend neutrality</EyebrowSm>
          <p className="mt-2 text-[12px] leading-relaxed text-steel-secondary">
            Yggdrasil's storage layer is backend-neutral. SQLite is the default for local single-host
            workshops. PostgreSQL is reserved for shared/team hosts. Multimodal retrieval providers
            (TDB, pgvector, others) are exposed as ordinary capability packages, never as kernel
            primitives.
          </p>
        </CardSection>
      </Card>
    </>
  );
}
