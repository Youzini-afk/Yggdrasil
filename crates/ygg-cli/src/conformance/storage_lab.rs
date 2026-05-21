//! Conformance tests for `official/storage-lab` (Storage Backend Neutrality Alpha S2 + S3).
//!
//! Covers:
//! 1. Contract shape — no kernel database terms
//! 2. Backend classes — no credentials/DSN/path
//! 3. Package state plan — scoped namespace, no official priority
//! 4. Put document preview — no real write
//! 5. Get document preview — no real read
//! 6. Query prefix preview — no query execution
//! 7. Delete tombstone preview — no real delete
//! 8. Export snapshot preview — redacted
//! 9. Raw secret rejected
//! 10. Unsafe ID rejected
//! 11. Blob contract shape — content-addressed, red lines, no forbidden namespace
//! 12. Put blob preview — content address deterministic, no storage/content/event
//! 13. Get blob metadata preview — no content returned
//! 14. Export blob manifest preview — refs only, no content
//! 15. Blob raw secret and unsafe ID rejected

use std::path::PathBuf;

use serde_json::json;
use ygg_runtime::CapabilityInvocationRequest;

use super::fixtures::*;
use crate::commands::manifest;

const PACKAGE_ID: &str = "official/storage-lab";

async fn load_storage_lab(
) -> anyhow::Result<ygg_runtime::Runtime<ygg_runtime::InMemoryEventStore>> {
    let (_store, runtime) = runtime();
    runtime
        .load_package(
            manifest::read_manifest(PathBuf::from(
                "packages/official/storage-lab/manifest.yaml",
            ))
            .await?,
        )
        .await?;
    Ok(runtime)
}

async fn invoke(
    runtime: &ygg_runtime::Runtime<ygg_runtime::InMemoryEventStore>,
    cap: &str,
    input: serde_json::Value,
) -> anyhow::Result<ygg_runtime::CapabilityInvocationResult> {
    runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: format!("{PACKAGE_ID}/{cap}"),
            caller_package_id: None,
            provider_package_id: Some(PACKAGE_ID.to_string()),
            version: None,
            input,
        })
        .await
        .map_err(Into::into)
}

/// Case 1: Contract shape — 8 capabilities, 3 surfaces, ordinary package,
/// no kernel database terms (kernel.sqlite.*, kernel.sql.*, kernel.database.*, etc.).
pub(crate) async fn contract_shape_no_kernel_database_terms() -> anyhow::Result<()> {
    let rt = load_storage_lab().await?;

    let contract = invoke(&rt, "describe_storage_contract", json!({})).await?;

    anyhow::ensure!(
        contract.output["kind"] == json!("storage_lab_contract"),
        "describe_storage_contract must return storage_lab_contract kind"
    );
    anyhow::ensure!(
        contract.output["package_kind"] == json!("ordinary"),
        "must be ordinary package"
    );

    // 3 surfaces
    let surfaces = contract.output["surfaces"].as_object().unwrap();
    anyhow::ensure!(surfaces.contains_key("forge_panel"), "must have forge_panel");
    anyhow::ensure!(surfaces.contains_key("assistant_action"), "must have assistant_action");
    anyhow::ensure!(surfaces.contains_key("home_card"), "must have home_card");

    // 12 capabilities
    anyhow::ensure!(
        contract.output["capabilities"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0)
            == 12,
        "describe_storage_contract must list 12 capabilities"
    );

    // No kernel database terms
    let output_str = serde_json::to_string(&contract.output).unwrap();
    let forbidden = [
        "kernel.sqlite.",
        "kernel.postgres.",
        "kernel.tdb.",
        "kernel.vector.",
        "kernel.embedding.",
        "kernel.collection.",
        "kernel.sql.",
        "kernel.database.",
    ];
    for token in &forbidden {
        anyhow::ensure!(
            !output_str.contains(token),
            "contract must not contain {}",
            token
        );
    }

    // No inference / no network
    anyhow::ensure!(contract.output["inference_performed"] == json!(false));
    anyhow::ensure!(contract.output["network_performed"] == json!(false));

    Ok(())
}

