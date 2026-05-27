import type {
  HomeCardContribution,
  QuickActionContribution,
  WorkshopCardContribution,
} from "@/surfaces/shell-contributions";

export const HOME_BUILTIN_QUICK_ACTION_PACKAGE_ID = "platform/home";
export const HOME_PACKAGE_QUICK_ACTION_LIMIT = 4;
export const HOME_WORKSHOP_CARD_LIMIT = 4;
export const HOME_CAPABILITY_CARD_LIMIT = 3;

export type HomeBuiltinQuickActionId = "install" | "open-folder" | "settings" | "switch-profile";

export interface HomeBuiltinQuickActionSpec {
  id: HomeBuiltinQuickActionId;
  title: string;
  iconHint: string;
}

export function createHomeBuiltinQuickActions(specs: HomeBuiltinQuickActionSpec[]): QuickActionContribution[] {
  return specs.map((spec, order) => ({
    id: spec.id,
    packageId: HOME_BUILTIN_QUICK_ACTION_PACKAGE_ID,
    slot: "quick_action",
    title: spec.title,
    iconHint: spec.iconHint,
    order,
  }));
}

export function mergeHomeQuickActions({
  builtin,
  packageActions,
  packageLimit = HOME_PACKAGE_QUICK_ACTION_LIMIT,
}: {
  builtin: QuickActionContribution[];
  packageActions: QuickActionContribution[];
  packageLimit?: number;
}): QuickActionContribution[] {
  return [...builtin, ...packageActions.slice(0, Math.max(0, packageLimit))];
}

export function limitHomeWorkshopCards(
  cards: WorkshopCardContribution[],
  limit = HOME_WORKSHOP_CARD_LIMIT,
): WorkshopCardContribution[] {
  return cards.slice(0, Math.max(0, limit));
}

export function limitHomeCapabilityCards(
  cards: HomeCardContribution[],
  limit = HOME_CAPABILITY_CARD_LIMIT,
): HomeCardContribution[] {
  return cards.slice(0, Math.max(0, limit));
}
