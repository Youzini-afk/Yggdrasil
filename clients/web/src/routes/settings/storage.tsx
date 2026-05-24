import { Folder } from "@/components/icons";
import { Card, CardSection } from "@/components/ui/card";
import { Eyebrow, EyebrowSm, PageTitle } from "@/components/ui/typography";
import { Skeleton } from "@/components/ui/skeleton";
import { useAsync, useKernel } from "@/lib/kernel-client";

interface StorageArea {
  label: string;
  description: string;
}

const STORAGE_AREAS: StorageArea[] = [
  { label: "Project data", description: "Project metadata, checkpoints, package state, and run records." },
  { label: "Package store", description: "Installed package sources and lockfile-managed revisions." },
  { label: "Profiles", description: "Host profiles passed to yg host serve --profile." },
  { label: "Secrets", description: "Encrypted platform and project secret stores. Raw values are never shown here." },
  { label: "Cache", description: "Generated bundles, tokenizer caches, and other rebuildable data." },
];

export function StoragePanel() {
  const client = useKernel();
  const diagnostics = useAsync(() => client.diagnostics().catch(() => null), [client]);

  const eventStoreKind =
    (diagnostics.data as { event_store?: { kind?: string } } | null)?.event_store?.kind ?? "sqlite";

  return (
    <>
      <header className="mb-8">
        <Eyebrow>Storage</Eyebrow>
        <PageTitle className="mt-2">Where your data lives</PageTitle>
        <p className="mt-2 max-w-[60ch] text-[13px] leading-relaxed text-steel-secondary">
          Yggdrasil keeps data on this machine by default. The UI summarizes storage areas without
          exposing host-specific absolute paths.
        </p>
      </header>

      <Card>
        <CardSection>
          <EyebrowSm>Storage areas</EyebrowSm>
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
              {STORAGE_AREAS.map((entry) => (
                <li
                  key={entry.label}
                  className="flex items-start justify-between gap-4 py-2.5 text-[13px]"
                >
                  <span className="text-steel-secondary">{entry.label}</span>
                  <span className="flex min-w-0 max-w-[42ch] items-start gap-2 text-right text-charcoal-ink">
                    <Folder size={12} className="shrink-0 text-steel-secondary" />
                    <span>{entry.description}</span>
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
