//! Handler for `official/agentic-forge-lab` capabilities.
//!
//! Phase A of Agentic Forge Beta: package-owned agent run lifecycle,
//! working state, plan graph contract. Deterministic, no-network,
//! no real model inference.

use serde_json::Value;

use super::InprocInvocation;

const PACKAGE_ID: &str = "official/agentic-forge-lab";

// ---------------------------------------------------------------------------
// Lifecycle states
// ---------------------------------------------------------------------------

const LIFECYCLE_STATES: &[&str] = &[
    "created",
    "prepared",
    "running",
    "paused",
    "waiting_for_approval",
    "completed",
    "failed",
    "cancelled",
    "archived",
];

fn is_valid_lifecycle_state(s: &str) -> bool {
    LIFECYCLE_STATES.contains(&s)
}

// ---------------------------------------------------------------------------
// Raw-secret detection (conservative, shared with kernel scanning)
// ---------------------------------------------------------------------------

const SECRET_FIELD_NAMES: &[&str] = &[
    "api_key",
    "secret",
    "token",
    "password",
    "private_key",
    "access_token",
    "refresh_token",
    "auth_token",
];

const SECRET_VALUE_PREFIXES: &[&str] = &["sk-", "Bearer ", "bearer "];

fn is_secret_ref_value(val: &str) -> bool {
    val.starts_with("secret_ref:")
        || val.starts_with("secretRef:")
        || val.starts_with("secret-ref:")
        || val.starts_with("host:")
}

fn looks_like_raw_secret_value(val: &str) -> bool {
    for prefix in SECRET_VALUE_PREFIXES {
        if val.starts_with(prefix) {
            return true;
        }
    }
    // High-entropy heuristic: long strings with mixed case/digits
    if val.len() >= 32 {
        let has_upper = val.chars().any(|c| c.is_ascii_uppercase());
        let has_lower = val.chars().any(|c| c.is_ascii_lowercase());
        let has_digit = val.chars().any(|c| c.is_ascii_digit());
        if has_upper && has_lower && has_digit && val.len() >= 40 {
            return true;
        }
    }
    false
}

/// Recursively scan a value for raw-secret-like content.
/// Returns true if any suspicious field name or value pattern is found.
fn contains_raw_secret(value: &Value) -> bool {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                let key_lower = key.to_lowercase();
                for secret_name in SECRET_FIELD_NAMES {
                    if key_lower == *secret_name || key_lower.contains(secret_name) {
                        if let Some(s) = val.as_str() {
                            if !is_secret_ref_value(s) {
                                return true;
                            }
                        } else if !val.is_null() {
                            return true;
                        }
                    }
                }
                if let Some(s) = val.as_str() {
                    if looks_like_raw_secret_value(s) {
                        return true;
                    }
                }
                if contains_raw_secret(val) {
                    return true;
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                if contains_raw_secret(item) {
                    return true;
                }
            }
        }
        _ => {}
    }
    false
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

