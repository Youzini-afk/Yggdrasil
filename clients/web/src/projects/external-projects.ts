import type { RegisteredCapability } from "../protocol/client";
import { YggProtocolClient } from "../protocol/client";
import { escapeHtml, formatJsonPreview } from "../utils/html";

const PROJECT_INTAKE = "official/project-intake-lab";
const WORKSPACE_LAB = "official/workspace-lab";

const DEMO_SOURCE_REF = "https://github.com/example/ygg-unadapted-tool.git";
const DEMO_WORKSPACE_REF = "ws-fixture-ygg-unadapted-tool";

const DEMO_METADATA = {
  package_json: {
    name: "ygg-unadapted-tool",
    version: "0.1.0",
    scripts: {
      dev: "vite --host 127.0.0.1",
      build: "vite build",
      test: "vitest run",
      postinstall: "node scripts/prepare.js",
    },
  },
  files: ["package.json", "src/main.ts", "README.md"],
  readme: "Unadapted external project fixture. No Ygg manifest required for intake.",
};

export interface ExternalProjectAggregation {
  available: boolean;
  missing_capabilities: string[];
  demo_source_ref: string;
  demo_workspace_ref: string;
  intake?: Record<string, unknown>;
  stack?: Record<string, unknown>;
  risk?: Record<string, unknown>;
  workspace_plan?: Record<string, unknown>;
  workspace?: Record<string, unknown>;
  entrypoints?: Record<string, unknown>;
  run_plan?: Record<string, unknown>;
  patch?: Record<string, unknown>;
  adapter_plan?: Record<string, unknown>;
  adapter_manifest?: Record<string, unknown>;
  adapter_wrapper?: Record<string, unknown>;
  adapter_fixture?: Record<string, unknown>;
  adapter_readiness?: Record<string, unknown>;
  errors: string[];
}

export async function buildExternalProjectAggregation(
  client: YggProtocolClient,
  capabilities: RegisteredCapability[],
): Promise<ExternalProjectAggregation> {
  const required = [
    `${PROJECT_INTAKE}/inspect_external_project_ref`,
    `${PROJECT_INTAKE}/detect_project_stack_from_metadata`,
    `${PROJECT_INTAKE}/draft_workspace_plan`,
    `${PROJECT_INTAKE}/draft_security_risk_summary`,
    `${PROJECT_INTAKE}/draft_adapter_plan`,
    `${WORKSPACE_LAB}/create_fixture_workspace`,
    `${WORKSPACE_LAB}/discover_workspace_entrypoints`,
    `${WORKSPACE_LAB}/plan_workspace_run`,
    `${WORKSPACE_LAB}/draft_workspace_patch`,
  ];
  const e5Required = [
    `${PROJECT_INTAKE}/generate_adapter_manifest_preview`,
    `${PROJECT_INTAKE}/generate_subprocess_wrapper_preview`,
    `${PROJECT_INTAKE}/generate_adapter_fixture_preview`,
    `${PROJECT_INTAKE}/check_adapter_readiness`,
  ];
  const capabilitySet = new Set(capabilities.map((capability) => capability.capability_id));
  const missing = required.filter((capabilityId) => !capabilitySet.has(capabilityId));
  const e5Available = e5Required.every((capabilityId) => capabilitySet.has(capabilityId));
  const model: ExternalProjectAggregation = {
    available: missing.length === 0,
    missing_capabilities: missing,
    demo_source_ref: DEMO_SOURCE_REF,
    demo_workspace_ref: DEMO_WORKSPACE_REF,
    errors: [],
  };
  if (!model.available) return model;

  async function capture(key: keyof ExternalProjectAggregation, capabilityId: string, input: unknown, provider: string) {
    try {
      const result = await client.invokeCapability(capabilityId, input, provider);
      if (isRecord(result)) {
        model[key] = result as never;
      } else {
        model.errors.push(`${capabilityId} returned non-object output`);
      }
    } catch (caught) {
      model.errors.push(`${capabilityId}: ${caught instanceof Error ? caught.message : String(caught)}`);
    }
  }

  await Promise.all([
    capture("intake", `${PROJECT_INTAKE}/inspect_external_project_ref`, { source_ref: DEMO_SOURCE_REF }, PROJECT_INTAKE),
    capture("stack", `${PROJECT_INTAKE}/detect_project_stack_from_metadata`, { metadata: DEMO_METADATA }, PROJECT_INTAKE),
    capture("risk", `${PROJECT_INTAKE}/draft_security_risk_summary`, { source_ref: DEMO_SOURCE_REF, metadata: DEMO_METADATA }, PROJECT_INTAKE),
    capture("workspace_plan", `${PROJECT_INTAKE}/draft_workspace_plan`, { source_ref: DEMO_SOURCE_REF, source_kind: "git" }, PROJECT_INTAKE),
    capture("adapter_plan", `${PROJECT_INTAKE}/draft_adapter_plan`, { source_ref: DEMO_SOURCE_REF, metadata: DEMO_METADATA }, PROJECT_INTAKE),
    capture("workspace", `${WORKSPACE_LAB}/create_fixture_workspace`, { workspace_ref: DEMO_WORKSPACE_REF, source_ref: DEMO_SOURCE_REF, stack_hint: "node", metadata: DEMO_METADATA }, WORKSPACE_LAB),
    capture("entrypoints", `${WORKSPACE_LAB}/discover_workspace_entrypoints`, { workspace_ref: DEMO_WORKSPACE_REF, stack_hint: "node", metadata: DEMO_METADATA }, WORKSPACE_LAB),
    capture("run_plan", `${WORKSPACE_LAB}/plan_workspace_run`, { workspace_ref: DEMO_WORKSPACE_REF, stack_hint: "node", scripts: DEMO_METADATA.package_json.scripts }, WORKSPACE_LAB),
    capture("patch", `${WORKSPACE_LAB}/draft_workspace_patch`, { workspace_ref: DEMO_WORKSPACE_REF, target_files: ["src/main.ts"], patch_summary: "Add Ygg adapter seam placeholder" }, WORKSPACE_LAB),
  ]);

  // E5: adapter generation preview (optional, degrades gracefully)
  if (e5Available) {
    const adapterPkgId = "thirdparty/example-ygg-unadapted-tool-adapter";
    await Promise.all([
      capture("adapter_manifest", `${PROJECT_INTAKE}/generate_adapter_manifest_preview`, { source_ref: DEMO_SOURCE_REF, source_kind: "git", adapter_package_id: adapterPkgId, capability_name: "invoke", entry_kind: "subprocess" }, PROJECT_INTAKE),
      capture("adapter_wrapper", `${PROJECT_INTAKE}/generate_subprocess_wrapper_preview`, { source_ref: DEMO_SOURCE_REF, source_kind: "git", adapter_package_id: adapterPkgId, capability_name: "invoke", language: "typescript" }, PROJECT_INTAKE),
      capture("adapter_fixture", `${PROJECT_INTAKE}/generate_adapter_fixture_preview`, { adapter_package_id: adapterPkgId, capability_name: "invoke" }, PROJECT_INTAKE),
      capture("adapter_readiness", `${PROJECT_INTAKE}/check_adapter_readiness`, { adapter_package_id: adapterPkgId, capability_name: "invoke", has_manifest: true, has_wrapper: true, has_fixture: true, source_ref: DEMO_SOURCE_REF }, PROJECT_INTAKE),
    ]);
  }

  return model;
}

