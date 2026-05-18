//! Handler for `official/capability-tool-bridge-lab` capabilities.
//!
//! The tool bridge discovers capabilities, previews permissions, and drafts
//! invocation/streaming plans. It never performs real capability calls —
//! it outputs deterministic plans and diagnostics only. As an ordinary
//! package it cannot privately call the runtime.

use serde_json::Value;

use super::InprocInvocation;

const PACKAGE_ID: &str = "official/capability-tool-bridge-lab";

/// Known secret field names that indicate a raw secret in input-like payloads.
const SECRET_FIELD_NAMES: &[&str] = &[
    "api_key",
    "secret",
    "token",
    "password",
    "secret_key",
    "access_token",
    "refresh_token",
    "private_key",
    "auth_token",
];

/// Value patterns that look like raw secrets.
fn looks_like_raw_secret(value: &str) -> bool {
    value.starts_with("sk-")
        || value.starts_with("Bearer ")
        || value.starts_with("ghp_")
        || value.starts_with("gho_")
        || value.starts_with("glpat-")
        || (value.len() >= 32
            && value.chars().filter(|c| c.is_ascii_alphanumeric()).count() == value.len()
            && value.chars().filter(|c| c.is_ascii_digit()).count() < value.len() / 2)
}

/// Scan input-like payloads for raw secrets.
fn scan_for_raw_secrets(input: &Value) -> (Vec<Value>, String) {
    let mut findings = Vec::new();
    scan_value(input, "", &mut findings);
    let redaction_state = if findings.is_empty() {
        "clean".to_string()
    } else {
        "unsafe_blocked".to_string()
    };
    (findings, redaction_state)
}

fn scan_value(value: &Value, path: &str, findings: &mut Vec<Value>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                // Check if this is a secret field name
                if SECRET_FIELD_NAMES.contains(&key.as_str()) {
                    if child.is_string() {
                        findings.push(serde_json::json!({
                            "path": child_path,
                            "field": key,
                            "reason": "secret_field_name"
                        }));
                    }
                } else if let Some(s) = child.as_str() {
                    if looks_like_raw_secret(s) {
                        findings.push(serde_json::json!({
                            "path": child_path,
                            "field": key,
                            "reason": "value_pattern"
                        }));
                    }
                }
                scan_value(child, &child_path, findings);
            }
        }
        Value::Array(arr) => {
            for (i, child) in arr.iter().enumerate() {
                let child_path = format!("{path}[{i}]");
                scan_value(child, &child_path, findings);
            }
        }
        _ => {}
    }
}

