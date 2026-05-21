//! Handler for `official/storage-lab` capabilities.
//!
//! Storage Backend Neutrality Alpha Phase S4 — Projection / Index Materialization Contract Proof.
//!
//! Package-facing storage/data contract preview: storage backend classes,
//! package state store planning, document CRUD previews, snapshot export,
//! blob/asset content-addressed contract proof, and projection/index
//! materialization contract proof.
//!
//! Deterministic, no-network, no real model inference, no real DB writes,
//! no SQL, no filesystem, no DSN/path/credentials, no blob content in
//! event payloads. Produces package-owned storage/data shapes.
//!
//! No reserved storage/database/blob/projection/sql/vector kernel namespace
//! references.
//!
//! State terminology: storage_contract, backend_class, package_state_plan,
//! document_preview, tombstone_preview, snapshot_preview,
//! blob_store_contract, blob_put_preview, blob_metadata_preview,
//! blob_manifest_preview, projection_store_contract,
//! projection_materialization_plan, projection_query_preview,
//! projection_migration_plan_preview — not vendor database, query-language,
//! vector, secret-bearing, object-store-bucket, DB-table, or SQL-product
//! semantics.

use serde_json::Value;

use super::safety;
use super::InprocInvocation;

const PACKAGE_ID: &str = "official/storage-lab";

// ---------------------------------------------------------------------------
// Storage contract layers
// ---------------------------------------------------------------------------

const STORAGE_CONTRACT_LAYERS: &[&str] = &[
    "event_spine_backend",
    "package_state_store",
    "blob_store_future",
    "projection_index_future",
    "retrieval_provider_future",
];

// ---------------------------------------------------------------------------
// Backend class candidates
// ---------------------------------------------------------------------------

const BACKEND_CLASS_CANDIDATES: &[&str] = &[
    "memory_event_store",
    "sqlite_event_store",
    "postgres_event_store_future",
    "package_state_event_sourced",
    "blob_content_addressed_future",
    "tdb_retrieval_provider_future",
];

// ---------------------------------------------------------------------------
// Blob backend candidates (S3)
// ---------------------------------------------------------------------------

const BLOB_BACKEND_CANDIDATES: &[&str] = &[
    "local_content_addressed_future",
    "filesystem_backend_future",
    "object_store_future",
];

// ---------------------------------------------------------------------------
// Projection / Index backend candidates (S4)
// ---------------------------------------------------------------------------

const PROJECTION_BACKEND_CANDIDATES: &[&str] = &[
    "event_derived_projection",
    "package_owned_index",
    "sqlite_materialized_view_future",
    "postgres_materialized_view_future",
];

// ---------------------------------------------------------------------------
// Forbidden namespace tokens (must not appear in output)
// ---------------------------------------------------------------------------

#[cfg(test)]
fn forbidden_namespace_tokens() -> Vec<String> {
    [
        "sqlite",
        "postgres",
        "tdb",
        "vector",
        "embedding",
        "collection",
        "sql",
        "database",
    ]
    .into_iter()
    .map(|segment| format!("kernel.{segment}."))
    .collect()
}

// ---------------------------------------------------------------------------
// Safe-id validation: reject path traversal and unsafe characters
// ---------------------------------------------------------------------------