export function renderHomeExternalProjects(model?: ExternalProjectAggregation): string {
  if (!model) return renderUnavailable("External Project Operating Plane", ["model not loaded yet"]);
  if (!model.available) return renderUnavailable("External Project Operating Plane", model.missing_capabilities);
  return `
    <section class="rail project-aggregation-home" aria-label="External project operating plane">
      <div class="rail-header"><h2>External Project Operating Plane</h2><span>safe intake · no execution</span></div>
      <div class="experience-grid">
        <article class="experience-card project-card">
          <div class="card-glow"></div>
          <p class="eyebrow">External Project</p>
          <h3>Unadapted Git project</h3>
          <p>${escapeHtml(model.demo_source_ref)}</p>
          <div class="surface-chip-row">
            ${badge(readText(model.intake, "source_kind") ?? "git")}
            ${badge(readText(model.stack, "detected_stack") ?? "unknown")}
            ${badge("no execution")}
            ${badge("proposal gated")}
          </div>
          <p class="guidance-note">Yggdrasil can inspect, plan, risk-score, and wrap a project before it becomes a package.</p>
          <button type="button" data-route="forge">Open in Forge</button>
        </article>
        <article class="experience-card project-card muted">
          <div class="card-glow"></div>
          <p class="eyebrow">Managed Workspace</p>
          <h3>Fixture workspace proof</h3>
          <p>${escapeHtml(model.demo_workspace_ref)}</p>
          <div class="surface-chip-row">
            ${badge(readText(model.workspace, "managed_workspace_kind") ?? "fixture")}
            ${badge("workspace_created_in_host=false")}
            ${badge("adapter next")}
          </div>
          <p class="guidance-note">The current UI uses deterministic package outputs only; real clone/install/run stays future policy-gated executor work.</p>
        </article>
      </div>
    </section>
  `;
}

