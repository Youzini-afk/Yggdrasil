import type { InstallDetectedKind, InstallPlan } from "@/protocol/client";

export function formatList(values: string[] | undefined, empty: string, limit = 3) {
  if (!values || values.length === 0) return empty;
  const shown = values.slice(0, limit).join(" · ");
  const remaining = values.length - limit;
  return remaining > 0 ? `${shown} · +${remaining} more` : shown;
}

export function shortHash(hash: string) {
  return hash.length > 12 ? `${hash.slice(0, 12)}…` : hash;
}

export function formatDetectedKind(kind: InstallDetectedKind | null) {
  switch (kind?.kind) {
    case "native":
      return "Native project";
    case "declared_external":
      return "Declared external";
    case "external":
      return "External";
    default:
      return "Detected";
  }
}

export function errorMessage(err: unknown) {
  return err instanceof Error ? err.message : String(err);
}

export function summarizeConformance(plan: InstallPlan) {
  let checks = 0;
  let failures = 0;
  let warnings = 0;
  for (const pkg of plan.packages) {
    const report = pkg.conformance;
    if (!report) continue;
    if (Array.isArray(report.checks)) {
      checks += report.checks.length;
      failures += report.checks.filter((check) => check.passed === false || check.status === "failed").length;
    }
    if (Array.isArray(report.failures)) failures += report.failures.length;
    if (Array.isArray(report.warnings)) warnings += report.warnings.length;
    if (report.passed === false && failures === 0) failures += 1;
  }
  if (checks === 0 && failures === 0 && warnings === 0) {
    return { hasFailures: false, label: "No conformance details returned" };
  }
  return {
    hasFailures: failures > 0,
    label: `${checks} check${checks === 1 ? "" : "s"}, ${failures} failure${failures === 1 ? "" : "s"}, ${warnings} warning${warnings === 1 ? "" : "s"}`,
  };
}