/// Case 2: Backend classes — capability flags only, no credentials/DSN/path.
pub(crate) async fn backend_classes_no_credentials() -> anyhow::Result<()> {
    let rt = load_storage_lab().await?;

    let result = invoke(&rt, "describe_backend_classes", json!({})).await?;

    anyhow::ensure!(
        result.output["kind"] == json!("backend_classes"),
        "must return backend_classes kind"
    );

    let output_str = serde_json::to_string(&result.output).unwrap();
    let lower = output_str.to_lowercase();
    for token in &["dsn", "connection_string", "password", "credential"] {
        anyhow::ensure!(
            !lower.contains(token),
            "backend_classes must not contain {}",
            token
        );
    }
    // No standalone SQL/table/collection/vector product terminology
    // (sqlite_event_store as a backend class identifier is allowed)
    anyhow::ensure!(
        !lower.contains("\"sql\""),
        "backend_classes must not contain standalone sql term"
    );
    anyhow::ensure!(
        !lower.contains("table"),
        "backend_classes must not contain table terminology"
    );
    anyhow::ensure!(
        !lower.contains("collection"),
        "backend_classes must not contain collection terminology"
    );
    anyhow::ensure!(
        !lower.contains("vector"),
        "backend_classes must not contain vector terminology"
    );

    // No kernel database namespace tokens
    let forbidden = [
        "kernel.sqlite.",
        "kernel.postgres.",
        "kernel.tdb.",
        "kernel.vector.",
        "kernel.embedding.",
        "kernel.collection.",
        "kernel.sql.",
        "kernel.database.",
    ];
    for token in &forbidden {
        anyhow::ensure!(
            !output_str.contains(token),
            "backend_classes must not contain {}",
            token
        );
    }

    // Backend classes have capability_flags
    let classes = result.output["backend_classes"].as_array().unwrap();
    for class in classes {
        anyhow::ensure!(
            class["capability_flags"].is_array(),
            "each backend class must have capability_flags"
        );
        anyhow::ensure!(
            class["status"].is_string(),
            "each backend class must have status"
        );
    }

    Ok(())
}

/// Case 3: Package state plan — namespace scoped to package, no official priority.
pub(crate) async fn package_state_plan_scoped() -> anyhow::Result<()> {
    let rt = load_storage_lab().await?;

    let plan = invoke(
        &rt,
        "plan_package_state_store",
        json!({
            "package_id": "thirdparty/my-app",
            "store_id": "userdata",
            "schema_hint": "document",
        }),
    )
    .await?;

    anyhow::ensure!(
        plan.output["kind"] == json!("package_state_plan"),
        "must return package_state_plan kind"
    );

    // Namespace must belong to the package
    let namespace = plan.output["namespace"].as_str().unwrap();
    anyhow::ensure!(
        namespace.starts_with("thirdparty/my-app/state/"),
        "namespace must belong to the owning package, got: {}",
        namespace
    );

    // Plan-only, requires approval
    anyhow::ensure!(plan.output["plan_only"] == json!(true));
    anyhow::ensure!(plan.output["requires_user_approval"] == json!(true));

    // Has quota hints
    anyhow::ensure!(plan.output["quota_hints"].is_object(), "must have quota_hints");

    // Has backend candidates
    anyhow::ensure!(
        plan.output["backend_candidates"].is_array(),
        "must have backend_candidates"
    );

    // No raw secrets or paths in output
    let output_str = serde_json::to_string(&plan.output).unwrap();
    let lower = output_str.to_lowercase();
    for token in &["dsn", "password", "credential", "connection_string", "file://"] {
        anyhow::ensure!(
            !lower.contains(token),
            "plan must not contain {}",
            token
        );
    }

    Ok(())
}

