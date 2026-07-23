interface PairingCredentialWindow {
  location: Pick<Location, "pathname" | "search" | "hash">;
  history: Pick<History, "state" | "replaceState">;
}

function currentWindow(): PairingCredentialWindow | undefined {
  return typeof window === "undefined" ? undefined : window;
}

/**
 * Keeps a scrubbed one-time pairing credential in memory across React
 * StrictMode remounts. The credential is never written to browser storage.
 */
export class PendingPairingCredentialLease {
  private consumed = false;
  private pending: string | undefined;

  constructor(
    private readonly browserWindow: PairingCredentialWindow | undefined = currentWindow(),
  ) {}

  resolve(): string | undefined {
    if (this.consumed) return this.pending;
    this.consumed = true;
    const hostWindow = this.browserWindow;
    if (!hostWindow) return undefined;
    try {
      const params = new URLSearchParams(hostWindow.location.search);
      const token = params.get("pairing_token")?.trim() || undefined;
      if (!params.has("pairing_token")) return undefined;
      params.delete("pairing_token");
      const search = params.toString();
      const cleanUrl = `${hostWindow.location.pathname}${search ? `?${search}` : ""}${hostWindow.location.hash}`;
      hostWindow.history.replaceState(hostWindow.history.state, "", cleanUrl);
      this.pending = token;
      return token;
    } catch {
      return undefined;
    }
  }

  clear(): void {
    this.pending = undefined;
  }
}
