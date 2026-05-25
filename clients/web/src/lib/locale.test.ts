import { chooseInitialLocale, lookupLabel, normalizeLocale } from "./locale";

function assertEqual<T>(actual: T, expected: T) {
  if (actual !== expected) {
    throw new Error(`expected ${String(expected)}, got ${String(actual)}`);
  }
}

assertEqual(normalizeLocale("zh"), "zh-CN");
assertEqual(normalizeLocale("zh-Hans-CN"), "zh-CN");
assertEqual(normalizeLocale("zh_CN"), "zh-CN");
assertEqual(normalizeLocale("en-US"), "en");
assertEqual(normalizeLocale("ja"), null);

assertEqual(chooseInitialLocale({ saved: "en", browser: "zh-CN" }), "en");
assertEqual(chooseInitialLocale({ saved: "ja", browser: "zh-TW" }), "zh-CN");
assertEqual(chooseInitialLocale({ saved: null, browser: "fr-FR" }), "zh-CN");
assertEqual(chooseInitialLocale({ saved: null, browser: "fr-FR", fallback: "en" }), "en");

assertEqual(lookupLabel("zh-CN", "languageName"), "简体中文");
assertEqual(lookupLabel("en", "topbarProject", "demo"), "Projects / demo");
