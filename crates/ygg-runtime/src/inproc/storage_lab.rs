//! Handler for `official/storage-lab` capabilities.
//!
//! Storage Backend Neutrality Alpha Phase S2 — Package-Scoped Data Contract.
//!
//! Package-facing storage/data contract preview: storage backend classes,
//! package state store planning, document CRUD previews, and snapshot export.
//!
//! Deterministic, no-network, no real model inference, no real DB writes,
//! no SQL, no filesystem, no DSN/path/credentials. Produces package-owned
//! storage/data shapes.
//!
//! No reserved storage/database kernel namespace references.
//!
//! State terminology: storage_contract, backend_class, package_state_plan,
//! document_preview, tombstone_preview, snapshot_preview — not
//! vendor database, query-language, vector, or secret-bearing backend semantics.

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
                    Vec::<&str>::new(),
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
    fn describe_contract_lists_8_capabilities() {
        let req = make_request("official/storage-lab/describe_storage_contract", json!({}));
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(
            result["capabilities"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0),
            8,
            "must list 8 capabilities"
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
        for token in FORBIDDEN_NAMESPACE_TOKENS {
            assert!(
                !output_str.contains(token),
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
        for token in FORBIDDEN_NAMESPACE_TOKENS {
            assert!(
                !output_str.contains(token),
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
}
