//! Generic/foundation capability handlers shared across official packages.
//!
//! These handlers match on `(provider_package_id, local_capability_name)` pairs,
//! requiring the capability_id to be under the provider_package_id namespace.
//! Only `official/` packages are served by these handlers; non-official packages
//! are rejected to prevent accidental fallback behavior.

use serde_json::Value;

use super::InprocInvocation;

/// Extract the local capability name when `capability_id` is under
/// `provider_package_id` namespace.
///
/// Returns `None` when `capability_id` does not start with
/// `"<provider_package_id>/"`.
///
/// Example: provider `official/asset-lab`, capability `official/asset-lab/preview`
///          => local name `preview`.
fn extract_local_name<'a>(capability_id: &'a str, provider_package_id: &str) -> Option<&'a str> {
    if !capability_id.starts_with(provider_package_id) {
        return None;
    }
    let rest = capability_id.get(provider_package_id.len()..)?;
    if !rest.starts_with('/') {
        return None;
    }
    Some(&rest[1..])
}

pub fn try_handle(request: &InprocInvocation) -> Option<anyhow::Result<Value>> {
    // Only serve official/ packages through the shared handlers.
    if !request.provider_package_id.starts_with("official/") {
        return None;
    }

    let local_name = extract_local_name(&request.capability_id, &request.provider_package_id)?;

    match local_name {
        "echo" => Some(Ok(request.input.clone())),
        "fail" => Some(Err(anyhow::anyhow!("official package-lab requested failure"))),
        "describe" => Some(describe(request)),
        "validate" => Some(validate()),
        "sample" => Some(sample(request)),
        "summarize" => Some(summarize(request)),
        "launch_plan" => Some(launch_plan(request)),
        "permission_preview" => Some(permission_preview(request)),
        "surface_graph" => Some(surface_graph(request)),
        "preview" => Some(preview(request)),
        "diff" => Some(diff(request)),
        "export" => Some(export(request)),
        "import_plan" => Some(import_plan(request)),
        "rebuild_plan" => Some(rebuild_plan(request)),
        "explain_source_events" => Some(explain_source_events(request)),
        "explain" => Some(explain(request)),
        "suggest" => Some(suggest()),
        "draft_branch_change" => Some(draft_branch_change(request)),
        "create_seed" => Some(create_seed(request)),
        "project" => Some(project(request)),
        _ => None,
    }
}

/// Returns an error for unhandled/unknown inproc capabilities.
///
/// Replaces the former permissive `fallback` that returned generic `{"ok": true}`
/// success for any unrecognized capability. Unknown capabilities must now fail
/// loudly so that callers receive a clear error instead of a misleading success.
pub fn unhandled_capability(request: &InprocInvocation) -> anyhow::Result<Value> {
    Err(anyhow::anyhow!(
        "no handler for inproc capability '{}' in package '{}'",
        request.capability_id,
        request.provider_package_id,
    ))
}

fn describe(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "package_id": request.provider_package_id,
        "capability_id": request.capability_id,
        "input_keys": request.input.as_object().map(|object| object.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
    }))
}

fn validate() -> anyhow::Result<Value> {
    Ok(serde_json::json!({"valid": true, "diagnostics": []}))
}

fn sample(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({"sample": request.input.get("schema").cloned().unwrap_or(Value::Null)}))
}

fn summarize(request: &InprocInvocation) -> anyhow::Result<Value> {
    let count = request.input.get("events").and_then(Value::as_array).map(|events| events.len()).unwrap_or(0);
    Ok(serde_json::json!({"event_count": count}))
}

fn launch_plan(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "composition_launch_plan",
        "composition_id": request.input.get("id").cloned().unwrap_or(Value::Null),
        "entry_surface_id": request.input.get("entry_surface_id").cloned().unwrap_or(Value::Null),
        "packages": request.input.get("packages").cloned().unwrap_or_else(|| serde_json::json!([])),
        "steps": ["validate manifest set", "resolve entry surface", "preview required permissions", "open session", "invoke launch capability"],
    }))
}

fn permission_preview(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "composition_permission_preview",
        "required_permissions": request.input.get("required_permissions").cloned().unwrap_or_else(|| serde_json::json!([])),
        "risk": request.input.get("risk").cloned().unwrap_or_else(|| serde_json::json!("medium")),
    }))
}

fn surface_graph(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "composition_surface_graph",
        "entry_surface_id": request.input.get("entry_surface_id").cloned().unwrap_or(Value::Null),
        "surfaces": request.input.get("surfaces").cloned().unwrap_or_else(|| serde_json::json!([])),
        "edges": request.input.get("edges").cloned().unwrap_or_else(|| serde_json::json!([])),
    }))
}

fn preview(request: &InprocInvocation) -> anyhow::Result<Value> {
    let content = request.input.get("content").and_then(Value::as_str).unwrap_or_default();
    Ok(serde_json::json!({
        "kind": "asset_preview",
        "asset_id": request.input.get("asset_id").cloned().unwrap_or(Value::Null),
        "mime": request.input.get("mime").and_then(Value::as_str).unwrap_or("application/octet-stream"),
        "content_length": content.len(),
        "preview": content.chars().take(120).collect::<String>(),
    }))
}

