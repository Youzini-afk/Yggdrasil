export const SHELL_SCHEMA_VERSION = 1;

export const SHELL_CONTRIBUTION_TITLE_MAX_LENGTH = 80;
export const SHELL_CONTRIBUTION_DESCRIPTION_MAX_LENGTH = 240;
export const SHELL_CONTRIBUTION_ICON_HINT_MAX_LENGTH = 40;

export type ShellContributionSlot = "quick_action" | "workshop_card" | "home_card";
export type WorkshopCardCategory = "tool" | "template" | "example";

export interface ShellContributionBase {
  id: string;
  packageId: string;
  slot: ShellContributionSlot;
  title: string;
  description?: string;
  iconHint?: string;
  order?: number;
}

export interface QuickActionContribution extends ShellContributionBase {
  slot: "quick_action";
  capabilityId?: string;
  surfaceId?: string;
}

export interface WorkshopCardContribution extends ShellContributionBase {
  slot: "workshop_card";
  category?: WorkshopCardCategory;
  capabilityId?: string;
  surfaceId?: string;
}

export interface HomeCardContribution extends ShellContributionBase {
  slot: "home_card";
  capabilityId?: string;
  surfaceId?: string;
}

export type ShellContribution = QuickActionContribution | WorkshopCardContribution | HomeCardContribution;

type UnknownRecord = Record<string, unknown>;

const FALLBACK_LOCALES = ["en", "zh-CN"];
const VALID_WORKSHOP_CATEGORIES = new Set<WorkshopCardCategory>(["tool", "template", "example"]);
const ICON_HINT_PATTERN = /^[a-z][a-z0-9_-]{0,39}$/i;

export function parseShellContribution(
  contribution: unknown,
  slot: ShellContributionSlot,
  locale: string,
): ShellContribution | null {
  const record = asRecord(contribution);
  if (!record) return null;

  const packageId = readString(record.package_id) ?? readString(record.packageId);
  const surface = asRecord(record.surface) ?? record;
  if (!packageId) return null;

  const surfaceSlot = readString(surface.slot);
  if (surfaceSlot !== slot) return null;

  const id = readString(surface.id);
  if (!id) return null;

  const metadata = asRecord(surface.metadata);
  if (!metadata || metadata.shell_schema_version !== SHELL_SCHEMA_VERSION) return null;

  if (slot !== "workshop_card" && metadata.category !== undefined) return null;

  const title = readLocalizedText(metadata.title, locale, SHELL_CONTRIBUTION_TITLE_MAX_LENGTH);
  if (!title) return null;

  const description = readLocalizedText(metadata.description, locale, SHELL_CONTRIBUTION_DESCRIPTION_MAX_LENGTH);
  const iconHint = readIconHint(metadata.icon_hint);
  if (metadata.icon_hint !== undefined && iconHint === null) return null;

  const order = readOrder(metadata.order);
  if (metadata.order !== undefined && order === null) return null;

  const base: ShellContributionBase = {
    id,
    packageId,
    slot,
    title,
    ...(description ? { description } : {}),
    ...(iconHint ? { iconHint } : {}),
    ...(order !== undefined && order !== null ? { order } : {}),
  };

  const capabilityId = readString(surface.capability_id) ?? readString(metadata.capability_id);
  const surfaceId = readString(surface.surface_id) ?? readString(metadata.surface_id);

  if (slot === "quick_action") {
    return {
      ...base,
      slot,
      ...(capabilityId ? { capabilityId } : {}),
      ...(surfaceId ? { surfaceId } : {}),
    };
  }

  if (slot === "workshop_card") {
    const category = readWorkshopCategory(metadata.category);
    if (metadata.category !== undefined && !category) return null;
    return {
      ...base,
      slot,
      ...(category ? { category } : {}),
      ...(capabilityId ? { capabilityId } : {}),
      ...(surfaceId ? { surfaceId } : {}),
    };
  }

  return {
    ...base,
    slot,
    ...(capabilityId ? { capabilityId } : {}),
    ...(surfaceId ? { surfaceId } : {}),
  };
}

export function parseShellContributions(
  contributions: unknown,
  slot: ShellContributionSlot,
  locale: string,
): ShellContribution[] {
  if (!Array.isArray(contributions)) return [];
  return contributions
    .map((item) => parseShellContribution(item, slot, locale))
    .filter((item): item is ShellContribution => item !== null)
    .sort(compareShellContributions);
}

export function compareShellContributions(a: ShellContribution, b: ShellContribution): number {
  const orderA = a.order ?? Number.MAX_SAFE_INTEGER;
  const orderB = b.order ?? Number.MAX_SAFE_INTEGER;
  if (orderA !== orderB) return orderA - orderB;
  const packageOrder = a.packageId.localeCompare(b.packageId);
  if (packageOrder !== 0) return packageOrder;
  return a.id.localeCompare(b.id);
}

function readLocalizedText(value: unknown, locale: string, maxLength: number): string | undefined {
  const localized = asRecord(value);
  if (!localized) return undefined;

  const seen = new Set<string>();
  const candidates = [locale, ...FALLBACK_LOCALES, ...Object.keys(localized)];
  for (const candidate of candidates) {
    if (seen.has(candidate)) continue;
    seen.add(candidate);
    const text = readString(localized[candidate]);
    if (!text || text.length > maxLength) continue;
    return text;
  }
  return undefined;
}

function readIconHint(value: unknown): string | undefined | null {
  if (value === undefined || value === null) return undefined;
  const iconHint = readString(value);
  if (!iconHint) return undefined;
  if (iconHint.length > SHELL_CONTRIBUTION_ICON_HINT_MAX_LENGTH) return null;
  if (!ICON_HINT_PATTERN.test(iconHint)) return null;
  return iconHint.toLowerCase();
}

function readOrder(value: unknown): number | undefined | null {
  if (value === undefined || value === null) return undefined;
  if (typeof value !== "number" || !Number.isFinite(value)) return null;
  return value;
}

function readWorkshopCategory(value: unknown): WorkshopCardCategory | undefined {
  const category = readString(value);
  if (!category) return undefined;
  return VALID_WORKSHOP_CATEGORIES.has(category as WorkshopCardCategory) ? (category as WorkshopCardCategory) : undefined;
}

function readString(value: unknown): string | undefined {
  if (typeof value !== "string") return undefined;
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : undefined;
}

function asRecord(value: unknown): UnknownRecord | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  return value as UnknownRecord;
}
