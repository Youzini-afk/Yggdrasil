import { isValidProjectId, parseHash, parseProjectPath, projectPath, serializeRoute } from "./router";

function assertDeepEqual(actual: unknown, expected: unknown) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

function assertEqual<T>(actual: T, expected: T) {
  if (actual !== expected) throw new Error(`expected ${String(expected)}, got ${String(actual)}`);
}

assertDeepEqual(parseHash("#/"), { kind: "home" });
assertDeepEqual(parseHash("#/settings/storage"), { kind: "settings", tab: "storage" });
assertDeepEqual(parseHash("#/project/youzini-afk__YdlTavern__2a47e5c"), {
  kind: "project",
  projectId: "youzini-afk__YdlTavern__2a47e5c",
});
assertDeepEqual(parseHash("#/project/%"), { kind: "home" });
assertDeepEqual(parseHash("#/project/bad%2Fid"), { kind: "home" });

assertEqual(serializeRoute({ kind: "settings", tab: "about" }), "#/settings/about");
assertEqual(projectPath("demo.project-1"), "/project/demo.project-1");
assertDeepEqual(parseProjectPath("/project/demo.project-1"), { kind: "project", projectId: "demo.project-1" });
assertDeepEqual(parseProjectPath("/project/bad%2Fid"), null);
assertDeepEqual(parseProjectPath("/project/demo/extra"), null);
assertEqual(isValidProjectId("demo_project-1.x"), true);
assertEqual(isValidProjectId("bad/id"), false);
assertEqual(isValidProjectId(".."), false);
assertEqual(isValidProjectId("a..b"), false);
assertEqual(isValidProjectId(".hidden"), false);
assertEqual(isValidProjectId("foo:bar"), false);
assertEqual(isValidProjectId("foo@bar"), false);
assertEqual(isValidProjectId(""), false);