fn diff(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "asset_diff",
        "from": request.input.get("from").cloned().unwrap_or(Value::Null),
        "to": request.input.get("to").cloned().unwrap_or(Value::Null),
        "requires_proposal": true,
    }))
}

fn export(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "asset_export",
        "asset_id": request.input.get("asset_id").cloned().unwrap_or(Value::Null),
        "format": request.input.get("format").and_then(Value::as_str).unwrap_or("json"),
        "content": request.input.get("content").cloned().unwrap_or(Value::Null),
    }))
}

fn import_plan(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "asset_import_plan",
        "requires_user_approval": true,
        "recommended_operation": "asset.put",
        "mime": request.input.get("mime").and_then(Value::as_str).unwrap_or("application/json"),
        "metadata": request.input.get("metadata").cloned().unwrap_or_else(|| serde_json::json!({})),
    }))
}

fn rebuild_plan(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "projection_rebuild_plan",
        "projection_id": request.input.get("projection_id").cloned().unwrap_or(Value::Null),
        "requires_user_approval": true,
        "recommended_operation": "projection.rebuild",
        "source_kind_prefix": request.input.get("source_kind_prefix").cloned().unwrap_or(Value::Null),
    }))
}

fn explain_source_events(request: &InprocInvocation) -> anyhow::Result<Value> {
    let event_count = request.input.get("events").and_then(Value::as_array).map(|events| events.len()).unwrap_or(0);
    Ok(serde_json::json!({
        "kind": "projection_source_events",
        "projection_id": request.input.get("projection_id").cloned().unwrap_or(Value::Null),
        "event_count": event_count,
        "source_kind_prefix": request.input.get("source_kind_prefix").cloned().unwrap_or(Value::Null),
    }))
}

fn explain(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "assistant_explanation",
        "summary": "Assistant package can explain protocol-visible context without private kernel access.",
        "context_keys": request.input.as_object().map(|object| object.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
    }))
}

fn suggest() -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "assistant_suggestions",
        "suggestions": ["inspect events", "fork before changing", "invoke package capability through public protocol"],
    }))
}

fn draft_branch_change(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "assistant_proposal",
        "requires_user_approval": true,
        "recommended_operation": "kernel.session.fork",
        "proposal": request.input,
    }))
}

fn create_seed(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "blank_experience_seed",
        "title": request.input.get("title").and_then(Value::as_str).unwrap_or("Blank Experience"),
        "seed": request.input,
    }))
}

fn project(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "blank_experience_projection",
        "state": request.input,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(provider: &str, cap: &str) -> InprocInvocation {
        InprocInvocation {
            capability_id: cap.to_string(),
            provider_package_id: provider.to_string(),
            input: serde_json::json!({}),
        }
    }

    #[test]
    fn extract_local_name_matches_namespace() {
        assert_eq!(extract_local_name("official/asset-lab/preview", "official/asset-lab"), Some("preview"));
        assert_eq!(extract_local_name("official/package-lab/echo", "official/package-lab"), Some("echo"));
    }

    #[test]
    fn extract_local_name_rejects_wrong_namespace() {
        assert_eq!(extract_local_name("thirdparty/pkg/preview", "official/asset-lab"), None);
        assert_eq!(extract_local_name("official/asset-lab/preview", "official/other"), None);
    }

    #[test]
    fn extract_local_name_rejects_partial_prefix() {
        // "official/asset" is a prefix of "official/asset-lab" but not a valid namespace
        assert_eq!(extract_local_name("official/asset-lab/preview", "official/asset"), None);
    }

    #[test]
    fn try_handle_official_preview() {
        let request = make_request("official/asset-lab", "official/asset-lab/preview");
        let result = try_handle(&request);
        assert!(result.is_some(), "official package preview should be handled");
        let output = result.unwrap().unwrap();
        assert_eq!(output["kind"], "asset_preview");
    }

    #[test]
    fn try_handle_rejects_non_official() {
        let request = make_request("thirdparty/pkg", "thirdparty/pkg/preview");
        assert!(try_handle(&request).is_none(), "non-official package should not be handled by common");
    }

    #[test]
    fn try_handle_rejects_wrong_namespace() {
        let request = make_request("official/other", "official/asset-lab/preview");
        assert!(try_handle(&request).is_none(), "wrong namespace should not be handled");
    }

    #[test]
    fn try_handle_unknown_local_name_returns_none() {
        let request = make_request("official/package-lab", "official/package-lab/nonexistent");
        assert!(try_handle(&request).is_none(), "unknown local name should return None");
    }

    #[test]
    fn unhandled_capability_returns_error() {
        let request = make_request("official/package-lab", "official/package-lab/unknown");
        let result = unhandled_capability(&request);
        assert!(result.is_err(), "unhandled capability should return an error");
    }
}
