//! Conformance tests for `official/project-intake-lab` (External Project Operating Plane Alpha Phase E1).
//!
//! Covers:
//! 1. Intake contract shape (7 capabilities, 3 surfaces, ordinary package, no execution)
//! 2. Source classification (git/npm/local/archive/unknown)
//! 3. Stack detection (node/rust/python/static/unknown) with npm lifecycle risk
//! 4. Workspace plan is plan-only
//! 5. Local path rejection (path traversal, home path, absolute sensitive path)
//! 6. Adapter plan is plan-only
//! 7. No forbidden namespace (kernel.project/workspace/git/npm/deploy/ide)
//! 8. No raw secrets in any capability

use std::path::PathBuf;

use serde_json::json;
use ygg_runtime::CapabilityInvocationRequest;

use super::fixtures::*;
use crate::commands::manifest;

const PACKAGE_ID: &str = "official/project-intake-lab";

async fn load_project_intake_lab(
) -> anyhow::Result<ygg_runtime::Runtime<ygg_runtime::InMemoryEventStore>> {
    let (_store, runtime) = runtime();
    runtime
        .load_package(
            manifest::read_manifest(PathBuf::from(
                "packages/official/project-intake-lab/manifest.yaml",
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

/// Case 1: Intake contract — 7 capabilities, 3 surfaces, ordinary package,
/// no execution, no forbidden namespace.
pub(crate) async fn project_intake_contract() -> anyhow::Result<()> {
    let rt = load_project_intake_lab().await?;

    let contract = invoke(&rt, "describe_intake_contract", json!({})).await?;

    anyhow::ensure!(
        contract.output["kind"] == json!("project_intake_contract"),
        "describe_intake_contract must return project_intake_contract kind"
    );
    anyhow::ensure!(
        contract.output["package_kind"] == json!("ordinary"),
        "must be ordinary package"
    );

    // 3 surfaces
    let surfaces = contract.output["surfaces"].as_object().unwrap();
    anyhow::ensure!(
        surfaces.contains_key("forge_panel"),
        "must have forge_panel"
    );
    anyhow::ensure!(
        surfaces.contains_key("assistant_action"),
        "must have assistant_action"
    );
    anyhow::ensure!(surfaces.contains_key("home_card"), "must have home_card");

    // 7 capabilities
    anyhow::ensure!(
        contract.output["capabilities"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0)
            == 7,
        "describe_intake_contract must list 7 capabilities"
    );

    // No execution
    anyhow::ensure!(contract.output["execution_performed"] == json!(false));
    anyhow::ensure!(contract.output["network_performed"] == json!(false));
    anyhow::ensure!(contract.output["inference_performed"] == json!(false));
    anyhow::ensure!(contract.output["filesystem_performed"] == json!(false));

    // No forbidden namespace
    let output_str = serde_json::to_string(&contract.output).unwrap();
    for token in &[
        "kernel.project.",
        "kernel.workspace.",
        "kernel.git.",
        "kernel.npm.",
        "kernel.deploy.",
        "kernel.ide.",
    ] {
        anyhow::ensure!(
            !output_str.contains(token),
            "describe_intake_contract must not contain {}",
            token
        );
    }

    Ok(())
}

/// Case 2: Source classification — git/npm/local/archive/unknown.
pub(crate) async fn project_intake_source_classification() -> anyhow::Result<()> {
    let rt = load_project_intake_lab().await?;

    // git
    let git = invoke(
        &rt,
        "inspect_external_project_ref",
        json!({"source_ref": "https://github.com/example/project.git"}),
    )
    .await?;
    anyhow::ensure!(git.output["source_kind"] == json!("git"));

    // npm
    let npm = invoke(
        &rt,
        "inspect_external_project_ref",
        json!({"source_ref": "npm:lodash"}),
    )
    .await?;
    anyhow::ensure!(npm.output["source_kind"] == json!("npm"));

    // local
    let local = invoke(
        &rt,
        "inspect_external_project_ref",
        json!({"source_ref": "./my-project"}),
    )
    .await?;
    anyhow::ensure!(local.output["source_kind"] == json!("local"));
    anyhow::ensure!(local.output["path_safety"] == json!("appears_safe"));

    // archive
    let archive = invoke(
        &rt,
        "inspect_external_project_ref",
        json!({"source_ref": "project.tar.gz"}),
    )
    .await?;
    anyhow::ensure!(archive.output["source_kind"] == json!("archive"));

    // no execution performed
    anyhow::ensure!(git.output["execution_performed"] == json!(false));
    anyhow::ensure!(git.output["network_performed"] == json!(false));

    Ok(())
}

/// Case 3: Stack detection — node with npm lifecycle risk, rust, python, unknown.
pub(crate) async fn project_intake_stack_detection() -> anyhow::Result<()> {
    let rt = load_project_intake_lab().await?;

    // Node with lifecycle scripts
    let node = invoke(
        &rt,
        "detect_project_stack_from_metadata",
        json!({"metadata": {
            "package_json": {
                "name": "test",
                "scripts": {
                    "preinstall": "echo pre",
                    "install": "echo install",
                    "postinstall": "echo post",
                    "prepare": "echo prepare",
                    "prepublish": "echo prepublish",
                    "start": "node index.js"
                }
            }
        }}),
    )
    .await?;

    anyhow::ensure!(node.output["detected_stack"] == json!("node"));
    let risks = node.output["npm_lifecycle_risks"].as_array().unwrap();
    anyhow::ensure!(
        risks.len() == 5,
        "must detect 5 npm lifecycle scripts, got {}",
        risks.len()
    );
    for risk in risks {
        anyhow::ensure!(
            risk["executes_code"] == json!(true),
            "npm lifecycle risk must have executes_code=true"
        );
        anyhow::ensure!(
            risk["requires_approval"] == json!(true),
            "npm lifecycle risk must have requires_approval=true"
        );
    }

    // Rust
    let rust = invoke(
        &rt,
        "detect_project_stack_from_metadata",
        json!({"metadata": {"cargo_toml": {"name": "test"}}}),
    )
    .await?;
    anyhow::ensure!(rust.output["detected_stack"] == json!("rust"));

    // Python
    let python = invoke(
        &rt,
        "detect_project_stack_from_metadata",
        json!({"metadata": {"pyproject": {"name": "test"}}}),
    )
    .await?;
    anyhow::ensure!(python.output["detected_stack"] == json!("python"));

    // Unknown
    let unknown = invoke(
        &rt,
        "detect_project_stack_from_metadata",
        json!({"metadata": {}}),
    )
    .await?;
    anyhow::ensure!(unknown.output["detected_stack"] == json!("unknown"));

    Ok(())
}

/// Case 4: Workspace plan is plan-only, no execution.
pub(crate) async fn project_intake_workspace_plan() -> anyhow::Result<()> {
    let rt = load_project_intake_lab().await?;

    let plan = invoke(
        &rt,
        "draft_workspace_plan",
        json!({"source_ref": "https://github.com/example/project.git", "source_kind": "git"}),
    )
    .await?;

    anyhow::ensure!(plan.output["kind"] == json!("project_workspace_plan"));
    anyhow::ensure!(plan.output["plan_only"] == json!(true));
    anyhow::ensure!(plan.output["requires_user_approval"] == json!(true));
    anyhow::ensure!(plan.output["execution_performed"] == json!(false));
    anyhow::ensure!(plan.output["network_performed"] == json!(false));

    Ok(())
}

/// Case 5: Local path rejection — path traversal, home path, absolute sensitive path.
pub(crate) async fn project_intake_local_path_rejection() -> anyhow::Result<()> {
    let rt = load_project_intake_lab().await?;

    // Path traversal
    let traversal = invoke(
        &rt,
        "inspect_external_project_ref",
        json!({"source_ref": "../../etc/passwd"}),
    )
    .await?;
    anyhow::ensure!(
        traversal.output["path_safety"] == json!("rejected"),
        "path traversal must be rejected"
    );

    // Home path
    let home = invoke(
        &rt,
        "inspect_external_project_ref",
        json!({"source_ref": "~/secret-project"}),
    )
    .await?;
    anyhow::ensure!(
        home.output["path_safety"] == json!("rejected"),
        "home path must be rejected"
    );

    // Absolute sensitive path
    let abs = invoke(
        &rt,
        "inspect_external_project_ref",
        json!({"source_ref": "/etc/shadow"}),
    )
    .await?;
    anyhow::ensure!(
        abs.output["path_safety"] == json!("rejected"),
        "absolute sensitive path must be rejected"
    );

    // Workspace plan also rejects unsafe local paths
    let ws_unsafe = invoke(
        &rt,
        "draft_workspace_plan",
        json!({"source_ref": "~/secret-project", "source_kind": "local"}),
    )
    .await?;
    anyhow::ensure!(
        ws_unsafe.output["kind"] == json!("project_intake_rejected"),
        "workspace plan must reject unsafe local paths"
    );
    anyhow::ensure!(ws_unsafe.output["redaction_state"] == json!("unsafe_blocked"));

    // Adapter plan also rejects unsafe local paths
    let ad_unsafe = invoke(
        &rt,
        "draft_adapter_plan",
        json!({"source_ref": "~/secret-project", "source_kind": "local"}),
    )
    .await?;
    anyhow::ensure!(
        ad_unsafe.output["kind"] == json!("project_intake_rejected"),
        "adapter plan must reject unsafe local paths"
    );

    Ok(())
}

/// Case 6: Adapter plan is plan-only, no execution.
pub(crate) async fn project_intake_adapter_plan() -> anyhow::Result<()> {
    let rt = load_project_intake_lab().await?;

    let plan = invoke(
        &rt,
        "draft_adapter_plan",
        json!({"source_ref": "./my-project", "source_kind": "local"}),
    )
    .await?;

    anyhow::ensure!(plan.output["kind"] == json!("project_adapter_plan"));
    anyhow::ensure!(plan.output["plan_only"] == json!(true));
    anyhow::ensure!(plan.output["requires_user_approval"] == json!(true));
    anyhow::ensure!(plan.output["execution_performed"] == json!(false));
    anyhow::ensure!(plan.output["network_performed"] == json!(false));

    // Must have proposed capabilities
    let caps = plan.output["proposed_capabilities"].as_array().unwrap();
    anyhow::ensure!(!caps.is_empty(), "adapter plan must propose capabilities");

    Ok(())
}

/// Case 7: No forbidden namespace in any capability output.
pub(crate) async fn project_intake_no_forbidden_namespace() -> anyhow::Result<()> {
    let rt = load_project_intake_lab().await?;

    let caps = [
        "describe_intake_contract",
        "inspect_external_project_ref",
        "detect_project_stack_from_metadata",
        "draft_workspace_plan",
        "draft_security_risk_summary",
        "list_candidate_entrypoints",
        "draft_adapter_plan",
    ];

    let forbidden = [
        "kernel.project.",
        "kernel.workspace.",
        "kernel.git.",
        "kernel.npm.",
        "kernel.deploy.",
        "kernel.ide.",
    ];

    for cap in &caps {
        let result = invoke(&rt, cap, json!({"source_ref": "./test"})).await?;
        let output_str = serde_json::to_string(&result.output).unwrap();
        for token in &forbidden {
            anyhow::ensure!(
                !output_str.contains(token),
                "{cap} must not contain {token}"
            );
        }
    }

    Ok(())
}

/// Case 8: No raw secrets in any capability.
pub(crate) async fn project_intake_no_raw_secrets() -> anyhow::Result<()> {
    let rt = load_project_intake_lab().await?;

    // inspect blocks raw secret
    let inspect = invoke(
        &rt,
        "inspect_external_project_ref",
        json!({"source_ref": "test", "api_key": "RawSecretExample1234567890abcdefABCDEF123456"}),
    )
    .await?;
    anyhow::ensure!(inspect.output["kind"] == json!("project_intake_rejected"));
    anyhow::ensure!(inspect.output["redaction_state"] == json!("unsafe_blocked"));

    // draft_workspace_plan blocks raw secret
    let ws = invoke(
        &rt,
        "draft_workspace_plan",
        json!({"source_ref": "test", "token": "Bearer abc123"}),
    )
    .await?;
    anyhow::ensure!(ws.output["kind"] == json!("project_intake_rejected"));

    // draft_adapter_plan blocks raw secret
    let ad = invoke(
        &rt,
        "draft_adapter_plan",
        json!({"source_ref": "test", "secret": "RawSecretExample1234567890abcdefABCDEF123456"}),
    )
    .await?;
    anyhow::ensure!(ad.output["kind"] == json!("project_intake_rejected"));

    Ok(())
}
