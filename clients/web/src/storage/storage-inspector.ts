import type { RegisteredCapability } from "../protocol/client";
import { YggProtocolClient } from "../protocol/client";
import { escapeHtml, formatJsonPreview } from "../utils/html";

const STORAGE_LAB = "official/storage-lab";
const TDB_RETRIEVAL_LAB = "official/tdb-retrieval-lab";

export interface StorageInspectorModel {
  available: boolean;
  missing_capabilities: string[];
  contract?: Record<string, unknown>;
  backend_classes?: Record<string, unknown>;
  package_state_plan?: Record<string, unknown>;
  blob_contract?: Record<string, unknown>;
  projection_contract?: Record<string, unknown>;
  retrieval_contract?: Record<string, unknown>;
  multimodal_plan?: Record<string, unknown>;
  tdb_contract?: Record<string, unknown>;
  tdb_real_seam?: Record<string, unknown>;
  errors: string[];
}

const REQUIRED = [
  `${STORAGE_LAB}/describe_storage_contract`,
  `${STORAGE_LAB}/describe_backend_classes`,
  `${STORAGE_LAB}/plan_package_state_store`,
  `${STORAGE_LAB}/describe_blob_store_contract`,
  `${STORAGE_LAB}/describe_projection_store_contract`,
  `${STORAGE_LAB}/describe_retrieval_provider_contract`,
  `${STORAGE_LAB}/draft_multimodal_index_plan`,
  `${TDB_RETRIEVAL_LAB}/describe_tdb_retrieval_contract`,
  `${TDB_RETRIEVAL_LAB}/describe_real_tdb_opt_in_seam`,
];

export async function buildStorageInspectorModel(
  client: YggProtocolClient,
  capabilities: RegisteredCapability[],
): Promise<StorageInspectorModel> {
  const availableIds = new Set(capabilities.map((capability) => capability.capability_id));
  const missing = REQUIRED.filter((capabilityId) => !availableIds.has(capabilityId));
  const model: StorageInspectorModel = {
    available: missing.length === 0,
    missing_capabilities: missing,
    errors: [],
  };
  if (!model.available) return model;

  async function capture(key: keyof StorageInspectorModel, capabilityId: string, input: unknown = {}, provider = STORAGE_LAB) {
    try {
      const result = await client.invokeCapability(capabilityId, input, provider);
      if (typeof result === "object" && result !== null && !Array.isArray(result)) {
        model[key] = result as never;
      } else {
        model.errors.push(`${capabilityId} returned non-object output`);
      }
    } catch (caught) {
      model.errors.push(`${capabilityId}: ${caught instanceof Error ? caught.message : String(caught)}`);
    }
  }

  await Promise.all([
    capture("contract", `${STORAGE_LAB}/describe_storage_contract`),
    capture("backend_classes", `${STORAGE_LAB}/describe_backend_classes`),
    capture("package_state_plan", `${STORAGE_LAB}/plan_package_state_store`, {
      package_id: "thirdparty/storage-preview",
      store_id: "creator-state",
      schema_hint: "document",
    }),
    capture("blob_contract", `${STORAGE_LAB}/describe_blob_store_contract`),
    capture("projection_contract", `${STORAGE_LAB}/describe_projection_store_contract`),
    capture("retrieval_contract", `${STORAGE_LAB}/describe_retrieval_provider_contract`),
    capture("multimodal_plan", `${STORAGE_LAB}/draft_multimodal_index_plan`, {
      package_id: "thirdparty/storage-preview",
      index_id: "asset-retrieval-preview",
      modalities: ["text", "image", "structured"],
      asset_refs: ["asset/example-card", "asset/example-image"],
      schema_hint: "creator_asset_index",
    }),
    capture("tdb_contract", `${TDB_RETRIEVAL_LAB}/describe_tdb_retrieval_contract`, {}, TDB_RETRIEVAL_LAB),
    capture("tdb_real_seam", `${TDB_RETRIEVAL_LAB}/describe_real_tdb_opt_in_seam`, {}, TDB_RETRIEVAL_LAB),
  ]);

  return model;
}

