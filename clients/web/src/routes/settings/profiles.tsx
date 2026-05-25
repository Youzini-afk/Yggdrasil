import { useMemo } from "react";
import { ArrowsLeftRight, GitBranch, Globe, Plus } from "@/components/icons";
import { Button } from "@/components/ui/button";
import { Card, CardSection } from "@/components/ui/card";
import { Eyebrow, EyebrowSm, PageTitle } from "@/components/ui/typography";
import { StatusPill } from "@/components/ui/status-pill";
import { Skeleton } from "@/components/ui/skeleton";
import { EmptyState } from "@/components/ui/empty-state";
import { useToast } from "@/components/ui/toast";
import { useAsync, useKernel } from "@/lib/kernel-client";
import { useT } from "@/lib/locale";
import { cn } from "@/lib/cn";

interface ProfileView {
  id: string;
  name: string;
  description: string;
  active: boolean;
  packagesAutoload: number;
  hostsAllowed: number;
}

export function ProfilesPanel() {
  const client = useKernel();
  const toast = useToast();
  const t = useT();

  const diagnostics = useAsync(() => client.diagnostics().catch(() => null), [client]);

  const profiles = useMemo<ProfileView[]>(() => {
    if (!diagnostics.data) return [];
    const d = diagnostics.data as {
      profile_name?: string;
      packages_loaded?: number;
      network_allowlist?: { hosts?: string[] };
      profile_description?: string;
    };
    const activeName = d.profile_name ?? "default";
    return [
      {
        id: activeName,
        name: activeName,
        description:
          d.profile_description ??
          t("profilesDefaultDescription", d.packages_loaded ?? 0, d.network_allowlist?.hosts?.length ?? 0),
        active: true,
        packagesAutoload: d.packages_loaded ?? 0,
        hostsAllowed: d.network_allowlist?.hosts?.length ?? 0,
      },
    ];
  }, [diagnostics.data, t]);

  const active = profiles.find((p) => p.active) ?? profiles[0];

  const handleSwitch = (id: string) => {
    if (active?.id === id) return;
    toast.push({
      variant: "warning",
      title: t("profilesSwitchRequiresRestart"),
      body: t("profilesSwitchBody", id),
      duration: 6000,
    });
  };

  return (
    <>
      <header className="mb-8">
        <Eyebrow>
          {diagnostics.loading
            ? t("profilesEyebrowLoading")
            : active
              ? t("profilesEyebrowActive", active.name)
              : t("profilesEyebrowNone")}
        </Eyebrow>
        <PageTitle className="mt-2">{t("profilesTitle")}</PageTitle>
        <p className="mt-2 max-w-[64ch] text-[13px] leading-relaxed text-steel-secondary">
          {t("profilesDescriptionPrefix")} {" "}
          <span className="font-mono text-charcoal-ink">yg host serve --profile</span>
          {t("profilesDescriptionSuffix")}
        </p>
      </header>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[7fr_4fr]">
        <section className="flex flex-col gap-3">
          <div className="flex items-center justify-between">
            <Eyebrow>{t("profilesOnMachine")}</Eyebrow>
            <Button
              tone="tertiary"
              size="sm"
              onClick={() =>
                toast.push({
                  variant: "info",
                  title: t("profilesCreateTitle"),
                  body: t("profilesCreateBody"),
                  duration: 4000,
                })
              }
            >
              <Plus size={14} />
              {t("profilesNew")}
            </Button>
          </div>
          {diagnostics.error ? (
            <Card>
              <EmptyState
                icon={<GitBranch />}
                title={t("profilesDiagnosticsErrorTitle")}
                body={t("profilesDiagnosticsErrorBody")}
                action={{ label: t("retry"), onClick: () => diagnostics.refresh() }}
              />
            </Card>
          ) : diagnostics.loading ? (
            <Card>
              <ul className="divide-y divide-whisper-border">
                {Array.from({ length: 2 }).map((_, idx) => (
                  <li key={idx} className="flex items-center gap-4 px-5 py-4">
                    <Skeleton className="size-5 rounded-full" />
                    <div className="flex-1 space-y-1.5">
                      <Skeleton className="h-3 w-44" />
                      <Skeleton className="h-2.5 w-72" />
                    </div>
                    <Skeleton className="h-3 w-20" />
                  </li>
                ))}
              </ul>
            </Card>
          ) : profiles.length === 0 ? (
            <Card>
              <EmptyState
                icon={<GitBranch />}
                title={t("profilesEmptyTitle")}
                body={t("profilesEmptyBody")}
              />
            </Card>
          ) : (
            <Card>
              <ul className="divide-y divide-whisper-border">
                {profiles.map((profile) => (
                  <li
                    key={profile.id}
                    onClick={() => handleSwitch(profile.id)}
                    className={cn(
                      "flex cursor-pointer items-center gap-4 px-5 py-4 transition",
                      profile.active &&
                        "border-l-[3px] border-l-aged-brass bg-aged-brass-surface-soft",
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
                          <StatusPill tone="accent" label={t("profilesActive")} showDot={false} />
                        ) : null}
                      </div>
                      <p className="mt-1 text-[12px] leading-snug text-steel-secondary">
                        {profile.description}
                      </p>
                    </div>
                  </li>
                ))}
              </ul>
            </Card>
          )}
        </section>

        {active ? (
          <Card>
            <CardSection>
              <EyebrowSm>{t("profilesActive")}</EyebrowSm>
              <h3 className="mt-3 font-display text-[20px] font-bold text-charcoal-ink">
                {active.name}
              </h3>
              <p className="mt-1 text-[12px] text-steel-secondary">{active.description}</p>
            </CardSection>

            <CardSection divided>
              <EyebrowSm>{t("profilesLoadedPackages")}</EyebrowSm>
              <p className="mt-2 font-mono text-[13px] text-charcoal-ink">
                {active.packagesAutoload}
              </p>
              <p className="mt-1 text-[11px] text-steel-secondary">
                {t("profilesLoadedPackagesHint")}
              </p>
            </CardSection>

            <CardSection divided>
              <EyebrowSm>{t("profilesNetworkAllowlist")}</EyebrowSm>
              {(diagnostics.data as { network_allowlist?: { hosts?: string[] } } | null)
                ?.network_allowlist?.hosts?.length ? (
                <ul className="mt-3 space-y-1.5">
                  {(
                    (diagnostics.data as { network_allowlist: { hosts: string[] } })
                      .network_allowlist.hosts ?? []
                  )
                    .slice(0, 6)
                    .map((host) => (
                      <li
                        key={host}
                        className="flex items-center gap-2 font-mono text-[12px] text-charcoal-ink"
                      >
                        <Globe size={12} className="text-steel-secondary" />
                        <span className="truncate">{host}</span>
                      </li>
                    ))}
                </ul>
              ) : (
                <p className="mt-2 text-[12px] text-muted-tone">{t("profilesOutboundBlocked")}</p>
              )}
            </CardSection>

            <CardSection divided>
              <Button
                tone="secondary"
                className="w-full"
                onClick={() =>
                  toast.push({
                    variant: "info",
                    title: t("profilesSwitchViaCli"),
                    body: "yg host serve --profile <path>",
                  })
                }
              >
                <ArrowsLeftRight size={14} />
                {t("profilesSwitch")}
              </Button>
              <p className="mt-2 text-center text-[11px] text-muted-tone">
                {t("profilesSwitchHint")}
              </p>
            </CardSection>
          </Card>
        ) : null}
      </div>
    </>
  );
}
