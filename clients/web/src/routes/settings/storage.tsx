import { Folder } from "@/components/icons";
import { Card, CardSection } from "@/components/ui/card";
import { Eyebrow, EyebrowSm, PageTitle } from "@/components/ui/typography";

export function StoragePanel() {
  return (
    <>
      <header className="mb-8">
        <Eyebrow>Storage</Eyebrow>
        <PageTitle className="mt-2">Where your data lives</PageTitle>
        <p className="mt-2 max-w-[60ch] text-[13px] leading-relaxed text-steel-secondary">
          Yggdrasil keeps everything on this machine. Open the path in Finder/Files to inspect, back
          up, or relocate.
        </p>
      </header>

      <Card>
        <CardSection>
          <EyebrowSm>Paths</EyebrowSm>
          <ul className="mt-3 divide-y divide-whisper-border">
            {[
              ["Root", "~/.yggdrasil"],
              ["Package store", "~/.yggdrasil/store"],
              ["Profiles", "~/.yggdrasil/profiles"],
              ["Trusted keys", "~/.yggdrasil/keys"],
              ["Cache", "~/.yggdrasil/cache"],
              ["Project secrets", "~/.yggdrasil/projects/<id>/secrets.dat"],
            ].map(([label, value]) => (
              <li key={label} className="flex items-center justify-between gap-4 py-2.5 text-[13px]">
                <span className="text-steel-secondary">{label}</span>
                <span className="flex items-center gap-2 font-mono text-charcoal-ink">
                  <Folder size={12} className="text-steel-secondary" />
                  {value}
                </span>
              </li>
            ))}
          </ul>
        </CardSection>
        <CardSection divided>
          <EyebrowSm>Backend</EyebrowSm>
          <p className="mt-2 text-[12px] leading-relaxed text-steel-secondary">
            Yggdrasil's storage layer is backend-neutral. SQLite is the default for local single-host
            workshops. PostgreSQL is reserved for shared/team hosts. Future multimodal retrieval
            providers (TDB, pgvector) are exposed as ordinary capability packages.
          </p>
        </CardSection>
      </Card>
    </>
  );
}