export function renderForgeStoragePanel(model?: StorageInspectorModel): string {
  if (!model) return renderUnavailable(["model not loaded yet"]);
  if (!model.available) return renderUnavailable(model.missing_capabilities);
  return `
    <div class="forge-section storage-inspector-panel">
      <div class="section-header">
        <h2>Storage / Data Backend Neutrality</h2>
        <span class="section-meta">SQLite local · Postgres future · TDB retrieval future</span>
      </div>
      ${model.errors.length ? `<p class="diagnostic-warn">${escapeHtml(model.errors.join(" · "))}</p>` : ""}
      <div class="storage-grid">
        ${storageTile("Event spine", model.contract, ["package_kind", "inference_performed", "network_performed"])}
        ${storageTile("Backend classes", model.backend_classes, ["kind"])}
        ${storageTile("Package state", model.package_state_plan, ["plan_only", "requires_user_approval", "write_performed"])}
        ${storageTile("Blob / asset", model.blob_contract, ["contract_type", "inference_performed", "network_performed"])}
        ${storageTile("Projection / index", model.projection_contract, ["kind", "inference_performed", "network_performed"])}
        ${storageTile("Retrieval / TDB slot", model.retrieval_contract, ["kind", "inference_performed", "network_performed"])}
        ${storageTile("Multimodal index plan", model.multimodal_plan, ["plan_only", "embedding_generated", "vectors_stored"])}
        ${storageTile("TDB adapter", model.tdb_contract, ["kind", "package_kind", "backend_role"])}
        ${storageTile("Real TDB opt-in seam", model.tdb_real_seam, ["kind", "status"])}
      </div>
      <div class="project-cta-row">
        ${chip("backend-neutral contract")}${chip("no SQL/DSN in protocol")}${chip("TDB as future provider slot")}${chip("public protocol only")}
      </div>
    </div>
  `;
}

export function renderAssistantStorageHints(model?: StorageInspectorModel): string {
  if (!model) return "";
  const state = model.available ? "ready" : "empty";
  const note = model.available
    ? "Storage-lab and TDB retrieval-lab can explain event spine, package state, blob, projection, retrieval, and real TDB opt-in seams without touching real DB backends."
    : `Missing ${model.missing_capabilities.length} storage capability/capabilities.`;
  return `
    <div class="agent-readiness-panel storage-assist-panel">
      <div class="agent-readiness-header">
        <span class="agent-readiness-badge ${state}">${model.available ? "◆" : "◇"}</span>
        <span class="agent-readiness-title">Storage backend neutrality lane</span>
      </div>
      <p class="agent-readiness-note">${escapeHtml(note)}</p>
      <div class="quick-actions">
        <button type="button" title="Contract preview only">Inspect storage contract</button>
        <button type="button" title="Plan-only path">Plan package store</button>
        <button type="button" title="Future provider slot">Review TDB slot</button>
      </div>
    </div>
  `;
}

function renderUnavailable(missing: string[]): string {
  return `
    <div class="forge-section storage-inspector-panel muted">
      <div class="section-header"><h2>Storage / Data Backend Neutrality</h2><span class="section-meta">unavailable</span></div>
      <p class="empty">Load <code>official/storage-lab</code> and <code>official/tdb-retrieval-lab</code> to inspect backend-neutral storage and TDB retrieval contracts.</p>
      ${missing.length ? `<details class="surface-metadata"><summary>Missing capabilities</summary><code>${formatJsonPreview(missing)}</code></details>` : ""}
    </div>
  `;
}

function storageTile(title: string, value: Record<string, unknown> | undefined, fields: string[]): string {
  return `
    <article class="storage-tile">
      <h3>${escapeHtml(title)}</h3>
      <div class="surface-chip-row">${fields.map((field) => chip(`${field}=${readDisplay(value, field)}`)).join("")}</div>
      <details class="surface-metadata"><summary>Inspect output preview</summary><code>${formatJsonPreview(value ?? { unavailable: true })}</code></details>
    </article>
  `;
}

function chip(text: string): string {
  return `<span class="surface-chip">${escapeHtml(text)}</span>`;
}

function readDisplay(value: Record<string, unknown> | undefined, key: string): string {
  if (!value) return "—";
  const candidate = value[key];
  if (typeof candidate === "string") return candidate;
  if (typeof candidate === "number" || typeof candidate === "boolean") return String(candidate);
  return "—";
}
