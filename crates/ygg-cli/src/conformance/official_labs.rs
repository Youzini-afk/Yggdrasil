use std::path::PathBuf;

use serde_json::json;
use ygg_runtime::{CapabilityInvocationRequest, ProtocolContext};

use super::fixtures::*;
use crate::commands::manifest;

pub(crate) async fn assistant_lab_proposal() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("packages/official/assistant-lab/manifest.yaml")).await?).await?;
    let assistant = json!({"kind": "assistant", "assistant_id": "assistant/lab", "delegated_user_id": "user/conformance"});
    let assistant_context = ProtocolContext { principal: serde_json::from_value(assistant.clone())?, transport: "conformance".to_string() };
    let denied = runtime
        .call_protocol(
            &assistant_context,
            "kernel.capability.invoke",
            json!({"capability_id": "official/assistant-lab/draft_branch_change", "input": {"change": "try branch"}}),
        )
        .await;
    anyhow::ensure!(denied.is_err(), "assistant package invocation should require grant");
    runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.permission.grant",
            json!({"principal": assistant, "permission": "capabilities.invoke", "scope": "official/assistant-lab"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let proposal = runtime
        .call_protocol(
            &assistant_context,
            "kernel.capability.invoke",
            json!({"capability_id": "official/assistant-lab/draft_branch_change", "input": {"change": "try branch"}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(proposal["output"]["requires_user_approval"] == json!(true), "assistant did not return an approval-gated proposal");
    let surfaces = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.surface.contribution.list", json!({"slot": "assistant_action"}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(surfaces.as_array().map(|items| items.len()).unwrap_or(0) == 1, "assistant surface contribution missing");
    Ok(())
}

pub(crate) async fn composition_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("packages/official/composition-lab/manifest.yaml")).await?).await?;
    let plan = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/composition-lab/launch_plan".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/composition-lab".to_string()),
            version: None,
            input: json!({
                "id": "example/composed-experience",
                "entry_surface_id": "example/composed-experience/entry",
                "packages": ["example/composed-experience"]
            }),
        })
        .await?;
    anyhow::ensure!(plan.output["kind"] == json!("composition_launch_plan"), "composition lab launch_plan returned wrong kind");
    let graph = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/composition-lab/surface_graph".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/composition-lab".to_string()),
            version: None,
            input: json!({"entry_surface_id": "example/composed-experience/entry", "surfaces": [{"slot": "experience_entry"}]}),
        })
        .await?;
    anyhow::ensure!(graph.output["kind"] == json!("composition_surface_graph"), "composition lab surface_graph returned wrong kind");
    Ok(())
}

pub(crate) async fn asset_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("packages/official/asset-lab/manifest.yaml")).await?).await?;
    let preview = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/asset-lab/preview".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/asset-lab".to_string()),
            version: None,
            input: json!({"asset_id": "asset/demo", "mime": "application/json", "content": "{\"hello\":\"world\"}"}),
        })
        .await?;
    anyhow::ensure!(preview.output["kind"] == json!("asset_preview"), "asset lab preview returned wrong kind");
    let import_plan = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/asset-lab/import_plan".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/asset-lab".to_string()),
            version: None,
            input: json!({"mime": "application/json", "metadata": {"source": "conformance"}}),
        })
        .await?;
    anyhow::ensure!(import_plan.output["requires_user_approval"] == json!(true), "asset import plan must require approval");
    Ok(())
}

pub(crate) async fn projection_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("packages/official/projection-lab/manifest.yaml")).await?).await?;
    let plan = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/projection-lab/rebuild_plan".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/projection-lab".to_string()),
            version: None,
            input: json!({"projection_id": "example/projection/state", "source_kind_prefix": "example/projection"}),
        })
        .await?;
    anyhow::ensure!(plan.output["kind"] == json!("projection_rebuild_plan"), "projection lab rebuild_plan returned wrong kind");
    let source = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/projection-lab/explain_source_events".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/projection-lab".to_string()),
            version: None,
            input: json!({"projection_id": "example/projection/state", "events": [{"sequence": 1}], "source_kind_prefix": "example/projection"}),
        })
        .await?;
    anyhow::ensure!(source.output["event_count"] == json!(1), "projection lab source event count mismatch");
    Ok(())
}

