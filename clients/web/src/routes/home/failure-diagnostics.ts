import type { FailureDetail } from "@/components/install/failure-modal";
import { formatRelativeAge } from "@/lib/format";
import type { YggProtocolClient, PackageRecord, SubprocessLogLine } from "@/protocol/client";

export function noFailureDiagnostic(projectName: string, reason: string): FailureDetail {
  return {
    projectName,
    title: "No diagnostic available",
    summary: reason,
    cause: "unavailable",
    log: [],
  };
}

export function failureDetailFromPackage(
  projectName: string,
  record: PackageRecord,
  logs: SubprocessLogLine[],
): FailureDetail {
  const failure = record.last_failure;
  const stderrLines = tail(
    failure?.stderr_tail_redacted.length
      ? failure.stderr_tail_redacted
      : logs.filter((log) => log.stream === "stderr").map((log) => log.line),
    8,
  );
  const fallbackLines = tail(logs.map((log) => `[${log.stream}] ${log.line}`), 20);
  return {
    projectName,
    title: `Package ${record.id} ${record.state}`,
    summary: failure?.reason ?? "Package status is degraded, but no failure summary was reported.",
    cause: failure?.reason ?? record.state,
    exitCode: failure?.exit_code ?? "—",
    failedAt: failure?.failed_at ? formatRelativeAge(failure.failed_at) : undefined,
    redactionState: failure?.redaction_state,
    log: stderrLines.length > 0 ? stderrLines : fallbackLines,
  };
}

export async function resolvePackageStatus(
  client: YggProtocolClient,
  packageRef: string,
  packageLookup: Map<string, PackageRecord>,
): Promise<PackageRecord | null> {
  const direct = await client.packageStatus(packageRef).catch<PackageRecord | null>(() => null);
  if (direct) return direct;

  const fileName = packageRef.split(/[\\/]/).pop();
  const packageDir = fileName?.match(/^manifest\.ya?ml$/i)
    ? packageRef.split(/[\\/]/).slice(-2, -1)[0]
    : undefined;
  const candidates = [packageRef, packageDir].filter(Boolean) as string[];
  return (
    candidates
      .flatMap((candidate) => [candidate, ...Array.from(packageLookup.values()).filter((record) => record.id.endsWith(`/${candidate}`)).map((record) => record.id)])
      .map((candidate) => packageLookup.get(candidate))
      .find(Boolean) ?? null
  );
}

export function tail<T>(items: T[], limit: number): T[] {
  return items.slice(Math.max(0, items.length - limit));
}