/// Case 4: Put document preview — write_performed=false.
pub(crate) async fn put_document_preview_no_write() -> anyhow::Result<()> {
    let rt = load_storage_lab().await?;

    let result = invoke(
        &rt,
        "put_document_preview",
        json!({
            "document_id": "my-pkg/doc/hello",
            "store_id": "default",
            "content": {"title": "Hello", "body": "World"},
        }),
    )
    .await?;

    anyhow::ensure!(
        result.output["kind"] == json!("document_put_preview"),
        "must return document_put_preview kind"
    );
    anyhow::ensure!(
        result.output["write_performed"] == json!(false),
        "write_performed must be false"
    );
    anyhow::ensure!(
        result.output["content_address"].is_string(),
        "must have content_address"
    );
    anyhow::ensure!(
        result.output["redacted_content"].is_object(),
        "must have redacted_content"
    );

    Ok(())
}

/// Case 5: Get document preview — read_performed=false.
pub(crate) async fn get_document_preview_no_read() -> anyhow::Result<()> {
    let rt = load_storage_lab().await?;

    let result = invoke(
        &rt,
        "get_document_preview",
        json!({
            "document_id": "my-pkg/doc/hello",
            "store_id": "default",
        }),
    )
    .await?;

    anyhow::ensure!(
        result.output["kind"] == json!("document_get_preview"),
        "must return document_get_preview kind"
    );
    anyhow::ensure!(
        result.output["read_performed"] == json!(false),
        "read_performed must be false"
    );

    Ok(())
}

/// Case 6: Query prefix preview — query_performed=false.
pub(crate) async fn query_prefix_preview_no_query_execution() -> anyhow::Result<()> {
    let rt = load_storage_lab().await?;

    let result = invoke(
        &rt,
        "query_document_prefix_preview",
        json!({
            "prefix": "my-pkg/doc",
            "store_id": "default",
        }),
    )
    .await?;

    anyhow::ensure!(
        result.output["kind"] == json!("document_query_preview"),
        "must return document_query_preview kind"
    );
    anyhow::ensure!(
        result.output["query_performed"] == json!(false),
        "query_performed must be false"
    );

    Ok(())
}

/// Case 7: Delete tombstone preview — delete_performed=false.
pub(crate) async fn delete_tombstone_preview_no_delete() -> anyhow::Result<()> {
    let rt = load_storage_lab().await?;

    let result = invoke(
        &rt,
        "delete_document_tombstone_preview",
        json!({
            "document_id": "my-pkg/doc/hello",
            "store_id": "default",
        }),
    )
    .await?;

    anyhow::ensure!(
        result.output["kind"] == json!("document_tombstone_preview"),
        "must return document_tombstone_preview kind"
    );
    anyhow::ensure!(
        result.output["delete_performed"] == json!(false),
        "delete_performed must be false"
    );
    anyhow::ensure!(
        result.output["tombstone_status"] == json!("preview_only"),
        "tombstone_status must be preview_only"
    );

    Ok(())
}

/// Case 8: Export snapshot preview — redacted, snapshot_exported=false.
pub(crate) async fn export_snapshot_preview_redacted() -> anyhow::Result<()> {
    let rt = load_storage_lab().await?;

    let result = invoke(
        &rt,
        "export_store_snapshot_preview",
        json!({
            "store_id": "default",
        }),
    )
    .await?;

    anyhow::ensure!(
        result.output["kind"] == json!("store_snapshot_preview"),
        "must return store_snapshot_preview kind"
    );
    anyhow::ensure!(
        result.output["snapshot_exported"] == json!(false),
        "snapshot_exported must be false"
    );
    anyhow::ensure!(
        result.output["redacted_entries"].is_object(),
        "must have redacted_entries"
    );

    Ok(())
}

