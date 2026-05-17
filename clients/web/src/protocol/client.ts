export interface ProtocolResponse<T = unknown> {
  id: string;
  result?: T;
  error?: { code: string; message: string; details?: unknown };
}

export interface PackageRecord {
  id: string;
  version: string;
  state: string;
  entry_kind: string;
  capability_count: number;
  hook_count: number;
}

export interface RegisteredCapability {
  capability_id: string;
  provider_package_id: string;
  version: string;
  streaming: boolean;
}

export interface KernelEvent {
  id: string;
  session_id: string;
  sequence: number;
  writer_package_id: string;
  kind: string;
  payload: unknown;
  metadata: unknown;
  created_at: string;
}

export class YggProtocolClient {
  constructor(private readonly baseUrl = "http://127.0.0.1:8787") {}

  async call<T>(method: string, params: unknown = {}): Promise<T> {
    const response = await fetch(`${this.baseUrl}/rpc`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ id: crypto.randomUUID(), method, params }),
    });
    const envelope = (await response.json()) as ProtocolResponse<T>;
    if (envelope.error) {
      throw new Error(`${envelope.error.code}: ${envelope.error.message}`);
    }
    return envelope.result as T;
  }

  packages() {
    return this.call<PackageRecord[]>("kernel.package.list");
  }

  capabilities() {
    return this.call<RegisteredCapability[]>("kernel.capability.discover");
  }

  diagnostics() {
    return this.call<Record<string, unknown>>("kernel.host.diagnostics");
  }

  openSession() {
    return this.call<{ id: string }>("kernel.session.open", {
      labels: ["forge-shell"],
      metadata: { surface: "forge" },
    });
  }

  listEvents(sessionId: string) {
    return this.call<KernelEvent[]>("kernel.event.list", { session_id: sessionId, limit: 50 });
  }

  subscribeEvents(sessionId: string, onEvent: (event: KernelEvent) => void) {
    const source = new EventSource(`${this.baseUrl}/kernel/event.subscribe/${encodeURIComponent(sessionId)}`);
    source.addEventListener("kernel.event", (message) => onEvent(JSON.parse((message as MessageEvent).data)));
    return () => source.close();
  }
}
