//! Conformance tests for `official/agentic-forge-lab` (Agentic Forge Beta Phase A).
//!
//! Proves: describe_contract, start_run plan graph/working state,
//! inspect/cancel/summarize, raw-secret blocking, no kernel agent namespace.

use std::path::PathBuf;

use serde_json::json;
use ygg_runtime::CapabilityInvocationRequest;

use super::fixtures::*;
use crate::commands::manifest;

const PACKAGE_ID: &str = "official/agentic-forge-lab";

async fn load_forge_lab() -> anyhow::Result<ygg_runtime::Runtime<ygg_runtime::InMemoryEventStore>> {
    let (_store, runtime) = runtime();
    runtime
        .load_package(manifest::read_manifest(PathBuf::from(
            "packages/official/agentic-forge-lab/manifest.yaml",
        ))
        .await?)
        .await?;
    Ok(runtime)
}

/// Phase A case 1: describe_contract returns all capabilities, lifecycle states,
/// plan graph fields, working state fields. No kernel agent namespace.
pub(crate) async fn agentic_forge_describe_contract() -> anyhow::Result<()> {
    let runtime = load_forge_lab().await?;

    let contract = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: format!("{PACKAGE_ID}/describe_contract"),
            caller_package_id: None,
            provider_package_id: Some(PACKAGE_ID.to_string()),
            version: None,
            input: json!({}),
        })
        .await?;

    anyhow::ensure!(
        contract.output["kind"] == json!("agentic_forge_contract"),
        "describe_contract must return agentic_forge_contract kind"
    );
    anyhow::ensure!(
        contract.output["lifecycle_states"].as_array().map(|a| a.len()).unwrap_or(0) == 9,
        "describe_contract must list 9 lifecycle states"
    );
    anyhow::ensure!(
        contract.output["capabilities"].as_array().map(|a| a.len()).unwrap_or(0) == 6,
        "describe_contract must list 6 capabilities"
    );
    anyhow::ensure!(
        contract.output["plan_graph_fields"].is_array(),
        "describe_contract must have plan_graph_fields"
    );
    anyhow::ensure!(
        contract.output["working_state_fields"].is_array(),
        "describe_contract must have working_state_fields"
    );
    anyhow::ensure!(
        contract.output["inference_performed"] == json!(false),
        "describe_contract must record inference_performed=false"
    );
    anyhow::ensure!(
        contract.output["network_performed"] == json!(false),
        "describe_contract must record network_performed=false"
    );

    // No kernel agent namespace
    let output_str = serde_json::to_string(&contract.output).unwrap();
    anyhow::ensure!(!output_str.contains("kernel.agent"), "describe_contract must not contain kernel.agent");
    anyhow::ensure!(!output_str.contains("kernel.model"), "describe_contract must not contain kernel.model");
    anyhow::ensure!(!output_str.contains("kernel.prompt"), "describe_contract must not contain kernel.prompt");

    Ok(())
}

/// Phase A case 2: start_run produces plan graph and working state with all
/// required fields. Plan graph has nodes/edges/status/revision/deterministic_mode.
/// Working state has run_id/owner_package/target_branch_ref/scratch_branch_ref/etc.
pub(crate) async fn agentic_forge_start_run() -> anyhow::Result<()> {
    let runtime = load_forge_lab().await?;

    let started = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: format!("{PACKAGE_ID}/start_run"),
            caller_package_id: None,
            provider_package_id: Some(PACKAGE_ID.to_string()),
            version: None,
            input: json!({
                "objective": "conformance test run",
                "target_branch_ref": "branch:target:main",
                "scratch_branch_ref": "branch:scratch:s1",
            }),
        })
        .await?;

    anyhow::ensure!(
        started.output["kind"] == json!("agentic_forge_run_started"),
        "start_run must return agentic_forge_run_started kind"
    );
    anyhow::ensure!(
        started.output["lifecycle_state"] == json!("prepared"),
        "start_run must return prepared lifecycle state"
    );
    anyhow::ensure!(
        started.output["run_id"].is_string(),
        "start_run must return a run_id"
    );

    // Plan graph checks
    let pg = &started.output["plan_graph"];
    anyhow::ensure!(pg["nodes"].is_array(), "plan_graph must have nodes");
    anyhow::ensure!(pg["edges"].is_array(), "plan_graph must have edges");
    anyhow::ensure!(pg["status"] == json!("prepared"), "plan_graph status must be prepared");
    anyhow::ensure!(pg["revision"] == json!(1), "plan_graph revision must be 1");
    anyhow::ensure!(pg["deterministic_mode"] == json!(true), "plan_graph deterministic_mode must be true");
    anyhow::ensure!(pg["approval_policy"].is_string(), "plan_graph must have approval_policy");
    anyhow::ensure!(pg["retry_policy"].is_object(), "plan_graph must have retry_policy");
    anyhow::ensure!(pg["input_refs"].is_array(), "plan_graph must have input_refs");
    anyhow::ensure!(pg["output_refs"].is_array(), "plan_graph must have output_refs");

    // Working state checks
    let ws = &started.output["working_state"];
    anyhow::ensure!(ws["run_id"].is_string(), "working_state must have run_id");
    anyhow::ensure!(ws["owner_package"] == json!(PACKAGE_ID), "working_state owner_package must match");
    anyhow::ensure!(ws["target_branch_ref"] == json!("branch:target:main"), "working_state target_branch_ref");
    anyhow::ensure!(ws["scratch_branch_ref"] == json!("branch:scratch:s1"), "working_state scratch_branch_ref");
    anyhow::ensure!(ws["current_objective"].is_string(), "working_state must have current_objective");
    anyhow::ensure!(ws["plan_graph_ref"].is_string(), "working_state must have plan_graph_ref");
    anyhow::ensure!(ws["candidate_refs"].is_array(), "working_state must have candidate_refs");
    anyhow::ensure!(ws["tool_observation_refs"].is_array(), "working_state must have tool_observation_refs");
    anyhow::ensure!(ws["inference_trace_refs"].is_array(), "working_state must have inference_trace_refs");
    anyhow::ensure!(ws["policy_state"].is_object(), "working_state must have policy_state");
    anyhow::ensure!(ws["policy_state"]["deterministic_mode"] == json!(true), "policy_state deterministic_mode");

    // Trace events
    anyhow::ensure!(
        started.output["trace_events"].as_array().map(|a| a.len()).unwrap_or(0) >= 2,
        "start_run must produce at least 2 trace events"
    );

    Ok(())
}

