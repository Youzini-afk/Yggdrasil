export {};

const importNode = new Function("specifier", "return import(specifier)") as (specifier: string) => Promise<Record<string, unknown>>;
const { readFileSync } = await importNode("node:fs") as { readFileSync(path: string, encoding: "utf8"): string };
const { resolve } = await importNode("node:path") as { resolve(...parts: string[]): string };
const { cwd } = await importNode("node:process") as { cwd(): string };

const bootstrap = readFileSync(resolve(cwd(), "public/surface-frame-bootstrap.js"), "utf8")
  .replace(/\r\n/g, "\n");

function assertContains(fragment: string) {
  if (!bootstrap.includes(fragment)) {
    throw new Error(`surface-frame-bootstrap.js must include ${fragment}`);
  }
}

assertContains("let bridgeToken = ''");
assertContains("globalThis.process ??= {};");
assertContains("globalThis.process.env ??= {};");
assertContains("globalThis.process.env.NODE_ENV ??= 'production';");
assertContains("bridge_token: bridgeToken");
assertContains("msg.bridge_token !== bridgeToken");
assertContains("if (e.source !== window.parent) return;");
assertContains("if (bridgeToken && msg.bridge_token !== bridgeToken) return;\n    pendingRpc.delete(msg.id);");
assertContains("if (mounted) return;");
assertContains("isAllowedAssetUrl(msg.bundleUrl)");
assertContains("code: 'invalid_bundle_url'");
assertContains("code: 'invalid_stylesheet_url'");
assertContains("type: 'mount.error'");
assertContains("postToHost({ type: 'rpc.call'");
assertContains("url.pathname.startsWith('/surface-bundles/')");
