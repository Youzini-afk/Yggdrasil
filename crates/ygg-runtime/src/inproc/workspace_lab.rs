//! Handler for `official/workspace-lab` capabilities.
//!
//! External Project Operating Plane Alpha Phase E2 — Workspace Action Policy Boundary.
//!
//! Workspace action policy boundary for external project workspaces.
//! No real execution, no shell, no clone, no install, no run.
//! Deny-by-default fake executor; all dangerous actions require approval.
//!
//! Capabilities:
//! - describe_workspace_contract: describe the workspace lab contract
//! - draft_workspace_creation: draft a plan-only workspace creation proposal
//! - explain_required_permissions: explain permissions required for workspace actions
//! - request_workspace_action: request a workspace action (deny-by-default, fake executor)
//! - summarize_workspace_audit: produce deterministic redacted audit summary
//!
//! Safety:
//! - Raw secret blocking (delegated to shared safety module)
//! - No kernel.project/workspace/git/npm/deploy namespace references in outputs
//! - No filesystem reads, no shell, no outbound, no execution
//! - Deny-by-default: executor_invoked=false, execution_performed=false
//! - Approval tokens are not honored in Alpha; proposal_required=true always

use serde_json::Value;

use super::safety;
use super::InprocInvocation;

const PACKAGE_ID: &str = "official/workspace-lab";

// ---------------------------------------------------------------------------
// Action taxonomy
// ---------------------------------------------------------------------------

const ACTION_TAXONOMY: &[(&str, &str, bool, bool, bool, bool)] = &[
    // (action, risk_level, requires_approval, executes_code, network_required, filesystem_write_required)
    ("clone_project", "high", true, false, true, true),
    ("read_metadata", "low", false, false, false, false),
    ("install_dependencies", "critical", true, true, true, true),
    ("run_command", "critical", true, true, false, true),
    ("run_tests", "high", true, true, false, true),
    ("stop_process", "medium", true, false, false, false),
    ("read_logs", "low", false, false, false, false),
    ("discover_entrypoints", "low", false, false, false, false),
    ("write_patch", "high", true, false, false, true),
    ("deploy_plan", "critical", true, true, true, true),
];

const VALID_ACTIONS: &[&str] = &[
    "clone_project",
    "read_metadata",
    "install_dependencies",
    "run_command",
    "run_tests",
    "stop_process",
    "read_logs",
    "discover_entrypoints",
    "write_patch",
    "deploy_plan",
];

// ---------------------------------------------------------------------------
// Forbidden namespace tokens (must not appear in outputs)
// ---------------------------------------------------------------------------

/// Kernel namespace tokens that must not appear in outputs.
const FORBIDDEN_NAMESPACE_TOKENS: &[&str] = &[
    "kernel.project.",
    "kernel.workspace.",
    "kernel.git.",
    "kernel.npm.",
    "kernel.deploy.",
    "kernel.ide.",
];

/// Check whether a string contains forbidden kernel namespace tokens.
/// Used in tests to verify outputs do not leak kernel namespaces.
#[allow(dead_code)]
fn contains_forbidden_namespace(s: &str) -> bool {
    FORBIDDEN_NAMESPACE_TOKENS.iter().any(|t| s.contains(t))
}

// ---------------------------------------------------------------------------
// Rejected output for raw-secret input
// ---------------------------------------------------------------------------

