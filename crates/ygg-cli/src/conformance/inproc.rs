use serde_json::json;
use ygg_core::{PackageContributions, PackageEntry, PermissionSet, SandboxPolicy};
use ygg_runtime::CapabilityInvocationRequest;

use super::fixtures::*;

/// Proves that a non-official inproc package with a `/preview` capability suffix
/// does NOT receive `asset_preview` output from the shared common handlers.
///
/// The common handlers are restricted to `official/` packages; a third-party
/// package routed through the same `official-foundation` inproc entry should
/// have its capabilities go unhandled rather than silently served by the
/// generic fallback.
pub(crate) async fn non_official_preview_rejected() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    // Register a non-official package that shares the official-foundation inproc entry
    // but whose package ID is outside the official/ namespace.
    runtime
        .load_package(ygg_core::PackageManifest {
            schema_version: 1,
            id: "thirdparty/asset-hijack".to_string(),
            version: "0.1.0".to_string(),
            display_name: None,
            description: None,
            author: None,
            license: None,
            entry: PackageEntry::RustInproc {
                crate_ref: "official-foundation".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            },
            provides: vec![ygg_core::CapabilityDescriptor {
                id: "thirdparty/asset-hijack/preview".to_string(),
                version: "0.1.0".to_string(),
                input_schema: serde_json::Value::Null,
                output_schema: serde_json::Value::Null,
                streaming: false,
                side_effects: Vec::new(),
                description: None,
            }],
            consumes: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet::default(),
            sandbox_policy: SandboxPolicy::default(),
        })
        .await?;
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "thirdparty/asset-hijack/preview".to_string(),
            caller_package_id: None,
            provider_package_id: Some("thirdparty/asset-hijack".to_string()),
            version: None,
            input: json!({"asset_id": "test", "content": "should not preview"}),
        })
        .await;
    anyhow::ensure!(result.is_err(), "non-official package with /preview suffix should not succeed (no asset_preview fallback)");
    Ok(())
}

/// Proves that an unknown/unimplemented inproc capability from an official package
/// returns an explicit error instead of a generic permissive success.
///
/// Before the package-aware fix, unhandled capabilities fell through to
/// `common::fallback` which returned `{"ok": true}`. This test ensures that
/// gap is closed: unknown capabilities must fail loudly.
pub(crate) async fn unknown_inproc_capability_errors() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    // Use an official package that goes through official-foundation inproc entry
    runtime
        .load_package(ygg_core::PackageManifest {
            schema_version: 1,
            id: "official/test-unknown-cap".to_string(),
            version: "0.1.0".to_string(),
            display_name: None,
            description: None,
            author: None,
            license: None,
            entry: PackageEntry::RustInproc {
                crate_ref: "official-foundation".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            },
            provides: vec![ygg_core::CapabilityDescriptor {
                id: "official/test-unknown-cap/nonexistent_action".to_string(),
                version: "0.1.0".to_string(),
                input_schema: serde_json::Value::Null,
                output_schema: serde_json::Value::Null,
                streaming: false,
                side_effects: Vec::new(),
                description: None,
            }],
            consumes: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet::default(),
            sandbox_policy: SandboxPolicy::default(),
        })
        .await?;
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/test-unknown-cap/nonexistent_action".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/test-unknown-cap".to_string()),
            version: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(result.is_err(), "unknown inproc capability should return an error, not generic success");
    let err_msg = result.unwrap_err().to_string();
    anyhow::ensure!(
        err_msg.contains("no handler for inproc capability"),
        "error message should mention missing handler, got: {err_msg}"
    );
    Ok(())
}