/// Case 9: Raw secret rejected across all mutating capabilities.
pub(crate) async fn raw_secret_rejected() -> anyhow::Result<()> {
    let rt = load_storage_lab().await?;

    // put_document_preview blocks raw secret
    let put = invoke(
        &rt,
        "put_document_preview",
        json!({
            "document_id": "doc1",
            "api_key": "RawSecretExample1234567890abcdefABCDEF123456",
        }),
    )
    .await?;
    anyhow::ensure!(put.output["kind"] == json!("storage_lab_rejected"));
    anyhow::ensure!(put.output["redaction_state"] == json!("unsafe_blocked"));

    // plan_package_state_store blocks raw secret
    let plan = invoke(
        &rt,
        "plan_package_state_store",
        json!({
            "package_id": "my-pkg",
            "token": "Bearer abc123",
        }),
    )
    .await?;
    anyhow::ensure!(plan.output["kind"] == json!("storage_lab_rejected"));

    // get_document_preview blocks raw secret
    let get = invoke(
        &rt,
        "get_document_preview",
        json!({
            "document_id": "doc1",
            "secret": "RawSecretExample1234567890abcdefABCDEF123456",
        }),
    )
    .await?;
    anyhow::ensure!(get.output["kind"] == json!("storage_lab_rejected"));

    Ok(())
}

/// Case 10: Unsafe ID rejected (path traversal, special characters).
pub(crate) async fn unsafe_id_rejected() -> anyhow::Result<()> {
    let rt = load_storage_lab().await?;

    // Path traversal in document_id
    let get = invoke(
        &rt,
        "get_document_preview",
        json!({
            "document_id": "../../../etc/passwd",
            "store_id": "default",
        }),
    )
    .await?;
    anyhow::ensure!(get.output["kind"] == json!("storage_lab_rejected"));

    // Path traversal in store_id
    let put = invoke(
        &rt,
        "put_document_preview",
        json!({
            "document_id": "doc1",
            "store_id": "../escape",
        }),
    )
    .await?;
    anyhow::ensure!(put.output["kind"] == json!("storage_lab_rejected"));

    // Empty package_id in plan
    let plan = invoke(
        &rt,
        "plan_package_state_store",
        json!({
            "package_id": "",
            "store_id": "test",
        }),
    )
    .await?;
    anyhow::ensure!(plan.output["kind"] == json!("storage_lab_rejected"));

    Ok(())
}

/// Case 11: Blob contract shape — content-addressed, backend candidates, red lines,
/// no kernel database/vector/blob namespace.
pub(crate) async fn blob_contract_shape() -> anyhow::Result<()> {
    let rt = load_storage_lab().await?;

    let contract = invoke(&rt, "describe_blob_store_contract", json!({})).await?;

    anyhow::ensure!(
        contract.output["kind"] == json!("blob_store_contract"),
        "must return blob_store_contract kind"
    );
    anyhow::ensure!(
        contract.output["contract_type"] == json!("content_addressed_blob_store"),
        "contract_type must be content_addressed_blob_store"
    );

    // Backend candidates
    let candidates = contract.output["backend_candidates"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("backend_candidates must be array"))?;
    anyhow::ensure!(candidates.len() >= 3, "must have at least 3 backend candidates");

    // Red lines
    let red_lines = contract.output["red_lines"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("red_lines must be array"))?;
    anyhow::ensure!(
        red_lines.contains(&json!("no_blob_content_in_events")),
        "must have no_blob_content_in_events red line"
    );
    anyhow::ensure!(
        red_lines.contains(&json!("no_raw_secrets")),
        "must have no_raw_secrets red line"
    );
    anyhow::ensure!(
        red_lines.contains(&json!("no_direct_filesystem_path_leak")),
        "must have no_direct_filesystem_path_leak red line"
    );
    anyhow::ensure!(
        red_lines.contains(&json!("content_address_required")),
        "must have content_address_required red line"
    );

    // No forbidden namespace tokens
    let output_str = serde_json::to_string(&contract.output).unwrap();
    let forbidden = [
        "kernel.sqlite.",
        "kernel.postgres.",
        "kernel.tdb.",
        "kernel.vector.",
        "kernel.embedding.",
        "kernel.collection.",
        "kernel.sql.",
        "kernel.database.",
        "kernel.blob.",
    ];
    for token in &forbidden {
        anyhow::ensure!(
            !output_str.contains(token),
            "blob contract must not contain {}",
            token
        );
    }

    // No credentials/paths/bucket names in backend candidates
    let lower = output_str.to_lowercase();
    for token in &["dsn", "connection_string", "password", "credential", "bucket", "s3://", "gcs://"] {
        anyhow::ensure!(
            !lower.contains(token),
            "blob contract must not contain {}",
            token
        );
    }

    // No inference / no network
    anyhow::ensure!(contract.output["inference_performed"] == json!(false));
    anyhow::ensure!(contract.output["network_performed"] == json!(false));

    Ok(())
}

