/**
 * Format helpers used across the settings panels and Home.
 */

const RELATIVE_THRESHOLDS: Array<[number, string, number]> = [
  [60_000, "just now", 0],
  [3_600_000, "minute", 60_000],
  [86_400_000, "hour", 3_600_000],
  [604_800_000, "day", 86_400_000],
  [2_628_000_000, "week", 604_800_000],
  [31_536_000_000, "month", 2_628_000_000],
];

export function formatRelativeAge(timestamp: string | number | Date | undefined): string {
  if (!timestamp) return "just now";
  const value = timestamp instanceof Date ? timestamp.getTime() : new Date(timestamp).getTime();
  if (Number.isNaN(value)) return "just now";
  const diff = Date.now() - value;
  if (diff < 0) return "just now";
  for (const [limit, unit, divisor] of RELATIVE_THRESHOLDS) {
    if (diff < limit) {
      if (unit === "just now") return unit;
      const n = Math.floor(diff / divisor);
      return `${n} ${unit}${n === 1 ? "" : "s"} ago`;
    }
  }
  const years = Math.floor(diff / 31_536_000_000);
  return `${years} year${years === 1 ? "" : "s"} ago`;
}

export function formatBytes(bytes: number | undefined, fractionDigits = 1): string {
  if (bytes == null || Number.isNaN(bytes)) return "—";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1_048_576) return `${(bytes / 1024).toFixed(fractionDigits)} KB`;
  if (bytes < 1_073_741_824) return `${(bytes / 1_048_576).toFixed(fractionDigits)} MB`;
  return `${(bytes / 1_073_741_824).toFixed(fractionDigits)} GB`;
}

export function formatGreetingTime(now = new Date()): string {
  const day = now.toLocaleDateString(undefined, { weekday: "short" }).toUpperCase();
  const time = now.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit", hour12: false });
  return `WORKSHOP · ${day} ${time}`;
}

/** Categorize a package_id into one of the inventory kinds for filtering. */
export function classifyPackageKind(packageId: string): "PROJECT" | "OFFICIAL" | "THIRD-PARTY" {
  if (packageId.startsWith("official/") || packageId.startsWith("official__")) return "OFFICIAL";
  if (
    packageId.includes("__") ||
    packageId.startsWith("github__") ||
    packageId.startsWith("local__") ||
    packageId.includes("/")
  ) {
    return "PROJECT";
  }
  return "THIRD-PARTY";
}
