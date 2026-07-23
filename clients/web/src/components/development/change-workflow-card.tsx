import { useCallback, useEffect, useMemo, useRef, useState, type Dispatch, type SetStateAction } from "react";
import { Button } from "@/components/ui/button";
import { Checkbox, Field, Input, Textarea } from "@/components/ui/input";
import { StatusPill, type StatusTone } from "@/components/ui/status-pill";
import { useKernel } from "@/lib/kernel-client";
import { useT } from "@/lib/locale";
import { useToast } from "@/components/ui/toast";
import type { LabelKey } from "@/lib/labels";
import type {
  DevelopmentChangeRecord,
  DevelopmentChangeStatus,
  DevelopmentFileOperationRequest,
  DevelopmentVerificationPlan,
} from "@/protocol/client";

type DraftOperation = "file_write" | "file_delete";

const ACTIVE_STATUSES = new Set<DevelopmentChangeStatus>(["staging", "verifying", "promoting"]);
const DEVELOPMENT_STATUS_KEYS: Record<DevelopmentChangeStatus, LabelKey> = {
  drafted: "projectFrameDevelopmentStatusDrafted",
  approved: "projectFrameDevelopmentStatusApproved",
  rejected: "projectFrameDevelopmentStatusRejected",
  staging: "projectFrameDevelopmentStatusStaging",
  verifying: "projectFrameDevelopmentStatusVerifying",
  promoting: "projectFrameDevelopmentStatusPromoting",
  verified: "projectFrameDevelopmentStatusVerified",
  committed: "projectFrameDevelopmentStatusCommitted",
  recovery_required: "projectFrameDevelopmentStatusRecoveryRequired",
  failed: "projectFrameDevelopmentStatusFailed",
};