export function renderForgeExternalProjectPanel(model?: ExternalProjectAggregation): string {
  if (!model) return renderUnavailable("Project Aggregation", ["model not loaded yet"]);
  if (!model.available) return renderUnavailable("Project Aggregation", model.missing_capabilities);
  return `
    <div class="forge-section project-aggregation-panel">
      <div class="section-header"><h2>External Projects / Managed Workspaces</h2><span class="section-meta">safe operating plane · no shell</span></div>
      ${model.errors.length ? `<p class="diagnostic-warn">${escapeHtml(model.errors.join(" · "))}</p>` : ""}
      <div class="project-grid">
        ${projectTile("Source intake", model.intake, ["source_kind", "classification_confidence", "path_safety"])}
        ${projectTile("Stack + lifecycle risk", model.stack, ["detected_stack", "confidence"])}
        ${projectTile("Workspace plan", model.workspace_plan, ["plan_only", "requires_user_approval", "execution_performed"])}
        ${projectTile("Fixture workspace", model.workspace, ["managed_workspace_kind", "detected_stack", "workspace_created_in_host"])}
        ${projectTile("Entrypoints", model.entrypoints, ["entrypoint_count", "execution_performed", "filesystem_performed"])}
        ${projectTile("Run plan", model.run_plan, ["executor_invoked", "execution_performed", "requires_approval"])}
        ${projectTile("Patch proposal", model.patch, ["proposal_required", "file_write_performed", "requires_approval"])}
        ${projectTile("Adapter candidate", model.adapter_plan, ["adapter_kind", "plan_only", "execution_performed"])}
        ${model.adapter_manifest ? projectTile("Adapter manifest", model.adapter_manifest, ["adapter_package_id", "capability_name", "entry_kind", "filesystem_performed"]) : ""}
        ${model.adapter_wrapper ? projectTile("Adapter wrapper", model.adapter_wrapper, ["language", "adapter_package_id", "execution_performed"]) : ""}
        ${model.adapter_readiness ? projectTile("Adapter readiness", model.adapter_readiness, ["ready", "capability_namespace_ok", "permissions_minimal", "fixture_present", "needs_approval_for_execution"]) : ""}
      </div>
      <div class="project-cta-row">${badge("inspect first")}${badge("approve before execution")}${badge("adapter/wrapper becomes package")}${badge("public protocol only")}</div>
    </div>
  `;
}

export function renderAssistantExternalProjectHints(model?: ExternalProjectAggregation): string {
  if (!model) return "";
  const state = model.available ? "ready" : "empty";
  const note = model.available
    ? "Inspect project, draft workspace plan, generate patch proposal, then adapter plan. All current actions are no-execution fixtures."
    : `Missing ${model.missing_capabilities.length} project capability/capabilities.`;
  return `
    <div class="agent-readiness-panel project-assist-panel">
      <div class="agent-readiness-header"><span class="agent-readiness-badge ${state}">${model.available ? "◆" : "◇"}</span><span class="agent-readiness-title">External project assistant lane</span></div>
      <p class="agent-readiness-note">${escapeHtml(note)}</p>
      <div class="quick-actions">
        <button type="button" title="No-execution capability path">Inspect project</button>
        <button type="button" title="Proposal-only path">Draft patch</button>
        <button type="button" title="E5 adapter generation path — manifest + wrapper + readiness">Generate adapter preview</button>
      </div>
    </div>
  `;
}

function renderUnavailable(title: string, missing: string[]): string {
  return `
    <div class="forge-section project-aggregation-panel muted">
      <div class="section-header"><h2>${escapeHtml(title)}</h2><span class="section-meta">unavailable</span></div>
      <p class="empty">Load <code>official/project-intake-lab</code> and <code>official/workspace-lab</code> to enable safe project aggregation.</p>
      ${missing.length ? `<details class="surface-metadata"><summary>Missing capabilities</summary><code>${formatJsonPreview(missing)}</code></details>` : ""}
    </div>
  `;
}

function projectTile(title: string, value: Record<string, unknown> | undefined, fields: string[]): string {
  const rows = fields.map((field) => `<span class="surface-chip">${escapeHtml(field)}=${escapeHtml(readDisplay(value, field))}</span>`).join("");
  return `
    <article class="project-tile">
      <h3>${escapeHtml(title)}</h3>
      <div class="surface-chip-row">${rows}</div>
      <details class="surface-metadata"><summary>Inspect output preview</summary><code>${formatJsonPreview(value ?? { unavailable: true })}</code></details>
    </article>
  `;
}

function badge(text: string): string {
  return `<span class="surface-chip">${escapeHtml(text)}</span>`;
}

function readDisplay(value: Record<string, unknown> | undefined, key: string): string {
  return readText(value, key) ?? "—";
}

function readText(value: Record<string, unknown> | undefined, key: string): string | undefined {
  if (!value) return undefined;
  const candidate = value[key];
  if (typeof candidate === "string") return candidate;
  if (typeof candidate === "number" || typeof candidate === "boolean") return String(candidate);
  return undefined;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