/// Case 12: Put blob preview — content address deterministic, no real storage,
/// no blob content in events.
pub(crate) async fn put_blob_preview_content_address_deterministic() -> anyhow::Result<()> {
    let rt = load_storage_lab().await?;

    // With content_hash provided → normalized sha256: prefix
    let result = invoke(
        &rt,
        "put_blob_preview",
        json!({
            "package_id": "thirdparty/my-app",
            "blob_id": "asset/avatar",
            "mime": "image/png",
            "size_bytes": 2048,
            "content_hash": "deadbeef1234",
        }),
    )
    .await?;

    anyhow::ensure!(
        result.output["kind"] == json!("blob_put_preview"),
        "must return blob_put_preview kind"
    );
    anyhow::ensure!(
        result.output["content_address"] == json!("sha256:deadbeef1234"),
        "content_hash must be normalized with sha256: prefix"
    );
    anyhow::ensure!(
        result.output["mime"] == json!("image/png"),
        "mime must match input"
    );
    anyhow::ensure!(
        result.output["size_bytes"] == json!(2048),
        "size_bytes must match input"
    );

    // Deterministic: same input → same content_address
    let result2 = invoke(
        &rt,
        "put_blob_preview",
        json!({
            "package_id": "thirdparty/my-app",
            "blob_id": "asset/avatar",
            "mime": "image/png",
            "size_bytes": 2048,
            "content_hash": "deadbeef1234",
        }),
    )
    .await?;
    anyhow::ensure!(
        result.output["content_address"] == result2.output["content_address"],
        "same input must produce same content_address"
    );

    Ok(())
}

/// Case 13: Put blob preview — no real storage, no blob content in event payload.
pub(crate) async fn put_blob_preview_no_storage_no_content_event() -> anyhow::Result<()> {
    let rt = load_storage_lab().await?;

    let result = invoke(
        &rt,
        "put_blob_preview",
        json!({
            "package_id": "my-pkg",
            "blob_id": "doc/readme",
            "content_sample": "Hello world",
        }),
    )
    .await?;

    anyhow::ensure!(
        result.output["kind"] == json!("blob_put_preview"),
        "must return blob_put_preview kind"
    );
    anyhow::ensure!(
        result.output["blob_stored"] == json!(false),
        "blob_stored must be false"
    );
    anyhow::ensure!(
        result.output["filesystem_performed"] == json!(false),
        "filesystem_performed must be false"
    );
    anyhow::ensure!(
        result.output["network_performed"] == json!(false),
        "network_performed must be false"
    );
    anyhow::ensure!(
        result.output["event_payload_contains_blob"] == json!(false),
        "event_payload_contains_blob must be false"
    );
    anyhow::ensure!(
        result.output["content_address"].is_string(),
        "must have content_address"
    );

    Ok(())
}

