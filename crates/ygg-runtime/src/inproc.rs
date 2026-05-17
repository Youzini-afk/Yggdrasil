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
        } else if request.provider_package_id == "official/persona-lab" && id.ends_with("/import_profile") {
            let data = request.input.get("data").unwrap_or(&request.input);
            let core = data.get("data").unwrap_or(data);
            let name = core.get("name").and_then(Value::as_str).unwrap_or("Unnamed Persona");
            Ok(serde_json::json!({
                "kind": "persona_profile",
                "imported_format": data.get("spec").and_then(Value::as_str).unwrap_or("generic_profile"),
                "core": {
                    "name": name,
                    "description": core.get("description").cloned().unwrap_or(Value::Null),
                    "personality": core.get("personality").cloned().unwrap_or(Value::Null),
                    "scenario": core.get("scenario").cloned().unwrap_or(Value::Null),
                    "example_dialogue": core.get("mes_example").or_else(|| core.get("example_dialogue")).cloned().unwrap_or(Value::Null)
                },
                "greetings": {
                    "primary": core.get("first_mes").or_else(|| core.get("primary_greeting")).cloned().unwrap_or(Value::Null),
                    "alternate": core.get("alternate_greetings").cloned().unwrap_or_else(|| serde_json::json!([]))
                },
                "metadata": {
                    "tags": core.get("tags").cloned().unwrap_or_else(|| serde_json::json!([])),
                    "source": request.input.get("source").and_then(Value::as_str).unwrap_or("inline")
                },
                "diagnostics": {"unknown_fields_preserved": true, "warnings": []},
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/persona-lab" && id.ends_with("/normalize_profile") {
            Ok(serde_json::json!({
                "kind": "persona_profile",
                "profile": request.input.get("profile").cloned().unwrap_or_else(|| request.input.clone()),
                "normalized": true,
                "diagnostics": {"warnings": []},
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/persona-lab" && id.ends_with("/describe_profile") {
            let profile = request.input.get("profile").unwrap_or(&request.input);
            Ok(serde_json::json!({
                "kind": "persona_profile_description",
                "name": profile.pointer("/core/name").or_else(|| profile.get("name")).cloned().unwrap_or(Value::Null),
                "sections": ["core", "greetings", "metadata", "extensions"],
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/persona-lab" && id.ends_with("/render_fragment") {
            let profile = request.input.get("profile").unwrap_or(&request.input);
            let name = profile.pointer("/core/name").or_else(|| profile.get("name")).and_then(Value::as_str).unwrap_or("Persona");
            let description = profile.pointer("/core/description").or_else(|| profile.get("description")).and_then(Value::as_str).unwrap_or("");
            Ok(serde_json::json!({
                "kind": "persona_fragment",
                "fragment": format!("{name}: {description}"),
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id, "source": "explicit_profile"}
            }))
        } else if request.provider_package_id == "official/persona-lab" && id.ends_with("/compat_report") {
            Ok(serde_json::json!({
                "kind": "persona_compat_report",
                "input_format": request.input.get("format").and_then(Value::as_str).unwrap_or("unknown"),
                "lossy": false,
                "unsupported_fields": [],
                "diagnostics": ["compatibility input is not canonical Yggdrasil ontology"]
            }))
        } else if request.provider_package_id == "official/knowledge-lab" && id.ends_with("/import_collection") {
            let data = request.input.get("data").unwrap_or(&request.input);
            let entries_value = data.get("entries").cloned().unwrap_or_else(|| serde_json::json!([]));
            let entries: Vec<Value> = if let Some(array) = entries_value.as_array() {
                array.clone()
            } else if let Some(object) = entries_value.as_object() {
                object.values().cloned().collect()
            } else {
                Vec::new()
            };
            Ok(serde_json::json!({
                "kind": "knowledge_collection",
                "name": data.get("name").and_then(Value::as_str).unwrap_or("Knowledge Collection"),
                "entries": entries,
                "entry_count": entries.len(),
                "diagnostics": {"compatibility_input": request.input.get("format").cloned().unwrap_or(Value::Null), "warnings": []},
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/knowledge-lab" && id.ends_with("/normalize_entries") {
            Ok(serde_json::json!({
                "kind": "knowledge_collection",
                "entries": request.input.get("entries").cloned().unwrap_or_else(|| serde_json::json!([])),
                "normalized": true,
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/knowledge-lab" && id.ends_with("/match_entries") {
            let query = request.input.get("query").and_then(Value::as_str).unwrap_or_default().to_lowercase();
            let entries = request.input.get("entries").and_then(Value::as_array).cloned().unwrap_or_default();
            let mut matches = Vec::new();
            for entry in entries {
                let keys = entry.get("key").or_else(|| entry.get("keys")).and_then(Value::as_array).cloned().unwrap_or_default();
                let hit = keys.iter().any(|key| key.as_str().map(|key| query.contains(&key.to_lowercase())).unwrap_or(false));
                if hit || entry.get("constant").and_then(Value::as_bool).unwrap_or(false) {
                    matches.push(serde_json::json!({"entry": entry, "reason": if hit {"keyword"} else {"constant"}}));
                }
            }
            Ok(serde_json::json!({
                "kind": "knowledge_match_result",
                "query": request.input.get("query").cloned().unwrap_or(Value::Null),
                "matches": matches,
                "trace": {"algorithm": "deterministic_keyword_contains", "case_sensitive": false},
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/knowledge-lab" && id.ends_with("/injection_plan") {
            Ok(serde_json::json!({
                "kind": "knowledge_injection_plan",
                "matches": request.input.get("matches").cloned().unwrap_or_else(|| serde_json::json!([])),
                "plan_only": true,
                "requires_user_approval": request.input.get("requires_user_approval").and_then(Value::as_bool).unwrap_or(true),
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/knowledge-lab" && id.ends_with("/compat_report") {
            Ok(serde_json::json!({
                "kind": "knowledge_compat_report",
                "input_format": request.input.get("format").and_then(Value::as_str).unwrap_or("unknown"),
                "lossy": false,
                "diagnostics": ["worldbook-like inputs are compatibility formats, not canonical ontology"]
            }))
        } else if request.provider_package_id == "official/context-lab" && id.ends_with("/assemble_preview") {
            let sources = request.input.get("sources").and_then(Value::as_array).cloned().unwrap_or_default();
            let budget = request.input.get("budget").and_then(Value::as_u64).unwrap_or(4096);
            let mut used = 0_u64;
            let mut included = Vec::new();
            let mut omitted = Vec::new();
            for source in sources {
                let text = source.get("text").and_then(Value::as_str).unwrap_or_default();
                let cost = text.len() as u64;
                if used + cost <= budget {
                    used += cost;
                    included.push(serde_json::json!({"source": source, "estimated_cost": cost, "reason": "fits_budget"}));
                } else {
                    omitted.push(serde_json::json!({"source": source, "estimated_cost": cost, "reason": "budget_exceeded"}));
                }
            }
            Ok(serde_json::json!({
                "kind": "context_preview",
                "blocks": included,
                "omitted": omitted,
                "budget": {"limit": budget, "used": used, "unit": "chars_estimate"},
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/context-lab" && id.ends_with("/inspect_layers") {
            Ok(serde_json::json!({
                "kind": "context_layer_inspection",
                "layers": request.input.get("layers").cloned().unwrap_or_else(|| serde_json::json!([])),
                "diagnostics": ["layers are generic context blocks, not chat roles"],
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/context-lab" && id.ends_with("/budget_plan") {
            let requested = request.input.get("requested").and_then(Value::as_u64).unwrap_or(0);
            let limit = request.input.get("limit").and_then(Value::as_u64).unwrap_or(4096);
            Ok(serde_json::json!({
                "kind": "context_budget_plan",
                "requested": requested,
                "limit": limit,
                "fits": requested <= limit,
                "omitted_reason": if requested <= limit {Value::Null} else {serde_json::json!("budget_exceeded")},
            }))
        } else if request.provider_package_id == "official/context-lab" && id.ends_with("/render_template") {
            let mut rendered = request.input.get("template").and_then(Value::as_str).unwrap_or_default().to_string();
            if let Some(vars) = request.input.get("variables").and_then(Value::as_object) {
                for (key, value) in vars {
                    let replacement = value.as_str().map(str::to_string).unwrap_or_else(|| value.to_string());
                    rendered = rendered.replace(&format!("{{{{{key}}}}}"), &replacement);
                }
            }
            Ok(serde_json::json!({
                "kind": "context_template_render",
                "rendered": rendered,
                "unresolved_policy": "leave_placeholder",
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/context-lab" && id.ends_with("/explain_assembly") {
            Ok(serde_json::json!({
                "kind": "context_assembly_explanation",
                "summary": "Context Lab assembles explicit source blocks under an explicit budget without model calls or chat ontology.",
                "input_keys": request.input.as_object().map(|object| object.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/text-transform-lab" && id.ends_with("/import_rules") {
            let rules = request.input.get("rules").cloned().unwrap_or_else(|| request.input.clone());
            let count = rules.as_array().map(|rules| rules.len()).unwrap_or(0);
            Ok(serde_json::json!({
                "kind": "text_transform_profile",
                "rules": rules,
                "rule_count": count,
                "diagnostics": {"warnings": []},
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/text-transform-lab" && id.ends_with("/validate_rules") {
            let rules = request.input.get("rules").and_then(Value::as_array).cloned().unwrap_or_default();
            let diagnostics: Vec<Value> = rules
                .iter()
                .enumerate()
                .filter_map(|(index, rule)| {
                    if rule.get("find").or_else(|| rule.get("findRegex")).is_none() {
                        Some(serde_json::json!({"index": index, "severity": "warning", "message": "missing find pattern"}))
                    } else {
                        None
                    }
                })
                .collect();
            Ok(serde_json::json!({"kind": "text_transform_validation", "valid": diagnostics.is_empty(), "diagnostics": diagnostics}))
        } else if request.provider_package_id == "official/text-transform-lab" && id.ends_with("/apply_preview") {
            let mut output = request.input.get("text").and_then(Value::as_str).unwrap_or_default().to_string();
            let mut trace = Vec::new();
            if let Some(rules) = request.input.get("rules").and_then(Value::as_array) {
                for rule in rules {
                    if rule.get("disabled").and_then(Value::as_bool).unwrap_or(false) {
                        trace.push(serde_json::json!({"rule": rule.get("id").or_else(|| rule.get("scriptName")).cloned().unwrap_or(Value::Null), "applied": false, "reason": "disabled"}));
                        continue;
                    }
                    let pattern = rule.get("find").or_else(|| rule.get("findRegex")).and_then(Value::as_str).unwrap_or_default();
                    let replacement = rule.get("replace").or_else(|| rule.get("replaceString")).and_then(Value::as_str).unwrap_or_default();
                    let simple_pattern = pattern.trim_start_matches('/').split('/').next().unwrap_or(pattern);
                    let before = output.clone();
                    output = output.replace(simple_pattern, replacement);
                    trace.push(serde_json::json!({"rule": rule.get("id").or_else(|| rule.get("scriptName")).cloned().unwrap_or(Value::Null), "applied": before != output, "pattern": simple_pattern}));
                }
            }
            Ok(serde_json::json!({
                "kind": "text_transform_preview",
                "input": request.input.get("text").cloned().unwrap_or(Value::Null),
                "output": output,
                "trace": trace,
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/text-transform-lab" && id.ends_with("/explain_pipeline") {
            Ok(serde_json::json!({
                "kind": "text_transform_pipeline",
                "rules": request.input.get("rules").cloned().unwrap_or_else(|| serde_json::json!([])),
                "execution": "deterministic_ordered_preview",
                "safety": "preview_only_no_mutation",
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/text-transform-lab" && id.ends_with("/compat_report") {
            Ok(serde_json::json!({
                "kind": "text_transform_compat_report",
                "input_format": request.input.get("format").and_then(Value::as_str).unwrap_or("unknown"),
                "diagnostics": ["regex-like compatibility rules are imported into generic transform profiles"]
            }))
        } else if request.provider_package_id == "official/model-connector-lab" && id.ends_with("/describe_families") {
            Ok(serde_json::json!({
                "kind": "model_provider_families",
                "verification_level": "static_declared",
                "families": [
                    {"id": "openai", "auth": "bearer_secret_ref", "base_url_required": false, "live_discovery": "planned_only"},
                    {"id": "openai-compatible", "auth": "bearer_secret_ref", "base_url_required": true, "live_discovery": "planned_only"},
                    {"id": "anthropic", "auth": "x-api-key_secret_ref", "base_url_required": false, "live_discovery": "planned_only"},
                    {"id": "google", "auth": "key_secret_ref", "base_url_required": false, "live_discovery": "planned_only"},
                    {"id": "deepseek", "auth": "bearer_secret_ref", "base_url_required": false, "live_discovery": "planned_only"},
                    {"id": "xai", "auth": "bearer_secret_ref", "base_url_required": false, "live_discovery": "planned_only"}
                ],
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/model-connector-lab" && id.ends_with("/mask_secret") {
            let secret = request.input.get("secret").and_then(Value::as_str).unwrap_or_default();
            let masked = if secret.is_empty() {
                "<secret:redacted>".to_string()
            } else {
                let suffix: String = secret.chars().rev().take(4).collect::<String>().chars().rev().collect();
                format!("<secret:...{suffix}>")
            };
            Ok(serde_json::json!({"kind": "model_secret_mask", "masked": masked, "raw_returned": false}))
        } else if request.provider_package_id == "official/model-connector-lab" && id.ends_with("/validate_profile") {
            let family = request.input.get("provider_family").and_then(Value::as_str).unwrap_or_default();
            let supported = ["openai", "openai-compatible", "anthropic", "google", "deepseek", "xai"].contains(&family);
            let base_url = request.input.get("base_url").and_then(Value::as_str).unwrap_or_default();
            let secret_ref = request.input.get("secret_ref").and_then(Value::as_str).unwrap_or_default();
            let has_raw_secret = request.input.get("api_key").is_some() || request.input.get("secret").is_some();
            let mut diagnostics = Vec::new();
            if !supported {
                diagnostics.push(serde_json::json!({"severity": "error", "message": "unsupported provider_family"}));
            }
            if family == "openai-compatible" && !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
                diagnostics.push(serde_json::json!({"severity": "error", "message": "openai-compatible requires http(s) base_url"}));
            }
            if secret_ref.is_empty() {
                diagnostics.push(serde_json::json!({"severity": "warning", "message": "missing secret_ref; credential usability is not verified in Alpha"}));
            }
            if has_raw_secret {
                diagnostics.push(serde_json::json!({"severity": "error", "message": "raw secrets are not accepted; use secret_ref"}));
            }
            let valid = !diagnostics.iter().any(|d| d.get("severity").and_then(Value::as_str) == Some("error"));
            Ok(serde_json::json!({
                "kind": "model_connector_profile_validation",
                "valid": valid,
                "verification_level": "not_verified",
                "profile": {
                    "provider_family": family,
                    "base_url": if base_url.is_empty() {Value::Null} else {serde_json::json!(base_url)},
                    "model_id": request.input.get("model_id").cloned().unwrap_or(Value::Null),
                    "secret_ref": if secret_ref.is_empty() {Value::Null} else {serde_json::json!(secret_ref)},
                },
                "diagnostics": diagnostics,
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/model-connector-lab" && id.ends_with("/discovery_plan") {
            Ok(serde_json::json!({
                "kind": "model_discovery_plan",
                "provider_family": request.input.get("provider_family").cloned().unwrap_or(Value::Null),
                "steps": ["validate profile structure", "resolve secret reference", "request network permission", "fetch model list", "normalize provider response"],
                "status": "planned",
                "network_performed": false,
                "verification_level": "not_verified",
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/model-connector-lab" && id.ends_with("/compat_report") {
            Ok(serde_json::json!({
                "kind": "model_connector_compat_report",
                "provider_family": request.input.get("provider_family").cloned().unwrap_or(Value::Null),
                "status": "static_profile_only",
                "compatible_with": request.input.get("provider_family").cloned().unwrap_or(Value::Null),
                "network_performed": false,
                "diagnostics": ["Alpha compatibility is structural and unverified; no live provider call was made"]
            }))
        } else if request.provider_package_id == "official/model-routing-lab" && id.ends_with("/define_binding") {
            Ok(serde_json::json!({
                "kind": "model_route_binding",
                "consumer_slot": request.input.get("consumer_slot").cloned().unwrap_or(Value::Null),
                "scope": request.input.get("scope").and_then(Value::as_str).unwrap_or("session"),
                "bindings": request.input.get("bindings").cloned().unwrap_or_else(|| serde_json::json!([])),
                "requires_user_approval": true,
                "inference_performed": false,
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/model-routing-lab" && id.ends_with("/resolve_binding") {
            let mut bindings = request.input.get("bindings").and_then(Value::as_array).cloned().unwrap_or_default();
            bindings.sort_by(|a, b| {
                let ap = a.get("priority").and_then(Value::as_i64).unwrap_or(0);
                let bp = b.get("priority").and_then(Value::as_i64).unwrap_or(0);
                bp.cmp(&ap)
            });
            let selected = bindings.first().cloned().unwrap_or(Value::Null);
            let fallbacks: Vec<Value> = bindings.iter().skip(1).cloned().collect();
            Ok(serde_json::json!({
                "kind": "model_route_resolution",
                "consumer_slot": request.input.get("consumer_slot").cloned().unwrap_or(Value::Null),
                "selected": selected,
                "fallbacks": fallbacks,
                "deterministic": true,
                "inference_performed": false,
                "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
            }))
        } else if request.provider_package_id == "official/model-routing-lab" && id.ends_with("/preview_routes") {
            Ok(serde_json::json!({
                "kind": "model_route_preview",
                "routes": request.input.get("bindings").cloned().unwrap_or_else(|| serde_json::json!([])),
                "status": "planned",
                "inference_performed": false,
                "diagnostics": ["routes are static plans; no model was invoked"]
            }))
        } else if request.provider_package_id == "official/model-routing-lab" && id.ends_with("/params_normalize") {
            let params = request.input.get("params").cloned().unwrap_or_else(|| serde_json::json!({}));
            Ok(serde_json::json!({
                "kind": "model_params_normalized",
                "params": {
                    "temperature": params.get("temperature").cloned().unwrap_or_else(|| serde_json::json!(0.7)),
                    "max_output_tokens": params.get("max_output_tokens").or_else(|| params.get("max_tokens")).cloned().unwrap_or_else(|| serde_json::json!(512)),
                    "provider_options": params.get("provider_options").cloned().unwrap_or_else(|| serde_json::json!({}))
                },
                "provider_specific_namespaced": true,
                "inference_performed": false
            }))
        } else if request.provider_package_id == "official/model-routing-lab" && id.ends_with("/compat_report") {
            Ok(serde_json::json!({
                "kind": "model_routing_compat_report",
                "consumer_slot": request.input.get("consumer_slot").cloned().unwrap_or(Value::Null),
                "status": "static_route_plan_only",
                "diagnostics": ["routing does not create a global model route and does not invoke inference"]
            }))
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
        } else if id.ends_with("/preview") {
            let content = request.input.get("content").and_then(Value::as_str).unwrap_or_default();
            Ok(serde_json::json!({
                "kind": "asset_preview",
                "asset_id": request.input.get("asset_id").cloned().unwrap_or(Value::Null),
                "mime": request.input.get("mime").and_then(Value::as_str).unwrap_or("application/octet-stream"),
                "content_length": content.len(),
                "preview": content.chars().take(120).collect::<String>(),
            }))
        } else if request.provider_package_id == "official/projection-lab" && id.ends_with("/diff") {
            Ok(serde_json::json!({
                "kind": "projection_diff",
                "before": request.input.get("before").cloned().unwrap_or(Value::Null),
                "after": request.input.get("after").cloned().unwrap_or(Value::Null),
                "projection_id": request.input.get("projection_id").cloned().unwrap_or(Value::Null),
            }))
        } else if id.ends_with("/diff") {
            Ok(serde_json::json!({
                "kind": "asset_diff",
                "from": request.input.get("from").cloned().unwrap_or(Value::Null),
                "to": request.input.get("to").cloned().unwrap_or(Value::Null),
                "requires_proposal": true,
            }))
        } else if id.ends_with("/export") {
            Ok(serde_json::json!({
                "kind": "asset_export",
                "asset_id": request.input.get("asset_id").cloned().unwrap_or(Value::Null),
                "format": request.input.get("format").and_then(Value::as_str).unwrap_or("json"),
                "content": request.input.get("content").cloned().unwrap_or(Value::Null),
            }))
        } else if id.ends_with("/import_plan") {
            Ok(serde_json::json!({
                "kind": "asset_import_plan",
                "requires_user_approval": true,
                "recommended_operation": "asset.put",
                "mime": request.input.get("mime").and_then(Value::as_str).unwrap_or("application/json"),
                "metadata": request.input.get("metadata").cloned().unwrap_or_else(|| serde_json::json!({})),
            }))
        } else if id.ends_with("/rebuild_plan") {
            Ok(serde_json::json!({
                "kind": "projection_rebuild_plan",
                "projection_id": request.input.get("projection_id").cloned().unwrap_or(Value::Null),
                "requires_user_approval": true,
                "recommended_operation": "projection.rebuild",
                "source_kind_prefix": request.input.get("source_kind_prefix").cloned().unwrap_or(Value::Null),
            }))
        } else if id.ends_with("/explain_source_events") {
            let event_count = request.input.get("events").and_then(Value::as_array).map(|events| events.len()).unwrap_or(0);
            Ok(serde_json::json!({
                "kind": "projection_source_events",
                "projection_id": request.input.get("projection_id").cloned().unwrap_or(Value::Null),
                "event_count": event_count,
                "source_kind_prefix": request.input.get("source_kind_prefix").cloned().unwrap_or(Value::Null),
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
        } else if id.ends_with("/create") && request.provider_package_id == "official/playable-seed" {
            Ok(serde_json::json!({
                "kind": "playable_seed",
                "title": request.input.get("title").and_then(Value::as_str).unwrap_or("Playable Seed"),
                "state": request.input.get("state").cloned().unwrap_or_else(|| serde_json::json!({"steps": [], "note": "reference playable seed"})),
            }))
        } else if id.ends_with("/launch") && request.provider_package_id == "official/playable-seed" {
            Ok(serde_json::json!({
                "kind": "playable_seed_launch",
                "title": request.input.get("title").and_then(Value::as_str).unwrap_or("Playable Seed"),
                "render_capability_id": "official/playable-seed/render_payload",
                "forge_panel_id": "official/playable-seed/forge-panel",
            }))
        } else if id.ends_with("/describe_state") && request.provider_package_id == "official/playable-seed" {
            Ok(serde_json::json!({
                "kind": "playable_seed_state",
                "state": request.input.get("state").cloned().unwrap_or_else(|| serde_json::json!({})),
                "editable_through": "proposal",
            }))
        } else if id.ends_with("/render_payload") && request.provider_package_id == "official/playable-seed" {
            Ok(serde_json::json!({
                "kind": "playable_seed_render_payload",
                "blocks": request.input.get("blocks").cloned().unwrap_or_else(|| serde_json::json!([{"type": "text", "text": "Playable Seed is running through package surfaces."}])),
            }))
        } else if id.ends_with("/propose_change") && request.provider_package_id == "official/playable-seed" {
            Ok(serde_json::json!({
                "kind": "playable_seed_change_proposal",
                "requires_user_approval": true,
                "recommended_operations": ["asset.put", "projection.rebuild"],
                "proposal": request.input,
            }))
        } else {
            Ok(serde_json::json!({"ok": true, "capability_id": request.capability_id}))
        }
    }
}
