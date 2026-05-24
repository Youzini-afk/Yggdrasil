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

export interface AssetRecord {
  id: string;
  origin_package_id: string;
  mime: string;
  hash: string;
  size_bytes: number;
  metadata: unknown;
}

export interface ProjectionRecord {
  id: string;
  session_id: string;
  source_kind_prefix?: string;
  state: unknown;
}

export interface ProposalRecord {
  id: string;
  status: string;
  target_session_id?: string;
  target_branch_id?: string;
  operations: unknown[];
  required_permissions: string[];
  expected_effects: unknown;
  result?: unknown;
}

export interface ProjectStorageSummary {
  data_bytes: number | null;
  cache_bytes: number | null;
  bundle_bytes: number | null;
  log_bytes: number | null;
  total_bytes: number | null;
  measured_at: string | null;
  measurement_state: "measured" | "unknown" | string;
}

export interface ProjectRecord {
  id: string;
  title: string;
  description?: string;
  type: "yggdrasil_native" | "external_wrapped" | "external_workspace";
  state: "installed" | "stopped" | "starting" | "running" | "stopping" | "failed" | "archived";
  icon?: string;
  entry_surface_id?: string;
  running_session_id?: string;
  storage_summary?: ProjectStorageSummary;
}

export class YggProtocolClient {
  constructor(private readonly baseUrl = "http://127.0.0.1:8787") {}

  invoke(method: string, params: unknown = {}) {
    return this.call(method, params);
  }

