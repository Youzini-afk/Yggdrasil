export interface ProtocolResponse<T = unknown> {
  id: string;
  result?: T;
  error?: { code: string; message: string; details?: unknown };
}

export const BROWSER_ACCESS_TOKEN_STORAGE_KEY = "ygg_http_access_token";

export class ProtocolHttpError extends Error {
  constructor(
    readonly status: number,
    readonly body: string,
  ) {
    super(`${status}: ${body || "HTTP error"}`);
    this.name = "ProtocolHttpError";
  }

  get isAuthError(): boolean {
    return this.status === 401;
  }
}

export function readBrowserAccessToken(): string | undefined {
  if (typeof window === "undefined") return undefined;

  try {
    return window.localStorage.getItem(BROWSER_ACCESS_TOKEN_STORAGE_KEY) ?? undefined;
  } catch {
    return undefined;
  }
}

export function storeBrowserAccessToken(token: string): void {
  if (typeof window === "undefined") return;

  try {
    window.localStorage.setItem(BROWSER_ACCESS_TOKEN_STORAGE_KEY, token);
  } catch {
    // Storage can be disabled in locked-down browsers; auth still works in memory.
  }
}

export function clearBrowserAccessToken(): void {
  if (typeof window === "undefined") return;

  try {
    window.localStorage.removeItem(BROWSER_ACCESS_TOKEN_STORAGE_KEY);
  } catch {
    // Best effort only.
  }
}

export function resolveBrowserAccessToken(): string | undefined {
  if (typeof window === "undefined") return undefined;

  try {
    const params = new URLSearchParams(window.location.search);
    const fromQuery = params.get("ygg_token") ?? params.get("access_token");
    if (fromQuery) {
      scrubTokenFromBrowserUrl(params);
      return fromQuery;
    }
  } catch {
    return undefined;
  }

  return readBrowserAccessToken();
}

export interface CapabilityInvocationResult<TOutput = unknown> {
  capability_id: string;
  correlation_id: string;
  duration_ms: number;
  output: TOutput;
  provider_package_id: string;
}

export interface InstallSource {
  root_url: string;
  root_ref?: string;
  lockfile?: string;
  require_signed?: boolean;
  strict_conformance?: boolean;
}

export interface InstallPlan {
  root_id: string;
  packages: InstallPlannedPackage[];
  permissions_summary: InstallPermissionsSummary;
  signature_summary: InstallSignatureSummary;
  integrity_summary: InstallIntegritySummary;
}

export interface InstallPlannedPackage {
  id: string;
  version: string;
  source: string;
  url?: string;
  ref?: string;
  path?: string;
  commit_sha?: string;
  manifest_hash: string;
  tree_hash: string;
  signed: boolean;
  signed_by?: string;
  permissions: {
    capabilities_invoke?: string[];
    network_hosts?: string[];
    secret_refs?: string[];
  };
  requires?: Array<{ id: string; source: unknown; version?: string }>;
  conformance?: InstallConformanceReport;
}

export interface InstallConformanceReport {
  passed?: boolean;
  checks?: Array<{ id?: string; status?: string; passed?: boolean; message?: string }>;
  failures?: unknown[];
  warnings?: unknown[];
  [key: string]: unknown;
}

export interface InstallPermissionsSummary {
  new_capabilities: string[];
  new_network_hosts: string[];
  new_secret_refs: string[];
}

export interface InstallSignatureSummary {
  all_signed: boolean;
  unsigned_packages: string[];
}

export interface InstallIntegritySummary {
  manifest_hashes_match_lockfile: boolean;
  drift_detected: unknown[];
}

export type InstallDetectedKind =
  | { kind: "native"; descriptor?: unknown }
  | { kind: "declared_external"; descriptor?: unknown }
  | { kind: "external"; has_manifest_yaml?: boolean };

export interface InstallConsent {
  approved_capabilities: string[];
  approved_network_hosts: string[];
  approved_secret_refs: string[];
}

export interface InstallExecuteResult {
  installed: Array<{ id: string }>;
  lockfile: string;
  project?: { project_id?: string } | null;
}