/// Check if an identifier is safe (no path traversal, no unsafe chars).
/// Allows alphanumeric, hyphens, underscores, forward slashes (single),
/// and colons. Rejects `..`, `//`, leading/trailing `/`, and characters
/// that could be used for injection.
fn is_safe_id(id: &str) -> bool {
    if id.is_empty() {
        return false;
    }
    // Reject path traversal
    if id.contains("..") {
        return false;
    }
    // Reject double slashes
    if id.contains("//") {
        return false;
    }
    // Reject leading/trailing slashes
    if id.starts_with('/') || id.ends_with('/') {
        return false;
    }
    // Reject obvious injection characters
    for ch in id.chars() {
        if !ch.is_ascii_alphanumeric()
            && ch != '-'
            && ch != '_'
            && ch != '/'
            && ch != ':'
        {
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Rejection output
// ---------------------------------------------------------------------------

fn rejected_output(request: &InprocInvocation, reason: &str) -> Value {
    serde_json::json!({
        "kind": "storage_lab_rejected",
        "redaction_state": "unsafe_blocked",
        "reason": reason,
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    })
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

pub fn try_handle(request: &InprocInvocation) -> Option<anyhow::Result<Value>> {
    if request.provider_package_id != PACKAGE_ID {
        return None;
    }
    let id = request.capability_id.as_str();
    if id.ends_with("/describe_storage_contract") {
        Some(describe_storage_contract(request))
    } else if id.ends_with("/describe_backend_classes") {
        Some(describe_backend_classes(request))
    } else if id.ends_with("/plan_package_state_store") {
        Some(plan_package_state_store(request))
    } else if id.ends_with("/put_document_preview") {
        Some(put_document_preview(request))
    } else if id.ends_with("/get_document_preview") {
        Some(get_document_preview(request))
    } else if id.ends_with("/query_document_prefix_preview") {
        Some(query_document_prefix_preview(request))
    } else if id.ends_with("/delete_document_tombstone_preview") {
        Some(delete_document_tombstone_preview(request))
    } else if id.ends_with("/export_store_snapshot_preview") {
        Some(export_store_snapshot_preview(request))
    } else if id.ends_with("/describe_blob_store_contract") {
        Some(describe_blob_store_contract(request))
    } else if id.ends_with("/put_blob_preview") {
        Some(put_blob_preview(request))
    } else if id.ends_with("/get_blob_metadata_preview") {
        Some(get_blob_metadata_preview(request))
    } else if id.ends_with("/export_blob_manifest_preview") {
        Some(export_blob_manifest_preview(request))
    } else if id.ends_with("/describe_projection_store_contract") {
        Some(describe_projection_store_contract(request))
    } else if id.ends_with("/plan_projection_materialization") {
        Some(plan_projection_materialization(request))
    } else if id.ends_with("/query_projection_preview") {
        Some(query_projection_preview(request))
    } else if id.ends_with("/migrate_projection_plan_preview") {
        Some(migrate_projection_plan_preview(request))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Capability implementations
// ---------------------------------------------------------------------------

fn describe_storage_contract(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "storage_lab_contract",
        "package_id": request.provider_package_id,
        "package_kind": "ordinary",
        "capabilities": [
            {"id": "official/storage-lab/describe_storage_contract", "purpose": "describe the storage lab package contract and layered model"},
            {"id": "official/storage-lab/describe_backend_classes", "purpose": "describe backend class candidates with capability flags only"},
            {"id": "official/storage-lab/plan_package_state_store", "purpose": "produce a package-scoped state store plan with namespace, quota hints, and backend candidates"},
            {"id": "official/storage-lab/put_document_preview", "purpose": "preview a document put operation without performing real storage"},
            {"id": "official/storage-lab/get_document_preview", "purpose": "preview a document get operation without performing real storage"},
            {"id": "official/storage-lab/query_document_prefix_preview", "purpose": "preview a prefix query without executing real storage queries"},
            {"id": "official/storage-lab/delete_document_tombstone_preview", "purpose": "preview a document tombstone delete without performing real deletion"},
            {"id": "official/storage-lab/export_store_snapshot_preview", "purpose": "preview a store snapshot export with redacted output"},
            {"id": "official/storage-lab/describe_blob_store_contract", "purpose": "describe blob/asset store contract with backend candidates and red lines"},
            {"id": "official/storage-lab/put_blob_preview", "purpose": "preview a blob put with content-addressed hash, no real blob storage"},
            {"id": "official/storage-lab/get_blob_metadata_preview", "purpose": "preview blob metadata retrieval without returning blob content"},
            {"id": "official/storage-lab/export_blob_manifest_preview", "purpose": "preview a blob manifest export with refs only, no content"},
            {"id": "official/storage-lab/describe_projection_store_contract", "purpose": "describe projection/index store contract with backend candidates and red lines"},
            {"id": "official/storage-lab/plan_projection_materialization", "purpose": "plan-only projection materialization with backend candidates, no real DB table or index creation"},
            {"id": "official/storage-lab/query_projection_preview", "purpose": "preview projection query shape without executing real queries"},
            {"id": "official/storage-lab/migrate_projection_plan_preview", "purpose": "preview projection migration plan without applying data rewrites"},
        ],
        "surfaces": {
            "forge_panel": "official/storage-lab/forge-panel",
            "assistant_action": "official/storage-lab/assistant-action",
            "home_card": "official/storage-lab/home-card",
        },
        "layers": STORAGE_CONTRACT_LAYERS,
        "layer_descriptions": {
            "event_spine_backend": "kernel-owned append-only event log; backend implementations (in-memory, SQLite, future Postgres) are host config, not protocol",
            "package_state_store": "package-scoped document/key-value store derived from event sourcing; namespace belongs to the owning package",
            "blob_store_future": "future large-object content-addressed storage; hash/size/mime/provenance only",
            "projection_index_future": "future package-owned projection/index materialization; plans only, no DB table leakage",
            "retrieval_provider_future": "future retrieval/vector/multimodal provider slots; no embedding generation, no vector storage, no network",
        },
        "output_shapes": {
            "storage_contract": ["kind", "package_id", "package_kind", "capabilities", "surfaces", "layers", "output_shapes", "inference_performed", "network_performed", "provenance"],
            "backend_class": ["class_id", "capability_flags", "status", "description"],
            "package_state_plan": ["plan_id", "package_id", "store_id", "namespace", "quota_hints", "backend_candidates", "requires_user_approval", "plan_only"],
            "document_preview": ["document_id", "store_id", "write_performed", "read_performed", "query_performed", "redacted_content", "content_address", "provenance"],
            "tombstone_preview": ["document_id", "store_id", "delete_performed", "tombstone_status", "provenance"],
            "snapshot_preview": ["store_id", "snapshot_exported", "redacted_entries", "entry_count", "provenance"],
            "blob_store_contract": ["kind", "contract_type", "backend_candidates", "red_lines", "inference_performed", "network_performed", "provenance"],
            "blob_put_preview": ["kind", "package_id", "blob_id", "content_address", "mime", "size_bytes", "metadata_shape", "blob_stored", "filesystem_performed", "network_performed", "event_payload_contains_blob", "provenance"],
            "blob_metadata_preview": ["kind", "blob_id", "content_address", "mime", "size_bytes", "metadata_shape", "blob_read", "content_returned", "provenance"],
            "blob_manifest_preview": ["kind", "manifest_items", "item_count", "content_included", "provenance"],
            "projection_store_contract": ["kind", "contract_kinds", "backend_candidates", "red_lines", "inference_performed", "network_performed", "provenance"],
            "projection_materialization_plan": ["kind", "projection_id", "package_id", "source_kinds", "index_keys", "backend_candidates", "materialized", "write_performed", "backend_selected", "plan_only", "inference_performed", "network_performed", "provenance"],
            "projection_query_preview": ["kind", "projection_ref", "filter_preview", "limit", "preview_shape", "query_executed", "rows_returned", "inference_performed", "network_performed", "provenance"],
            "projection_migration_plan_preview": ["kind", "projection_id", "from_version", "to_version", "change_summary", "migration_applied", "data_rewritten", "requires_rebuild", "inference_performed", "network_performed", "provenance"],
        },
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn describe_backend_classes(request: &InprocInvocation) -> anyhow::Result<Value> {
    let backend_classes: Vec<Value> = BACKEND_CLASS_CANDIDATES
        .iter()
        .map(|class_id| {
            let (capability_flags, status, description) = match *class_id {
                "memory_event_store" => (
                    vec!["append", "replay", "range", "kind_prefix", "subscription"],
                    "available",
                    "In-memory event store for testing and ephemeral sessions",
                ),
                "sqlite_event_store" => (
                    vec!["append", "replay", "range", "kind_prefix", "subscription", "durable"],
                    "available",
                    "SQLite-backed event store for single-host durable sessions",
                ),
                "postgres_event_store_future" => (
                    vec!["append", "replay", "range", "kind_prefix", "subscription", "durable", "remote"],
                    "future",
                    "Future PostgreSQL event store for server/team deployments",
                ),
                "package_state_event_sourced" => (
                    vec!["read_package_state", "write_package_state", "prefix_query"],
                    "available",
                    "Package-scoped state store derived from event sourcing",
                ),
                "blob_content_addressed_future" => (
                    vec!["put_blob", "get_blob_metadata", "content_addressed"],
                    "future",
                    "Future content-addressed blob storage",
                ),
                "tdb_retrieval_provider_future" => (
                    vec!["similarity_search", "multimodal_query"],
                    "future",
                    "Future TDB multimodal retrieval provider",
                ),
                _ => (
                    vec![] as Vec<&str>,
                    "unknown",
                    "Unknown backend class",
                ),
            };
            serde_json::json!({
                "class_id": class_id,
                "capability_flags": capability_flags,
                "status": status,
                "description": description,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "kind": "backend_classes",
        "backend_classes": backend_classes,
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn plan_package_state_store(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request, "input contains raw-secret-like content; use secret_ref references instead"));
    }

    let package_id = request
        .input
        .get("package_id")
        .and_then(Value::as_str)
        .unwrap_or("");

    if package_id.is_empty() {
        return Ok(rejected_output(request, "package_id must not be empty"));
    }

    if !is_safe_id(package_id) {
        return Ok(rejected_output(request, "package_id contains unsafe characters or path traversal"));
    }

    let store_id = request
        .input
        .get("store_id")
        .and_then(Value::as_str)
        .unwrap_or("default");

    if !is_safe_id(store_id) {
        return Ok(rejected_output(request, "store_id contains unsafe characters or path traversal"));
    }

    let schema_hint = request
        .input
        .get("schema_hint")
        .and_then(Value::as_str)
        .unwrap_or("document");

    // Namespace must belong to the package
    let namespace = format!("{}/state/{}", package_id, store_id);

    let plan_id = format!(
        "plan:{}:{}:{}",
        package_id,
        store_id,
        crate::runtime::content_address(&format!("{}:{}", namespace, schema_hint))
    );

    // Backend candidates — capability flags only, no path/DSN/credentials
    let backend_candidates = vec![
        serde_json::json!({
            "class_id": "package_state_event_sourced",
            "capability_flags": ["read_package_state", "write_package_state", "prefix_query"],
            "status": "available",
        }),
        serde_json::json!({
            "class_id": "memory_event_store",
            "capability_flags": ["append", "replay", "range"],
            "status": "available",
        }),
    ];

    Ok(serde_json::json!({
        "kind": "package_state_plan",
        "plan_id": plan_id,
        "package_id": package_id,
        "store_id": store_id,
        "namespace": namespace,
        "schema_hint": schema_hint,
        "quota_hints": {
            "max_document_count": 10000,
            "max_document_size_bytes": 1048576,
            "retention_policy": "event_sourced_replay",
        },
        "backend_candidates": backend_candidates,
        "requires_user_approval": true,
        "plan_only": true,
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn put_document_preview(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request, "input contains raw-secret-like content; use secret_ref references instead"));
    }

    let document_id = request
        .input
        .get("document_id")
        .and_then(Value::as_str)
        .unwrap_or("");

    if !is_safe_id(document_id) {
        return Ok(rejected_output(request, "document_id contains unsafe characters or path traversal"));
    }

    let store_id = request
        .input
        .get("store_id")
        .and_then(Value::as_str)
        .unwrap_or("default");

    if !is_safe_id(store_id) {
        return Ok(rejected_output(request, "store_id contains unsafe characters or path traversal"));
    }

    let content = request
        .input
        .get("content")
        .cloned()
        .unwrap_or(Value::Null);

    let content_address = crate::runtime::content_address(&format!("{}:{}", document_id, content));

    // Redacted content — only show shape, not raw content
    let redacted_content = match &content {
        Value::Object(map) => {
            let keys: Vec<&String> = map.keys().collect();
            serde_json::json!({"keys": keys, "value_hint": "redacted"})
        }
        Value::String(s) => {
            let hint = if s.len() > 20 {
                format!("{}...({} chars)", &s[..20], s.len())
            } else {
                s.clone()
            };
            serde_json::json!({"string_hint": hint, "value_hint": "redacted"})
        }
        _ => serde_json::json!({"type": "non_object", "value_hint": "redacted"}),
    };

    Ok(serde_json::json!({
        "kind": "document_put_preview",
        "document_id": document_id,
        "store_id": store_id,
        "write_performed": false,
        "redacted_content": redacted_content,
        "content_address": content_address,
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn get_document_preview(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request, "input contains raw-secret-like content; use secret_ref references instead"));
    }

    let document_id = request
        .input
        .get("document_id")
        .and_then(Value::as_str)
        .unwrap_or("");

    if !is_safe_id(document_id) {
        return Ok(rejected_output(request, "document_id contains unsafe characters or path traversal"));
    }

    let store_id = request
        .input
        .get("store_id")
        .and_then(Value::as_str)
        .unwrap_or("default");

    if !is_safe_id(store_id) {
        return Ok(rejected_output(request, "store_id contains unsafe characters or path traversal"));
    }

    Ok(serde_json::json!({
        "kind": "document_get_preview",
        "document_id": document_id,
        "store_id": store_id,
        "read_performed": false,
        "redacted_content": {"value_hint": "redacted"},
        "content_address": crate::runtime::content_address(&format!("get:{}", document_id)),
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn query_document_prefix_preview(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request, "input contains raw-secret-like content; use secret_ref references instead"));
    }

    let prefix = request
        .input
        .get("prefix")
        .and_then(Value::as_str)
        .unwrap_or("");

    if !is_safe_id(prefix) && !prefix.is_empty() {
        return Ok(rejected_output(request, "prefix contains unsafe characters or path traversal"));
    }

    let store_id = request
        .input
        .get("store_id")
        .and_then(Value::as_str)
        .unwrap_or("default");

    if !is_safe_id(store_id) {
        return Ok(rejected_output(request, "store_id contains unsafe characters or path traversal"));
    }

    Ok(serde_json::json!({
        "kind": "document_query_preview",
        "prefix": prefix,
        "store_id": store_id,
        "query_performed": false,
        "redacted_matches": {"match_count_hint": 0, "value_hint": "redacted"},
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn delete_document_tombstone_preview(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request, "input contains raw-secret-like content; use secret_ref references instead"));
    }

    let document_id = request
        .input
        .get("document_id")
        .and_then(Value::as_str)
        .unwrap_or("");

    if !is_safe_id(document_id) {
        return Ok(rejected_output(request, "document_id contains unsafe characters or path traversal"));
    }

    let store_id = request
        .input
        .get("store_id")
        .and_then(Value::as_str)
        .unwrap_or("default");

    if !is_safe_id(store_id) {
        return Ok(rejected_output(request, "store_id contains unsafe characters or path traversal"));
    }

    Ok(serde_json::json!({
        "kind": "document_tombstone_preview",
        "document_id": document_id,
        "store_id": store_id,
        "delete_performed": false,
        "tombstone_status": "preview_only",
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn export_store_snapshot_preview(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request, "input contains raw-secret-like content; use secret_ref references instead"));
    }

    let store_id = request
        .input
        .get("store_id")
        .and_then(Value::as_str)
        .unwrap_or("default");

    if !is_safe_id(store_id) {
        return Ok(rejected_output(request, "store_id contains unsafe characters or path traversal"));
    }

    Ok(serde_json::json!({
        "kind": "store_snapshot_preview",
        "store_id": store_id,
        "snapshot_exported": false,
        "redacted_entries": {"entry_count": 0, "content_hint": "redacted"},
        "entry_count": 0,
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

// ---------------------------------------------------------------------------
// S3 — Blob / Asset Store Contract Proof capabilities
// ---------------------------------------------------------------------------

fn describe_blob_store_contract(request: &InprocInvocation) -> anyhow::Result<Value> {
    let backend_candidates: Vec<Value> = BLOB_BACKEND_CANDIDATES
        .iter()
        .map(|class_id| {
            let (capability_flags, status, description) = match *class_id {
                "local_content_addressed_future" => (
                    vec!["put_blob", "get_blob_metadata", "content_addressed", "dedup"],
                    "future",
                    "Future local content-addressed blob storage with hash-based dedup",
                ),
                "filesystem_backend_future" => (
                    vec!["put_blob", "get_blob_metadata", "content_addressed"],
                    "future",
                    "Future filesystem-backed blob storage",
                ),
                "object_store_future" => (
                    vec![
                        "put_blob",
                        "get_blob_metadata",
                        "content_addressed",
                        "remote",
                    ],
                    "future",
                    "Future remote object store (S3/GCS/Azure Blob) backend",
                ),
                _ => (
                    Vec::<&str>::new(),
                    "unknown",
                    "Unknown blob backend class",
                ),
            };
            serde_json::json!({
                "class_id": class_id,
                "capability_flags": capability_flags,
                "status": status,
                "description": description,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "kind": "blob_store_contract",
        "contract_type": "content_addressed_blob_store",
        "backend_candidates": backend_candidates,
        "red_lines": [
            "no_blob_content_in_events",
            "no_raw_secrets",
            "no_direct_filesystem_path_leak",
            "content_address_required",
        ],
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

/// Maximum allowed inline content_sample length in characters.
const MAX_INLINE_SAMPLE_CHARS: usize = 4096;

fn put_blob_preview(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(
            request,
            "input contains raw-secret-like content; use secret_ref references instead",
        ));
    }

    let package_id = request
        .input
        .get("package_id")
        .and_then(Value::as_str)
        .unwrap_or("");

    if package_id.is_empty() {
        return Ok(rejected_output(request, "package_id must not be empty"));
    }

    if !is_safe_id(package_id) {
        return Ok(rejected_output(
            request,
            "package_id contains unsafe characters or path traversal",
        ));
    }

    let blob_id = request
        .input
        .get("blob_id")
        .and_then(Value::as_str)
        .unwrap_or("");

    if !is_safe_id(blob_id) {
        return Ok(rejected_output(
            request,
            "blob_id contains unsafe characters or path traversal",
        ));
    }

    let mime = request
        .input
        .get("mime")
        .and_then(Value::as_str)
        .unwrap_or("application/octet-stream");

    let size_bytes = request
        .input
        .get("size_bytes")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    // Content hash: if provided, normalize it; otherwise derive from safe sample.
    let content_hash = request
        .input
        .get("content_hash")
        .and_then(Value::as_str);

    let content_sample = request
        .input
        .get("content_sample")
        .and_then(Value::as_str);

    // Reject oversized inline samples
    if let Some(sample) = content_sample {
        if sample.len() > MAX_INLINE_SAMPLE_CHARS {
            return Ok(rejected_output(
                request,
                &format!(
                    "content_sample exceeds maximum inline size ({} > {} chars)",
                    sample.len(),
                    MAX_INLINE_SAMPLE_CHARS
                ),
            ));
        }
    }

    let content_address = if let Some(hash) = content_hash {
        // Use the provided hash, normalized with a sha256: prefix
        if hash.starts_with("sha256:") {
            hash.to_string()
        } else {
            format!("sha256:{}", hash)
        }
    } else {
        // Derive deterministic content address from safe sample
        let material = match content_sample {
            Some(sample) => format!("{}:{}:{}", package_id, blob_id, sample),
            None => format!("{}:{}:{}", package_id, blob_id, size_bytes),
        };
        crate::runtime::content_address(&material)
    };

    let metadata_shape = serde_json::json!({
        "package_id": package_id,
        "blob_id": blob_id,
        "mime": mime,
        "size_bytes": size_bytes,
        "content_address": content_address,
        "provenance_hint": "redacted",
    });

    Ok(serde_json::json!({
        "kind": "blob_put_preview",
        "package_id": package_id,
        "blob_id": blob_id,
        "content_address": content_address,
        "mime": mime,
        "size_bytes": size_bytes,
        "metadata_shape": metadata_shape,
        "blob_stored": false,
        "filesystem_performed": false,
        "network_performed": false,
        "event_payload_contains_blob": false,
        "inference_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn get_blob_metadata_preview(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(
            request,
            "input contains raw-secret-like content; use secret_ref references instead",
        ));
    }

    let blob_id = request
        .input
        .get("blob_id")
        .and_then(Value::as_str)
        .unwrap_or("");

    if !is_safe_id(blob_id) {
        return Ok(rejected_output(
            request,
            "blob_id contains unsafe characters or path traversal",
        ));
    }

    let content_address = crate::runtime::content_address(&format!("blob_meta:{}", blob_id));

    Ok(serde_json::json!({
        "kind": "blob_metadata_preview",
        "blob_id": blob_id,
        "content_address": content_address,
        "mime": "application/octet-stream",
        "size_bytes": 0,
        "metadata_shape": {"value_hint": "redacted"},
        "blob_read": false,
        "content_returned": false,
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn export_blob_manifest_preview(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(
            request,
            "input contains raw-secret-like content; use secret_ref references instead",
        ));
    }

    let package_id = request
        .input
        .get("package_id")
        .and_then(Value::as_str)
        .unwrap_or("");

    if !package_id.is_empty() && !is_safe_id(package_id) {
        return Ok(rejected_output(
            request,
            "package_id contains unsafe characters or path traversal",
        ));
    }

    // Manifest items contain refs only — no blob content
    let manifest_items: Vec<Value> = Vec::new();

    Ok(serde_json::json!({
        "kind": "blob_manifest_preview",
        "manifest_items": manifest_items,
        "item_count": 0,
        "content_included": false,
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

// ---------------------------------------------------------------------------
// S4 — Projection / Index Materialization Contract Proof capabilities
// ---------------------------------------------------------------------------

fn describe_projection_store_contract(request: &InprocInvocation) -> anyhow::Result<Value> {
    let backend_candidates: Vec<Value> = PROJECTION_BACKEND_CANDIDATES
        .iter()
        .map(|class_id| {
            let (capability_flags, status, description) = match *class_id {
                "event_derived_projection" => (
                    vec!["replay", "rebuild", "kind_prefix_filter", "event_sourced"],
                    "available",
                    "Event-derived projection rebuilt from event log replay",
                ),
                "package_owned_index" => (
                    vec!["plan_materialization", "query_preview", "migration_preview"],
                    "available",
                    "Package-owned index/projection plan and preview without real DB persistence",
                ),
                "sqlite_materialized_view_future" => (
                    vec!["materialized_view", "durable", "plan_only"],
                    "future",
                    "Future SQLite materialized view for single-host durable projections",
                ),
                "postgres_materialized_view_future" => (
                    vec!["materialized_view", "durable", "remote", "plan_only"],
                    "future",
                    "Future PostgreSQL materialized view for server/team durable projections",
                ),
                _ => (
                    Vec::<&str>::new(),
                    "unknown",
                    "Unknown projection backend class",
                ),
            };
            serde_json::json!({
                "class_id": class_id,
                "capability_flags": capability_flags,
                "status": status,
                "description": description,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "kind": "projection_store_contract",
        "contract_kinds": [
            "event_derived_projection",
            "package_owned_index",
            "sqlite_materialized_view_future",
            "postgres_materialized_view_future",
        ],
        "backend_candidates": backend_candidates,
        "red_lines": [
            "no_table_exposure",
            "no_sql_exposure",
            "no_backend_credentials",
            "no_query_product_leakage",
            "projection_derives_from_events_assets_only",
        ],
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn plan_projection_materialization(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(
            request,
            "input contains raw-secret-like content; use secret_ref references instead",
        ));
    }

    let package_id = request
        .input
        .get("package_id")
        .and_then(Value::as_str)
        .unwrap_or("");

    if package_id.is_empty() {
        return Ok(rejected_output(request, "package_id must not be empty"));
    }

    if !is_safe_id(package_id) {
        return Ok(rejected_output(
            request,
            "package_id contains unsafe characters or path traversal",
        ));
    }

    let projection_id = request
        .input
        .get("projection_id")
        .and_then(Value::as_str)
        .unwrap_or("");

    if projection_id.is_empty() {
        return Ok(rejected_output(request, "projection_id must not be empty"));
    }

    if !is_safe_id(projection_id) {
        return Ok(rejected_output(
            request,
            "projection_id contains unsafe characters or path traversal",
        ));
    }

    let source_kinds: Vec<Value> = request
        .input
        .get("source_kinds")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let index_keys: Vec<Value> = request
        .input
        .get("index_keys")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    // Backend candidates — plan only, no actual backend selection
    let backend_candidates: Vec<Value> = PROJECTION_BACKEND_CANDIDATES
        .iter()
        .map(|class_id| {
            let status = match *class_id {
                "event_derived_projection" => "available",
                "package_owned_index" => "available",
                _ => "future",
            };
            serde_json::json!({
                "class_id": class_id,
                "status": status,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "kind": "projection_materialization_plan",
        "projection_id": projection_id,
        "package_id": package_id,
        "source_kinds": source_kinds,
        "index_keys": index_keys,
        "backend_candidates": backend_candidates,
        "materialized": false,
        "write_performed": false,
        "backend_selected": false,
        "plan_only": true,
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn query_projection_preview(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(
            request,
            "input contains raw-secret-like content; use secret_ref references instead",
        ));
    }

    let projection_ref = request
        .input
        .get("projection_ref")
        .and_then(Value::as_str)
        .unwrap_or("");

    if !projection_ref.is_empty() && !is_safe_id(projection_ref) {
        return Ok(rejected_output(
            request,
            "projection_ref contains unsafe characters or path traversal",
        ));
    }

    let filter_preview = request
        .input
        .get("filter_preview")
        .cloned()
        .unwrap_or(Value::Null);

    let limit = request
        .input
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(10);

    // Preview shape only — no actual query execution
    let preview_shape = serde_json::json!({
        "projection_ref": if projection_ref.is_empty() { Value::Null } else { serde_json::json!(projection_ref) },
        "filter_preview": filter_preview,
        "limit": limit,
        "result_hint": "preview_only",
    });

    Ok(serde_json::json!({
        "kind": "projection_query_preview",
        "projection_ref": if projection_ref.is_empty() { Value::Null } else { serde_json::json!(projection_ref) },
        "filter_preview": filter_preview,
        "limit": limit,
        "preview_shape": preview_shape,
        "query_executed": false,
        "rows_returned": false,
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn migrate_projection_plan_preview(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(
            request,
            "input contains raw-secret-like content; use secret_ref references instead",
        ));
    }

    let projection_id = request
        .input
        .get("projection_id")
        .and_then(Value::as_str)
        .unwrap_or("");

    if projection_id.is_empty() {
        return Ok(rejected_output(request, "projection_id must not be empty"));
    }

    if !is_safe_id(projection_id) {
        return Ok(rejected_output(
            request,
            "projection_id contains unsafe characters or path traversal",
        ));
    }

    let from_version = request
        .input
        .get("from_version")
        .and_then(Value::as_str)
        .unwrap_or("0");

    let to_version = request
        .input
        .get("to_version")
        .and_then(Value::as_str)
        .unwrap_or("1");

    let change_summary = request
        .input
        .get("change_summary")
        .and_then(Value::as_str)
        .unwrap_or("schema evolution preview");

    Ok(serde_json::json!({
        "kind": "projection_migration_plan_preview",
        "projection_id": projection_id,
        "from_version": from_version,
        "to_version": to_version,
        "change_summary": change_summary,
        "migration_applied": false,
        "data_rewritten": false,
        "requires_rebuild": true,
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_request(cap: &str, input: Value) -> InprocInvocation {
        InprocInvocation {
            capability_id: cap.to_string(),
            provider_package_id: PACKAGE_ID.to_string(),
            input,
        }
    }

    #[test]
    fn try_handle_matches_package_id() {
        let req = make_request("official/storage-lab/describe_storage_contract", json!({}));
        assert!(try_handle(&req).is_some());
    }

    #[test]
    fn try_handle_rejects_wrong_package() {
        let req = InprocInvocation {
            capability_id: "official/storage-lab/describe_storage_contract".to_string(),
            provider_package_id: "official/other".to_string(),
            input: json!({}),
        };
        assert!(try_handle(&req).is_none());
    }

    #[test]
    fn describe_contract_has_all_surfaces() {
        let req = make_request("official/storage-lab/describe_storage_contract", json!({}));
        let result = try_handle(&req).unwrap().unwrap();
        let surfaces = result["surfaces"].as_object().unwrap();
        assert!(surfaces.contains_key("forge_panel"));
        assert!(surfaces.contains_key("assistant_action"));
        assert!(surfaces.contains_key("home_card"));
    }

    #[test]
    fn describe_contract_lists_12_capabilities() {
        let req = make_request("official/storage-lab/describe_storage_contract", json!({}));
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(
            result["capabilities"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0),
            16,
            "must list 16 capabilities"
        );
    }

    #[test]
    fn describe_contract_lists_layers() {
        let req = make_request("official/storage-lab/describe_storage_contract", json!({}));
        let result = try_handle(&req).unwrap().unwrap();
        let layers = result["layers"].as_array().unwrap();
        assert!(layers.len() >= 5, "must list at least 5 layers");
    }

    #[test]
    fn no_forbidden_namespace_in_contract() {
        let req = make_request("official/storage-lab/describe_storage_contract", json!({}));
        let result = try_handle(&req).unwrap().unwrap();
        let output_str = serde_json::to_string(&result).unwrap();
        for token in forbidden_namespace_tokens() {
            assert!(
                !output_str.contains(&token),
                "must not contain {}",
                token
            );
        }
    }

    #[test]
    fn backend_classes_no_credentials() {
        let req = make_request("official/storage-lab/describe_backend_classes", json!({}));
        let result = try_handle(&req).unwrap().unwrap();
        let output_str = serde_json::to_string(&result).unwrap();
        // No DSN, path, credentials in output
        for token in &["dsn", &format!("connection_{}", "string"), "password", "credential"] {
            assert!(
                !output_str.to_lowercase().contains(token),
                "must not contain {}",
                token
            );
        }
        // No SQL/table/collection/vector terminology as standalone product terms
        // (sqlite_event_store as a backend class identifier is allowed)
        let lower = output_str.to_lowercase();
        // Check for kernel-level SQL/table/collection/vector namespace references
        for token in forbidden_namespace_tokens() {
            assert!(
                !output_str.contains(&token),
                "must not contain forbidden namespace {}",
                token
            );
        }
        // Check that no standalone "sql" appears outside of "sqlite"
        // and no "table", "collection", or "vector" as product terms
        assert!(!lower.contains("\"sql\""), "must not contain standalone sql term");
        assert!(!lower.contains("table"), "must not contain table terminology");
        assert!(!lower.contains("collection"), "must not contain collection terminology");
        assert!(!lower.contains("vector"), "must not contain vector terminology");
    }

    #[test]
    fn plan_rejects_empty_package_id() {
        let req = make_request(
            "official/storage-lab/plan_package_state_store",
            json!({"package_id": "", "store_id": "test"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("storage_lab_rejected"));
    }

    #[test]
    fn plan_rejects_path_traversal_package_id() {
        let req = make_request(
            "official/storage-lab/plan_package_state_store",
            json!({"package_id": "../etc/passwd", "store_id": "test"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("storage_lab_rejected"));
    }

    #[test]
    fn plan_namespace_belongs_to_package() {
        let req = make_request(
            "official/storage-lab/plan_package_state_store",
            json!({"package_id": "my-pkg", "store_id": "data"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        let namespace = result["namespace"].as_str().unwrap();
        assert!(namespace.starts_with("my-pkg/state/"), "namespace must belong to package");
    }

    #[test]
    fn put_document_preview_no_write() {
        let req = make_request(
            "official/storage-lab/put_document_preview",
            json!({"document_id": "doc1", "store_id": "default", "content": {"key": "value"}}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("document_put_preview"));
        assert_eq!(result["write_performed"], json!(false));
        assert!(result["content_address"].is_string());
    }

    #[test]
    fn get_document_preview_no_read() {
        let req = make_request(
            "official/storage-lab/get_document_preview",
            json!({"document_id": "doc1", "store_id": "default"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("document_get_preview"));
        assert_eq!(result["read_performed"], json!(false));
    }

    #[test]
    fn query_prefix_preview_no_query() {
        let req = make_request(
            "official/storage-lab/query_document_prefix_preview",
            json!({"prefix": "my-pkg/doc", "store_id": "default"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("document_query_preview"));
        assert_eq!(result["query_performed"], json!(false));
    }

    #[test]
    fn delete_tombstone_preview_no_delete() {
        let req = make_request(
            "official/storage-lab/delete_document_tombstone_preview",
            json!({"document_id": "doc1", "store_id": "default"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("document_tombstone_preview"));
        assert_eq!(result["delete_performed"], json!(false));
    }

    #[test]
    fn export_snapshot_preview_redacted() {
        let req = make_request(
            "official/storage-lab/export_store_snapshot_preview",
            json!({"store_id": "default"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("store_snapshot_preview"));
        assert_eq!(result["snapshot_exported"], json!(false));
        assert!(result["redacted_entries"].is_object());
    }

    #[test]
    fn raw_secret_rejected() {
        let req = make_request(
            "official/storage-lab/put_document_preview",
            json!({"document_id": "doc1", "api_key": "RawSecretExample1234567890abcdefABCDEF123456"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("storage_lab_rejected"));
        assert_eq!(result["redaction_state"], json!("unsafe_blocked"));
    }

    #[test]
    fn unsafe_id_rejected() {
        let req = make_request(
            "official/storage-lab/get_document_preview",
            json!({"document_id": "../../../etc/passwd", "store_id": "default"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("storage_lab_rejected"));
    }

    // ------- S3 Blob / Asset Store Contract Proof tests -------

    #[test]
    fn describe_blob_store_contract_shape() {
        let req = make_request("official/storage-lab/describe_blob_store_contract", json!({}));
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("blob_store_contract"));
        assert_eq!(result["contract_type"], json!("content_addressed_blob_store"));
        let candidates = result["backend_candidates"].as_array().unwrap();
        assert!(candidates.len() >= 3, "must have at least 3 backend candidates");
        let red_lines = result["red_lines"].as_array().unwrap();
        assert!(red_lines.contains(&json!("no_blob_content_in_events")));
        assert!(red_lines.contains(&json!("no_raw_secrets")));
        assert!(red_lines.contains(&json!("no_direct_filesystem_path_leak")));
        assert!(red_lines.contains(&json!("content_address_required")));
    }

    #[test]
    fn put_blob_preview_content_address_deterministic() {
        let req = make_request(
            "official/storage-lab/put_blob_preview",
            json!({
                "package_id": "my-pkg",
                "blob_id": "asset/image-1",
                "mime": "image/png",
                "size_bytes": 1024,
                "content_hash": "abc123def456",
            }),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("blob_put_preview"));
        // content_hash provided → normalized with sha256: prefix
        assert_eq!(result["content_address"], json!("sha256:abc123def456"));
        assert_eq!(result["blob_stored"], json!(false));
        assert_eq!(result["filesystem_performed"], json!(false));
        assert_eq!(result["network_performed"], json!(false));
        assert_eq!(result["event_payload_contains_blob"], json!(false));
    }

    #[test]
    fn put_blob_preview_no_storage_no_content_event() {
        let req = make_request(
            "official/storage-lab/put_blob_preview",
            json!({
                "package_id": "my-pkg",
                "blob_id": "doc/readme",
                "content_sample": "Hello world sample",
            }),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("blob_put_preview"));
        assert_eq!(result["blob_stored"], json!(false));
        assert_eq!(result["event_payload_contains_blob"], json!(false));
        assert!(result["content_address"].is_string());
    }

    #[test]
    fn get_blob_metadata_preview_no_content() {
        let req = make_request(
            "official/storage-lab/get_blob_metadata_preview",
            json!({"blob_id": "my-pkg/asset/image-1"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("blob_metadata_preview"));
        assert_eq!(result["blob_read"], json!(false));
        assert_eq!(result["content_returned"], json!(false));
    }

    #[test]
    fn export_blob_manifest_refs_only() {
        let req = make_request(
            "official/storage-lab/export_blob_manifest_preview",
            json!({"package_id": "my-pkg"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("blob_manifest_preview"));
        assert_eq!(result["content_included"], json!(false));
        assert_eq!(result["item_count"], json!(0));
    }

    #[test]
    fn blob_raw_secret_and_unsafe_id_rejected() {
        // Raw secret in put_blob_preview
        let req = make_request(
            "official/storage-lab/put_blob_preview",
            json!({
                "package_id": "my-pkg",
                "blob_id": "doc/1",
                "api_key": "RawSecretExample1234567890abcdefABCDEF123456",
            }),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("storage_lab_rejected"));

        // Unsafe blob_id (path traversal)
        let req2 = make_request(
            "official/storage-lab/put_blob_preview",
            json!({
                "package_id": "my-pkg",
                "blob_id": "../../etc/passwd",
            }),
        );
        let result2 = try_handle(&req2).unwrap().unwrap();
        assert_eq!(result2["kind"], json!("storage_lab_rejected"));

        // Oversized content_sample
        let big_sample = "x".repeat(5000);
        let req3 = make_request(
            "official/storage-lab/put_blob_preview",
            json!({
                "package_id": "my-pkg",
                "blob_id": "doc/1",
                "content_sample": big_sample,
            }),
        );
        let result3 = try_handle(&req3).unwrap().unwrap();
        assert_eq!(result3["kind"], json!("storage_lab_rejected"));
    }

    #[test]
    fn put_blob_preview_sha256_prefix_preserved() {
        // Already has sha256: prefix — should be preserved as-is
        let req = make_request(
            "official/storage-lab/put_blob_preview",
            json!({
                "package_id": "my-pkg",
                "blob_id": "asset/hash-test",
                "content_hash": "sha256:abc123",
            }),
        );
        let result = try_handle(&req).unwrap().unwrap();
        // sha256: prefix already present → kept as-is (not doubled)
        assert_eq!(result["content_address"], json!("sha256:abc123"));
    }

    #[test]
    fn blob_no_forbidden_namespace() {
        let req = make_request("official/storage-lab/describe_blob_store_contract", json!({}));
        let result = try_handle(&req).unwrap().unwrap();
        let output_str = serde_json::to_string(&result).unwrap();
        for token in forbidden_namespace_tokens() {
            assert!(
                !output_str.contains(&token),
                "must not contain {}",
                token
            );
        }
    }

    // ------- S4 Projection / Index Materialization Contract Proof tests -------

    #[test]
    fn describe_projection_store_contract_shape() {
        let req = make_request(
            "official/storage-lab/describe_projection_store_contract",
            json!({}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("projection_store_contract"));
        let contract_kinds = result["contract_kinds"].as_array().unwrap();
        assert!(contract_kinds.len() >= 4, "must list at least 4 contract kinds");
        let candidates = result["backend_candidates"].as_array().unwrap();
        assert!(candidates.len() >= 4, "must have at least 4 backend candidates");
        let red_lines = result["red_lines"].as_array().unwrap();
        assert!(red_lines.contains(&json!("no_table_exposure")));
        assert!(red_lines.contains(&json!("no_sql_exposure")));
        assert!(red_lines.contains(&json!("no_backend_credentials")));
        assert!(red_lines.contains(&json!("projection_derives_from_events_assets_only")));
    }

    #[test]
    fn plan_projection_materialization_plan_only() {
        let req = make_request(
            "official/storage-lab/plan_projection_materialization",
            json!({
                "package_id": "my-pkg",
                "projection_id": "my-pkg/projection/board-state",
                "source_kinds": ["my-pkg/event/action"],
                "index_keys": ["board_id", "sequence"],
            }),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("projection_materialization_plan"));
        assert_eq!(result["materialized"], json!(false));
        assert_eq!(result["write_performed"], json!(false));
        assert_eq!(result["backend_selected"], json!(false));
        assert_eq!(result["plan_only"], json!(true));
        let candidates = result["backend_candidates"].as_array().unwrap();
        assert!(candidates.len() >= 4, "must have backend candidates");
    }

    #[test]
    fn query_projection_preview_no_execution() {
        let req = make_request(
            "official/storage-lab/query_projection_preview",
            json!({
                "projection_ref": "my-pkg/projection/board-state",
                "filter_preview": {"board_id": "abc"},
                "limit": 5,
            }),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("projection_query_preview"));
        assert_eq!(result["query_executed"], json!(false));
        assert_eq!(result["rows_returned"], json!(false));
        assert!(result["preview_shape"].is_object());
    }

    #[test]
    fn migrate_projection_plan_no_rewrite() {
        let req = make_request(
            "official/storage-lab/migrate_projection_plan_preview",
            json!({
                "projection_id": "my-pkg/projection/board-state",
                "from_version": "1",
                "to_version": "2",
                "change_summary": "added sequence index",
            }),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("projection_migration_plan_preview"));
        assert_eq!(result["migration_applied"], json!(false));
        assert_eq!(result["data_rewritten"], json!(false));
        assert_eq!(result["requires_rebuild"], json!(true));
    }

    #[test]
    fn projection_rejects_raw_secret() {
        let req = make_request(
            "official/storage-lab/plan_projection_materialization",
            json!({
                "package_id": "my-pkg",
                "projection_id": "my-pkg/projection/test",
                "api_key": "RawSecretExample1234567890abcdefABCDEF123456",
            }),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("storage_lab_rejected"));
        assert_eq!(result["redaction_state"], json!("unsafe_blocked"));
    }

    #[test]
    fn projection_no_db_table_leakage() {
        let req = make_request(
            "official/storage-lab/describe_projection_store_contract",
            json!({}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        let output_str = serde_json::to_string(&result).unwrap();
        let lower = output_str.to_lowercase();
        // No SQL/table/collection/vector/database namespace terms as product terms.
        // "no_table_exposure" / "no_sql_exposure" in red_lines are negation terms (blocking
        // the leakage), not leakage themselves — filter them out before checking.
        let lower_no_negation = lower
            .replace("no_table_exposure", "")
            .replace("no_sql_exposure", "");
        assert!(!lower_no_negation.contains("\"sql\""), "must not contain standalone sql term");
        assert!(!lower_no_negation.contains("table"), "must not contain table terminology");
        assert!(!lower_no_negation.contains("collection"), "must not contain collection terminology");
        assert!(!lower_no_negation.contains("vector"), "must not contain vector terminology");
        assert!(!lower_no_negation.contains("\"database\""), "must not contain database terminology");
        // No kernel namespace tokens
        for token in forbidden_namespace_tokens() {
            assert!(
                !output_str.contains(&token),
                "must not contain forbidden namespace {}",
                token
            );
        }
    }

    #[test]
    fn projection_plan_rejects_empty_and_unsafe_ids() {
        // Empty projection_id
        let req = make_request(
            "official/storage-lab/plan_projection_materialization",
            json!({
                "package_id": "my-pkg",
                "projection_id": "",
            }),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("storage_lab_rejected"));

        // Unsafe projection_id (path traversal)
        let req2 = make_request(
            "official/storage-lab/plan_projection_materialization",
            json!({
                "package_id": "my-pkg",
                "projection_id": "../../etc/passwd",
            }),
        );
        let result2 = try_handle(&req2).unwrap().unwrap();
        assert_eq!(result2["kind"], json!("storage_lab_rejected"));

        // Empty package_id
        let req3 = make_request(
            "official/storage-lab/plan_projection_materialization",
            json!({
                "package_id": "",
                "projection_id": "my-pkg/proj/1",
            }),
        );
        let result3 = try_handle(&req3).unwrap().unwrap();
        assert_eq!(result3["kind"], json!("storage_lab_rejected"));
    }

    #[test]
    fn projection_migration_rejects_empty_and_unsafe_projection_id() {
        // Empty projection_id
        let req = make_request(
            "official/storage-lab/migrate_projection_plan_preview",
            json!({
                "projection_id": "",
            }),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("storage_lab_rejected"));

        // Unsafe projection_id
        let req2 = make_request(
            "official/storage-lab/migrate_projection_plan_preview",
            json!({
                "projection_id": "../../../etc/passwd",
            }),
        );
        let result2 = try_handle(&req2).unwrap().unwrap();
        assert_eq!(result2["kind"], json!("storage_lab_rejected"));
    }

    #[test]
    fn projection_query_rejects_unsafe_ref() {
        let req = make_request(
            "official/storage-lab/query_projection_preview",
            json!({
                "projection_ref": "../../etc/passwd",
            }),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("storage_lab_rejected"));
    }

    #[test]
    fn projection_query_and_migration_reject_raw_secret() {
        // Raw secret in query_projection_preview
        let req = make_request(
            "official/storage-lab/query_projection_preview",
            json!({
                "projection_ref": "my-pkg/proj/1",
                "token": "Bearer RawSecretExample1234567890abcdefABCDEF123456",
            }),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("storage_lab_rejected"));

        // Raw secret in migrate_projection_plan_preview
        let req2 = make_request(
            "official/storage-lab/migrate_projection_plan_preview",
            json!({
                "projection_id": "my-pkg/proj/1",
                "api_key": "RawSecretExample1234567890abcdefABCDEF123456",
            }),
        );
        let result2 = try_handle(&req2).unwrap().unwrap();
        assert_eq!(result2["kind"], json!("storage_lab_rejected"));
    }
}
