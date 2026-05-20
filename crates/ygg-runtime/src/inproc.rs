use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ygg_core::{CapabilityId, PackageId};

mod agentic_forge_lab;
mod capability_tool_bridge_lab;
mod common;
mod context_lab;
mod experience_runtime_lab;
mod inference_local_lab;
mod inference_playtest_lab;
mod knowledge_lab;
mod model_connector_lab;
mod model_provider_lab;
mod model_routing_lab;
mod persona_lab;
mod pi_agent_runtime_lab;
mod playable_creation_board;
mod playable_seed;
mod projection_lab;
mod text_transform_lab;
mod thirdparty_agent_runtime;

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
        entries.insert(
            entry_key("example-echo-rust-inproc", "register"),
            Arc::new(EchoInprocPackage),
        );
        entries.insert(
            entry_key("example-hook-inproc", "register"),
            Arc::new(HookInprocPackage),
        );
        entries.insert(
            entry_key("official-foundation", "register"),
            Arc::new(OfficialFoundationPackage),
        );
        entries.insert(
            entry_key("example-thirdparty-agent-runtime", "register"),
            Arc::new(ThirdpartyAgentRuntimePackage),
        );
        Self {
            entries: Arc::new(entries),
        }
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
            Ok(
                serde_json::json!({"decision": "allow", "metadata_patch": {"hook_trace": request.provider_package_id}}),
            )
        } else {
            Ok(serde_json::json!({"decision": "allow"}))
        }
    }
}

struct OfficialFoundationPackage;

#[async_trait]
impl InprocPackage for OfficialFoundationPackage {
    async fn invoke(&self, request: InprocInvocation) -> anyhow::Result<Value> {
        // Try package-specific handlers first (order preserved from original)
        if let Some(result) = persona_lab::try_handle(&request) {
            return result;
        }
        if let Some(result) = knowledge_lab::try_handle(&request) {
            return result;
        }
        if let Some(result) = context_lab::try_handle(&request) {
            return result;
        }
        if let Some(result) = text_transform_lab::try_handle(&request) {
            return result;
        }
        if let Some(result) = model_connector_lab::try_handle(&request) {
            return result;
        }
        if let Some(result) = model_provider_lab::try_handle(&request) {
            return result;
        }
        if let Some(result) = model_routing_lab::try_handle(&request) {
            return result;
        }
        if let Some(result) = pi_agent_runtime_lab::try_handle(&request) {
            return result;
        }
        // agentic-forge-lab handlers: package-owned run lifecycle / working state / plan graph
        if let Some(result) = agentic_forge_lab::try_handle(&request) {
            return result;
        }
        // projection-lab /diff must be tried before the generic /diff
        if let Some(result) = projection_lab::try_handle(&request) {
            return result;
        }
        // playable-seed handlers checked before generic capability suffixes
        if let Some(result) = playable_seed::try_handle(&request) {
            return result;
        }
        // capability-tool-bridge-lab handlers checked before generic capability suffixes
        if let Some(result) = capability_tool_bridge_lab::try_handle(&request) {
            return result;
        }
        // inference-local-lab handlers: deterministic non-HTTP fake inference provider proof
        if let Some(result) = inference_local_lab::try_handle(&request) {
            return result;
        }
        // inference-playtest-lab handlers: Ygg-native inference proposal vertical slice
        if let Some(result) = inference_playtest_lab::try_handle(&request) {
            return result;
        }
        // experience-runtime-lab handlers: Experience Beta 0 thin runtime contract
        if let Some(result) = experience_runtime_lab::try_handle(&request) {
            return result;
        }
        // playable-creation-board handlers: Experience Beta 1 first real playable vertical slice
        if let Some(result) = playable_creation_board::try_handle(&request) {
            return result;
        }
        // Package-aware generic capability handlers (namespace-scoped matching)
        if let Some(result) = common::try_handle(&request) {
            return result;
        }
        // No handler matched — fail loudly instead of returning permissive success
        common::unhandled_capability(&request)
    }
}

struct ThirdpartyAgentRuntimePackage;

#[async_trait]
impl InprocPackage for ThirdpartyAgentRuntimePackage {
    async fn invoke(&self, request: InprocInvocation) -> anyhow::Result<Value> {
        if let Some(result) = thirdparty_agent_runtime::try_handle(&request) {
            return result;
        }
        common::unhandled_capability(&request)
    }
}
