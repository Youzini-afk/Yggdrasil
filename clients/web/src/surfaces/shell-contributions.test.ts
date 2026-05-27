import {
  parseShellContribution,
  parseShellContributions,
  type QuickActionContribution,
  type ShellContributionSlot,
  type WorkshopCardContribution,
} from "./shell-contributions";
import {
  HOME_BUILTIN_QUICK_ACTION_PACKAGE_ID,
  createHomeBuiltinQuickActions,
  limitHomeWorkshopCards,
  mergeHomeQuickActions,
} from "../routes/home/shell-actions";

function assertEqual<T>(actual: T, expected: T) {
  if (actual !== expected) {
    throw new Error(`expected ${String(expected)}, got ${String(actual)}`);
  }
}

function assertDeepEqual(actual: unknown, expected: unknown) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

function assertOk(value: unknown, message: string) {
  if (!value) throw new Error(message);
}

function contribution(metadata: Record<string, unknown>, overrides: Record<string, unknown> = {}, slot: ShellContributionSlot = "quick_action") {
  return {
    package_id: "pkg/demo",
    surface: {
      id: "surface-a",
      slot,
      metadata,
      ...overrides,
    },
  };
}

const baseMetadata = {
  shell_schema_version: 1,
  title: { en: "Run", "zh-CN": "运行" },
};

const localeFallback = parseShellContribution(
  contribution({
    shell_schema_version: 1,
    title: { fr: "Lancer", en: "Run", "zh-CN": "运行" },
    description: { "zh-CN": "中文描述" },
  }),
  "quick_action",
  "fr",
);
assertOk(localeFallback, "expected localized contribution");
assertEqual(localeFallback?.title, "Lancer");
assertEqual(localeFallback?.description, "中文描述");

const enFallback = parseShellContribution(
  contribution({
    shell_schema_version: 1,
    title: { en: "English", "zh-CN": "中文" },
  }),
  "quick_action",
  "de",
);
assertEqual(enFallback?.title, "English");

assertEqual(parseShellContribution(contribution({ title: { en: "Legacy" } }), "quick_action", "en"), null);
assertEqual(parseShellContribution(contribution({ shell_schema_version: 1, description: { en: "No title" } }), "quick_action", "en"), null);
assertEqual(parseShellContribution(contribution({ ...baseMetadata, title: { en: "   " } }), "quick_action", "en"), null);
assertEqual(parseShellContribution(contribution({ shell_schema_version: 1, title: { en: "x".repeat(81) } }), "quick_action", "en"), null);
assertEqual(parseShellContribution(contribution({ ...baseMetadata, description: { en: "x".repeat(241) } }), "quick_action", "en"), null);

const unknownIcon = parseShellContribution(contribution({ ...baseMetadata, icon_hint: "mystery" }), "quick_action", "en");
assertEqual(unknownIcon?.iconHint, "mystery");

assertEqual(parseShellContribution(contribution({ ...baseMetadata, icon_hint: "https://example.test/icon.svg" }), "quick_action", "en"), null);
assertEqual(parseShellContribution(contribution({ ...baseMetadata, icon_hint: "bad/icon" }), "quick_action", "en"), null);

const metadataCapabilityIgnored = parseShellContribution(
  contribution({ ...baseMetadata, capability_id: "other/package/run" }),
  "quick_action",
  "en",
);
assertEqual(metadataCapabilityIgnored?.capabilityId, undefined);

const validCategory = parseShellContribution(
  contribution({ ...baseMetadata, category: "template" }, {}, "workshop_card"),
  "workshop_card",
  "en",
);
assertEqual(validCategory?.slot, "workshop_card");
assertEqual(validCategory && "category" in validCategory ? validCategory.category : undefined, "template");
assertEqual(parseShellContribution(contribution({ ...baseMetadata, category: "marketing" }, {}, "workshop_card"), "workshop_card", "en"), null);
assertEqual(parseShellContribution(contribution({ ...baseMetadata, category: "tool" }), "quick_action", "en"), null);