fn provider_candidates(input: &Value) -> Vec<String> {
    input
        .get("providers")
        .and_then(Value::as_array)
        .map(|providers| providers.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default()
}

fn provider_matches_candidates(provider_package_id: &str, candidates: &[String]) -> bool {
    candidates.is_empty() || candidates.iter().any(|candidate| candidate == provider_package_id)
}

pub fn try_handle(request: &InprocInvocation) -> Option<anyhow::Result<Value>> {
    if request.provider_package_id != PACKAGE_ID {
        return None;
    }
    let id = request.capability_id.as_str();
    if id.ends_with("/discover_tools") {
        Some(discover_tools(request))
    } else if id.ends_with("/preview_tool_permissions") {
        Some(preview_tool_permissions(request))
    } else if id.ends_with("/invoke_tool") {
        Some(invoke_tool(request))
    } else if id.ends_with("/stream_tool") {
        Some(stream_tool(request))
    } else if id.ends_with("/explain_tool_call") {
        Some(explain_tool_call(request))
    } else if id.ends_with("/echo") {
        Some(echo(request))
    } else {
        None
    }
}

/// discover_tools: receives a `capabilities` array, outputs `agent_tool` descriptors.
///
/// If the same capability_id appears from multiple providers and no explicit
/// provider is given, the tool is marked `ambiguous`/`rejected`. No preference
/// is given to official providers.
fn discover_tools(request: &InprocInvocation) -> anyhow::Result<Value> {
    let (secret_findings, redaction_state) = scan_for_raw_secrets(&request.input);
    if !secret_findings.is_empty() {
        return Ok(serde_json::json!({
            "kind": "tool_bridge_discovery",
            "tools": [],
            "redaction_state": "unsafe_blocked",
            "secret_findings": secret_findings,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    let capabilities = request
        .input
        .get("capabilities")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut tools = Vec::new();
    for cap in &capabilities {
        let cap_id = cap.get("capability_id").and_then(Value::as_str).unwrap_or("");
        let providers = cap.get("providers").and_then(Value::as_array);
        let explicit_provider = cap.get("provider_package_id").and_then(Value::as_str);

        if explicit_provider.is_some() && !explicit_provider.unwrap().is_empty() {
            let provider = explicit_provider.unwrap();
            let candidates: Vec<String> = providers
                .map(|items| items.iter().filter_map(Value::as_str).map(str::to_string).collect())
                .unwrap_or_default();
            if provider_matches_candidates(provider, &candidates) {
                // Explicit provider selected and either no candidate list was supplied
                // or it matches the candidate set.
                tools.push(serde_json::json!({
                    "tool_type": "agent_tool",
                    "capability_id": cap_id,
                    "provider_package_id": provider,
                    "status": "available",
                    "ambiguous": false
                }));
            } else {
                tools.push(serde_json::json!({
                    "tool_type": "agent_tool",
                    "capability_id": cap_id,
                    "provider_package_id": provider,
                    "status": "rejected",
                    "ambiguous": false,
                    "rejection_reason": "provider_not_in_candidates",
                    "available_providers": candidates
                }));
            }
        } else if let Some(providers) = providers {
            if providers.len() > 1 {
                // Ambiguous: multiple providers, no explicit choice
                tools.push(serde_json::json!({
                    "tool_type": "agent_tool",
                    "capability_id": cap_id,
                    "provider_package_id": null,
                    "status": "rejected",
                    "ambiguous": true,
                    "available_providers": providers
                }));
            } else if providers.len() == 1 {
                let provider_id = providers[0].as_str().unwrap_or("");
                tools.push(serde_json::json!({
                    "tool_type": "agent_tool",
                    "capability_id": cap_id,
                    "provider_package_id": provider_id,
                    "status": "available",
                    "ambiguous": false
                }));
            } else {
                tools.push(serde_json::json!({
                    "tool_type": "agent_tool",
                    "capability_id": cap_id,
                    "provider_package_id": null,
                    "status": "no_provider",
                    "ambiguous": false
                }));
            }
        } else {
            // No provider info — single or unknown
            tools.push(serde_json::json!({
                "tool_type": "agent_tool",
                "capability_id": cap_id,
                "provider_package_id": null,
                "status": "unknown_provider",
                "ambiguous": false
            }));
        }
    }

    Ok(serde_json::json!({
        "kind": "tool_bridge_discovery",
        "tools": tools,
        "redaction_state": redaction_state,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

/// preview_tool_permissions: receives tool/caller/principal/grants,
/// outputs allowed/missing_permissions/required_provider_package_id.
fn preview_tool_permissions(request: &InprocInvocation) -> anyhow::Result<Value> {
    let (secret_findings, redaction_state) = scan_for_raw_secrets(&request.input);
    if !secret_findings.is_empty() {
        return Ok(serde_json::json!({
            "kind": "tool_bridge_permission_preview",
            "allowed": false,
            "missing_permissions": [],
            "redaction_state": "unsafe_blocked",
            "secret_findings": secret_findings,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    let grants = request
        .input
        .get("grants")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let required_permissions = request
        .input
        .get("required_permissions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let provider_package_id = request
        .input
        .get("provider_package_id")
        .and_then(Value::as_str)
        .unwrap_or("");

    // Check which required permissions are missing from grants
    let granted_values: Vec<&str> = grants
        .iter()
        .filter_map(|g| g.as_str())
        .collect();
    let missing: Vec<Value> = required_permissions
        .iter()
        .filter(|rp| {
            let rp_str = rp.as_str().unwrap_or("");
            !granted_values.contains(&rp_str) && !granted_values.contains(&"*")
        })
        .cloned()
        .collect();

    let allowed = missing.is_empty() && !provider_package_id.is_empty();

    Ok(serde_json::json!({
        "kind": "tool_bridge_permission_preview",
        "allowed": allowed,
        "missing_permissions": missing,
        "required_provider_package_id": if provider_package_id.is_empty() { Value::Null } else { serde_json::json!(provider_package_id) },
        "redaction_state": redaction_state,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

/// invoke_tool: returns invocation_plan with method kernel.capability.invoke.
///
/// If ambiguous or missing provider, returns rejected. Does not actually invoke.
fn invoke_tool(request: &InprocInvocation) -> anyhow::Result<Value> {
    let (secret_findings, redaction_state) = scan_for_raw_secrets(&request.input);
    if !secret_findings.is_empty() {
        return Ok(serde_json::json!({
            "kind": "tool_bridge_invocation_plan",
            "method": "kernel.capability.invoke",
            "capability_id": request.input.get("capability_id").cloned().unwrap_or(Value::Null),
            "provider_package_id": null,
            "status": "rejected",
            "rejection_reason": "raw_secret_detected",
            "redaction_state": "unsafe_blocked",
            "secret_findings": secret_findings,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    let capability_id = request
        .input
        .get("capability_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let provider_package_id = request
        .input
        .get("provider_package_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let ambiguous = request
        .input
        .get("ambiguous")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let candidates = provider_candidates(&request.input);

    if provider_package_id.is_empty() {
        return Ok(serde_json::json!({
            "kind": "tool_bridge_invocation_plan",
            "method": "kernel.capability.invoke",
            "capability_id": capability_id,
            "provider_package_id": null,
            "status": "rejected",
            "rejection_reason": "missing_provider",
            "redaction_state": redaction_state,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    if ambiguous {
        return Ok(serde_json::json!({
            "kind": "tool_bridge_invocation_plan",
            "method": "kernel.capability.invoke",
            "capability_id": capability_id,
            "provider_package_id": provider_package_id,
            "status": "rejected",
            "rejection_reason": "ambiguous_provider",
            "redaction_state": redaction_state,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    if !provider_matches_candidates(provider_package_id, &candidates) {
        return Ok(serde_json::json!({
            "kind": "tool_bridge_invocation_plan",
            "method": "kernel.capability.invoke",
            "capability_id": capability_id,
            "provider_package_id": provider_package_id,
            "status": "rejected",
            "rejection_reason": "provider_not_in_candidates",
            "available_providers": candidates,
            "redaction_state": redaction_state,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    Ok(serde_json::json!({
        "kind": "tool_bridge_invocation_plan",
        "method": "kernel.capability.invoke",
        "capability_id": capability_id,
        "provider_package_id": provider_package_id,
        "status": "plan_ready",
        "requires_user_approval": true,
        "redaction_state": redaction_state,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

/// stream_tool: returns method kernel.capability.stream plan.
/// Provider must be explicit; ambiguous or missing provider is rejected.
fn stream_tool(request: &InprocInvocation) -> anyhow::Result<Value> {
    let (secret_findings, redaction_state) = scan_for_raw_secrets(&request.input);
    if !secret_findings.is_empty() {
        return Ok(serde_json::json!({
            "kind": "tool_bridge_stream_plan",
            "method": "kernel.capability.stream",
            "capability_id": request.input.get("capability_id").cloned().unwrap_or(Value::Null),
            "provider_package_id": null,
            "status": "rejected",
            "rejection_reason": "raw_secret_detected",
            "redaction_state": "unsafe_blocked",
            "secret_findings": secret_findings,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    let capability_id = request
        .input
        .get("capability_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let provider_package_id = request
        .input
        .get("provider_package_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let candidates = provider_candidates(&request.input);

    if provider_package_id.is_empty() {
        return Ok(serde_json::json!({
            "kind": "tool_bridge_stream_plan",
            "method": "kernel.capability.stream",
            "capability_id": capability_id,
            "provider_package_id": null,
            "status": "rejected",
            "rejection_reason": "missing_provider",
            "redaction_state": redaction_state,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    let ambiguous = request
        .input
        .get("ambiguous")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if ambiguous {
        return Ok(serde_json::json!({
            "kind": "tool_bridge_stream_plan",
            "method": "kernel.capability.stream",
            "capability_id": capability_id,
            "provider_package_id": provider_package_id,
            "status": "rejected",
            "rejection_reason": "ambiguous_provider",
            "redaction_state": redaction_state,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    if !provider_matches_candidates(provider_package_id, &candidates) {
        return Ok(serde_json::json!({
            "kind": "tool_bridge_stream_plan",
            "method": "kernel.capability.stream",
            "capability_id": capability_id,
            "provider_package_id": provider_package_id,
            "status": "rejected",
            "rejection_reason": "provider_not_in_candidates",
            "available_providers": candidates,
            "redaction_state": redaction_state,
            "provenance": {
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id
            }
        }));
    }

    Ok(serde_json::json!({
        "kind": "tool_bridge_stream_plan",
        "method": "kernel.capability.stream",
        "capability_id": capability_id,
        "provider_package_id": provider_package_id,
        "status": "plan_ready",
        "requires_user_approval": true,
        "redaction_state": redaction_state,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

/// explain_tool_call: returns an audit-safe summary of a tool call plan.
fn explain_tool_call(request: &InprocInvocation) -> anyhow::Result<Value> {
    let (secret_findings, redaction_state) = scan_for_raw_secrets(&request.input);

    let capability_id = request
        .input
        .get("capability_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let provider_package_id = request
        .input
        .get("provider_package_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let method = request
        .input
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or("kernel.capability.invoke");

    let summary = format!(
        "Tool call explanation: method={}, capability_id={}, provider={}. No raw secrets in output.",
        method,
        capability_id,
        if provider_package_id.is_empty() { "unspecified" } else { provider_package_id }
    );

    Ok(serde_json::json!({
        "kind": "tool_bridge_explanation",
        "summary": summary,
        "method": method,
        "capability_id": capability_id,
        "provider_package_id": if provider_package_id.is_empty() { Value::Null } else { serde_json::json!(provider_package_id) },
        "redaction_state": redaction_state,
        "secret_findings": secret_findings,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

/// echo: package conformance passthrough.
fn echo(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "tool_bridge_echo",
        "input": request.input,
        "provenance": {
            "package_id": request.provider_package_id,
            "capability_id": request.capability_id
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(cap: &str, input: serde_json::Value) -> InprocInvocation {
        InprocInvocation {
            capability_id: format!("official/capability-tool-bridge-lab/{cap}"),
            provider_package_id: PACKAGE_ID.to_string(),
            input,
        }
    }

    #[test]
    fn discover_tools_ambiguous_rejected() {
        let request = make_request(
            "discover_tools",
            serde_json::json!({
                "capabilities": [
                    {
                        "capability_id": "example/echo",
                        "providers": ["official/pkg-a", "thirdparty/pkg-b"]
                    }
                ]
            }),
        );
        let result = discover_tools(&request).unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools[0]["status"], "rejected");
        assert_eq!(tools[0]["ambiguous"], true);
    }

    #[test]
    fn discover_tools_explicit_provider_available() {
        let request = make_request(
            "discover_tools",
            serde_json::json!({
                "capabilities": [
                    {
                        "capability_id": "example/echo",
                        "providers": ["official/pkg-a", "thirdparty/pkg-b"],
                        "provider_package_id": "thirdparty/pkg-b"
                    }
                ]
            }),
        );
        let result = discover_tools(&request).unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools[0]["status"], "available");
        assert_eq!(tools[0]["provider_package_id"], "thirdparty/pkg-b");
    }

    #[test]
    fn discover_tools_explicit_provider_must_match_candidates() {
        let request = make_request(
            "discover_tools",
            serde_json::json!({
                "capabilities": [
                    {
                        "capability_id": "example/echo",
                        "providers": ["official/pkg-a", "thirdparty/pkg-b"],
                        "provider_package_id": "thirdparty/unknown"
                    }
                ]
            }),
        );
        let result = discover_tools(&request).unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools[0]["status"], "rejected");
        assert_eq!(tools[0]["rejection_reason"], "provider_not_in_candidates");
    }

    #[test]
    fn discover_tools_no_official_preference() {
        // When ambiguous, official is NOT preferred
        let request = make_request(
            "discover_tools",
            serde_json::json!({
                "capabilities": [
                    {
                        "capability_id": "example/echo",
                        "providers": ["official/pkg-a", "thirdparty/pkg-b"]
                    }
                ]
            }),
        );
        let result = discover_tools(&request).unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools[0]["status"], "rejected");
        assert_eq!(tools[0]["ambiguous"], true);
        // No auto-selection of official provider
        assert!(tools[0]["provider_package_id"].is_null());
    }

    #[test]
    fn invoke_tool_missing_provider_rejected() {
        let request = make_request(
            "invoke_tool",
            serde_json::json!({
                "capability_id": "example/echo"
            }),
        );
        let result = invoke_tool(&request).unwrap();
        assert_eq!(result["status"], "rejected");
        assert_eq!(result["rejection_reason"], "missing_provider");
    }

    #[test]
    fn invoke_tool_ambiguous_rejected() {
        let request = make_request(
            "invoke_tool",
            serde_json::json!({
                "capability_id": "example/echo",
                "provider_package_id": "official/pkg-a",
                "ambiguous": true
            }),
        );
        let result = invoke_tool(&request).unwrap();
        assert_eq!(result["status"], "rejected");
        assert_eq!(result["rejection_reason"], "ambiguous_provider");
    }

    #[test]
    fn invoke_tool_provider_must_match_candidates() {
        let request = make_request(
            "invoke_tool",
            serde_json::json!({
                "capability_id": "example/echo",
                "provider_package_id": "thirdparty/unknown",
                "providers": ["official/pkg-a", "thirdparty/pkg-b"]
            }),
        );
        let result = invoke_tool(&request).unwrap();
        assert_eq!(result["status"], "rejected");
        assert_eq!(result["rejection_reason"], "provider_not_in_candidates");
    }

    #[test]
    fn invoke_tool_explicit_third_party_works() {
        let request = make_request(
            "invoke_tool",
            serde_json::json!({
                "capability_id": "example/echo",
                "provider_package_id": "thirdparty/my-tool"
            }),
        );
        let result = invoke_tool(&request).unwrap();
        assert_eq!(result["status"], "plan_ready");
        assert_eq!(result["provider_package_id"], "thirdparty/my-tool");
    }

    #[test]
    fn preview_tool_permissions_denied_reports_missing() {
        let request = make_request(
            "preview_tool_permissions",
            serde_json::json!({
                "required_permissions": ["capabilities.invoke"],
                "grants": [],
                "provider_package_id": "official/echo"
            }),
        );
        let result = preview_tool_permissions(&request).unwrap();
        assert_eq!(result["allowed"], false);
        let missing = result["missing_permissions"].as_array().unwrap();
        assert_eq!(missing.len(), 1);
    }

    #[test]
    fn raw_secret_payload_unsafe_blocked() {
        let request = make_request(
            "invoke_tool",
            serde_json::json!({
                "capability_id": "example/echo",
                "provider_package_id": "official/pkg",
                "api_key": "sk-raw-secret-here"
            }),
        );
        let result = invoke_tool(&request).unwrap();
        assert_eq!(result["redaction_state"], "unsafe_blocked");
        assert_eq!(result["status"], "rejected");
    }

    #[test]
    fn stream_tool_missing_provider_rejected() {
        let request = make_request(
            "stream_tool",
            serde_json::json!({
                "capability_id": "example/stream"
            }),
        );
        let result = stream_tool(&request).unwrap();
        assert_eq!(result["status"], "rejected");
        assert_eq!(result["rejection_reason"], "missing_provider");
    }

    #[test]
    fn try_handle_wrong_package_returns_none() {
        let request = InprocInvocation {
            capability_id: "official/capability-tool-bridge-lab/discover_tools".to_string(),
            provider_package_id: "other/package".to_string(),
            input: serde_json::json!({}),
        };
        assert!(try_handle(&request).is_none());
    }
}
