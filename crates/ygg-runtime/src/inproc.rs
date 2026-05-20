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
mod experience_observability_lab;
mod experience_runtime_lab;
mod inference_local_lab;
mod inference_playtest_lab;
mod knowledge_lab;
mod memory_lab;
mod model_connector_lab;
mod model_provider_lab;
mod model_routing_lab;
mod persona_lab;
mod pi_agent_runtime_lab;
mod playable_creation_board;
mod playable_seed;
mod projection_lab;
mod project_intake_lab;
pub mod safety;
mod sharing_lab;
mod text_transform_lab;
mod thirdparty_agent_runtime;
mod workspace_lab;

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

/// Dispatch an official package invocation using provider-package indexed dispatch.
///
/// Each `official/*` package is dispatched directly based on `provider_package_id`,
/// falling through to `common::try_handle` when the specific handler returns `None`
/// or when the package_id is an unknown official package.
/// Non-official packages are never served by `common::try_handle`.
fn dispatch_official(request: &InprocInvocation) -> anyhow::Result<Value> {
    // Try the package-specific handler first, then fall through to common
    // namespace-scoped handlers if the specific handler doesn't match.
    let specific_result = match request.provider_package_id.as_str() {
        "official/persona-lab" => persona_lab::try_handle(request),
        "official/knowledge-lab" => knowledge_lab::try_handle(request),
        "official/context-lab" => context_lab::try_handle(request),
        "official/text-transform-lab" => text_transform_lab::try_handle(request),
        "official/model-connector-lab" => model_connector_lab::try_handle(request),
        "official/model-provider-lab" => model_provider_lab::try_handle(request),
        "official/model-routing-lab" => model_routing_lab::try_handle(request),
        "official/pi-agent-runtime-lab" => pi_agent_runtime_lab::try_handle(request),
        "official/agentic-forge-lab" => agentic_forge_lab::try_handle(request),
        // projection-lab /diff must be tried before the generic /diff
        "official/projection-lab" => projection_lab::try_handle(request),
        // playable-seed handlers checked before generic capability suffixes
        "official/playable-seed" => playable_seed::try_handle(request),
        // capability-tool-bridge-lab handlers checked before generic capability suffixes
        "official/capability-tool-bridge-lab" => capability_tool_bridge_lab::try_handle(request),
        "official/inference-local-lab" => inference_local_lab::try_handle(request),
        "official/inference-playtest-lab" => inference_playtest_lab::try_handle(request),
        "official/experience-runtime-lab" => experience_runtime_lab::try_handle(request),
        "official/playable-creation-board" => playable_creation_board::try_handle(request),
        "official/memory-lab" => memory_lab::try_handle(request),
        "official/experience-observability-lab" => experience_observability_lab::try_handle(request),
        "official/sharing-lab" => sharing_lab::try_handle(request),
        "official/project-intake-lab" => project_intake_lab::try_handle(request),
        "official/workspace-lab" => workspace_lab::try_handle(request),
        _ => None,
    };

    if let Some(result) = specific_result {
        return result;
    }

    // Fall through to common namespace-scoped handlers for any official package.
    // Non-official packages are rejected by common::try_handle (it checks the prefix).
    if let Some(result) = common::try_handle(request) {
        return result;
    }

    // No handler matched — fail loudly
    common::unhandled_capability(request)
}

struct OfficialFoundationPackage;

#[async_trait]
impl InprocPackage for OfficialFoundationPackage {
    async fn invoke(&self, request: InprocInvocation) -> anyhow::Result<Value> {
        dispatch_official(&request)
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