const sorted = parseShellContributions(
  [
    { package_id: "pkg/b", surface: { id: "b", slot: "quick_action", metadata: { ...baseMetadata, order: 20 } } },
    { package_id: "pkg/a", surface: { id: "c", slot: "quick_action", metadata: { ...baseMetadata, order: 20 } } },
    { package_id: "pkg/a", surface: { id: "a", slot: "quick_action", metadata: { ...baseMetadata, order: 10 } } },
    { package_id: "pkg/a", surface: { id: "b", slot: "quick_action", metadata: { ...baseMetadata, order: 20 } } },
  ],
  "quick_action",
  "en",
);
assertDeepEqual(sorted.map((item) => `${item.order}:${item.packageId}:${item.id}`), [
  "10:pkg/a:a",
  "20:pkg/a:b",
  "20:pkg/a:c",
  "20:pkg/b:b",
]);

const builtinQuickActions = createHomeBuiltinQuickActions([
  { id: "install", title: "Install URL", iconHint: "plus" },
  { id: "open-folder", title: "Data folder", iconHint: "folder" },
  { id: "settings", title: "Settings", iconHint: "settings" },
  { id: "switch-profile", title: "Switch profile", iconHint: "terminal" },
]);
assertDeepEqual(builtinQuickActions.map((item) => item.id), ["install", "open-folder", "settings", "switch-profile"]);
assertDeepEqual(
  builtinQuickActions.map((item) => item.packageId),
  Array.from({ length: 4 }, () => HOME_BUILTIN_QUICK_ACTION_PACKAGE_ID),
);
assertDeepEqual(
  builtinQuickActions.map((item) => item.capabilityId ?? item.surfaceId ?? null),
  [null, null, null, null],
);

const packageQuickActions = parseShellContributions(
  [
    { package_id: "pkg/a", surface: { id: "a", slot: "quick_action", metadata: { ...baseMetadata, order: 1 } } },
    { package_id: "pkg/b", surface: { id: "b", slot: "quick_action", metadata: { ...baseMetadata, order: 2 } } },
    { package_id: "pkg/c", surface: { id: "c", slot: "quick_action", metadata: { ...baseMetadata, order: 3 } } },
    { package_id: "pkg/d", surface: { id: "d", slot: "quick_action", metadata: { ...baseMetadata, order: 4 } } },
    { package_id: "pkg/e", surface: { id: "e", slot: "quick_action", metadata: { ...baseMetadata, order: 5 } } },
  ],
  "quick_action",
  "en",
).filter((item): item is QuickActionContribution => item.slot === "quick_action");
const mergedQuickActions = mergeHomeQuickActions({ builtin: builtinQuickActions, packageActions: packageQuickActions, packageLimit: 4 });
assertDeepEqual(mergedQuickActions.map((item) => `${item.packageId}:${item.id}`), [
  `${HOME_BUILTIN_QUICK_ACTION_PACKAGE_ID}:install`,
  `${HOME_BUILTIN_QUICK_ACTION_PACKAGE_ID}:open-folder`,
  `${HOME_BUILTIN_QUICK_ACTION_PACKAGE_ID}:settings`,
  `${HOME_BUILTIN_QUICK_ACTION_PACKAGE_ID}:switch-profile`,
  "pkg/a:a",
  "pkg/b:b",
  "pkg/c:c",
  "pkg/d:d",
]);

const workshopCards = parseShellContributions(
  [0, 1, 2, 3, 4, 5, 6].map((index) => ({
    package_id: `pkg/${index}`,
    surface: { id: `card-${index}`, slot: "workshop_card", metadata: { ...baseMetadata, order: index } },
  })),
  "workshop_card",
  "en",
).filter((item): item is WorkshopCardContribution => item.slot === "workshop_card");
assertEqual(limitHomeWorkshopCards(workshopCards).length, 4);