pub(crate) async fn playable_seed() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("packages/official/playable-seed/manifest.yaml")).await?).await?;
    let launch = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/playable-seed/launch".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/playable-seed".to_string()),
            version: None,
            input: json!({"title": "Conformance Seed"}),
        })
        .await?;
    anyhow::ensure!(launch.output["kind"] == json!("playable_seed_launch"), "playable seed launch returned wrong kind");
    let render = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/playable-seed/render_payload".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/playable-seed".to_string()),
            version: None,
            input: json!({}),
        })
        .await?;
    anyhow::ensure!(render.output["kind"] == json!("playable_seed_render_payload"), "playable seed render returned wrong kind");
    let proposal = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/playable-seed/propose_change".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/playable-seed".to_string()),
            version: None,
            input: json!({"change": "add one seed block"}),
        })
        .await?;
    anyhow::ensure!(proposal.output["requires_user_approval"] == json!(true), "playable seed change must require approval");
    let surfaces = runtime.list_surface_contributions(Some("experience_entry".to_string())).await;
    let has_entry = surfaces
        .as_array()
        .map(|records| records.iter().any(|record| record["package_id"] == json!("official/playable-seed")))
        .unwrap_or(false);
    anyhow::ensure!(has_entry, "playable seed entry surface missing");
    Ok(())
}

pub(crate) async fn persona_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("packages/official/persona-lab/manifest.yaml")).await?).await?;
    let imported = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/persona-lab/import_profile".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/persona-lab".to_string()),
            version: None,
            input: json!({"source": "conformance", "data": {"spec": "chara_card_v2", "data": {"name": "Mira", "description": "Maps dream cities", "extensions": {"unknown": true}}}}),
        })
        .await?;
    anyhow::ensure!(imported.output["kind"] == json!("persona_profile"), "persona import returned wrong kind");
    anyhow::ensure!(imported.output["core"]["name"] == json!("Mira"), "persona import lost name");
    let fragment = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/persona-lab/render_fragment".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/persona-lab".to_string()),
            version: None,
            input: json!({"profile": imported.output}),
        })
        .await?;
    anyhow::ensure!(fragment.output["kind"] == json!("persona_fragment"), "persona render returned wrong kind");
    anyhow::ensure!(fragment.output.get("provenance").is_some(), "persona render missing provenance");
    Ok(())
}

pub(crate) async fn knowledge_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("packages/official/knowledge-lab/manifest.yaml")).await?).await?;
    let imported = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/knowledge-lab/import_collection".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/knowledge-lab".to_string()),
            version: None,
            input: json!({"format": "worldbook-like", "data": {"name": "Dream City", "entries": {"1": {"key": ["bell"], "content": "Alleys rotate."}}}}),
        })
        .await?;
    anyhow::ensure!(imported.output["kind"] == json!("knowledge_collection"), "knowledge import returned wrong kind");
    let matched = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/knowledge-lab/match_entries".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/knowledge-lab".to_string()),
            version: None,
            input: json!({"query": "the bell rings", "entries": imported.output["entries"]}),
        })
        .await?;
    anyhow::ensure!(matched.output["kind"] == json!("knowledge_match_result"), "knowledge match returned wrong kind");
    anyhow::ensure!(matched.output["matches"].as_array().map(|m| !m.is_empty()).unwrap_or(false), "knowledge match missed keyword");
    let plan = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/knowledge-lab/injection_plan".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/knowledge-lab".to_string()),
            version: None,
            input: json!({"matches": matched.output["matches"]}),
        })
        .await?;
    anyhow::ensure!(plan.output["kind"] == json!("knowledge_injection_plan"), "knowledge plan returned wrong kind");
    anyhow::ensure!(plan.output["plan_only"] == json!(true), "knowledge injection must be plan-only");
    Ok(())
}

pub(crate) async fn context_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("packages/official/context-lab/manifest.yaml")).await?).await?;
    let preview = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/context-lab/assemble_preview".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/context-lab".to_string()),
            version: None,
            input: json!({"budget": 20, "sources": [{"id": "short", "text": "fits"}, {"id": "long", "text": "this source should be omitted by budget"}]}),
        })
        .await?;
    anyhow::ensure!(preview.output["kind"] == json!("context_preview"), "context preview returned wrong kind");
    anyhow::ensure!(preview.output["omitted"].as_array().map(|o| !o.is_empty()).unwrap_or(false), "context preview should report omitted sources");
    let rendered = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/context-lab/render_template".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/context-lab".to_string()),
            version: None,
            input: json!({"template": "Hello {{name}}", "variables": {"name": "Yggdrasil"}}),
        })
        .await?;
    anyhow::ensure!(rendered.output["rendered"] == json!("Hello Yggdrasil"), "context template render failed");
    Ok(())
}

