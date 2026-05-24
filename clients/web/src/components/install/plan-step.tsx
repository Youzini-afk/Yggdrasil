import { type ReactNode } from "react";
import {
  CheckCircle,
  Download,
  Folder as PIconFolder,
  GitBranch as PIconGit,
  GithubLogo,
  Globe as PIconGlobe,
  Key as PIconKey,
  Warning,
} from "@/components/icons";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/input";
import { ModalFooter, ModalHeader } from "@/components/ui/modal";
import { StatusPill } from "@/components/ui/status-pill";
import { EyebrowSm } from "@/components/ui/typography";
import { cn } from "@/lib/cn";
import type { InstallDetectedKind, InstallPlan } from "@/protocol/client";
import { formatDetectedKind, formatList, shortHash, summarizeConformance } from "./install-format";

export function PlanStep({
  url,
  plan,
  detectedKind,
  approvedPermissions,
  onApprovalChange,
  onBack,
  onCancel,
  onConfirm,
  installing,
}: {
  url: string;
  plan: InstallPlan;
  detectedKind: InstallDetectedKind | null;
  approvedPermissions: boolean;
  onApprovalChange: (v: boolean) => void;
  onBack: () => void;
  onCancel: () => void;
  onConfirm: () => void;
  installing: boolean;
}) {
  const rootPackage = plan.packages.find((pkg) => pkg.id === plan.root_id) ?? plan.packages[0];
  const permissionGroups = [
    {
      label: "Capabilities",
      detail: formatList(plan.permissions_summary.new_capabilities, "No new capability invokes"),
      count: plan.permissions_summary.new_capabilities.length,
    },
    {
      label: "Network",
      detail: formatList(plan.permissions_summary.new_network_hosts, "No new network hosts"),
      count: plan.permissions_summary.new_network_hosts.length,
    },
    {
      label: "Secrets",
      detail: formatList(plan.permissions_summary.new_secret_refs, "No new secret refs"),
      count: plan.permissions_summary.new_secret_refs.length,
    },
  ];
  const conformance = summarizeConformance(plan);
  const isExternal = detectedKind?.kind === "external";

  return (
    <>
      <ModalHeader
        eyebrow="Install — Step 2 of 3"
        title="Review the install plan"
        description="Install Lab resolved this plan. Approve requested permissions to begin installation."
      />

      {/* Project identity */}
      <div className="flex items-center gap-3 rounded-[12px] border border-whisper-border bg-aged-brass-surface-soft px-4 py-3">
        <GithubLogo size={20} className="text-charcoal-ink shrink-0" />
        <span className="flex-1 truncate font-mono text-[13px] text-charcoal-ink">{url}</span>
        <StatusPill tone="accent" label="RESOLVED" />
        <span className="font-mono text-[11px] text-muted-tone">root: {plan.root_id}</span>
      </div>

      {isExternal ? (
        <div className="mt-4 rounded-[12px] border border-aged-brass-border bg-aged-brass-surface-soft px-4 py-3 text-[12px] text-charcoal-ink">
          <div className="flex items-start gap-2">
            <Warning size={15} className="mt-0.5 shrink-0 text-aged-brass-deep" />
            <div>
              <p className="font-medium">External adapter generation is CLI-only in this build.</p>
              <p className="mt-1 text-steel-secondary">
                The package plan is real, but the web UI will not execute it without a project descriptor.
              </p>
            </div>
          </div>
        </div>
      ) : null}

      {/* Project metadata */}
      <section className="mt-6">
        <EyebrowSm>Project</EyebrowSm>
        <dl className="mt-3 grid grid-cols-2 gap-x-8 gap-y-2 text-[12px]">
          {[
            ["Kind", formatDetectedKind(detectedKind), "accent"],
            ["Root package", plan.root_id, "mono"],
            ["Version", rootPackage?.version ?? "—", "mono"],
            ["Source", rootPackage?.source ?? "—"],
            ["Commit", rootPackage?.commit_sha ? shortHash(rootPackage.commit_sha) : "—", "mono"],
            ["Signed", plan.signature_summary.all_signed ? "All signed" : "Unsigned packages", plan.signature_summary.all_signed ? "accent" : "neutral"],
          ].map(([label, value, hint]) => (
            <div key={label as string} className="flex justify-between">
              <dt className="font-medium text-steel-secondary">{label}</dt>
              <dd className={cn("text-charcoal-ink", hint === "mono" && "font-mono")}>
                {hint === "accent" ? (
                  <span className="inline-flex items-center gap-1.5">
                    <span className="size-1.5 rounded-full bg-aged-brass" aria-hidden />
                    {value}
                  </span>
                ) : hint === "neutral" ? (
                  <span className="inline-flex items-center gap-1.5">
                    <span className="size-1.5 rounded-full bg-steel-secondary" aria-hidden />
                    {value}
                  </span>
                ) : (
                  value
                )}
              </dd>
            </div>
          ))}
        </dl>
      </section>

      <div className="my-6 h-px bg-whisper-border" />

      {/* Dependencies */}
      <section>
        <div className="flex items-center justify-between">
          <EyebrowSm>Packages</EyebrowSm>
          <span className="text-[11px] text-steel-secondary">
            {plan.packages.length} package{plan.packages.length === 1 ? "" : "s"} will be installed
          </span>
        </div>
        <ul className="mt-3 divide-y divide-whisper-border">
          {plan.packages.map((pkg) => (
            <li key={`${pkg.id}:${pkg.tree_hash}`} className="flex items-center gap-3 py-2.5">
              <PackageMarker signed={pkg.signed} />
              <span className="flex-1 truncate font-mono text-[12px] text-charcoal-ink">{pkg.id}</span>
              <span className="font-mono text-[11px] text-muted-tone">{pkg.version}</span>
              <StatusPill tone={pkg.id === plan.root_id ? "accent" : "neutral"} label={pkg.source.toUpperCase()} showDot={false} />
            </li>
          ))}
        </ul>
      </section>

      <div className="my-6 h-px bg-whisper-border" />

      {/* Permissions */}
      <section>
        <div className="flex items-center justify-between">
          <EyebrowSm>Permissions requested</EyebrowSm>
          <span className="text-[11px] text-steel-secondary">
            {permissionGroups.reduce((sum, group) => sum + group.count, 0)} total entries
          </span>
        </div>
        <ul className="mt-3 divide-y divide-whisper-border">
          {permissionGroups.map((p) => (
            <li key={p.label} className="flex gap-3 py-3">
              <span className="rounded-full bg-aged-brass-surface-soft p-2 text-aged-brass shrink-0">
                <PermissionIcon label={p.label} />
              </span>
              <div className="min-w-0 flex-1">
                <p className="text-[12px] font-medium text-charcoal-ink">{p.label}</p>
                <p className="mt-0.5 truncate text-[11px] text-steel-secondary">{p.detail}</p>
              </div>
            </li>
          ))}
        </ul>
      </section>

      <div className="my-6 h-px bg-whisper-border" />

      <section className="grid gap-3 text-[12px] md:grid-cols-3">
        <SummaryCard
          icon={<CheckCircle size={16} className="text-aged-brass" />}
          title="Signatures"
          body={
            plan.signature_summary.all_signed
              ? "All packages signed"
              : `Unsigned: ${formatList(plan.signature_summary.unsigned_packages, "none")}`
          }
        />
        <SummaryCard
          icon={plan.integrity_summary.manifest_hashes_match_lockfile ? <CheckCircle size={16} className="text-aged-brass" /> : <Warning size={16} className="text-aged-brass-deep" />}
          title="Integrity"
          body={
            plan.integrity_summary.manifest_hashes_match_lockfile
              ? "No lockfile drift detected"
              : `${plan.integrity_summary.drift_detected.length} drift item${plan.integrity_summary.drift_detected.length === 1 ? "" : "s"}`
          }
        />
        <SummaryCard
          icon={conformance.hasFailures ? <Warning size={16} className="text-aged-brass-deep" /> : <CheckCircle size={16} className="text-aged-brass" />}
          title="Conformance"
          body={conformance.label}
        />
      </section>

      <ModalFooter className="justify-between">
        <Checkbox
          checked={approvedPermissions}
          onCheckedChange={onApprovalChange}
          label="Approve requested permissions"
        />
        <div className="flex items-center gap-3">
          <Button tone="tertiary" onClick={onBack} disabled={installing}>
            Back
          </Button>
          <Button tone="secondary" onClick={onCancel} disabled={installing}>
            Cancel
          </Button>
          <Button tone="primary" onClick={onConfirm} disabled={!approvedPermissions || installing || isExternal}>
            <Download size={14} />
            {installing ? "Installing…" : "Install"}
          </Button>
        </div>
      </ModalFooter>
    </>
  );
}

function PermissionIcon({ label }: { label: string }) {
  switch (label) {
    case "Network":
      return <PIconGlobe size={16} />;
    case "Secrets":
      return <PIconKey size={16} />;
    case "Filesystem":
      return <PIconFolder size={16} />;
    default:
      return <PIconGit size={16} />;
  }
}

function PackageMarker({ signed }: { signed: boolean }) {
  return signed ? (
    <CheckCircle size={14} className="shrink-0 text-aged-brass" weight="fill" />
  ) : (
    <span className="flex size-3.5 shrink-0 items-center justify-center rounded-full border border-whisper-border-strong/70">
      <span className="size-1.5 rounded-full bg-muted-tone" aria-hidden />
    </span>
  );
}

function SummaryCard({ icon, title, body }: { icon: ReactNode; title: string; body: string }) {
  return (
    <div className="rounded-[12px] border border-whisper-border bg-pure-surface p-3">
      <div className="flex items-center gap-2">
        {icon}
        <p className="font-medium text-charcoal-ink">{title}</p>
      </div>
      <p className="mt-2 line-clamp-2 text-[11px] leading-snug text-steel-secondary">{body}</p>
    </div>
  );
}
