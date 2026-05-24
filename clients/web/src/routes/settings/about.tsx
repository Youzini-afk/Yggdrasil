import { Bug, ChatCircle, GithubLogo, BookOpen, Newspaper } from "@/components/icons";
import { Card, CardSection } from "@/components/ui/card";
import { Eyebrow, EyebrowSm, PageTitle } from "@/components/ui/typography";

export function AboutPanel() {
  return (
    <>
      <header className="mb-8">
        <Eyebrow>About</Eyebrow>
        <PageTitle className="mt-3">Yggdrasil</PageTitle>
        <p className="mt-3 text-[15px] text-steel-secondary">Open platform for play and creation.</p>
      </header>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[2fr_1fr]">
        <div className="flex flex-col gap-4">
          <Card>
            <CardSection>
              <dl className="grid grid-cols-2 gap-x-6 gap-y-3 sm:grid-cols-4">
                {[
                  ["Version", "v0.1.0-alpha"],
                  ["Build", "2a47e5c"],
                  ["Released", "2026-05-14"],
                  ["Channel", "alpha"],
                ].map(([label, value]) => (
                  <div key={label}>
                    <dt className="font-mono text-[10px] uppercase tracking-[0.12em] text-muted-tone">
                      {label}
                    </dt>
                    <dd className="mt-1 font-mono text-[14px] text-charcoal-ink">{value}</dd>
                  </div>
                ))}
              </dl>
            </CardSection>
          </Card>

          <Card>
            <CardSection>
              <EyebrowSm>What Yggdrasil is</EyebrowSm>
              <div className="mt-3 max-w-[60ch] space-y-3 text-[14px] leading-relaxed text-charcoal-ink">
                <p>
                  Yggdrasil is a kernel and a contract. The kernel hosts your projects in sandboxes.
                  The contract lets any project — official, community, or self-built — participate
                  as a first-class citizen.
                </p>
                <p>
                  It runs on your machine, with your keys, your files, your network. There is no
                  SaaS account, no central registry, no telemetry. Projects you install live in the
                  local platform data directory and stay there until you remove them.
                </p>
                <p className="text-steel-secondary">
                  The shell you are looking at right now is one of many possible UIs. Anyone can
                  write another. The platform is the contract — not this window.
                </p>
              </div>
            </CardSection>
          </Card>

          <Card>
            <CardSection>
              <EyebrowSm>Credits</EyebrowSm>
              <ul className="mt-3 divide-y divide-whisper-border text-[12px]">
                {[
                  ["Built on", "Rust 1.84 · TypeScript 5.7 · Tauri 2.x", "mono"],
                  ["Fonts", "Bricolage Grotesque · Geist · JetBrains Mono"],
                  ["Icons", "Phosphor Icons (1.4 · MIT)"],
                ].map(([label, value, font]) => (
                  <li key={label} className="flex items-center justify-between gap-4 py-2.5">
                    <span className="text-steel-secondary">{label}</span>
                    <span
                      className={
                        font === "mono"
                          ? "font-mono text-charcoal-ink"
                          : "text-charcoal-ink"
                      }
                    >
                      {value}
                    </span>
                  </li>
                ))}
              </ul>
            </CardSection>
          </Card>
        </div>

        <div className="flex flex-col gap-4">
          <Card>
            <CardSection>
              <EyebrowSm>License</EyebrowSm>
              <h3 className="mt-2 font-display text-[17px] font-bold text-charcoal-ink">
                AGPL-3.0-or-later
              </h3>
              <p className="mt-2 max-w-[32ch] text-[12px] leading-relaxed text-steel-secondary">
                Free to use, modify, run. Network use requires source disclosure.
              </p>
              <a
                href="https://github.com/Youzini-afk/Yggdrasil/blob/main/LICENSE"
                target="_blank"
                rel="noopener noreferrer"
                className="mt-3 inline-block text-[12px] font-medium text-charcoal-ink underline underline-offset-4 decoration-1 hover:decoration-aged-brass"
              >
                Read full license →
              </a>
            </CardSection>
          </Card>

          <Card>
            <CardSection>
              <EyebrowSm>Links</EyebrowSm>
              <ul className="mt-3 divide-y divide-whisper-border">
                {[
                  { Icon: GithubLogo, label: "Source code", href: "https://github.com/Youzini-afk/Yggdrasil" },
                  { Icon: BookOpen, label: "Documentation", href: "https://github.com/Youzini-afk/Yggdrasil/tree/main/docs" },
                  { Icon: Bug, label: "Report an issue", href: "https://github.com/Youzini-afk/Yggdrasil/issues" },
                  { Icon: ChatCircle, label: "Community", href: "#" },
                  { Icon: Newspaper, label: "Changelog", href: "https://github.com/Youzini-afk/Yggdrasil/blob/main/CHANGELOG.md" },
                ].map(({ Icon, label, href }) => (
                  <li key={label}>
                    <a
                      href={href}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="flex items-center justify-between gap-2 py-2.5 text-[13px] font-medium text-charcoal-ink hover:text-aged-brass-deep"
                    >
                      <span className="flex items-center gap-2">
                        <Icon size={16} className="text-steel-secondary" />
                        {label}
                      </span>
                      <span className="font-mono text-[10px] text-muted-tone">↗</span>
                    </a>
                  </li>
                ))}
              </ul>
            </CardSection>
          </Card>

          <Card>
            <CardSection>
              <EyebrowSm>Gratitude</EyebrowSm>
              <p className="mt-3 max-w-[36ch] text-[12px] leading-relaxed text-steel-secondary">
                SillyTavern community for the asset formats and extension API patterns referenced in
                YdlTavern compatibility work.
              </p>
            </CardSection>
          </Card>
        </div>
      </div>
    </>
  );
}
