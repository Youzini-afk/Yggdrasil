import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const bootstrap = readFileSync(resolve(process.cwd(), "public/surface-frame-bootstrap.js"), "utf8");

function assertContains(fragment: string) {
  if (!bootstrap.includes(fragment)) {
    throw new Error(`surface-frame-bootstrap.js must include ${fragment}`);
  }
}

assertContains("let bridgeToken = ''");
assertContains("bridge_token: bridgeToken");
assertContains("msg.bridge_token !== bridgeToken");
assertContains("type: 'mount.error'");
assertContains("postToHost({ type: 'rpc.call'");
