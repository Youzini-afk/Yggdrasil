import { useState } from "react";
import { ArrowsLeftRight, GitBranch, Globe, Pencil, Plus } from "@/components/icons";
import { Button } from "@/components/ui/button";
import { Card, CardSection } from "@/components/ui/card";
import { Eyebrow, EyebrowSm, PageTitle } from "@/components/ui/typography";
import { StatusPill } from "@/components/ui/status-pill";
import { useToast } from "@/components/ui/toast";
import { cn } from "@/lib/cn";

interface ProfileEntry {
  id: string;
  name: string;
  description: string;
  editedAge: string;
  active: boolean;
  autoload: Array<{ name: string; tone: "running" | "stopped" | "failed" }>;
  network: string[];
  resolver: { store: string; envAllowlist: string; projectScope: string };
}

const MOCK_PROFILES: ProfileEntry[] = [
  {
    id: "forge-alpha",
    name: "forge-alpha",
    description: "Default workshop · 3 projects autoload · 6 hosts allowed",
    editedAge: "3d ago",
    active: true,
    autoload: [
      { name: "YdlTavern", tone: "running" },
      { name: "Coding Workshop", tone: "stopped" },
      { name: "Image Studio", tone: "failed" },
    ],
    network: [
      "api.openai.com",
      "api.anthropic.com",
      "generativelanguage.googleapis.com",
      "+ 3 more",
    ],
    resolver: { store: "enabled", envAllowlist: "4 keys", projectScope: "fallback to platform" },
  },
  {
    id: "sandbox",
    name: "sandbox",
    description: "No autoload · network blocked · for testing third-party packages",
    editedAge: "1w ago",
    active: false,
    autoload: [],
    network: [],
    resolver: { store: "disabled", envAllowlist: "0 keys", projectScope: "isolated" },
  },
  {
    id: "restricted",
    name: "restricted",
    description: "Read-only project access · audit-only · fixed secret allowlist",
    editedAge: "2w ago",
    active: false,
    autoload: [],
    network: ["api.openai.com"],
    resolver: { store: "read-only", envAllowlist: "1 key", projectScope: "isolated" },
  },
  {
    id: "local-dev",
    name: "local-dev",
    description: "Loads packages from sibling repos · debug logging on",
    editedAge: "1mo ago",
    active: false,
    autoload: [],
    network: ["*.localhost"],
    resolver: { store: "enabled", envAllowlist: "all", projectScope: "fallback to platform" },
  },
];