pub(crate) async fn text_transform_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("packages/official/text-transform-lab/manifest.yaml")).await?).await?;
    let preview = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/text-transform-lab/apply_preview".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/text-transform-lab".to_string()),
            version: None,
            input: json!({"text": "Mira whispers now", "rules": [{"id": "quiet", "find": "whispers", "replace": "says quietly"}]}),
        })
        .await?;
    anyhow::ensure!(preview.output["kind"] == json!("text_transform_preview"), "text transform preview returned wrong kind");
    anyhow::ensure!(preview.output["output"] == json!("Mira says quietly now"), "text transform did not apply deterministic replacement");
    anyhow::ensure!(preview.output["trace"].as_array().map(|t| !t.is_empty()).unwrap_or(false), "text transform missing trace");
    let validation = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/text-transform-lab/validate_rules".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/text-transform-lab".to_string()),
            version: None,
            input: json!({"rules": [{"id": "bad"}]}),
        })
        .await?;
    anyhow::ensure!(validation.output["valid"] == json!(false), "invalid transform rule should be reported");
    Ok(())
}

pub(crate) async fn model_connector_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("packages/official/model-connector-lab/manifest.yaml")).await?).await?;
    let families = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/model-connector-lab/describe_families".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/model-connector-lab".to_string()),
            version: None,
            input: json!({}),
        })
        .await?;
    anyhow::ensure!(families.output["kind"] == json!("model_provider_families"), "model connector families wrong kind");
    anyhow::ensure!(families.output["families"].as_array().map(|f| f.len() >= 6).unwrap_or(false), "expected declared provider families");
    let valid = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/model-connector-lab/validate_profile".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/model-connector-lab".to_string()),
            version: None,
            input: json!({"provider_family": "openai-compatible", "base_url": "http://127.0.0.1:11434/v1", "model_id": "fixture", "secret_ref": "env:LOCAL_KEY"}),
        })
        .await?;
    anyhow::ensure!(valid.output["valid"] == json!(true), "valid connector profile rejected");
    anyhow::ensure!(valid.output["verification_level"] == json!("not_verified"), "connector Alpha must not claim live verification");
    let raw_secret = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/model-connector-lab/validate_profile".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/model-connector-lab".to_string()),
            version: None,
            input: json!({"provider_family": "openai", "model_id": "fixture", "api_key": "sk-should-not-be-accepted"}),
        })
        .await?;
    anyhow::ensure!(raw_secret.output["valid"] == json!(false), "raw secret profile should be invalid");
    let plan = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/model-connector-lab/discovery_plan".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/model-connector-lab".to_string()),
            version: None,
            input: json!({"provider_family": "google"}),
        })
        .await?;
    anyhow::ensure!(plan.output["network_performed"] == json!(false), "discovery plan must not perform network");
    Ok(())
}

pub(crate) async fn model_routing_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("packages/official/model-routing-lab/manifest.yaml")).await?).await?;
    let bindings = json!([
        {"profile_id": "profile/low", "priority": 1, "fallback": true},
        {"profile_id": "profile/high", "priority": 10, "fallback": false}
    ]);
    let resolved = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/model-routing-lab/resolve_binding".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/model-routing-lab".to_string()),
            version: None,
            input: json!({"consumer_slot": "play.primary", "bindings": bindings}),
        })
        .await?;
    anyhow::ensure!(resolved.output["kind"] == json!("model_route_resolution"), "model routing resolution wrong kind");
    anyhow::ensure!(resolved.output["selected"]["profile_id"] == json!("profile/high"), "model routing did not select highest priority");
    anyhow::ensure!(resolved.output["inference_performed"] == json!(false), "model routing must not invoke inference");
    let params = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/model-routing-lab/params_normalize".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/model-routing-lab".to_string()),
            version: None,
            input: json!({"params": {"temperature": 0.2, "max_tokens": 128, "provider_options": {"openai": {"reasoning_effort": "low"}}}}),
        })
        .await?;
    anyhow::ensure!(params.output["kind"] == json!("model_params_normalized"), "model params normalization wrong kind");
    anyhow::ensure!(params.output["params"]["max_output_tokens"] == json!(128), "model params did not normalize max_tokens");
    anyhow::ensure!(params.output["provider_specific_namespaced"] == json!(true), "provider-specific params must stay namespaced");
    Ok(())
}
