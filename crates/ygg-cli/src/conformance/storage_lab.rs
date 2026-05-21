//! Conformance tests for `official/storage-lab` (Storage Backend Neutrality Alpha S2).
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

    // 8 capabilities
    anyhow::ensure!(
        contract.output["capabilities"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0)
            == 8,
        "describe_storage_contract must list 8 capabilities"
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
