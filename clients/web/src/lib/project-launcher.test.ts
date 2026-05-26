import { openProjectInTab, projectTabTargetName, type ProjectTabWindow } from "./project-launcher";

function assertEqual<T>(actual: T, expected: T) {
  if (actual !== expected) throw new Error(`expected ${String(expected)}, got ${String(actual)}`);
}

function assertOk(value: unknown, message: string) {
  if (!value) throw new Error(message);
}

const target = projectTabTargetName("project/with/slash");
assertOk(/^ygg-project-[a-z0-9]+$/.test(target), "target name must be sanitized");
assertOk(!target.includes("project/with/slash"), "target must not include raw project id");

let openArgs: [string, string, string | undefined] | null = null;
let assigned = "";
const opened = { opener: {} } as Window;
const hostWindow: ProjectTabWindow = {
  open(url, targetName, features) {
    openArgs = [url, targetName, features];
    return opened;
  },
  location: {
    assign(url: string) {
      assigned = url;
    },
  },
};

assertEqual(openProjectInTab("demo-project", hostWindow), true);
assertEqual(openArgs?.[0], "/project/demo-project");
assertEqual(openArgs?.[2], "noopener,noreferrer");
assertEqual(opened.opener, null);
assertEqual(assigned, "");

const popupBlocked: ProjectTabWindow = {
  open() {
    return null;
  },
  location: {
    assign(url: string) {
      assigned = url;
    },
  },
};

assertEqual(openProjectInTab("fallback-project", popupBlocked), false);
assertEqual(assigned, "");
assertEqual(openProjectInTab("bad/id", popupBlocked), false);
