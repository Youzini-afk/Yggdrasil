import {
  BROWSER_ACCESS_TOKEN_STORAGE_KEY,
  clearBrowserAccessToken,
  ProtocolHttpError,
  resolveBrowserAccessToken,
  storeBrowserAccessToken,
  YggProtocolClient,
} from "./client";

function assertEqual<T>(actual: T, expected: T) {
  if (actual !== expected) {
    throw new Error(`expected ${String(expected)}, got ${String(actual)}`);
  }
}

function assertDeepEqual(actual: unknown, expected: unknown) {
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {
    throw new Error(`expected ${expectedJson}, got ${actualJson}`);
  }
}

async function rejectsWithHttpStatus(promise: Promise<unknown>, status: number) {
  try {
    await promise;
  } catch (err: unknown) {
    if (err instanceof ProtocolHttpError) {
      assertEqual(err.status, status);
      assertEqual(err.isAuthError, status === 401);
      return;
    }
    throw err;
  }
  throw new Error(`expected rejection with HTTP status ${status}`);
}

class MemoryStorage {
  private values = new Map<string, string>();

  getItem(key: string): string | null {
    return this.values.get(key) ?? null;
  }

  setItem(key: string, value: string): void {
    this.values.set(key, value);
  }

  removeItem(key: string): void {
    this.values.delete(key);
  }
}

function installWindow(search: string) {
  const storage = new MemoryStorage();
  let replacedUrl = "";

  Object.defineProperty(globalThis, "window", {
    configurable: true,
    value: {
      location: {
        origin: "http://web.test",
        pathname: "/app",
        search,
        hash: "#frag",
      },
      localStorage: storage,
      history: {
        state: { ok: true },
        replaceState: (_state: unknown, _title: string, url: string) => {
          replacedUrl = url;
        },
      },
    },
  });

  return {
    storage,
    replacedUrl: () => replacedUrl,
  };
}

const { storage, replacedUrl } = installWindow("?foo=bar&ygg_token=query-token&access_token=ignored");
assertEqual(resolveBrowserAccessToken(), "query-token");
assertEqual(storage.getItem(BROWSER_ACCESS_TOKEN_STORAGE_KEY), null);
assertEqual(replacedUrl(), "/app?foo=bar#frag");

storeBrowserAccessToken("valid-token");
assertEqual(storage.getItem(BROWSER_ACCESS_TOKEN_STORAGE_KEY), "valid-token");
clearBrowserAccessToken();
assertEqual(storage.getItem(BROWSER_ACCESS_TOKEN_STORAGE_KEY), null);

const secondWindow = installWindow("");
secondWindow.storage.setItem(BROWSER_ACCESS_TOKEN_STORAGE_KEY, "stored-token");
assertEqual(resolveBrowserAccessToken(), "stored-token");
assertEqual(secondWindow.replacedUrl(), "");

const thirdWindow = installWindow("?access_token=bad-token");
assertEqual(resolveBrowserAccessToken(), "bad-token");
clearBrowserAccessToken();
assertEqual(thirdWindow.storage.getItem(BROWSER_ACCESS_TOKEN_STORAGE_KEY), null);

const originalFetch = globalThis.fetch;
const originalCrypto = globalThis.crypto;

Object.defineProperty(globalThis, "crypto", {
  configurable: true,
  value: { randomUUID: () => "request-id" },
});

globalThis.fetch = (async () =>
  new Response("missing token", {
    status: 401,
    statusText: "Unauthorized",
  })) as typeof fetch;

await rejectsWithHttpStatus(new YggProtocolClient("http://host.test", "bad-token").diagnostics(), 401);

const capturedRequests: unknown[] = [];
globalThis.fetch = (async (_input: RequestInfo | URL, init?: RequestInit) => {
  const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined;
  capturedRequests.push(body);

  if (body?.method === "kernel.v1.session.open") {
    return Response.json({ id: body.id, result: { id: "install-session" } });
  }

  if (body?.method === "kernel.v1.capability.invoke") {
    return Response.json({
      id: body.id,
      result: {
        capability_id: body.params.capability_id,
        correlation_id: "corr-1",
        duration_ms: 1,
        provider_package_id: body.params.provider_package_id,
        output: {
          plan: {
            root_id: "official/test-project",
            packages: [],
            permissions_summary: {
              new_capabilities: [],
              new_network_hosts: [],
              new_secret_refs: [],
            },
            signature_summary: {
              all_signed: false,
              unsigned_packages: [],
            },
            integrity_summary: {
              manifest_hashes_match_lockfile: true,
              drift_detected: [],
            },
          },
        },
      },
    });
  }

  throw new Error(`unexpected method ${body?.method}`);
}) as typeof fetch;

await new YggProtocolClient("http://host.test", "valid-token").resolveInstallPlan({
  root_url: "https://github.com/Youzini-afk/Yggdrasil-Tavern",
});

const sessionOpenRequest = capturedRequests[0] as { method?: string; params?: Record<string, unknown> };
assertEqual(sessionOpenRequest.method, "kernel.v1.session.open");
assertDeepEqual(sessionOpenRequest.params?.active_package_set, ["official/install-lab"]);
assertDeepEqual(sessionOpenRequest.params?.labels, ["install", "official/install-lab"]);

globalThis.fetch = originalFetch;
Object.defineProperty(globalThis, "crypto", {
  configurable: true,
  value: originalCrypto,
});
