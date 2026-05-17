use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ygg_core::{CapabilityId, PackageId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InprocInvocation {
    pub capability_id: CapabilityId,
    pub provider_package_id: PackageId,
    #[serde(default)]
    pub input: Value,
}

#[async_trait]
pub trait InprocPackage: Send + Sync {
    async fn invoke(&self, request: InprocInvocation) -> anyhow::Result<Value>;
}

#[derive(Clone, Default)]
pub struct InprocPackageCatalog {
    entries: Arc<HashMap<String, Arc<dyn InprocPackage>>>,
}

impl InprocPackageCatalog {
    pub fn with_default_examples() -> Self {
        let mut entries: HashMap<String, Arc<dyn InprocPackage>> = HashMap::new();
        entries.insert(entry_key("example-echo-rust-inproc", "register"), Arc::new(EchoInprocPackage));
        entries.insert(entry_key("example-hook-inproc", "register"), Arc::new(HookInprocPackage));
        entries.insert(entry_key("official-foundation", "register"), Arc::new(OfficialFoundationPackage));
        Self { entries: Arc::new(entries) }
    }

    pub fn lookup(&self, crate_ref: &str, symbol: &str) -> Option<Arc<dyn InprocPackage>> {
        self.entries.get(&entry_key(crate_ref, symbol)).cloned()
    }
}

fn entry_key(crate_ref: &str, symbol: &str) -> String {
    format!("{crate_ref}::{symbol}")
}

struct EchoInprocPackage;

#[async_trait]
impl InprocPackage for EchoInprocPackage {
    async fn invoke(&self, request: InprocInvocation) -> anyhow::Result<Value> {
        Ok(request.input)
    }
}

struct HookInprocPackage;

#[async_trait]
impl InprocPackage for HookInprocPackage {
    async fn invoke(&self, request: InprocInvocation) -> anyhow::Result<Value> {
        if request.capability_id.ends_with("/veto") {
            Ok(serde_json::json!({"decision": "veto", "reason": "hook package veto"}))
        } else if request.capability_id.ends_with("/trace") {
            Ok(serde_json::json!({"decision": "allow", "metadata_patch": {"hook_trace": request.provider_package_id}}))
        } else {
            Ok(serde_json::json!({"decision": "allow"}))
        }
    }
}

struct OfficialFoundationPackage;

#[async_trait]
impl InprocPackage for OfficialFoundationPackage {
    async fn invoke(&self, request: InprocInvocation) -> anyhow::Result<Value> {
        let id = request.capability_id.as_str();
        if id.ends_with("/echo") {
            Ok(request.input)
        } else if id.ends_with("/fail") {
            anyhow::bail!("official package-lab requested failure")
        } else if id.ends_with("/describe") {
            Ok(serde_json::json!({
                "package_id": request.provider_package_id,
                "capability_id": request.capability_id,
                "input_keys": request.input.as_object().map(|object| object.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
            }))
        } else if id.ends_with("/validate") {
            Ok(serde_json::json!({"valid": true, "diagnostics": []}))
        } else if id.ends_with("/sample") {
            Ok(serde_json::json!({"sample": request.input.get("schema").cloned().unwrap_or(Value::Null)}))
        } else if id.ends_with("/summarize") {
            let count = request.input.get("events").and_then(Value::as_array).map(|events| events.len()).unwrap_or(0);
            Ok(serde_json::json!({"event_count": count}))
        } else if id.ends_with("/launch_plan") {
            Ok(serde_json::json!({
                "kind": "composition_launch_plan",
                "composition_id": request.input.get("id").cloned().unwrap_or(Value::Null),
                "entry_surface_id": request.input.get("entry_surface_id").cloned().unwrap_or(Value::Null),
                "packages": request.input.get("packages").cloned().unwrap_or_else(|| serde_json::json!([])),
                "steps": ["validate manifest set", "resolve entry surface", "preview required permissions", "open session", "invoke launch capability"],
            }))
        } else if id.ends_with("/permission_preview") {
            Ok(serde_json::json!({
                "kind": "composition_permission_preview",
                "required_permissions": request.input.get("required_permissions").cloned().unwrap_or_else(|| serde_json::json!([])),
                "risk": request.input.get("risk").cloned().unwrap_or_else(|| serde_json::json!("medium")),
            }))
        } else if id.ends_with("/surface_graph") {
            Ok(serde_json::json!({
                "kind": "composition_surface_graph",
                "entry_surface_id": request.input.get("entry_surface_id").cloned().unwrap_or(Value::Null),
                "surfaces": request.input.get("surfaces").cloned().unwrap_or_else(|| serde_json::json!([])),
                "edges": request.input.get("edges").cloned().unwrap_or_else(|| serde_json::json!([])),
            }))
        } else if id.ends_with("/explain") {
            Ok(serde_json::json!({
                "kind": "assistant_explanation",
                "summary": "Assistant package can explain protocol-visible context without private kernel access.",
                "context_keys": request.input.as_object().map(|object| object.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
            }))
        } else if id.ends_with("/suggest") {
            Ok(serde_json::json!({
                "kind": "assistant_suggestions",
                "suggestions": ["inspect events", "fork before changing", "invoke package capability through public protocol"],
            }))
        } else if id.ends_with("/draft_branch_change") {
            Ok(serde_json::json!({
                "kind": "assistant_proposal",
                "requires_user_approval": true,
                "recommended_operation": "kernel.session.fork",
                "proposal": request.input,
            }))
        } else if id.ends_with("/create_seed") {
            Ok(serde_json::json!({
                "kind": "blank_experience_seed",
                "title": request.input.get("title").and_then(Value::as_str).unwrap_or("Blank Experience"),
                "seed": request.input,
            }))
        } else if id.ends_with("/project") {
            Ok(serde_json::json!({
                "kind": "blank_experience_projection",
                "state": request.input,
            }))
        } else {
            Ok(serde_json::json!({"ok": true, "capability_id": request.capability_id}))
        }
    }
}