export function ProfilesPanel() {
  const toast = useToast();
  const [profiles, setProfiles] = useState(MOCK_PROFILES);
  const active = profiles.find((p) => p.active) ?? profiles[0];

  const handleSwitch = (id: string) => {
    if (profiles.find((p) => p.active)?.id === id) return;
    setProfiles((current) => current.map((p) => ({ ...p, active: p.id === id })));
    toast.push({
      variant: "warning",
      title: "Profile switch requires restart",
      body: `Restart the host to activate ${id}.`,
      duration: 6000,
    });
  };

  return (
    <>
      <header className="mb-8">
        <Eyebrow>Profiles · Active: {active.name}</Eyebrow>
        <PageTitle className="mt-2">Workshop profiles</PageTitle>
        <p className="mt-2 max-w-[64ch] text-[13px] leading-relaxed text-steel-secondary">
          A profile bundles host configuration: which projects autoload, which outbound hosts are
          allowed, secret resolver settings. Switch profiles to swap workshop modes — daily use,
          sandbox, restricted.
        </p>
      </header>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[7fr_4fr]">
        <section className="flex flex-col gap-3">
          <div className="flex items-center justify-between">
            <Eyebrow>Profiles</Eyebrow>
            <Button tone="tertiary" size="sm">
              <Plus size={14} />
              New profile
            </Button>
          </div>
          <Card>
            <ul className="divide-y divide-whisper-border">
              {profiles.map((profile) => (
                <li
                  key={profile.id}
                  onClick={() => handleSwitch(profile.id)}
                  className={cn(
                    "flex cursor-pointer items-center gap-4 px-5 py-4 transition",
                    profile.active && "border-l-[3px] border-l-aged-brass bg-aged-brass-surface-soft",
                    !profile.active && "hover:bg-whisper-border-strong/30",
                  )}
                >
                  <GitBranch
                    size={18}
                    className={profile.active ? "text-aged-brass" : "text-steel-secondary"}
                  />
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="font-display text-[16px] font-bold text-charcoal-ink">
                        {profile.name}
                      </span>
                      {profile.active ? (
                        <StatusPill tone="accent" label="ACTIVE" showDot={false} />
                      ) : null}
                    </div>
                    <p className="mt-1 text-[12px] leading-snug text-steel-secondary">
                      {profile.description}
                    </p>
                  </div>
                  <span className="shrink-0 font-mono text-[11px] text-muted-tone">
                    edited {profile.editedAge}
                  </span>
                </li>
              ))}
            </ul>
          </Card>
        </section>

        <Card>
          <CardSection>
            <EyebrowSm>Active profile</EyebrowSm>
            <div className="mt-3 flex items-center justify-between">
              <h3 className="font-display text-[20px] font-bold text-charcoal-ink">
                {active.name}
              </h3>
              <Button tone="icon" size="icon-sm" aria-label="Edit profile">
                <Pencil size={14} />
              </Button>
            </div>
          </CardSection>

          <CardSection divided>
            <EyebrowSm>Autoload on start</EyebrowSm>
            {active.autoload.length === 0 ? (
              <p className="mt-2 text-[12px] text-muted-tone">No projects autoload.</p>
            ) : (
              <ul className="mt-3 space-y-1.5">
                {active.autoload.map((entry) => (
                  <li key={entry.name} className="flex items-center gap-2 font-mono text-[12px]">
                    <span
                      className={cn(
                        "size-1.5 rounded-full",
                        entry.tone === "running" && "bg-aged-brass animate-[pulse-dot_2.4s_ease-in-out_infinite]",
                        entry.tone === "stopped" && "bg-steel-secondary",
                        entry.tone === "failed" && "bg-deep-rust",
                      )}
                    />
                    <span className="text-charcoal-ink">{entry.name}</span>
                  </li>
                ))}
              </ul>
            )}
          </CardSection>

          <CardSection divided>
            <EyebrowSm>Network allowlist</EyebrowSm>
            {active.network.length === 0 ? (
              <p className="mt-2 text-[12px] text-muted-tone">All outbound blocked.</p>
            ) : (
              <ul className="mt-3 space-y-1.5">
                {active.network.map((host) => (
                  <li key={host} className="flex items-center gap-2 font-mono text-[12px] text-charcoal-ink">
                    <Globe size={12} className="text-steel-secondary" />
                    <span>{host}</span>
                  </li>
                ))}
              </ul>
            )}
          </CardSection>

          <CardSection divided>
            <EyebrowSm>Secret resolver</EyebrowSm>
            <dl className="mt-3 space-y-2 text-[12px]">
              {[
                ["Store", active.resolver.store],
                ["Env allowlist", active.resolver.envAllowlist],
                ["Project scope", active.resolver.projectScope],
              ].map(([label, value]) => (
                <div key={label} className="flex items-center justify-between">
                  <dt className="text-steel-secondary">{label}</dt>
                  <dd className="font-mono text-charcoal-ink">{value}</dd>
                </div>
              ))}
            </dl>
          </CardSection>

          <CardSection divided>
            <Button tone="primary" className="w-full">
              <ArrowsLeftRight size={14} />
              Switch to another profile…
            </Button>
            <p className="mt-2 text-center text-[11px] text-muted-tone">
              Switching restarts the host. Project state is preserved.
            </p>
          </CardSection>
        </Card>
      </div>
    </>
  );
}
