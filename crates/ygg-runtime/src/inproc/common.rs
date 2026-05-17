//! Generic/foundation capability handlers shared across official packages.
//!
//! These handlers match on `capability_id` suffix only, without requiring
//! a specific `provider_package_id`. They serve as the fallback layer
//! after all package-specific handlers have been tried.

use serde_json::Value;

use super::InprocInvocation;

pub fn try_handle(request: &InprocInvocation) -> Option<anyhow::Result<Value>> {
    let id = request.capability_id.as_str();
    if id.ends_with("/echo") {
        Some(Ok(request.input.clone()))
    } else if id.ends_with("/fail") {
        Some(Err(anyhow::anyhow!("official package-lab requested failure")))
    } else if id.ends_with("/describe") {
        Some(describe(request))
    } else if id.ends_with("/validate") {
        Some(validate())
    } else if id.ends_with("/sample") {
        Some(sample(request))
    } else if id.ends_with("/summarize") {
        Some(summarize(request))
    } else if id.ends_with("/launch_plan") {
        Some(launch_plan(request))
    } else if id.ends_with("/permission_preview") {
        Some(permission_preview(request))
    } else if id.ends_with("/surface_graph") {
        Some(surface_graph(request))
    } else if id.ends_with("/preview") {
        Some(preview(request))
    } else if id.ends_with("/diff") {
        Some(diff(request))
    } else if id.ends_with("/export") {
        Some(export(request))
    } else if id.ends_with("/import_plan") {
        Some(import_plan(request))
    } else if id.ends_with("/rebuild_plan") {
        Some(rebuild_plan(request))
    } else if id.ends_with("/explain_source_events") {
        Some(explain_source_events(request))
    } else if id.ends_with("/explain") {
        Some(explain(request))
    } else if id.ends_with("/suggest") {
        Some(suggest())
    } else if id.ends_with("/draft_branch_change") {
        Some(draft_branch_change(request))
    } else if id.ends_with("/create_seed") {
        Some(create_seed(request))
    } else if id.ends_with("/project") {
        Some(project(request))
    } else {
        None
    }
}

pub fn fallback(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({"ok": true, "capability_id": request.capability_id}))
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