/// Phase A case 3: inspect_run, cancel_run, summarize_run produce correct
/// lifecycle transitions and observability summaries.
pub(crate) async fn agentic_forge_inspect_cancel_summarize() -> anyhow::Result<()> {
    let runtime = load_forge_lab().await?;

    // Inspect
    let inspection = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: format!("{PACKAGE_ID}/inspect_run"),
            caller_package_id: None,
            provider_package_id: Some(PACKAGE_ID.to_string()),
            version: None,
            input: json!({
                "run_id": "run_conformance",
                "lifecycle_state": "running",
            }),
        })
        .await?;

    anyhow::ensure!(
        inspection.output["kind"] == json!("agentic_forge_run_inspection"),
        "inspect_run must return agentic_forge_run_inspection kind"
    );
    anyhow::ensure!(
        inspection.output["run_id"] == json!("run_conformance"),
        "inspect_run must return correct run_id"
    );
    anyhow::ensure!(
        inspection.output["lifecycle_state"] == json!("running"),
        "inspect_run must return running state"
    );
    anyhow::ensure!(
        inspection.output["working_state"].is_object(),
        "inspect_run must have working_state"
    );
    anyhow::ensure!(
        inspection.output["plan_graph_ref"].is_string(),
        "inspect_run must have plan_graph_ref"
    );

    // Cancel
    let cancelled = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: format!("{PACKAGE_ID}/cancel_run"),
            caller_package_id: None,
            provider_package_id: Some(PACKAGE_ID.to_string()),
            version: None,
            input: json!({
                "run_id": "run_conformance",
                "lifecycle_state": "running",
            }),
        })
        .await?;

    anyhow::ensure!(
        cancelled.output["kind"] == json!("agentic_forge_run_cancelled"),
        "cancel_run must return agentic_forge_run_cancelled kind"
    );
    anyhow::ensure!(
        cancelled.output["lifecycle_state"] == json!("cancelled"),
        "cancel_run must set lifecycle_state to cancelled"
    );
    anyhow::ensure!(
        cancelled.output["previous_state"] == json!("running"),
        "cancel_run must record previous_state"
    );

    // Summarize
    let summary = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: format!("{PACKAGE_ID}/summarize_run"),
            caller_package_id: None,
            provider_package_id: Some(PACKAGE_ID.to_string()),
            version: None,
            input: json!({
                "run_id": "run_conformance",
                "lifecycle_state": "cancelled",
                "trace_events": [{"a": 1}, {"b": 2}, {"c": 3}],
            }),
        })
        .await?;

    anyhow::ensure!(
        summary.output["kind"] == json!("agentic_forge_run_summary"),
        "summarize_run must return agentic_forge_run_summary kind"
    );
    anyhow::ensure!(
        summary.output["trace_event_count"] == json!(3),
        "summarize_run must count trace events"
    );
    anyhow::ensure!(
        summary.output["inference_performed"] == json!(false),
        "summarize_run must record inference_performed=false"
    );
    anyhow::ensure!(
        summary.output["network_performed"] == json!(false),
        "summarize_run must record network_performed=false"
    );

    Ok(())
}