pub fn try_handle(request: &InprocInvocation) -> Option<anyhow::Result<Value>> {
    if request.provider_package_id != PACKAGE_ID {
        return None;
    }
    let id = request.capability_id.as_str();
    if id.ends_with("/describe_contract") {
        Some(describe_contract(request))
    } else if id.ends_with("/start_run") {
        Some(start_run(request))
    } else if id.ends_with("/inspect_run") {
        Some(inspect_run(request))
    } else if id.ends_with("/cancel_run") {
        Some(cancel_run(request))
    } else if id.ends_with("/summarize_run") {
        Some(summarize_run(request))
    } else if id.ends_with("/export_plan_graph") {
        Some(export_plan_graph(request))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Capability implementations
// ---------------------------------------------------------------------------

fn describe_contract(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "agentic_forge_contract",
        "package_id": request.provider_package_id,
        "capabilities": [
            {"id": "official/agentic-forge-lab/describe_contract", "purpose": "describe the agentic forge package contract"},
            {"id": "official/agentic-forge-lab/start_run", "purpose": "start a deterministic agent run with plan graph and working state"},
            {"id": "official/agentic-forge-lab/inspect_run", "purpose": "inspect an existing run's working state and lifecycle"},
            {"id": "official/agentic-forge-lab/cancel_run", "purpose": "cancel a running or paused run"},
            {"id": "official/agentic-forge-lab/summarize_run", "purpose": "produce an observability summary of a run"},
            {"id": "official/agentic-forge-lab/export_plan_graph", "purpose": "export the plan graph artifact of a run"},
        ],
        "lifecycle_states": LIFECYCLE_STATES,
        "plan_graph_fields": [
            "nodes", "edges", "status", "revision", "input_refs",
            "output_refs", "approval_policy", "retry_policy", "deterministic_mode"
        ],
        "working_state_fields": [
            "run_id", "owner_package", "target_branch_ref", "scratch_branch_ref",
            "current_objective", "local_context_refs", "plan_graph_ref",
            "candidate_refs", "tool_observation_refs", "inference_trace_refs",
            "policy_state"
        ],
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn start_run(request: &InprocInvocation) -> anyhow::Result<Value> {
    // Check for raw-secret-like content in input
    if contains_raw_secret(&request.input) {
        return Ok(serde_json::json!({
            "kind": "agentic_forge_run_rejected",
            "redaction_state": "unsafe_blocked",
            "reason": "input contains raw-secret-like content; use secret_ref references instead",
            "inference_performed": false,
            "network_performed": false,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    let run_id = format!("run_{}", deterministic_id(&request.input));
    let objective = request.input
        .get("objective")
        .and_then(Value::as_str)
        .unwrap_or("deterministic agentic forge run");

    let plan_graph = build_plan_graph(&run_id, objective);
    let working_state = build_working_state(
        &run_id,
        &request.provider_package_id,
        &request.input,
    );

    Ok(serde_json::json!({
        "kind": "agentic_forge_run_started",
        "run_id": run_id,
        "lifecycle_state": "prepared",
        "plan_graph": plan_graph,
        "working_state": working_state,
        "trace_events": [
            {
                "event_type": "run_created",
                "run_id": run_id,
                "timestamp": 0,
                "payload": {"step": "plan_only", "status": "deterministic"}
            },
            {
                "event_type": "run_prepared",
                "run_id": run_id,
                "timestamp": 1,
                "payload": {"objective": objective, "plan_revision": 1}
            }
        ],
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn inspect_run(request: &InprocInvocation) -> anyhow::Result<Value> {
    let run_id = request.input
        .get("run_id")
        .and_then(Value::as_str)
        .unwrap_or("run_unknown");

    let state = request.input
        .get("lifecycle_state")
        .and_then(Value::as_str)
        .filter(|s| is_valid_lifecycle_state(s))
        .unwrap_or("running");

    Ok(serde_json::json!({
        "kind": "agentic_forge_run_inspection",
        "run_id": run_id,
        "lifecycle_state": state,
        "working_state": build_working_state(&run_id, &request.provider_package_id, &request.input),
        "plan_graph_ref": format!("plan_graph:{}", run_id),
        "candidate_refs": request.input.get("candidate_refs").cloned().unwrap_or(serde_json::json!([])),
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn cancel_run(request: &InprocInvocation) -> anyhow::Result<Value> {
    let run_id = request.input
        .get("run_id")
        .and_then(Value::as_str)
        .unwrap_or("run_unknown");

    Ok(serde_json::json!({
        "kind": "agentic_forge_run_cancelled",
        "run_id": run_id,
        "previous_state": request.input.get("lifecycle_state").and_then(Value::as_str).unwrap_or("running"),
        "lifecycle_state": "cancelled",
        "trace_events": [
            {
                "event_type": "run_cancelled",
                "run_id": run_id,
                "timestamp": 0,
                "payload": {"reason": "user_cancelled", "status": "deterministic"}
            }
        ],
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn summarize_run(request: &InprocInvocation) -> anyhow::Result<Value> {
    let run_id = request.input
        .get("run_id")
        .and_then(Value::as_str)
        .unwrap_or("run_unknown");
    let event_count = request.input
        .get("trace_events")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);
    let node_count = request.input
        .get("plan_graph")
        .and_then(|pg| pg.get("nodes"))
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(3);
    let candidate_count = request.input
        .get("candidate_refs")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(serde_json::json!({
        "kind": "agentic_forge_run_summary",
        "run_id": run_id,
        "lifecycle_state": request.input.get("lifecycle_state").and_then(Value::as_str).unwrap_or("completed"),
        "trace_event_count": event_count,
        "plan_node_count": node_count,
        "candidate_count": candidate_count,
        "tool_observation_count": 0,
        "inference_trace_count": 0,
        "inference_performed": false,
        "network_performed": false,
        "summary": format!("Run {run_id}: {event_count} trace events, {node_count} plan nodes, {candidate_count} candidates, no inference, no network"),
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn export_plan_graph(request: &InprocInvocation) -> anyhow::Result<Value> {
    let run_id = request.input
        .get("run_id")
        .and_then(Value::as_str)
        .unwrap_or("run_unknown");
    let objective = request.input
        .get("objective")
        .and_then(Value::as_str)
        .unwrap_or("deterministic agentic forge run");

    let plan_graph = build_plan_graph(run_id, objective);

    Ok(serde_json::json!({
        "kind": "agentic_forge_plan_graph",
        "run_id": run_id,
        "plan_graph": plan_graph,
        "inference_performed": false,
        "network_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

// ---------------------------------------------------------------------------
// Shape builders
// ---------------------------------------------------------------------------

fn deterministic_id(input: &Value) -> String {
    // Simple deterministic ID from input content, no random
    let objective = input.get("objective").and_then(Value::as_str).unwrap_or("default");
    let len = objective.len();
    format!("{:04x}", len.wrapping_mul(31).wrapping_add(0xaf))
}

fn build_plan_graph(run_id: &str, objective: &str) -> Value {
    serde_json::json!({
        "nodes": [
            {
                "node_id": format!("{run_id}_node_observe"),
                "kind": "observe",
                "label": "Observe context",
                "status": "pending"
            },
            {
                "node_id": format!("{run_id}_node_plan"),
                "kind": "tool_call",
                "label": objective,
                "status": "pending"
            },
            {
                "node_id": format!("{run_id}_node_propose"),
                "kind": "propose",
                "label": "Produce candidate",
                "status": "pending"
            }
        ],
        "edges": [
            {
                "from_node_id": format!("{run_id}_node_observe"),
                "to_node_id": format!("{run_id}_node_plan"),
                "kind": "sequential"
            },
            {
                "from_node_id": format!("{run_id}_node_plan"),
                "to_node_id": format!("{run_id}_node_propose"),
                "kind": "sequential"
            }
        ],
        "status": "prepared",
        "revision": 1,
        "input_refs": [],
        "output_refs": [],
        "approval_policy": "fork_then_approve",
        "retry_policy": {"max_retries": 0, "backoff": "none"},
        "deterministic_mode": true
    })
}

fn build_working_state(run_id: &str, owner_package: &str, input: &Value) -> Value {
    serde_json::json!({
        "run_id": run_id,
        "owner_package": owner_package,
        "target_branch_ref": input.get("target_branch_ref").and_then(Value::as_str).unwrap_or("branch:target:default"),
        "scratch_branch_ref": input.get("scratch_branch_ref").and_then(Value::as_str).unwrap_or("branch:scratch:default"),
        "current_objective": input.get("objective").and_then(Value::as_str).unwrap_or("deterministic agentic forge run"),
        "local_context_refs": input.get("local_context_refs").cloned().unwrap_or(serde_json::json!([])),
        "plan_graph_ref": format!("plan_graph:{}", run_id),
        "candidate_refs": input.get("candidate_refs").cloned().unwrap_or(serde_json::json!([])),
        "tool_observation_refs": input.get("tool_observation_refs").cloned().unwrap_or(serde_json::json!([])),
        "inference_trace_refs": input.get("inference_trace_refs").cloned().unwrap_or(serde_json::json!([])),
        "policy_state": {
            "approval_policy": "fork_then_approve",
            "retry_budget_remaining": 0,
            "deterministic_mode": true
        }
    })
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
        let req = make_request("official/agentic-forge-lab/describe_contract", json!({}));
        assert!(try_handle(&req).is_some());
    }

    #[test]
    fn try_handle_rejects_wrong_package() {
        let req = InprocInvocation {
            capability_id: "official/agentic-forge-lab/describe_contract".to_string(),
            provider_package_id: "official/other".to_string(),
            input: json!({}),
        };
        assert!(try_handle(&req).is_none());
    }

    #[test]
    fn describe_contract_returns_lifecycle_states() {
        let req = make_request("official/agentic-forge-lab/describe_contract", json!({}));
        let result = try_handle(&req).unwrap().unwrap();
        let states = result["lifecycle_states"].as_array().unwrap();
        assert_eq!(states.len(), 9);
        assert_eq!(states[0], json!("created"));
        assert_eq!(states[8], json!("archived"));
    }

    #[test]
    fn start_run_returns_plan_graph_and_working_state() {
        let req = make_request("official/agentic-forge-lab/start_run", json!({"objective": "test run"}));
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("agentic_forge_run_started"));
        assert_eq!(result["lifecycle_state"], json!("prepared"));
        assert!(result["plan_graph"]["nodes"].is_array());
        assert!(result["working_state"]["run_id"].is_string());
        assert_eq!(result["plan_graph"]["deterministic_mode"], json!(true));
    }

    #[test]
    fn start_run_blocks_raw_secret() {
        let req = make_request("official/agentic-forge-lab/start_run", json!({"objective": "test", "api_key": "RawSecretExample1234567890abcdefABCDEF123456"}));
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("agentic_forge_run_rejected"));
        assert_eq!(result["redaction_state"], json!("unsafe_blocked"));
    }

    #[test]
    fn start_run_blocks_bearer_secret() {
        let req = make_request("official/agentic-forge-lab/start_run", json!({"objective": "test", "token": "Bearer abc123"}));
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["redaction_state"], json!("unsafe_blocked"));
    }

    #[test]
    fn start_run_accepts_secret_ref() {
        let req = make_request("official/agentic-forge-lab/start_run", json!({"objective": "test", "api_key": "secret_ref:env:MY_KEY"}));
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("agentic_forge_run_started"));
        assert_ne!(result["redaction_state"], json!("unsafe_blocked"));
    }

    #[test]
    fn cancel_run_sets_cancelled_state() {
        let req = make_request("official/agentic-forge-lab/cancel_run", json!({"run_id": "run_test", "lifecycle_state": "running"}));
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("agentic_forge_run_cancelled"));
        assert_eq!(result["lifecycle_state"], json!("cancelled"));
    }

    #[test]
    fn summarize_run_returns_observability() {
        let req = make_request("official/agentic-forge-lab/summarize_run", json!({"run_id": "run_test", "trace_events": [{"a": 1}, {"b": 2}]}));
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("agentic_forge_run_summary"));
        assert_eq!(result["trace_event_count"], json!(2));
    }

    #[test]
    fn export_plan_graph_returns_nodes_edges() {
        let req = make_request("official/agentic-forge-lab/export_plan_graph", json!({"run_id": "run_test"}));
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("agentic_forge_plan_graph"));
        assert!(result["plan_graph"]["nodes"].as_array().unwrap().len() >= 1);
        assert!(result["plan_graph"]["approval_policy"].is_string());
    }

    #[test]
    fn no_kernel_agent_namespace_in_output() {
        let req = make_request("official/agentic-forge-lab/start_run", json!({"objective": "test"}));
        let result = try_handle(&req).unwrap().unwrap();
        let output_str = serde_json::to_string(&result).unwrap();
        assert!(!output_str.contains("kernel.agent"));
        assert!(!output_str.contains("kernel.model"));
        assert!(!output_str.contains("kernel.prompt"));
        assert!(!output_str.contains("kernel.memory"));
        assert!(!output_str.contains("kernel.turn"));
    }

    #[test]
    fn all_lifecycle_states_valid() {
        for state in LIFECYCLE_STATES {
            assert!(is_valid_lifecycle_state(state));
        }
        assert!(!is_valid_lifecycle_state("unknown_state"));
    }

    #[test]
    fn contains_raw_secret_detects_sk_prefix() {
        assert!(contains_raw_secret(&json!({"api_key": "RawSecretExample1234567890abcdefABCDEF123456"})));
        assert!(contains_raw_secret(&json!({"token": "Bearer xyz"})));
        assert!(!contains_raw_secret(&json!({"api_key": "secret_ref:env:MY_KEY"})));
        assert!(!contains_raw_secret(&json!({"api_key": "secret-ref:env:MY_KEY"})));
        assert!(!contains_raw_secret(&json!({"api_key": "host:env:MY_KEY"})));
        assert!(!contains_raw_secret(&json!({"objective": "safe text"})));
    }
}