  async invokeWithSession(method: string, params: unknown = {}, sessionId: string): Promise<unknown> {
    const response = await fetch(`${this.baseUrl}/rpc`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ id: crypto.randomUUID(), method, params, session_id: sessionId }),
    });
    const envelope = (await response.json()) as ProtocolResponse<unknown>;
    if (envelope.error) {
      throw new Error(`${envelope.error.code}: ${envelope.error.message}`);
    }
    return envelope.result;
  }

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
    return this.call<PackageRecord[]>("kernel.v1.package.list");
  }

  capabilities() {
    return this.call<RegisteredCapability[]>("kernel.v1.capability.discover");
  }

  diagnostics() {
    return this.call<Record<string, unknown>>("kernel.v1.host.diagnostics");
  }

  assets() {
    return this.call<AssetRecord[]>("kernel.v1.asset.list");
  }

  projections() {
    return this.call<ProjectionRecord[]>("kernel.v1.projection.list");
  }

  proposals() {
    return this.call<ProposalRecord[]>("kernel.v1.proposal.list");
  }

  approveProposal(proposalId: string) {
    return this.call<ProposalRecord>("kernel.v1.proposal.approve", { proposal_id: proposalId, reason: "web-forge" });
  }

  applyProposal(proposalId: string) {
    return this.call<ProposalRecord>("kernel.v1.proposal.apply", { proposal_id: proposalId });
  }

  surfaceContributions(slot?: string) {
    return this.call<SurfaceContributionRecord[]>("kernel.v1.surface.contribution.list", slot ? { slot } : {});
  }

  describeSurface(surfaceId: string) {
    return this.call<SurfaceContributionRecord>("kernel.v1.surface.contribution.describe", { surface_id: surfaceId });
  }

  async listProjects(): Promise<ProjectRecord[]> {
    const result = await this.invoke("kernel.v1.project.list", {});
    return (result as { projects: ProjectRecord[] }).projects;
  }

  async getProject(projectId: string): Promise<ProjectRecord & { state_details?: Record<string, unknown>; paths?: Record<string, unknown> }> {
    const result = await this.invoke("kernel.v1.project.get", { project_id: projectId });
    const descriptor = result as {
      project?: Omit<ProjectRecord, "type" | "state" | "storage_summary"> & { type?: ProjectRecord["type"] };
      state?: ProjectRecord["state"];
      paths?: Record<string, unknown>;
      running_session_id?: string;
      storage_summary?: ProjectStorageSummary;
    };
    return {
      ...(descriptor.project as ProjectRecord),
      state: descriptor.state ?? "installed",
      paths: descriptor.paths,
      running_session_id: descriptor.running_session_id,
      storage_summary: descriptor.storage_summary,
    };
  }

  async startProject(projectId: string): Promise<{
    project_id: string;
    previous_state: string;
    new_state: string;
    session_id: string;
    already_running: boolean;
  }> {
    return await this.invoke("kernel.v1.project.start", { project_id: projectId }) as {
      project_id: string;
      previous_state: string;
      new_state: string;
      session_id: string;
      already_running: boolean;
    };
  }

  async stopProject(projectId: string): Promise<{ project_id: string; previous_state: string; new_state: string; session_id?: string }> {
    return await this.invoke("kernel.v1.project.stop", { project_id: projectId }) as { project_id: string; previous_state: string; new_state: string; session_id?: string };
  }

  async getProjectStatus(projectId: string): Promise<{
    project_id: string;
    state: string;
    sessions_count: number;
    secrets_count: number;
    storage_summary?: ProjectStorageSummary;
  }> {
    return await this.invoke("kernel.v1.project.status", { project_id: projectId }) as {
      project_id: string;
      state: string;
      sessions_count: number;
      secrets_count: number;
      storage_summary?: ProjectStorageSummary;
    };
  }

  openSession(labels: string[] = [], metadata: Record<string, unknown> = {}) {
    return this.call<{ id: string }>("kernel.v1.session.open", { labels, metadata });
  }

  forkSession(parentSessionId: string, forkedFromSequence: number, metadata: Record<string, unknown> = {}) {
    return this.call<{ id: string }>("kernel.v1.session.fork", {
      parent_session_id: parentSessionId,
      forked_from_sequence: forkedFromSequence,
      metadata,
    });
  }

  invokeCapability(capabilityId: string, input: unknown, providerPackageId?: string) {
    return this.call("kernel.v1.capability.invoke", {
      capability_id: capabilityId,
      input,
      ...(providerPackageId ? { provider_package_id: providerPackageId } : {}),
    });
  }

  listEvents(sessionId: string) {
    return this.call<KernelEvent[]>("kernel.v1.event.list", { session_id: sessionId, limit: 50 });
  }

  subscribeEvents(sessionId: string | undefined, onEvent: (event: KernelEvent) => void) {
    const targetSession = sessionId ?? "kernel_project_lifecycle";
    const source = new EventSource(`${this.baseUrl}/kernel/v1/event.subscribe/${encodeURIComponent(targetSession)}`);
    source.addEventListener("kernel.v1.event", (message) => onEvent(JSON.parse((message as MessageEvent).data)));
    return () => source.close();
  }

  /* ────────────────────────────────────────────────────────────────
     Secret store — wraps `official/secret-store-lab` capabilities.
     The host injects raw values via secret_ref; the UI never reads
     raw secret values.
     ──────────────────────────────────────────────────────────────── */

  async secretsHealth(): Promise<{
    store_path: string;
    exists: boolean;
    secret_count: number;
    key_source: string;
  }> {
    return (await this.invokeCapability("official/secret-store-lab/health", {})) as {
      store_path: string;
      exists: boolean;
      secret_count: number;
      key_source: string;
    };
  }

  async listSecrets(projectId?: string): Promise<string[]> {
    if (projectId) {
      const result = (await this.invokeCapability("official/secret-store-lab/list_project_secrets", {
        project_id: projectId,
      })) as { names: string[] };
      return result.names ?? [];
    }
    const result = (await this.invokeCapability("official/secret-store-lab/list_secrets", {})) as {
      names: string[];
    };
    return result.names ?? [];
  }

  async putSecret(name: string, value: string, projectId?: string): Promise<{ created: boolean }> {
    const capability = projectId
      ? "official/secret-store-lab/put_project_secret"
      : "official/secret-store-lab/put_secret";
    const params = projectId ? { project_id: projectId, name, value } : { name, value };
    const result = (await this.invokeCapability(capability, params)) as { created: boolean };
    return { created: result.created };
  }

  async deleteSecret(name: string, projectId?: string): Promise<{ removed: boolean }> {
    const capability = projectId
      ? "official/secret-store-lab/delete_project_secret"
      : "official/secret-store-lab/delete_secret";
    const params = projectId ? { project_id: projectId, name } : { name };
    const result = (await this.invokeCapability(capability, params)) as { removed: boolean };
    return { removed: result.removed };
  }
}