fn rejected_output(request: &InprocInvocation) -> Value {
    serde_json::json!({
        "kind": "workspace_lab_rejected",
        "redaction_state": "unsafe_blocked",
        "reason": "input contains raw-secret-like content; use secret_ref references instead",
        "executor_invoked": false,
        "execution_performed": false,
        "inference_performed": false,
        "network_performed": false,
        "filesystem_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    })
}

// ---------------------------------------------------------------------------
// Lookup action taxonomy entry
// ---------------------------------------------------------------------------

fn lookup_action(action: &str) -> Option<(&'static str, &'static str, bool, bool, bool, bool)> {
    ACTION_TAXONOMY
        .iter()
        .find(|(a, _, _, _, _, _)| *a == action)
        .map(|(a, risk, appr, code, net, fs)| (*a, *risk, *appr, *code, *net, *fs))
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

pub fn try_handle(request: &InprocInvocation) -> Option<anyhow::Result<Value>> {
    if request.provider_package_id != PACKAGE_ID {
        return None;
    }
    let id = request.capability_id.as_str();
    if id.ends_with("/describe_workspace_contract") {
        Some(describe_workspace_contract(request))
    } else if id.ends_with("/draft_workspace_creation") {
        Some(draft_workspace_creation(request))
    } else if id.ends_with("/explain_required_permissions") {
        Some(explain_required_permissions(request))
    } else if id.ends_with("/request_workspace_action") {
        Some(request_workspace_action(request))
    } else if id.ends_with("/summarize_workspace_audit") {
        Some(summarize_workspace_audit(request))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Capability implementations
// ---------------------------------------------------------------------------

fn describe_workspace_contract(request: &InprocInvocation) -> anyhow::Result<Value> {
    let action_entries: Vec<Value> = ACTION_TAXONOMY
        .iter()
        .map(|(action, risk, appr, code, net, fs)| {
            serde_json::json!({
                "action": action,
                "risk_level": risk,
                "requires_approval": appr,
                "executes_code": code,
                "network_required": net,
                "filesystem_write_required": fs,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "kind": "workspace_lab_contract",
        "package_id": request.provider_package_id,
        "package_kind": "ordinary",
        "capabilities": [
            {"id": "official/workspace-lab/describe_workspace_contract", "purpose": "describe the workspace lab contract and action taxonomy"},
            {"id": "official/workspace-lab/draft_workspace_creation", "purpose": "draft a plan-only workspace creation proposal, no direct workspace creation"},
            {"id": "official/workspace-lab/explain_required_permissions", "purpose": "explain permissions required for workspace actions"},
            {"id": "official/workspace-lab/request_workspace_action", "purpose": "request a workspace action with deny-by-default policy; no real execution"},
            {"id": "official/workspace-lab/summarize_workspace_audit", "purpose": "produce deterministic redacted audit summary; no raw env/logs/commands"},
        ],
        "surfaces": {
            "forge_panel": "official/workspace-lab/forge-panel",
            "assistant_action": "official/workspace-lab/assistant-action",
            "home_card": "official/workspace-lab/home-card",
        },
        "action_taxonomy": action_entries,
        "policy_defaults": {
            "default_decision": "denied_by_default",
            "requires_approval": true,
            "executor_invoked": false,
            "execution_performed": false,
            "proposal_required": true,
            "approval_token_honored": false,
        },
        "audit_shape": {
            "fields": ["workspace_ref", "action", "policy_decision", "executor_invoked", "execution_performed", "proposal_required", "audit_preview"],
            "redaction": "no raw env, no raw logs, no raw commands, no raw secrets",
        },
        "inference_performed": false,
        "network_performed": false,
        "execution_performed": false,
        "filesystem_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn draft_workspace_creation(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request));
    }

    let workspace_ref = request
        .input
        .get("workspace_ref")
        .and_then(Value::as_str)
        .unwrap_or("");

    let source_ref = request
        .input
        .get("source_ref")
        .and_then(Value::as_str)
        .unwrap_or("");

    // Collect required actions and their risk levels
    let mut proposed_actions: Vec<Value> = Vec::new();
    let mut risk_notes: Vec<Value> = Vec::new();

    // Typical workspace creation needs: clone + read_metadata + install_dependencies + discover_entrypoints
    for action in &["clone_project", "read_metadata", "install_dependencies", "discover_entrypoints"] {
        if let Some((_, risk, appr, code, net, fs)) = lookup_action(action) {
            proposed_actions.push(serde_json::json!({
                "action": action,
                "risk_level": risk,
                "requires_approval": appr,
                "executes_code": code,
                "network_required": net,
                "filesystem_write_required": fs,
            }));
        }
    }

    risk_notes.push(serde_json::json!({
        "kind": "deny_by_default",
        "note": "all workspace actions are denied by default; each requires explicit approval via proposal"
    }));
    risk_notes.push(serde_json::json!({
        "kind": "no_execution",
        "note": "Alpha phase does not execute any workspace action; executor_invoked is always false"
    }));
    if !source_ref.is_empty() {
        risk_notes.push(serde_json::json!({
            "kind": "source_ref_present",
            "note": "source ref provided; clone or fetch may be required which needs network and policy approval"
        }));
    }

    Ok(serde_json::json!({
        "kind": "workspace_creation_draft",
        "plan_only": true,
        "requires_user_approval": true,
        "workspace_ref": if workspace_ref.is_empty() { Value::Null } else { serde_json::json!(workspace_ref) },
        "source_ref": if source_ref.is_empty() { Value::Null } else { serde_json::json!(source_ref) },
        "proposed_actions": proposed_actions,
        "risk_notes": risk_notes,
        "executor_invoked": false,
        "execution_performed": false,
        "inference_performed": false,
        "network_performed": false,
        "filesystem_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn explain_required_permissions(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request));
    }

    let action = request
        .input
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("");

    let mut explanations: Vec<Value> = Vec::new();

    if action.is_empty() {
        // Explain all actions
        for (act, risk, appr, code, net, fs) in ACTION_TAXONOMY {
            explanations.push(serde_json::json!({
                "action": act,
                "risk_level": risk,
                "requires_approval": appr,
                "executes_code": code,
                "network_required": net,
                "filesystem_write_required": fs,
                "explanation": format_permission_explanation(act, *risk, *appr, *code, *net, *fs),
            }));
        }
    } else if let Some((_, risk, appr, code, net, fs)) = lookup_action(action) {
        explanations.push(serde_json::json!({
            "action": action,
            "risk_level": risk,
            "requires_approval": appr,
            "executes_code": code,
            "network_required": net,
            "filesystem_write_required": fs,
            "explanation": format_permission_explanation(action, risk, appr, code, net, fs),
        }));
    } else {
        return Ok(serde_json::json!({
            "kind": "workspace_lab_rejected",
            "redaction_state": "action_unknown",
            "reason": format!("action '{}' is not in the action taxonomy; request denied", action),
            "executor_invoked": false,
            "execution_performed": false,
            "inference_performed": false,
            "network_performed": false,
            "filesystem_performed": false,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    Ok(serde_json::json!({
        "kind": "workspace_permission_explanation",
        "explanations": explanations,
        "policy_defaults": {
            "default_decision": "denied_by_default",
            "requires_approval": true,
            "approval_token_honored": false,
        },
        "inference_performed": false,
        "network_performed": false,
        "execution_performed": false,
        "filesystem_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn request_workspace_action(request: &InprocInvocation) -> anyhow::Result<Value> {
    // Extract known workspace-lab fields before raw-secret scanning.
    // `approval_token` contains "token" which the shared safety module
    // would flag; we need to handle it specially here.
    let action = request
        .input
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("");

    let workspace_ref = request
        .input
        .get("workspace_ref")
        .and_then(Value::as_str)
        .unwrap_or("");

    let source_ref = request
        .input
        .get("source_ref")
        .and_then(Value::as_str)
        .unwrap_or("");

    let approval_token = request
        .input
        .get("approval_token")
        .and_then(Value::as_str);

    let policy = request
        .input
        .get("policy")
        .and_then(Value::as_str)
        .unwrap_or("");

    // Raw-secret blocking: scan input excluding known workspace-lab fields
    // that the safety module would false-positive on (approval_token contains "token").
    let input_for_scan = {
        let mut obj = request.input.as_object().cloned().unwrap_or_default();
        obj.remove("approval_token"); // remove before scanning; token is a protocol field, not a secret
        serde_json::Value::Object(obj)
    };

    if safety::contains_raw_secret(&input_for_scan) {
        return Ok(serde_json::json!({
            "kind": "workspace_lab_rejected",
            "redaction_state": "unsafe_blocked",
            "reason": "input contains raw-secret-like content; use secret_ref references instead",
            "executor_invoked": false,
            "execution_performed": false,
            "proposal_required": true,
            "inference_performed": false,
            "network_performed": false,
            "filesystem_performed": false,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    // Validate action is in taxonomy
    if !VALID_ACTIONS.contains(&action) {
        return Ok(serde_json::json!({
            "kind": "workspace_action_rejected",
            "policy_decision": "denied",
            "reason": format!("action '{}' is not in the action taxonomy; fail-closed", action),
            "executor_invoked": false,
            "execution_performed": false,
            "proposal_required": true,
            "audit_preview": {
                "action": if action.is_empty() { Value::Null } else { serde_json::json!(action) },
                "workspace_ref": if workspace_ref.is_empty() { Value::Null } else { serde_json::json!(workspace_ref) },
                "policy_decision": "denied",
                "denial_reason": "unknown_action",
            },
            "inference_performed": false,
            "network_performed": false,
            "filesystem_performed": false,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    let (_, risk, requires_approval, executes_code, network_required, fs_write) =
        lookup_action(action).unwrap();

    // Policy/action mismatch: if policy claims "allow" but the action taxonomy
    // requires approval or the policy value is invalid, fail closed.
    let valid_policies = ["", "deny", "require_approval", "propose"];
    let policy_mismatch = !valid_policies.contains(&policy)
        || (policy == "allow" && requires_approval);

    if policy_mismatch {
        return Ok(serde_json::json!({
            "kind": "workspace_action_rejected",
            "policy_decision": "denied",
            "reason": "policy/action mismatch: policy must not claim 'allow' for approval-required actions; fail-closed",
            "executor_invoked": false,
            "execution_performed": false,
            "proposal_required": true,
            "audit_preview": {
                "action": action,
                "workspace_ref": if workspace_ref.is_empty() { Value::Null } else { serde_json::json!(workspace_ref) },
                "policy_decision": "denied",
                "denial_reason": "policy_action_mismatch",
            },
            "inference_performed": false,
            "network_performed": false,
            "filesystem_performed": false,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    // Approval tokens are not honored in Alpha phase
    let approval_token_rejected = approval_token.is_some();

    // Build audit preview (no raw env/logs/commands/secrets)
    let mut audit_preview = serde_json::json!({
        "action": action,
        "workspace_ref": if workspace_ref.is_empty() { Value::Null } else { serde_json::json!(workspace_ref) },
        "policy_decision": "denied_by_default",
        "risk_level": risk,
        "requires_approval": requires_approval,
        "executes_code": executes_code,
        "network_required": network_required,
        "filesystem_write_required": fs_write,
        "approval_token_present": approval_token.is_some(),
        "approval_token_honored": false,
    });

    if !source_ref.is_empty() {
        audit_preview["source_ref_provided"] = serde_json::json!(true);
    }

    // Deny-by-default: always output denied, no execution
    Ok(serde_json::json!({
        "kind": "workspace_action_denied_by_default",
        "policy_decision": "denied_by_default",
        "executor_invoked": false,
        "execution_performed": false,
        "proposal_required": true,
        "audit_preview": audit_preview,
        "risk_level": risk,
        "requires_approval": requires_approval,
        "executes_code": executes_code,
        "network_required": network_required,
        "filesystem_write_required": fs_write,
        "approval_token_rejected": approval_token_rejected,
        "approval_token_honored": false,
        "inference_performed": false,
        "network_performed": false,
        "filesystem_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

fn summarize_workspace_audit(request: &InprocInvocation) -> anyhow::Result<Value> {
    // Check top-level fields (excluding action_history) for raw secrets.
    // action_history entries are individually redacted below.
    let top_level_for_scan = {
        let mut obj = request.input.as_object().cloned().unwrap_or_default();
        obj.remove("action_history"); // handled per-entry below
        serde_json::Value::Object(obj)
    };
    if safety::contains_raw_secret(&top_level_for_scan) {
        return Ok(rejected_output(request));
    }

    let workspace_ref = request
        .input
        .get("workspace_ref")
        .and_then(Value::as_str)
        .unwrap_or("");

    let action_history = request
        .input
        .get("action_history")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    // Produce deterministic redacted audit summary
    // Never include raw env, logs, commands, or secrets
    let mut action_summaries: Vec<Value> = Vec::new();
    let mut total_actions = 0u64;
    let mut denied_count = 0u64;
    let mut approved_count = 0u64;
    let mut pending_count = 0u64;

    for entry in &action_history {
        if safety::contains_raw_secret(entry) {
            // Skip entries containing raw secrets — redact them
            action_summaries.push(serde_json::json!({
                "redacted": true,
                "redaction_reason": "entry contained raw-secret-like content",
            }));
            denied_count += 1;
            total_actions += 1;
            continue;
        }

        let action = entry
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let decision = entry
            .get("policy_decision")
            .and_then(Value::as_str)
            .unwrap_or("unknown");

        // Build redacted summary — only safe fields
        let mut summary = serde_json::json!({
            "action": if VALID_ACTIONS.contains(&action) { action } else { "redacted_unknown" },
            "policy_decision": decision,
            "executor_invoked": entry.get("executor_invoked").and_then(Value::as_bool).unwrap_or(false),
            "execution_performed": entry.get("execution_performed").and_then(Value::as_bool).unwrap_or(false),
        });

        // Include risk_level from taxonomy if known
        if let Some((_, risk, _, _, _, _)) = lookup_action(action) {
            summary["risk_level"] = serde_json::json!(risk);
        }

        // Strip any raw fields that might have leaked in
        for key in &["raw_command", "raw_env", "raw_log", "raw_output", "secret", "api_key", "token", "password"] {
            if let Some(obj) = summary.as_object_mut() {
                obj.remove(*key);
            }
        }

        action_summaries.push(summary);
        total_actions += 1;
        match decision {
            "denied" | "denied_by_default" => denied_count += 1,
            "approved" | "allowed" => approved_count += 1,
            "pending" | "proposed" => pending_count += 1,
            _ => {}
        }
    }

    Ok(serde_json::json!({
        "kind": "workspace_audit_summary",
        "workspace_ref": if workspace_ref.is_empty() { Value::Null } else { serde_json::json!(workspace_ref) },
        "total_actions": total_actions,
        "denied_count": denied_count,
        "approved_count": approved_count,
        "pending_count": pending_count,
        "action_summaries": action_summaries,
        "redaction_applied": true,
        "redaction_policy": "no raw env, no raw logs, no raw commands, no raw secrets",
        "inference_performed": false,
        "network_performed": false,
        "execution_performed": false,
        "filesystem_performed": false,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

// ---------------------------------------------------------------------------
// Permission explanation formatting
// ---------------------------------------------------------------------------

fn format_permission_explanation(
    action: &str,
    risk: &str,
    appr: bool,
    code: bool,
    net: bool,
    fs: bool,
) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if appr {
        parts.push("requires explicit user approval");
    }
    if code {
        parts.push("executes arbitrary code");
    }
    if net {
        parts.push("requires network access");
    }
    if fs {
        parts.push("writes to filesystem");
    }
    let detail = if parts.is_empty() {
        "read-only, no side effects".to_string()
    } else {
        parts.join("; ")
    };
    format!(
        "action '{}' has {} risk and {}",
        action,
        risk,
        detail
    )
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
        let req = make_request(
            "official/workspace-lab/describe_workspace_contract",
            json!({}),
        );
        assert!(try_handle(&req).is_some());
    }

    #[test]
    fn try_handle_rejects_wrong_package() {
        let req = InprocInvocation {
            capability_id: "official/workspace-lab/describe_workspace_contract".to_string(),
            provider_package_id: "official/other".to_string(),
            input: json!({}),
        };
        assert!(try_handle(&req).is_none());
    }

    #[test]
    fn describe_contract_has_all_surfaces() {
        let req = make_request(
            "official/workspace-lab/describe_workspace_contract",
            json!({}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        let surfaces = result["surfaces"].as_object().unwrap();
        assert!(surfaces.contains_key("forge_panel"));
        assert!(surfaces.contains_key("assistant_action"));
        assert!(surfaces.contains_key("home_card"));
    }

    #[test]
    fn describe_contract_lists_5_capabilities() {
        let req = make_request(
            "official/workspace-lab/describe_workspace_contract",
            json!({}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(
            result["capabilities"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0),
            5,
            "must list 5 capabilities"
        );
    }

    #[test]
    fn describe_contract_lists_10_action_taxonomy_entries() {
        let req = make_request(
            "official/workspace-lab/describe_workspace_contract",
            json!({}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(
            result["action_taxonomy"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0),
            10,
            "must list 10 action taxonomy entries"
        );
    }

    #[test]
    fn describe_contract_deny_by_default() {
        let req = make_request(
            "official/workspace-lab/describe_workspace_contract",
            json!({}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(
            result["policy_defaults"]["default_decision"],
            json!("denied_by_default")
        );
        assert_eq!(
            result["policy_defaults"]["executor_invoked"],
            json!(false)
        );
        assert_eq!(
            result["policy_defaults"]["execution_performed"],
            json!(false)
        );
    }

    #[test]
    fn draft_creation_is_plan_only() {
        let req = make_request(
            "official/workspace-lab/draft_workspace_creation",
            json!({"workspace_ref": "ws-001", "source_ref": "https://example.com/project.git"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("workspace_creation_draft"));
        assert_eq!(result["plan_only"], json!(true));
        assert_eq!(result["requires_user_approval"], json!(true));
        assert_eq!(result["executor_invoked"], json!(false));
        assert_eq!(result["execution_performed"], json!(false));
    }

    #[test]
    fn request_action_denied_by_default() {
        let req = make_request(
            "official/workspace-lab/request_workspace_action",
            json!({"action": "clone_project", "workspace_ref": "ws-001"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("workspace_action_denied_by_default"));
        assert_eq!(result["policy_decision"], json!("denied_by_default"));
        assert_eq!(result["executor_invoked"], json!(false));
        assert_eq!(result["execution_performed"], json!(false));
        assert_eq!(result["proposal_required"], json!(true));
    }

    #[test]
    fn request_action_unknown_action_denied() {
        let req = make_request(
            "official/workspace-lab/request_workspace_action",
            json!({"action": "hack_the_planet", "workspace_ref": "ws-001"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("workspace_action_rejected"));
        assert_eq!(result["policy_decision"], json!("denied"));
    }

    #[test]
    fn request_action_policy_mismatch_fail_closed() {
        let req = make_request(
            "official/workspace-lab/request_workspace_action",
            json!({"action": "install_dependencies", "workspace_ref": "ws-001", "policy": "allow"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("workspace_action_rejected"));
        assert_eq!(result["policy_decision"], json!("denied"));
    }

    #[test]
    fn request_action_approval_token_not_honored() {
        let req = make_request(
            "official/workspace-lab/request_workspace_action",
            json!({"action": "clone_project", "workspace_ref": "ws-001", "approval_token": "fake-token-12345"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["policy_decision"], json!("denied_by_default"));
        assert_eq!(result["executor_invoked"], json!(false));
        assert_eq!(result["execution_performed"], json!(false));
        assert_eq!(result["approval_token_rejected"], json!(true));
        assert_eq!(result["approval_token_honored"], json!(false));
    }

    #[test]
    fn raw_secret_blocked() {
        let req = make_request(
            "official/workspace-lab/request_workspace_action",
            json!({"action": "clone_project", "workspace_ref": "ws-001", "api_key": "RawSecretExample1234567890abcdefABCDEF123456"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("workspace_lab_rejected"));
        assert_eq!(result["redaction_state"], json!("unsafe_blocked"));
    }

    #[test]
    fn audit_summary_redacted() {
        let req = make_request(
            "official/workspace-lab/summarize_workspace_audit",
            json!({
                "workspace_ref": "ws-001",
                "action_history": [
                    {"action": "clone_project", "policy_decision": "denied_by_default", "executor_invoked": false, "execution_performed": false},
                    {"action": "read_metadata", "policy_decision": "approved", "executor_invoked": false, "execution_performed": false},
                    {"action": "run_command", "policy_decision": "pending", "executor_invoked": false, "execution_performed": false, "raw_command": "rm -rf /"},
                ]
            }),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("workspace_audit_summary"));
        assert_eq!(result["total_actions"], json!(3));
        assert_eq!(result["denied_count"], json!(1));
        assert_eq!(result["approved_count"], json!(1));
        assert_eq!(result["pending_count"], json!(1));
        assert_eq!(result["redaction_applied"], json!(true));

        // Verify no raw_command leaked
        let output_str = serde_json::to_string(&result).unwrap();
        assert!(!output_str.contains("rm -rf"));
        assert!(!output_str.contains("raw_command"));
        assert!(!output_str.contains("raw_env"));
        assert!(!output_str.contains("raw_log"));
    }

    #[test]
    fn audit_summary_raw_secret_redacted() {
        let req = make_request(
            "official/workspace-lab/summarize_workspace_audit",
            json!({
                "workspace_ref": "ws-001",
                "action_history": [
                    {"action": "clone_project", "policy_decision": "denied_by_default", "executor_invoked": false, "execution_performed": false},
                    {"action": "run_command", "policy_decision": "approved", "executor_invoked": false, "execution_performed": false, "secret": "RawSecretExample1234567890abcdefABCDEF123456"},
                ]
            }),
        );
        let result = try_handle(&req).unwrap().unwrap();
        // The entry with secret should be redacted
        let summaries = result["action_summaries"].as_array().unwrap();
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[1]["redacted"], json!(true));
        assert_eq!(result["denied_count"], json!(2));
    }

    #[test]
    fn no_forbidden_namespace_in_contract() {
        let req = make_request(
            "official/workspace-lab/describe_workspace_contract",
            json!({}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        let output_str = serde_json::to_string(&result).unwrap();
        assert!(!contains_forbidden_namespace(&output_str), "must not contain forbidden namespace tokens");
    }

    #[test]
    fn no_forbidden_namespace_all_capabilities() {
        let caps = [
            "describe_workspace_contract",
            "draft_workspace_creation",
            "explain_required_permissions",
            "request_workspace_action",
            "summarize_workspace_audit",
        ];
        for cap in &caps {
            let req = make_request(
                &format!("official/workspace-lab/{}", cap),
                json!({"workspace_ref": "ws-001", "action": "read_metadata"}),
            );
            let result = try_handle(&req).unwrap().unwrap();
            let output_str = serde_json::to_string(&result).unwrap();
            assert!(
                !contains_forbidden_namespace(&output_str),
                "{} must not contain forbidden namespace tokens",
                cap
            );
        }
    }

    #[test]
    fn no_execution_performed() {
        let caps = [
            "describe_workspace_contract",
            "draft_workspace_creation",
            "explain_required_permissions",
            "request_workspace_action",
            "summarize_workspace_audit",
        ];
        for cap in &caps {
            let req = make_request(
                &format!("official/workspace-lab/{}", cap),
                json!({"workspace_ref": "ws-001", "action": "clone_project"}),
            );
            let result = try_handle(&req).unwrap().unwrap();
            assert_eq!(
                result["execution_performed"],
                json!(false),
                "{} must have execution_performed=false",
                cap
            );
            assert_eq!(
                result["network_performed"],
                json!(false),
                "{} must have network_performed=false",
                cap
            );
            assert_eq!(
                result["inference_performed"],
                json!(false),
                "{} must have inference_performed=false",
                cap
            );
            assert_eq!(
                result["filesystem_performed"],
                json!(false),
                "{} must have filesystem_performed=false",
                cap
            );
        }
    }

    #[test]
    fn action_taxonomy_complete() {
        assert_eq!(ACTION_TAXONOMY.len(), 10);
        assert_eq!(VALID_ACTIONS.len(), 10);
        for (action, _, _, _, _, _) in ACTION_TAXONOMY {
            assert!(VALID_ACTIONS.contains(action), "action {} must be in VALID_ACTIONS", action);
        }
    }

    #[test]
    fn action_taxonomy_risk_annotations() {
        // clone_project: high risk, requires approval, no code exec, network + fs write
        let (_, risk, appr, code, net, fs) = lookup_action("clone_project").unwrap();
        assert_eq!(risk, "high");
        assert!(appr);
        assert!(!code);
        assert!(net);
        assert!(fs);

        // read_metadata: low risk, no approval, no code, no network, no fs write
        let (_, risk, appr, code, net, fs) = lookup_action("read_metadata").unwrap();
        assert_eq!(risk, "low");
        assert!(!appr);
        assert!(!code);
        assert!(!net);
        assert!(!fs);

        // install_dependencies: critical, requires approval, executes code
        let (_, risk, appr, code, net, fs) = lookup_action("install_dependencies").unwrap();
        assert_eq!(risk, "critical");
        assert!(appr);
        assert!(code);
        assert!(net);
        assert!(fs);

        // run_command: critical, executes code
        let (_, risk, appr, code, _, _) = lookup_action("run_command").unwrap();
        assert_eq!(risk, "critical");
        assert!(appr);
        assert!(code);

        // deploy_plan: critical, executes code, network + fs write
        let (_, risk, appr, code, net, fs) = lookup_action("deploy_plan").unwrap();
        assert_eq!(risk, "critical");
        assert!(appr);
        assert!(code);
        assert!(net);
        assert!(fs);
    }

    #[test]
    fn explain_permissions_for_single_action() {
        let req = make_request(
            "official/workspace-lab/explain_required_permissions",
            json!({"action": "clone_project"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("workspace_permission_explanation"));
        let explanations = result["explanations"].as_array().unwrap();
        assert_eq!(explanations.len(), 1);
        assert_eq!(explanations[0]["action"], json!("clone_project"));
    }

    #[test]
    fn explain_permissions_for_unknown_action() {
        let req = make_request(
            "official/workspace-lab/explain_required_permissions",
            json!({"action": "teleport"}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("workspace_lab_rejected"));
        assert_eq!(result["redaction_state"], json!("action_unknown"));
    }

    #[test]
    fn explain_permissions_all_when_no_action() {
        let req = make_request(
            "official/workspace-lab/explain_required_permissions",
            json!({}),
        );
        let result = try_handle(&req).unwrap().unwrap();
        assert_eq!(result["kind"], json!("workspace_permission_explanation"));
        let explanations = result["explanations"].as_array().unwrap();
        assert_eq!(explanations.len(), 10);
    }
}
