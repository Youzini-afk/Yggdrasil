//! Conformance tests for `official/workspace-lab` (External Project Operating Plane Alpha Phase E2).
//!
//! Covers:
//! 1. Contract shape (5 capabilities, 3 surfaces, ordinary package, deny-by-default, 10 action taxonomy)
//! 2. Action taxonomy deny-by-default (all dangerous actions denied, no execution)
//! 3. Policy mismatch fail-closed (policy "allow" for approval-required action → denied)
//! 4. Raw secret blocked (raw API key / Bearer token in input → rejected)
//! 5. Audit redacted (no raw env/logs/commands/secrets in audit summary)
//! 6. No forbidden namespace (no kernel.project/workspace/git/npm/deploy/ide in any output)
//! 7. No execution performed (executor_invoked=false, execution_performed=false always)

use std::path::PathBuf;

use serde_json::json;
use ygg_runtime::CapabilityInvocationRequest;

use super::fixtures::*;
use crate::commands::manifest;

const PACKAGE_ID: &str = "official/workspace-lab";

async fn load_workspace_lab(
) -> anyhow::Result<ygg_runtime::Runtime<ygg_runtime::InMemoryEventStore>> {
    let (_store, runtime) = runtime();
    runtime
        .load_package(
            manifest::read_manifest(PathBuf::from(
                "packages/official/workspace-lab/manifest.yaml",
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

/// Case 1: Contract shape — 5 capabilities, 3 surfaces, ordinary package,
/// deny-by-default, 10 action taxonomy, no forbidden namespace, no execution.
pub(crate) async fn workspace_lab_contract() -> anyhow::Result<()> {
    let rt = load_workspace_lab().await?;

    let contract = invoke(&rt, "describe_workspace_contract", json!({})).await?;

    anyhow::ensure!(
        contract.output["kind"] == json!("workspace_lab_contract"),
        "describe_workspace_contract must return workspace_lab_contract kind"
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
    anyhow::ensure!(
        surfaces.contains_key("home_card"),
        "must have home_card"
    );

    // 5 capabilities
    anyhow::ensure!(
        contract.output["capabilities"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0)
            == 5,
        "describe_workspace_contract must list 5 capabilities"
    );

    // 10 action taxonomy entries
    anyhow::ensure!(
        contract.output["action_taxonomy"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0)
            == 10,
        "describe_workspace_contract must list 10 action taxonomy entries"
    );

    // Deny-by-default
    anyhow::ensure!(
        contract.output["policy_defaults"]["default_decision"]
            == json!("denied_by_default")
    );
    anyhow::ensure!(
        contract.output["policy_defaults"]["executor_invoked"] == json!(false)
    );
    anyhow::ensure!(
        contract.output["policy_defaults"]["execution_performed"] == json!(false)
    );
    anyhow::ensure!(
        contract.output["policy_defaults"]["proposal_required"] == json!(true)
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
            "describe_workspace_contract must not contain {}",
            token
        );
    }

    Ok(())
}

/// Case 2: Action taxonomy deny-by-default — all dangerous actions denied,
/// no execution, proposal_required=true, approval_token not honored.
pub(crate) async fn workspace_lab_action_deny_default() -> anyhow::Result<()> {
    let rt = load_workspace_lab().await?;

    // Test dangerous actions: clone_project, install_dependencies, run_command, deploy_plan
    for action in &[
        "clone_project",
        "install_dependencies",
        "run_command",
        "deploy_plan",
    ] {
        let result = invoke(
            &rt,
            "request_workspace_action",
            json!({"action": action, "workspace_ref": "ws-test"}),
        )
        .await?;
        anyhow::ensure!(
            result.output["policy_decision"] == json!("denied_by_default"),
            "action {} must be denied_by_default",
            action
        );
        anyhow::ensure!(
            result.output["executor_invoked"] == json!(false),
            "action {} must have executor_invoked=false",
            action
        );
        anyhow::ensure!(
            result.output["execution_performed"] == json!(false),
            "action {} must have execution_performed=false",
            action
        );
        anyhow::ensure!(
            result.output["proposal_required"] == json!(true),
            "action {} must have proposal_required=true",
            action
        );
    }

    // Approval token not honored
    let with_token = invoke(
        &rt,
        "request_workspace_action",
        json!({"action": "clone_project", "workspace_ref": "ws-test", "approval_token": "pretend-token"}),
    )
    .await?;
    anyhow::ensure!(
        with_token.output["policy_decision"] == json!("denied_by_default"),
        "approval_token must not bypass deny-by-default"
    );
    anyhow::ensure!(
        with_token.output["approval_token_rejected"] == json!(true),
        "approval_token must be rejected"
    );
    anyhow::ensure!(
        with_token.output["approval_token_honored"] == json!(false),
        "approval_token must not be honored"
    );

    Ok(())
}

/// Case 3: Policy/action mismatch fail-closed — policy "allow" for
/// approval-required actions → denied; unknown policy → denied.
pub(crate) async fn workspace_lab_policy_mismatch() -> anyhow::Result<()> {
    let rt = load_workspace_lab().await?;

    // policy "allow" with approval-required action → fail closed
    let mismatch = invoke(
        &rt,
        "request_workspace_action",
        json!({"action": "install_dependencies", "workspace_ref": "ws-test", "policy": "allow"}),
    )
    .await?;
    anyhow::ensure!(
        mismatch.output["kind"] == json!("workspace_action_rejected"),
        "policy/action mismatch must produce workspace_action_rejected"
    );
    anyhow::ensure!(
        mismatch.output["policy_decision"] == json!("denied"),
        "policy/action mismatch must produce policy_decision=denied"
    );
    anyhow::ensure!(
        mismatch.output["executor_invoked"] == json!(false),
        "policy mismatch must not invoke executor"
    );

    // unknown policy → fail closed
    let unknown = invoke(
        &rt,
        "request_workspace_action",
        json!({"action": "run_command", "workspace_ref": "ws-test", "policy": "auto_approve"}),
    )
    .await?;
    anyhow::ensure!(
        unknown.output["policy_decision"] == json!("denied"),
        "unknown policy must be denied"
    );

    // unknown action → fail closed
    let bad_action = invoke(
        &rt,
        "request_workspace_action",
        json!({"action": "rm_rf", "workspace_ref": "ws-test"}),
    )
    .await?;
    anyhow::ensure!(
        bad_action.output["kind"] == json!("workspace_action_rejected"),
        "unknown action must produce workspace_action_rejected"
    );
    anyhow::ensure!(
        bad_action.output["policy_decision"] == json!("denied"),
        "unknown action must produce policy_decision=denied"
    );

    Ok(())
}

/// Case 4: Raw secret blocked — raw API key / Bearer token in input → rejected.
pub(crate) async fn workspace_lab_raw_secret_blocked() -> anyhow::Result<()> {
    let rt = load_workspace_lab().await?;

    // request_workspace_action blocks raw secret
    let with_secret = invoke(
        &rt,
        "request_workspace_action",
        json!({"action": "clone_project", "workspace_ref": "ws-test", "api_key": "RawSecretExample1234567890abcdefABCDEF123456"}),
    )
    .await?;
    anyhow::ensure!(
        with_secret.output["kind"] == json!("workspace_lab_rejected"),
        "raw secret must produce workspace_lab_rejected"
    );
    anyhow::ensure!(
        with_secret.output["redaction_state"] == json!("unsafe_blocked"),
        "raw secret must produce redaction_state=unsafe_blocked"
    );

    // draft_workspace_creation blocks raw secret
    let draft_secret = invoke(
        &rt,
        "draft_workspace_creation",
        json!({"workspace_ref": "ws-test", "token": "Bearer abc123"}),
    )
    .await?;
    anyhow::ensure!(
        draft_secret.output["kind"] == json!("workspace_lab_rejected"),
        "raw secret in draft must produce workspace_lab_rejected"
    );

    // summarize_workspace_audit blocks raw secret in top-level input
    let audit_secret = invoke(
        &rt,
        "summarize_workspace_audit",
        json!({"workspace_ref": "ws-test", "secret": "RawSecretExample1234567890abcdefABCDEF123456"}),
    )
    .await?;
    anyhow::ensure!(
        audit_secret.output["kind"] == json!("workspace_lab_rejected"),
        "raw secret in audit must produce workspace_lab_rejected"
    );

    Ok(())
}

/// Case 5: Audit redacted — no raw env/logs/commands/secrets in audit summary.
pub(crate) async fn workspace_lab_audit_redacted() -> anyhow::Result<()> {
    let rt = load_workspace_lab().await?;

    let audit = invoke(
        &rt,
        "summarize_workspace_audit",
        json!({
            "workspace_ref": "ws-audit",
            "action_history": [
                {"action": "clone_project", "policy_decision": "denied_by_default", "executor_invoked": false, "execution_performed": false},
                {"action": "read_metadata", "policy_decision": "approved", "executor_invoked": false, "execution_performed": false},
                {"action": "run_command", "policy_decision": "pending", "executor_invoked": false, "execution_performed": false, "raw_command": "rm -rf /", "raw_env": "SECRET=abc", "raw_log": "password=xyz"},
            ]
        }),
    )
    .await?;

    anyhow::ensure!(
        audit.output["kind"] == json!("workspace_audit_summary"),
        "must produce workspace_audit_summary"
    );
    anyhow::ensure!(
        audit.output["redaction_applied"] == json!(true),
        "must have redaction_applied=true"
    );

    // Verify no raw content leaked
    let output_str = serde_json::to_string(&audit.output).unwrap();
    anyhow::ensure!(
        !output_str.contains("rm -rf"),
        "audit must not contain raw commands"
    );
    anyhow::ensure!(
        !output_str.contains("raw_command"),
        "audit must not contain raw_command field"
    );
    anyhow::ensure!(
        !output_str.contains("raw_env"),
        "audit must not contain raw_env field"
    );
    anyhow::ensure!(
        !output_str.contains("raw_log"),
        "audit must not contain raw_log field"
    );
    anyhow::ensure!(
        !output_str.contains("SECRET=abc"),
        "audit must not contain raw env values"
    );
    anyhow::ensure!(
        !output_str.contains("password=xyz"),
        "audit must not contain raw log values"
    );

    // Verify counts
    anyhow::ensure!(
        audit.output["total_actions"] == json!(3),
        "must count 3 total actions"
    );

    Ok(())
}

/// Case 6: No forbidden namespace in any capability output.
pub(crate) async fn workspace_lab_no_forbidden_namespace() -> anyhow::Result<()> {
    let rt = load_workspace_lab().await?;

    let caps = [
        "describe_workspace_contract",
        "draft_workspace_creation",
        "explain_required_permissions",
        "request_workspace_action",
        "summarize_workspace_audit",
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
        let result = invoke(
            &rt,
            cap,
            json!({"workspace_ref": "ws-test", "action": "clone_project"}),
        )
        .await?;
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

/// Case 7: No execution performed across all capabilities.
pub(crate) async fn workspace_lab_no_execution() -> anyhow::Result<()> {
    let rt = load_workspace_lab().await?;

    let caps = [
        "describe_workspace_contract",
        "draft_workspace_creation",
        "explain_required_permissions",
        "request_workspace_action",
        "summarize_workspace_audit",
    ];

    for cap in &caps {
        let result = invoke(
            &rt,
            cap,
            json!({"workspace_ref": "ws-test", "action": "run_command"}),
        )
        .await?;
        anyhow::ensure!(
            result.output["execution_performed"] == json!(false),
            "{cap} must have execution_performed=false"
        );
        anyhow::ensure!(
            result.output["network_performed"] == json!(false),
            "{cap} must have network_performed=false"
        );
        anyhow::ensure!(
            result.output["inference_performed"] == json!(false),
            "{cap} must have inference_performed=false"
        );
        anyhow::ensure!(
            result.output["filesystem_performed"] == json!(false),
            "{cap} must have filesystem_performed=false"
        );
    }

    Ok(())
}