/// Case 14: Get blob metadata preview — no blob content returned.
pub(crate) async fn get_blob_metadata_preview_no_content() -> anyhow::Result<()> {
    let rt = load_storage_lab().await?;

    let result = invoke(
        &rt,
        "get_blob_metadata_preview",
        json!({
            "blob_id": "my-pkg/asset/avatar",
        }),
    )
    .await?;

    anyhow::ensure!(
        result.output["kind"] == json!("blob_metadata_preview"),
        "must return blob_metadata_preview kind"
    );
    anyhow::ensure!(
        result.output["blob_read"] == json!(false),
        "blob_read must be false"
    );
    anyhow::ensure!(
        result.output["content_returned"] == json!(false),
        "content_returned must be false"
    );
    anyhow::ensure!(
        result.output["content_address"].is_string(),
        "must have content_address"
    );

    Ok(())
}

/// Case 15: Export blob manifest preview — refs only, no content.
pub(crate) async fn export_blob_manifest_refs_only() -> anyhow::Result<()> {
    let rt = load_storage_lab().await?;

    let result = invoke(
        &rt,
        "export_blob_manifest_preview",
        json!({
            "package_id": "my-pkg",
        }),
    )
    .await?;

    anyhow::ensure!(
        result.output["kind"] == json!("blob_manifest_preview"),
        "must return blob_manifest_preview kind"
    );
    anyhow::ensure!(
        result.output["content_included"] == json!(false),
        "content_included must be false"
    );
    anyhow::ensure!(
        result.output["manifest_items"].is_array(),
        "must have manifest_items array"
    );

    Ok(())
}

/// Case 16: Blob raw secret and unsafe ID rejected.
pub(crate) async fn blob_raw_secret_and_unsafe_id_rejected() -> anyhow::Result<()> {
    let rt = load_storage_lab().await?;

    // Raw secret in put_blob_preview
    let put = invoke(
        &rt,
        "put_blob_preview",
        json!({
            "package_id": "my-pkg",
            "blob_id": "doc/1",
            "api_key": "RawSecretExample1234567890abcdefABCDEF123456",
        }),
    )
    .await?;
    anyhow::ensure!(put.output["kind"] == json!("storage_lab_rejected"));
    anyhow::ensure!(put.output["redaction_state"] == json!("unsafe_blocked"));

    // Path traversal in blob_id
    let put2 = invoke(
        &rt,
        "put_blob_preview",
        json!({
            "package_id": "my-pkg",
            "blob_id": "../../etc/passwd",
        }),
    )
    .await?;
    anyhow::ensure!(put2.output["kind"] == json!("storage_lab_rejected"));

    // Path traversal in package_id
    let put3 = invoke(
        &rt,
        "put_blob_preview",
        json!({
            "package_id": "../escape",
            "blob_id": "doc/1",
        }),
    )
    .await?;
    anyhow::ensure!(put3.output["kind"] == json!("storage_lab_rejected"));

    // Raw secret in get_blob_metadata_preview
    let get = invoke(
        &rt,
        "get_blob_metadata_preview",
        json!({
            "blob_id": "doc/1",
            "secret": "Bearer abc123xyz456",
        }),
    )
    .await?;
    anyhow::ensure!(get.output["kind"] == json!("storage_lab_rejected"));

    // Raw secret in export_blob_manifest_preview
    let export = invoke(
        &rt,
        "export_blob_manifest_preview",
        json!({
            "package_id": "my-pkg",
            "token": "RawSecretExample1234567890abcdefABCDEF123456",
        }),
    )
    .await?;
    anyhow::ensure!(export.output["kind"] == json!("storage_lab_rejected"));

    // Oversized content_sample rejected
    let big_sample = "x".repeat(5000);
    let put4 = invoke(
        &rt,
        "put_blob_preview",
        json!({
            "package_id": "my-pkg",
            "blob_id": "doc/big",
            "content_sample": big_sample,
        }),
    )
    .await?;
    anyhow::ensure!(put4.output["kind"] == json!("storage_lab_rejected"));

    Ok(())
}
