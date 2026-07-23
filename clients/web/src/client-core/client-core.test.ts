import {
  BROWSER_ACCESS_TOKEN_STORAGE_KEY,
  BrowserCredentialProvider,
  PendingCredentialLease,
} from "./credentials";
import { PROJECT_SHELL_HISTORY_STATE, shouldReturnToShellHistory } from "./platform-adapter";
import { normalizeHostBaseUrl, resolveHostBaseUrl } from "./host-endpoint";

function assertEqual<T>(actual: T, expected: T) {
  if (actual !== expected) throw new Error(`expected ${String(expected)}, got ${String(actual)}`);
}

class MemoryStorage {
  values = new Map<string, string>();
  getItem(key: string) { return this.values.get(key) ?? null; }
  setItem(key: string, value: string) { this.values.set(key, value); }
  removeItem(key: string) { this.values.delete(key); }
}

const storage = new MemoryStorage();
let replaced = "";
const credential = new BrowserCredentialProvider({
  localStorage: storage,
  location: {
    pathname: "/project/demo",
    search: "?keep=1&ygg_token=&access_token=fallback-token",
    hash: "#view",
  },
  history: {
    state: { preserved: true },
    replaceState: (_state, _title, url) => { replaced = String(url); },
  },
});

assertEqual(credential.consumeBootstrap(), "fallback-token");
assertEqual(replaced, "/project/demo?keep=1#view");
credential.write("stored-token");
assertEqual(storage.getItem(BROWSER_ACCESS_TOKEN_STORAGE_KEY), "stored-token");
assertEqual(credential.read(), "stored-token");
credential.clear();
assertEqual(credential.read(), undefined);

let bootstrapReads = 0;
const pending = new PendingCredentialLease({
  read: () => undefined,
  write: () => {},
  clear: () => {},
  consumeBootstrap: () => {
    bootstrapReads += 1;
    return bootstrapReads === 1 ? "one-time-token" : undefined;
  },
});
assertEqual(pending.resolve(), "one-time-token");
assertEqual(pending.resolve(), "one-time-token");
assertEqual(bootstrapReads, 1);
pending.clear();
assertEqual(pending.resolve(), undefined);

assertEqual(shouldReturnToShellHistory({ length: 5, state: null }), false);
assertEqual(shouldReturnToShellHistory({ length: 5, state: { [PROJECT_SHELL_HISTORY_STATE]: true } }), true);

assertEqual(normalizeHostBaseUrl("https://host.example///"), "https://host.example");
assertEqual(resolveHostBaseUrl("https://host.example/root/"), "https://host.example/root");
