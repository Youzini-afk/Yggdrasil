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

export interface SurfaceActivation {
  launch_capability_id?: string;
  session_template?: Record<string, unknown>;
  input_schema?: unknown;
}

export interface SurfacePermissionRequirement {
  permission: string;
  scope?: string;
  reason?: string;
  risk: "low" | "medium" | "high";
}

export interface SurfaceContribution {
  id: string;
  version: string;
  slot: string;
  title: string;
  description?: string;
  capability_id?: string;
  activation: SurfaceActivation;
  required_permissions: SurfacePermissionRequirement[];
  approval_policy?: "none" | "user_approval" | "fork_then_approve";
  metadata: Record<string, unknown>;
}

export interface SurfaceContributionRecord {
  package_id: string;
  entry_kind: string;
  package_state: string;
  surface: SurfaceContribution;
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

  surfaceContributions(slot?: string) {
    return this.call<SurfaceContributionRecord[]>("kernel.surface.contribution.list", slot ? { slot } : {});
  }

  describeSurface(surfaceId: string) {
    return this.call<SurfaceContributionRecord>("kernel.surface.contribution.describe", { surface_id: surfaceId });
  }

  openSession(labels: string[] = [], metadata: Record<string, unknown> = {}) {
    return this.call<{ id: string }>("kernel.session.open", { labels, metadata });
  }

  forkSession(parentSessionId: string, forkedFromSequence: number, metadata: Record<string, unknown> = {}) {
    return this.call<{ id: string }>("kernel.session.fork", {
      parent_session_id: parentSessionId,
      forked_from_sequence: forkedFromSequence,
      metadata,
    });
  }

  invokeCapability(capabilityId: string, input: unknown, providerPackageId?: string) {
    return this.call("kernel.capability.invoke", {
      capability_id: capabilityId,
      input,
      ...(providerPackageId ? { provider_package_id: providerPackageId } : {}),
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