export interface InstallUninstallResult {
  removed_from_profile: boolean;
  store_path_orphaned?: string | null;
  store_paths_orphaned?: string[];
  project?: { project_id: string; data_action: string } | null;
}

const INSTALL_LAB_PROVIDER = "official/install-lab";
const INSTALL_LAB_CAPABILITIES = {
  resolvePlan: `${INSTALL_LAB_PROVIDER}/resolve_plan`,
  detectKind: `${INSTALL_LAB_PROVIDER}/detect_kind`,
  executePlan: `${INSTALL_LAB_PROVIDER}/execute_plan`,
  uninstall: `${INSTALL_LAB_PROVIDER}/uninstall`,
} as const;

function normalizeInstallRootUrl(input: string): string {
  const trimmed = input.trim();
  if (!trimmed) return trimmed;
  if (trimmed.startsWith("~")) {
    throw new Error("Home-relative paths are not accepted from the web UI. Use an absolute path or HTTPS Git URL.");
  }
  if (/^[\w.-]+\/[\w./-]+(?:\.git)?(?:#.+)?$/.test(trimmed) && !trimmed.includes("://")) {
    return `https://${trimmed}`;
  }
  return trimmed;
}

export interface PackageRecord {
  id: string;
  version: string;
  state: string;
  entry_kind: string;
  capability_count: number;
  hook_count: number;
  last_failure?: PackageFailureSummary;
}

export interface PackageFailureSummary {
  package_id: string;
  reason: string;
  exit_code?: string | null;
  signal?: string | null;
  failed_at: string;
  stderr_tail_redacted: string[];
  log_tail_redacted: SubprocessLogLine[];
  stderr_truncated: boolean;
  redaction_state: "redacted" | "safe" | "not_captured" | "policy_ref" | "unsafe_blocked";
  state: string;
}

export interface SubprocessLogLine {
  package_id: string;
  stream: string;
  line: string;
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
  allowed_capability_ids?: string[];
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
  packages?: string[];
}

export class YggProtocolClient {
  private readonly accessToken?: string;

  constructor(private readonly baseUrl = "http://127.0.0.1:8787", accessToken?: string | null) {
    this.accessToken = accessToken === undefined ? resolveBrowserAccessToken() : accessToken || undefined;
  }

  invoke(method: string, params: unknown = {}) {
    return this.call(method, params);
  }

  async invokeWithSession(method: string, params: unknown = {}, sessionId: string): Promise<unknown> {
    const response = await fetch(`${this.baseUrl}/rpc`, {
      method: "POST",
      headers: this.rpcHeaders(),
      body: JSON.stringify({ id: crypto.randomUUID(), method, params, session_id: sessionId }),
    });
    await throwForHttpError(response);
    const envelope = (await response.json()) as ProtocolResponse<unknown>;
    if (envelope.error) {
      throw new Error(`${envelope.error.code}: ${envelope.error.message}`);
    }
    return envelope.result;
  }

  async call<T>(method: string, params: unknown = {}): Promise<T> {
    const response = await fetch(`${this.baseUrl}/rpc`, {
      method: "POST",
      headers: this.rpcHeaders(),
      body: JSON.stringify({ id: crypto.randomUUID(), method, params }),
    });
    await throwForHttpError(response);
    const envelope = (await response.json()) as ProtocolResponse<T>;
    if (envelope.error) {
      throw new Error(`${envelope.error.code}: ${envelope.error.message}`);
    }
    return envelope.result as T;
  }

  packages() {
    return this.call<PackageRecord[]>("kernel.v1.package.list");
  }

  packageStatus(packageId: string) {
    return this.call<PackageRecord>("kernel.v1.package.status", { package_id: packageId });
  }

  packageLogs(packageId: string) {
    return this.call<SubprocessLogLine[]>("kernel.v1.package.logs", { package_id: packageId });
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

  async getProject(projectId: string): Promise<ProjectRecord & { state_details?: Record<string, unknown>; packages?: string[] }> {
    const result = await this.invoke("kernel.v1.project.get", { project_id: projectId });
    const descriptor = result as {
      project?: Omit<ProjectRecord, "type" | "state" | "storage_summary"> & { type?: ProjectRecord["type"]; packages?: string[] };
      state?: ProjectRecord["state"];
      running_session_id?: string;
      storage_summary?: ProjectStorageSummary;
    };
    return {
      ...(descriptor.project as ProjectRecord),
      state: descriptor.state ?? "installed",
      running_session_id: descriptor.running_session_id,
      storage_summary: descriptor.storage_summary,
      packages: descriptor.project?.packages,
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

  openSession(labels: string[] = [], metadata: Record<string, unknown> = {}, activePackageSet: string[] = []) {
    return this.call<{ id: string }>("kernel.v1.session.open", {
      active_package_set: activePackageSet,
      labels,
      metadata,
    });
  }

  forkSession(parentSessionId: string, forkedFromSequence: number, metadata: Record<string, unknown> = {}) {
    return this.call<{ id: string }>("kernel.v1.session.fork", {
      parent_session_id: parentSessionId,
      forked_from_sequence: forkedFromSequence,
      metadata,
    });
  }

  invokeCapability<TOutput = unknown>(
    capabilityId: string,
    input: unknown,
    providerPackageId?: string,
    sessionId?: string,
  ): Promise<CapabilityInvocationResult<TOutput>> {
    return this.call("kernel.v1.capability.invoke", {
      capability_id: capabilityId,
      input,
      ...(providerPackageId ? { provider_package_id: providerPackageId } : {}),
      ...(sessionId ? { session_id: sessionId } : {}),
    });
  }

  private async invokeInstallLab<TOutput>(capabilityId: string, input: unknown): Promise<TOutput> {
    const session = await this.openSession(["install", "official/install-lab"], {
      source: "clients/web",
      capability_id: capabilityId,
    }, [INSTALL_LAB_PROVIDER]);
    const result = await this.invokeCapability<TOutput>(capabilityId, input, INSTALL_LAB_PROVIDER, session.id);
    return result.output;
  }

  async resolveInstallPlan(source: InstallSource): Promise<InstallPlan> {
    const rootUrl = normalizeInstallRootUrl(source.root_url);
    if (!/^https:\/\//i.test(rootUrl)) {
      throw new Error("The web install flow accepts public HTTPS Git URLs only. Use the CLI for local folders.");
    }
    const output = await this.invokeInstallLab<{ plan: InstallPlan }>(INSTALL_LAB_CAPABILITIES.resolvePlan, {
      root_url: rootUrl,
      root_ref: source.root_ref ?? "HEAD",
      ...(source.lockfile ? { lockfile: source.lockfile } : {}),
      ...(source.require_signed !== undefined ? { require_signed: source.require_signed } : {}),
      ...(source.strict_conformance !== undefined ? { strict_conformance: source.strict_conformance } : {}),
    });
    return output.plan;
  }

  async detectInstallKind(source: Pick<InstallSource, "root_url" | "root_ref">): Promise<InstallDetectedKind> {
    const rootUrl = normalizeInstallRootUrl(source.root_url);
    if (!/^https:\/\//i.test(rootUrl)) {
      throw new Error("The web install flow accepts public HTTPS Git URLs only. Use the CLI for local folders.");
    }
    const isLocalSource = false;
    return await this.invokeInstallLab<InstallDetectedKind>(INSTALL_LAB_CAPABILITIES.detectKind, {
      [isLocalSource ? "path" : "url"]: rootUrl,
      root_ref: source.root_ref ?? "HEAD",
    });
  }

  async executeInstallPlan(
    plan: InstallPlan,
    consent: InstallConsent = {
      approved_capabilities: plan.permissions_summary.new_capabilities,
      approved_network_hosts: plan.permissions_summary.new_network_hosts,
      approved_secret_refs: plan.permissions_summary.new_secret_refs,
    },
    profile = "default",
  ): Promise<InstallExecuteResult> {
    return await this.invokeInstallLab<InstallExecuteResult>(INSTALL_LAB_CAPABILITIES.executePlan, {
      plan,
      consent,
      profile,
    });
  }

  async uninstallProject(projectId: string, profile = "default"): Promise<InstallUninstallResult> {
    return await this.invokeInstallLab<InstallUninstallResult>(INSTALL_LAB_CAPABILITIES.uninstall, {
      project_id: projectId,
      profile,
      delete_project_data: false,
    });
  }

  listEvents(sessionId: string) {
    return this.call<KernelEvent[]>("kernel.v1.event.list", { session_id: sessionId, limit: 50 });
  }

  subscribeEvents(sessionId: string | undefined, onEvent: (event: KernelEvent) => void) {
    const targetSession = sessionId ?? "kernel_project_lifecycle";
    const source = new EventSource(this.eventSubscribeUrl(targetSession));
    source.addEventListener("kernel.v1.event", (message) => onEvent(JSON.parse((message as MessageEvent).data)));
    return () => source.close();
  }

  private rpcHeaders(): Record<string, string> {
    return {
      "content-type": "application/json",
      ...(this.accessToken ? { authorization: `Bearer ${this.accessToken}` } : {}),
    };
  }

  private eventSubscribeUrl(sessionId: string): string {
    const url = new URL(`${this.baseUrl}/kernel/v1/event.subscribe/${encodeURIComponent(sessionId)}`);
    if (this.accessToken) {
      url.searchParams.set("access_token", this.accessToken);
    }
    return url.toString();
  }

  /* ────────────────────────────────────────────────────────────────
     Secret store — wraps `official/secret-store-lab` capabilities.
     The host injects raw values via secret_ref; the UI never reads
     raw secret values.
     ──────────────────────────────────────────────────────────────── */

  async secretsHealth(): Promise<{
    exists: boolean;
    secret_count: number;
    key_source: string;
  }> {
    return (await this.invokeCapability<{
      exists: boolean;
      secret_count: number;
      key_source: string;
    }>("official/secret-store-lab/health", {})).output;
  }

  async listSecrets(projectId?: string): Promise<string[]> {
    if (projectId) {
      const result = (await this.invokeCapability<{ names: string[] }>("official/secret-store-lab/list_project_secrets", {
        project_id: projectId,
      })).output;
      return result.names ?? [];
    }
    const result = (await this.invokeCapability<{ names: string[] }>("official/secret-store-lab/list_secrets", {})).output;
    return result.names ?? [];
  }

  async putSecret(name: string, value: string, projectId?: string): Promise<{ created: boolean }> {
    const capability = projectId
      ? "official/secret-store-lab/put_project_secret"
      : "official/secret-store-lab/put_secret";
    const params = projectId ? { project_id: projectId, name, value } : { name, value };
    const result = (await this.invokeCapability<{ created: boolean }>(capability, params)).output;
    return { created: result.created };
  }

  async deleteSecret(name: string, projectId?: string): Promise<{ removed: boolean }> {
    const capability = projectId
      ? "official/secret-store-lab/delete_project_secret"
      : "official/secret-store-lab/delete_secret";
    const params = projectId ? { project_id: projectId, name } : { name };
    const result = (await this.invokeCapability<{ removed: boolean }>(capability, params)).output;
    return { removed: result.removed };
  }
}

async function throwForHttpError(response: Response): Promise<void> {
  if (response.ok) return;

  const body = await response.text().catch(() => response.statusText || "HTTP error");
  throw new ProtocolHttpError(response.status, body);
}

function scrubTokenFromBrowserUrl(params: URLSearchParams) {
  if (typeof window === "undefined" || !window.history?.replaceState) return;
  if (!params.has("ygg_token") && !params.has("access_token")) return;

  params.delete("ygg_token");
  params.delete("access_token");
  const search = params.toString();
  const nextUrl = `${window.location.pathname}${search ? `?${search}` : ""}${window.location.hash}`;
  window.history.replaceState(window.history.state, "", nextUrl);
}
