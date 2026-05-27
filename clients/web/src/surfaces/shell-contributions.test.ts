import {
  parseShellContribution,
  parseShellContributions,
  type ShellContributionSlot,
} from "./shell-contributions";

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
    title: { de: "", en: "English", "zh-CN": "中文" },
  }),
  "quick_action",
  "de",
);
assertEqual(enFallback?.title, "English");

assertEqual(parseShellContribution(contribution({ title: { en: "Legacy" } }), "quick_action", "en"), null);
assertEqual(parseShellContribution(contribution({ shell_schema_version: 1, description: { en: "No title" } }), "quick_action", "en"), null);
assertEqual(parseShellContribution(contribution({ ...baseMetadata, title: { en: "   " } }), "quick_action", "en"), null);

const unknownIcon = parseShellContribution(contribution({ ...baseMetadata, icon_hint: "mystery" }), "quick_action", "en");
assertEqual(unknownIcon?.iconHint, "mystery");

assertEqual(parseShellContribution(contribution({ ...baseMetadata, icon_hint: "https://example.test/icon.svg" }), "quick_action", "en"), null);

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
