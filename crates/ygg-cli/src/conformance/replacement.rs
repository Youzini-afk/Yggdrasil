//! Conformance cases for Phase H5: proving official seed is replaceable
//! with a third-party package, no kernel privilege or hardcoding required.

use std::path::PathBuf;

use serde_json::json;
use ygg_runtime::{CapabilityInvocationRequest, ProtocolContext};

use super::fixtures::*;
use crate::commands::{composition, manifest};

/// Proves that the third-party playable-seed replacement package loads and its
/// surfaces are discoverable through `kernel.surface.contribution.list`.
pub(crate) async fn thirdparty_seed_surfaces() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime
        .load_package(manifest::read_manifest(PathBuf::from("examples/packages/thirdparty-playable-seed/manifest.yaml")).await?)
        .await?;

    // List all surfaces — should include at least the 5 from thirdparty-playable-seed
    let all = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.surface.contribution.list",
            json!({}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(all.as_array().map(|items| items.len()).unwrap_or(0) >= 5, "third-party surfaces not listed");

    // Check experience_entry slot
    let entries = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.surface.contribution.list",
            json!({"slot": "experience_entry"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let has_entry = entries
        .as_array()
        .map(|items| items.iter().any(|r| r["package_id"] == json!("thirdparty/playable-seed")))
        .unwrap_or(false);
    anyhow::ensure!(has_entry, "third-party experience_entry surface missing");

    // Check play_renderer slot
    let renderers = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.surface.contribution.list",
            json!({"slot": "play_renderer"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let has_renderer = renderers
        .as_array()
        .map(|items| items.iter().any(|r| r["package_id"] == json!("thirdparty/playable-seed")))
        .unwrap_or(false);
    anyhow::ensure!(has_renderer, "third-party play_renderer surface missing");

    // Check forge_panel slot
    let forge_panels = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.surface.contribution.list",
            json!({"slot": "forge_panel"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let has_forge = forge_panels
        .as_array()
        .map(|items| items.iter().any(|r| r["package_id"] == json!("thirdparty/playable-seed")))
        .unwrap_or(false);
    anyhow::ensure!(has_forge, "third-party forge_panel surface missing");

    // Check assistant_action slot
    let assistants = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.surface.contribution.list",
            json!({"slot": "assistant_action"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let has_assist = assistants
        .as_array()
        .map(|items| items.iter().any(|r| r["package_id"] == json!("thirdparty/playable-seed")))
        .unwrap_or(false);
    anyhow::ensure!(has_assist, "third-party assistant_action surface missing");

    // Check asset_editor slot
    let editors = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.surface.contribution.list",
            json!({"slot": "asset_editor"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let has_editor = editors
        .as_array()
        .map(|items| items.iter().any(|r| r["package_id"] == json!("thirdparty/playable-seed")))
        .unwrap_or(false);
    anyhow::ensure!(has_editor, "third-party asset_editor surface missing");

    Ok(())
}

/// Proves that capability invocation works for the third-party playable-seed
/// package through normal routing.
pub(crate) async fn thirdparty_seed_invocation() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime
        .load_package(manifest::read_manifest(PathBuf::from("examples/packages/thirdparty-playable-seed/manifest.yaml")).await?)
        .await?;

    // Invoke the launch capability through normal routing (echo inproc echoes input)
    let launch = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "thirdparty/playable-seed/launch".to_string(),
            caller_package_id: None,
            provider_package_id: Some("thirdparty/playable-seed".to_string()),
            version: None,
            input: json!({"title": "Community Seed"}),
        })
        .await?;
    // Echo inproc returns input, so verify the input is echoed
    anyhow::ensure!(launch.output["title"] == json!("Community Seed"), "third-party launch capability did not echo input");

    // Invoke render_payload capability
    let render = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "thirdparty/playable-seed/render_payload".to_string(),
            caller_package_id: None,
            provider_package_id: Some("thirdparty/playable-seed".to_string()),
            version: None,
            input: json!({}),
        })
        .await?;
    anyhow::ensure!(render.provider_package_id == "thirdparty/playable-seed", "third-party render routed to wrong provider");

    Ok(())
}

/// Proves that when both an official and a third-party package provide the same
/// capability ID, the kernel does NOT prefer the official package. The ambiguous
/// route is rejected, requiring explicit provider selection — same as any other
/// duplicate-provider scenario.
pub(crate) async fn ambiguous_no_official_priority() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();

    // Load an official package that provides a shared capability ID
    runtime
        .load_package(echo_package("official/replacement-fixture", "shared/playable-seed/launch"))
        .await?;
    // Load a third-party package that provides the SAME capability ID
    runtime
        .load_package(echo_package("thirdparty/replacement-fixture", "shared/playable-seed/launch"))
        .await?;

    // Without explicit provider, the route should be ambiguous — NOT preferred to official
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "shared/playable-seed/launch".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "ambiguous route should be rejected, official package should NOT win");

    // With explicit third-party provider, it should work — proving no official priority
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "shared/playable-seed/launch".to_string(),
            caller_package_id: None,
            provider_package_id: Some("thirdparty/replacement-fixture".to_string()),
            version: None,
            input: json!({"source": "thirdparty"}),
        })
        .await?;
    anyhow::ensure!(
        result.provider_package_id == "thirdparty/replacement-fixture",
        "explicit third-party provider was not used"
    );

    Ok(())
}

/// Proves that composition check passes with the third-party replacement
/// composition descriptor.
pub(crate) async fn composition_thirdparty() -> anyhow::Result<()> {
    composition::composition_check(PathBuf::from("examples/compositions/playable-seed-replacement/composition.yaml")).await?;
    Ok(())
}