export function DevelopmentWorkflowCard({ projectId }: { projectId: string }) {
  const client = useKernel();
  const t = useT();
  const toast = useToast();
  const [changes, setChanges] = useState<DevelopmentChangeRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState<string | null>(null);
  const [goal, setGoal] = useState("");
  const [path, setPath] = useState("");
  const [content, setContent] = useState("");
  const [operation, setOperation] = useState<DraftOperation>("file_write");
  const [executable, setExecutable] = useState(false);
  const [dockerBuild, setDockerBuild] = useState(false);
  const [dockerfile, setDockerfile] = useState("Dockerfile");
  const [allowNetwork, setAllowNetwork] = useState(false);
  const loadGeneration = useRef(0);
  const loadInFlight = useRef(false);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      loadGeneration.current += 1;
    };
  }, []);

  const load = useCallback(async () => {
    if (loadInFlight.current) return;
    loadInFlight.current = true;
    const generation = ++loadGeneration.current;
    try {
      const result = await client.listProjectChanges(projectId);
      if (mounted.current && generation === loadGeneration.current) {
        setChanges((current) => mergeChangeList(current, result.changes));
      }
    } catch (error) {
      if (mounted.current && generation === loadGeneration.current) {
        toast.push({
          variant: "error",
          title: t("projectFrameDevelopmentLoadFailed"),
          body: errorMessage(error),
        });
      }
    } finally {
      loadInFlight.current = false;
      if (mounted.current && generation === loadGeneration.current) {
        setLoading(false);
      }
    }
  }, [client, projectId, t, toast]);

  useEffect(() => {
    void load();
  }, [load]);

  const hasActiveChange = useMemo(
    () => changes.some((change) => ACTIVE_STATUSES.has(change.status)),
    [changes],
  );

  useEffect(() => {
    if (!hasActiveChange) return;
    const timer = window.setInterval(() => void load(), 1800);
    return () => window.clearInterval(timer);
  }, [hasActiveChange, load]);

  const onDraft = useCallback(async () => {
    if (!goal.trim() || !path.trim() || (operation === "file_write" && !content)) return;
    setBusy("draft");
    try {
      const fileOperation: DevelopmentFileOperationRequest = operation === "file_write"
        ? { op: "file_write", path: path.trim(), content, executable }
        : { op: "file_delete", path: path.trim() };
      const verification: DevelopmentVerificationPlan = dockerBuild
        ? {
            kind: "docker_build",
            dockerfile: dockerfile.trim() || "Dockerfile",
            network_mode: allowNetwork ? "bridge" : "none",
          }
        : { kind: "static_validation" };
      const change = await client.draftProjectChange(projectId, {
        goal: goal.trim(),
        operations: [fileOperation],
        verification,
        idempotency_key: newDevelopmentIdempotencyKey(),
      });
      if (!mounted.current) return;
      loadGeneration.current += 1;
      setLoading(false);
      setChanges((current) => [change, ...current.filter((item) => item.change_set.id !== change.change_set.id)]);
      setGoal("");
      setPath("");
      setContent("");
      toast.push({ variant: "success", title: t("projectFrameDevelopmentDrafted") });
    } catch (error) {
      if (mounted.current) {
        toast.push({ variant: "error", title: t("projectFrameDevelopmentDraftFailed"), body: errorMessage(error) });
      }
    } finally {
      if (mounted.current) setBusy(null);
    }
  }, [allowNetwork, client, content, dockerBuild, dockerfile, executable, goal, operation, path, projectId, t, toast]);

  const decide = useCallback(async (change: DevelopmentChangeRecord, approved: boolean) => {
    if (approved && typeof window !== "undefined" && !window.confirm(t("projectFrameDevelopmentApproveConfirm"))) return;
    setBusy(change.change_set.id);
    try {
      const updated = await client.approveProjectChange(
        projectId,
        change.change_set.id,
        approved,
        approved ? "approved from the project development console" : "rejected from the project development console",
      );
      if (!mounted.current) return;
      loadGeneration.current += 1;
      setLoading(false);
      replaceChange(setChanges, updated);
    } catch (error) {
      if (mounted.current) {
        toast.push({ variant: "error", title: t("projectFrameDevelopmentDecisionFailed"), body: errorMessage(error) });
      }
    } finally {
      if (mounted.current) setBusy(null);
    }
  }, [client, projectId, t, toast]);

  const execute = useCallback(async (change: DevelopmentChangeRecord) => {
    if (typeof window !== "undefined" && !window.confirm(t("projectFrameDevelopmentExecuteConfirm"))) return;
    setBusy(change.change_set.id);
    try {
      const result = await client.executeProjectChange(projectId, change.change_set.id);
      if (!mounted.current) return;
      loadGeneration.current += 1;
      setLoading(false);
      replaceChange(setChanges, result.change);
      toast.push({
        variant: result.accepted ? "success" : "info",
        title: result.accepted
          ? t("projectFrameDevelopmentExecutionStarted")
          : t("projectFrameDevelopmentExecutionAlreadyActive"),
      });
      if (!result.accepted) void load();
    } catch (error) {
      if (mounted.current) {
        toast.push({ variant: "error", title: t("projectFrameDevelopmentExecutionFailed"), body: errorMessage(error) });
      }
    } finally {
      if (mounted.current) setBusy(null);
    }
  }, [client, load, projectId, t, toast]);

  const recover = useCallback(async (change: DevelopmentChangeRecord) => {
    setBusy(change.change_set.id);
    try {
      const updated = await client.recoverProjectChange(projectId, change.change_set.id);
      if (!mounted.current) return;
      loadGeneration.current += 1;
      setLoading(false);
      replaceChange(setChanges, updated);
      toast.push({ variant: "success", title: t("projectFrameDevelopmentRecovered") });
    } catch (error) {
      if (mounted.current) {
        toast.push({ variant: "error", title: t("projectFrameDevelopmentRecoveryFailed"), body: errorMessage(error) });
      }
    } finally {
      if (mounted.current) setBusy(null);
    }
  }, [client, projectId, t, toast]);

  const exportBundle = useCallback(async (change: DevelopmentChangeRecord) => {
    setBusy(change.change_set.id);
    try {
      const bundle = await client.getProjectChangeBundle(projectId, change.change_set.id);
      if (!mounted.current) return;
      const blob = new Blob([JSON.stringify(bundle, null, 2)], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = `${change.change_set.id}.ygg-change.json`;
      anchor.click();
      URL.revokeObjectURL(url);
    } catch (error) {
      if (mounted.current) {
        toast.push({ variant: "error", title: t("projectFrameDevelopmentExportFailed"), body: errorMessage(error) });
      }
    } finally {
      if (mounted.current) setBusy(null);
    }
  }, [client, projectId, t, toast]);

  return (
    <div className="space-y-4">
      <div className="rounded-[16px] border border-aged-brass/40 bg-warm-bone p-4">
        <div className="grid gap-3 lg:grid-cols-2">
          <Field label={t("projectFrameDevelopmentGoal")} required>
            <Input value={goal} onChange={(event) => setGoal(event.target.value)} placeholder={t("projectFrameDevelopmentGoalPlaceholder")} />
          </Field>
          <Field label={t("projectFrameDevelopmentTarget")} required>
            <Input value={path} onChange={(event) => setPath(event.target.value)} placeholder="src/example.ts" className="font-mono" />
          </Field>
          <Field label={t("projectFrameDevelopmentOperation")}>
            <select
              value={operation}
              onChange={(event) => setOperation(event.target.value as DraftOperation)}
              className="h-10 rounded-[10px] border border-whisper-border bg-transparent px-3 text-[13px] text-charcoal-ink outline-none focus-visible:border-aged-brass"
            >
              <option value="file_write">{t("projectFrameDevelopmentWrite")}</option>
              <option value="file_delete">{t("projectFrameDevelopmentDelete")}</option>
            </select>
          </Field>
          <div className="flex flex-wrap items-end gap-4 pb-2">
            {operation === "file_write" ? <Checkbox checked={executable} onCheckedChange={setExecutable} label={t("projectFrameDevelopmentExecutable")} /> : null}
            <Checkbox checked={dockerBuild} onCheckedChange={setDockerBuild} label={t("projectFrameDevelopmentDockerBuild")} />
            {dockerBuild ? <Checkbox checked={allowNetwork} onCheckedChange={setAllowNetwork} label={t("projectFrameDevelopmentAllowNetwork")} /> : null}
          </div>
          {dockerBuild ? (
            <Field label={t("projectFrameDevelopmentDockerfile")}>
              <Input value={dockerfile} onChange={(event) => setDockerfile(event.target.value)} className="font-mono" />
            </Field>
          ) : null}
          {operation === "file_write" ? (
            <Field label={t("projectFrameDevelopmentContent")} required className={dockerBuild ? "lg:col-span-2" : "lg:col-span-2"}>
              <Textarea value={content} onChange={(event) => setContent(event.target.value)} className="min-h-[180px] font-mono text-[12px]" spellCheck={false} />
            </Field>
          ) : null}
        </div>
        <div className="mt-4 flex flex-wrap items-center justify-between gap-3">
          <p className="max-w-3xl text-[11px] leading-relaxed text-steel-secondary">{t("projectFrameDevelopmentSafetyHint")}</p>
          <Button
            tone="primary"
            size="sm"
            disabled={busy !== null || !goal.trim() || !path.trim() || (operation === "file_write" && !content)}
            onClick={() => void onDraft()}
          >
            {busy === "draft" ? t("projectFrameDevelopmentDrafting") : t("projectFrameDevelopmentDraft")}
          </Button>
        </div>
      </div>

      <div className="rounded-[16px] border border-whisper-border bg-pure-surface p-4">
        <div className="flex items-center justify-between gap-3">
          <div>
            <p className="font-display text-[15px] font-bold text-charcoal-ink">{t("projectFrameDevelopmentHistory")}</p>
            <p className="mt-1 text-[12px] text-steel-secondary">{t("projectFrameDevelopmentHistoryDescription")}</p>
          </div>
          <Button tone="tertiary" size="sm" onClick={() => void load()} disabled={loading}>{t("projectFrameDevelopmentRefresh")}</Button>
        </div>
        {loading ? <p className="mt-4 text-[12px] text-steel-secondary">{t("projectFrameDiagnosticsLoading")}</p> : changes.length === 0 ? (
          <p className="mt-4 text-[12px] text-steel-secondary">{t("projectFrameDevelopmentEmpty")}</p>
        ) : (
          <div className="mt-4 space-y-2">
            {changes.slice(0, 8).map((change) => {
              const isBusy = busy === change.change_set.id;
              const verification = change.verification_plan.kind === "docker_build"
                ? `${t("projectFrameDevelopmentVerificationDocker")} · ${change.verification_plan.dockerfile ?? "Dockerfile"} · ${change.verification_plan.network_mode ?? "none"}`
                : t("projectFrameDevelopmentVerificationStatic");
              return (
                <div key={change.change_set.id} className="rounded-[12px] border border-whisper-border bg-warm-bone p-3">
                  <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                    <div className="min-w-0">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="font-mono text-[11px] text-charcoal-ink">{change.change_set.id}</span>
                        <StatusPill tone={developmentStatusTone(change.status)} label={t(DEVELOPMENT_STATUS_KEYS[change.status])} />
                        <span className="rounded-full border border-whisper-border bg-pure-surface px-2 py-0.5 font-mono text-[10px] text-steel-secondary">{change.workspace_ownership}</span>
                      </div>
                      <p className="mt-2 text-[12px] text-charcoal-ink">{intentSummary(change)}</p>
                      <p className="mt-1 truncate font-mono text-[10px] text-muted-tone" title={change.proposed_tree_digest ?? change.base_tree_digest}>
                        {change.proposed_tree_digest ?? change.base_tree_digest}
                      </p>
                      {change.error ? <p className="mt-2 text-[11px] text-deep-rust">{change.error}</p> : null}
                      {change.workspace_ownership === "linked_local" ? <p className="mt-2 text-[11px] text-steel-secondary">{t("projectFrameDevelopmentLinkedHint")}</p> : null}
                      <div className="mt-3 grid gap-2 rounded-[10px] border border-whisper-border bg-pure-surface p-3 text-[11px] sm:grid-cols-2">
                        <div>
                          <p className="font-semibold text-charcoal-ink">{t("projectFrameDevelopmentReviewOperations")}</p>
                          <ul className="mt-1 space-y-1 text-steel-secondary">
                            {change.change_set.operations.map((item, index) => (
                              <li key={`${item.op}-${item.target ?? index}`} className="font-mono">
                                {item.op} · {item.target ?? "—"}
                              </li>
                            ))}
                          </ul>
                        </div>
                        <div>
                          <p className="font-semibold text-charcoal-ink">{t("projectFrameDevelopmentReviewVerification")}</p>
                          <p className="mt-1 font-mono text-steel-secondary">{verification}</p>
                        </div>
                        <div>
                          <p className="font-semibold text-charcoal-ink">{t("projectFrameDevelopmentReviewAuthority")}</p>
                          <div className="mt-1 flex flex-wrap gap-1">
                            {change.change_set.required_authority.map((authority) => (
                              <span key={authority} className="rounded-full bg-warm-bone px-2 py-0.5 font-mono text-[10px] text-steel-secondary">{authority}</span>
                            ))}
                          </div>
                        </div>
                        <div>
                          <p className="font-semibold text-charcoal-ink">{t("projectFrameDevelopmentReviewEffects")}</p>
                          <p className="mt-1 break-all font-mono text-[10px] text-steel-secondary">{compactJson(change.change_set.expected_effects)}</p>
                        </div>
                      </div>
                      {change.approval_decision ? (
                        <div className="mt-2 rounded-[10px] border border-whisper-border bg-pure-surface p-3 text-[11px]">
                          <p className="font-semibold text-charcoal-ink">{t("projectFrameDevelopmentApprovalRecord")}</p>
                          <p className="mt-1 font-mono text-steel-secondary">
                            {change.approval_decision.outcome} · {change.approval_decision.principal.kind} · {new Date(change.approval_decision.decided_at).toLocaleString()}
                          </p>
                          {change.approval_decision.reason ? <p className="mt-1 text-steel-secondary">{change.approval_decision.reason}</p> : null}
                          <div className="mt-2 flex flex-wrap gap-1">
                            {change.approval_decision.evaluated_authority.map((authority) => (
                              <span key={authority} className="rounded-full bg-warm-bone px-2 py-0.5 font-mono text-[10px] text-steel-secondary">{authority}</span>
                            ))}
                          </div>
                          {change.approval_ref ? <p className="mt-2 break-all font-mono text-[10px] text-muted-tone">{change.approval_ref.digest}</p> : null}
                        </div>
                      ) : null}
                      {change.recovery_kind ? (
                        <div className="mt-2 rounded-[10px] border border-deep-rust/30 bg-pure-surface p-3 text-[11px]">
                          <p className="font-semibold text-deep-rust">{t("projectFrameDevelopmentRecoveryTarget")}</p>
                          <p className="mt-1 font-mono text-steel-secondary">{change.recovery_kind}</p>
                          {change.managed_promotion ? (
                            <p className="mt-1 break-all font-mono text-[10px] text-muted-tone">
                              {change.managed_promotion.previous_tree_digest} → {change.managed_promotion.proposed_tree_digest} · {change.managed_promotion.destination_preexisting ? "destination_preexisting" : "destination_created_by_change"}
                            </p>
                          ) : null}
                        </div>
                      ) : null}
                    </div>
                    <div className="flex shrink-0 flex-wrap gap-2">
                      <Button tone="tertiary" size="sm" disabled={isBusy} onClick={() => void exportBundle(change)}>{t("projectFrameDevelopmentExport")}</Button>
                      {change.status === "drafted" ? (
                        <>
                          <Button tone="secondary" size="sm" disabled={isBusy} onClick={() => void decide(change, false)}>{t("projectFrameDevelopmentReject")}</Button>
                          <Button tone="primary" size="sm" disabled={isBusy} onClick={() => void decide(change, true)}>{t("projectFrameDevelopmentApprove")}</Button>
                        </>
                      ) : null}
                      {change.status === "approved" ? <Button tone="primary" size="sm" disabled={isBusy} onClick={() => void execute(change)}>{t("projectFrameDevelopmentExecute")}</Button> : null}
                      {change.status === "recovery_required" ? <Button tone="primary" size="sm" disabled={isBusy} onClick={() => void recover(change)}>{t("projectFrameDevelopmentRecover")}</Button> : null}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

function replaceChange(
  setChanges: Dispatch<SetStateAction<DevelopmentChangeRecord[]>>,
  updated: DevelopmentChangeRecord,
) {
  setChanges((current) => {
    const existing = current.find((item) => item.change_set.id === updated.change_set.id);
    if (existing && existing.revision > updated.revision) return current;
    return [updated, ...current.filter((item) => item.change_set.id !== updated.change_set.id)];
  });
}

function mergeChangeList(
  current: DevelopmentChangeRecord[],
  incoming: DevelopmentChangeRecord[],
): DevelopmentChangeRecord[] {
  const currentById = new Map(current.map((item) => [item.change_set.id, item]));
  return incoming.map((item) => {
    const existing = currentById.get(item.change_set.id);
    return existing && existing.revision > item.revision ? existing : item;
  });
}

function intentSummary(change: DevelopmentChangeRecord): string {
  const goal = change.intent.goal;
  if (goal && typeof goal === "object" && "summary" in goal && typeof (goal as { summary?: unknown }).summary === "string") {
    return (goal as { summary: string }).summary;
  }
  return change.change_set.operations.map((operation) => operation.target ?? operation.op).join(", ");
}

function developmentStatusTone(status: DevelopmentChangeStatus): StatusTone {
  if (status === "committed" || status === "verified") return "stopped";
  if (status === "failed" || status === "rejected" || status === "recovery_required") return "failed";
  if (ACTIVE_STATUSES.has(status)) return "starting";
  if (status === "approved") return "accent";
  return "neutral";
}

function compactJson(value: unknown): string {
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}

function newDevelopmentIdempotencyKey(): string {
  const entropy = globalThis.crypto?.randomUUID?.().replaceAll("-", "")
    ?? Math.random().toString(36).slice(2);
  return `dev-web-${Date.now().toString(36)}-${entropy.slice(0, 20)}`;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
