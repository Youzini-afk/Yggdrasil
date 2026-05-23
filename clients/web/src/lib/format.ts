/** Format helpers — short, no dependencies. */

const SECOND = 1000;
const MINUTE = 60 * SECOND;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

export function formatRelativeAge(timestamp: string | number | Date): string {
  const ms = typeof timestamp === "number"
    ? timestamp
    : typeof timestamp === "string"
      ? new Date(timestamp).getTime()
      : timestamp.getTime();
  const elapsed = Date.now() - ms;
  if (Number.isNaN(elapsed) || elapsed < 0) return "—";
  if (elapsed < MINUTE) return "just now";
  if (elapsed < HOUR) return `${Math.floor(elapsed / MINUTE)}m ago`;
  if (elapsed < DAY) return `${Math.floor(elapsed / HOUR)}h ago`;
  if (elapsed < 7 * DAY) return `${Math.floor(elapsed / DAY)}d ago`;
  return new Date(ms).toLocaleDateString();
}

export function formatGreetingTime(date = new Date()): string {
  const days = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"] as const;
  const day = days[date.getDay()];
  const hh = String(date.getHours()).padStart(2, "0");
  const mm = String(date.getMinutes()).padStart(2, "0");
  return `WORKSHOP · ${day} ${hh}:${mm}`;
}

export function pluralize(count: number, singular: string, plural?: string): string {
  return `${count} ${count === 1 ? singular : plural ?? `${singular}s`}`;
}