/// Phase A case 4: raw-secret-like input is blocked with
/// redaction_state=unsafe_blocked and no raw secret echo.
pub(crate) async fn agentic_forge_raw_secret_blocked() -> anyhow::Result<()> {
    let runtime = load_forge_lab().await?;

    // Try start_run with a raw credential-looking value
    let blocked = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: format!("{PACKAGE_ID}/start_run"),
            caller_package_id: None,
            provider_package_id: Some(PACKAGE_ID.to_string()),
            version: None,
            input: json!({
                "objective": "secret exfil test",
                "api_key": "RawSecretExample1234567890abcdefABCDEF123456",
            }),
        })
        .await?;

    anyhow::ensure!(
        blocked.output["kind"] == json!("agentic_forge_run_rejected"),
        "start_run with raw secret must be rejected"
    );
    anyhow::ensure!(
        blocked.output["redaction_state"] == json!("unsafe_blocked"),
        "redaction_state must be unsafe_blocked"
    );

    // Ensure no raw secret echo in output
    let output_str = serde_json::to_string(&blocked.output).unwrap();
    anyhow::ensure!(
        !output_str.contains("RawSecretExample1234567890abcdefABCDEF123456"),
        "rejected output must not echo raw secret"
    );

    // Also test Bearer token
    let bearer_blocked = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: format!("{PACKAGE_ID}/start_run"),
            caller_package_id: None,
            provider_package_id: Some(PACKAGE_ID.to_string()),
            version: None,
            input: json!({
                "objective": "bearer exfil test",
                "token": "rawBearerPlaceholder1234567890ABCDEF",
            }),
        })
        .await?;

    anyhow::ensure!(
        bearer_blocked.output["redaction_state"] == json!("unsafe_blocked"),
        "Bearer token must be blocked"
    );

    // secret_ref should be accepted
    let allowed = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: format!("{PACKAGE_ID}/start_run"),
            caller_package_id: None,
            provider_package_id: Some(PACKAGE_ID.to_string()),
            version: None,
            input: json!({
                "objective": "safe run",
                "api_key": "secret_ref:env:MY_KEY",
            }),
        })
        .await?;

    anyhow::ensure!(
        allowed.output["kind"] == json!("agentic_forge_run_started"),
        "start_run with secret_ref should succeed"
    );

    let allowed_host_ref = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: format!("{PACKAGE_ID}/start_run"),
            caller_package_id: None,
            provider_package_id: Some(PACKAGE_ID.to_string()),
            version: None,
            input: json!({
                "objective": "safe host ref run",
                "api_key": "host:env:MY_KEY",
            }),
        })
        .await?;

    anyhow::ensure!(
        allowed_host_ref.output["kind"] == json!("agentic_forge_run_started"),
        "start_run with host:env secret ref should succeed"
    );

    Ok(())
}

/// Phase A case 5: outputs contain no kernel.agent/model/prompt/memory/turn
/// namespace references.
pub(crate) async fn agentic_forge_no_kernel_agent_namespace() -> anyhow::Result<()> {
    let runtime = load_forge_lab().await?;

    // Collect outputs from all capabilities
    let contract = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: format!("{PACKAGE_ID}/describe_contract"),
            caller_package_id: None,
            provider_package_id: Some(PACKAGE_ID.to_string()),
            version: None,
            input: json!({}),
        })
        .await?;

    let started = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: format!("{PACKAGE_ID}/start_run"),
            caller_package_id: None,
            provider_package_id: Some(PACKAGE_ID.to_string()),
            version: None,
            input: json!({"objective": "namespace test"}),
        })
        .await?;

    let exported = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: format!("{PACKAGE_ID}/export_plan_graph"),
            caller_package_id: None,
            provider_package_id: Some(PACKAGE_ID.to_string()),
            version: None,
            input: json!({"run_id": "run_ns_test"}),
        })
        .await?;

    // Check all outputs for forbidden kernel namespaces
    for (label, output) in [
        ("describe_contract", &contract.output),
        ("start_run", &started.output),
        ("export_plan_graph", &exported.output),
    ] {
        let output_str = serde_json::to_string(output).unwrap();
        anyhow::ensure!(
            !output_str.contains("kernel.agent"),
            "{label} must not contain kernel.agent"
        );
        anyhow::ensure!(
            !output_str.contains("kernel.model"),
            "{label} must not contain kernel.model"
        );
        anyhow::ensure!(
            !output_str.contains("kernel.prompt"),
            "{label} must not contain kernel.prompt"
        );
        anyhow::ensure!(
            !output_str.contains("kernel.memory"),
            "{label} must not contain kernel.memory"
        );
        anyhow::ensure!(
            !output_str.contains("kernel.turn"),
            "{label} must not contain kernel.turn"
        );
    }

    Ok(())
}
