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
                "packages": ["example/composed-experience"],
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

/// Test composition-lab diagnostics output with v2 fields (capabilities, permissions, replacements, compatibility).
pub(crate) async fn composition_lab_diagnostics() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("packages/official/composition-lab/manifest.yaml")).await?).await?;

    // launch_plan with v2 fields
    let plan = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/composition-lab/launch_plan".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/composition-lab".to_string()),
            version: None,
            input: json!({
                "id": "example/diag-comp",
                "entry_surface_id": "example/diag-comp/entry",
                "packages": ["example/diag-comp"],
                "required_capabilities": ["example/diag-comp/echo"],
                "optional_packages": ["example/optional-extra"],
                "permission_expectations": ["capabilities.invoke"],
                "replacement_candidates": ["example/diag-comp-alt"],
                "compatibility_notes": ["Requires kernel v0.1.0"],
                "surfaces": [{"slot": "experience_entry"}],
            }),
        })
        .await?;
    anyhow::ensure!(plan.output["kind"] == json!("composition_launch_plan"), "composition lab launch_plan v2 returned wrong kind");
    anyhow::ensure!(plan.output.get("required_capabilities").is_some(), "launch_plan missing required_capabilities");
    anyhow::ensure!(plan.output.get("permission_expectations").is_some(), "launch_plan missing permission_expectations");
    anyhow::ensure!(plan.output.get("replacement_candidates").is_some(), "launch_plan missing replacement_candidates");
    anyhow::ensure!(plan.output.get("compatibility_notes").is_some(), "launch_plan missing compatibility_notes");

    // surface_graph with v2 fields
    let graph = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/composition-lab/surface_graph".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/composition-lab".to_string()),
            version: None,
            input: json!({
                "entry_surface_id": "example/diag-comp/entry",
                "surfaces": [{"slot": "experience_entry"}],
                "required_capabilities": ["example/diag-comp/echo"],
                "permission_expectations": ["capabilities.invoke"],
                "replacement_candidates": ["example/diag-comp-alt"],
                "compatibility_notes": ["Requires kernel v0.1.0"],
            }),
        })
        .await?;
    anyhow::ensure!(graph.output["kind"] == json!("composition_surface_graph"), "composition lab surface_graph v2 returned wrong kind");
    anyhow::ensure!(graph.output.get("required_capabilities").is_some(), "surface_graph missing required_capabilities");
    anyhow::ensure!(graph.output.get("permission_expectations").is_some(), "surface_graph missing permission_expectations");
    anyhow::ensure!(graph.output.get("replacement_candidates").is_some(), "surface_graph missing replacement_candidates");
    anyhow::ensure!(graph.output.get("compatibility_notes").is_some(), "surface_graph missing compatibility_notes");

    // compat_report capability
    let report = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/composition-lab/compat_report".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/composition-lab".to_string()),
            version: None,
            input: json!({
                "id": "example/diag-comp",
                "required_capabilities": ["example/diag-comp/echo", "example/missing/cap"],
                "available_capabilities": ["example/diag-comp/echo"],
                "permission_expectations": ["capabilities.invoke"],
                "replacement_candidates": ["example/diag-comp-alt"],
                "compatibility_notes": ["Requires kernel v0.1.0"],
                "surfaces": [{"slot": "experience_entry"}],
            }),
        })
        .await?;
    anyhow::ensure!(report.output["kind"] == json!("composition_compat_report"), "composition lab compat_report returned wrong kind");
    anyhow::ensure!(report.output.get("missing_required_capabilities").is_some(), "compat_report missing missing_required_capabilities");
    let missing = report.output["missing_required_capabilities"].as_array().unwrap();
    anyhow::ensure!(missing.len() == 1, "compat_report should report exactly 1 missing capability");
    anyhow::ensure!(missing[0] == json!("example/missing/cap"), "compat_report wrong missing capability");

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

pub(crate) async fn capability_tool_bridge_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("packages/official/capability-tool-bridge-lab/manifest.yaml")).await?).await?;

    // discover_tools: ambiguous providers marked rejected
    let discovery = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/capability-tool-bridge-lab/discover_tools".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/capability-tool-bridge-lab".to_string()),
            version: None,
            input: json!({
                "capabilities": [
                    {
                        "capability_id": "example/echo",
                        "providers": ["official/pkg-a", "thirdparty/pkg-b"]
                    }
                ]
            }),
        })
        .await?;
    anyhow::ensure!(discovery.output["kind"] == json!("tool_bridge_discovery"), "tool bridge discovery wrong kind");
    let tools = discovery.output["tools"].as_array().unwrap();
    anyhow::ensure!(tools.len() == 1, "tool bridge discovery should return 1 tool");
    anyhow::ensure!(tools[0]["status"] == json!("rejected"), "ambiguous tool should be rejected");
    anyhow::ensure!(tools[0]["ambiguous"] == json!(true), "ambiguous tool should be flagged");
    // No official preference
    anyhow::ensure!(tools[0]["provider_package_id"].is_null(), "ambiguous tool should not auto-select official provider");

    // discover_tools: explicit third-party provider works as plan
    let discovery_explicit = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/capability-tool-bridge-lab/discover_tools".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/capability-tool-bridge-lab".to_string()),
            version: None,
            input: json!({
                "capabilities": [
                    {
                        "capability_id": "example/echo",
                        "providers": ["official/pkg-a", "thirdparty/pkg-b"],
                        "provider_package_id": "thirdparty/pkg-b"
                    }
                ]
            }),
        })
        .await?;
    let tools_explicit = discovery_explicit.output["tools"].as_array().unwrap();
    anyhow::ensure!(tools_explicit[0]["status"] == json!("available"), "explicit third-party provider should be available");
    anyhow::ensure!(tools_explicit[0]["provider_package_id"] == json!("thirdparty/pkg-b"), "explicit provider should be preserved");

    // discover_tools: explicit provider must be in candidate providers when candidates are supplied
    let discovery_bad_provider = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/capability-tool-bridge-lab/discover_tools".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/capability-tool-bridge-lab".to_string()),
            version: None,
            input: json!({
                "capabilities": [
                    {
                        "capability_id": "example/echo",
                        "providers": ["official/pkg-a", "thirdparty/pkg-b"],
                        "provider_package_id": "thirdparty/not-a-provider"
                    }
                ]
            }),
        })
        .await?;
    let bad_provider_tools = discovery_bad_provider.output["tools"].as_array().unwrap();
    anyhow::ensure!(bad_provider_tools[0]["status"] == json!("rejected"), "explicit non-candidate provider should be rejected");
    anyhow::ensure!(bad_provider_tools[0]["rejection_reason"] == json!("provider_not_in_candidates"), "explicit non-candidate provider wrong reason");

    // invoke_tool: missing provider rejected
    let invoke_missing = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/capability-tool-bridge-lab/invoke_tool".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/capability-tool-bridge-lab".to_string()),
            version: None,
            input: json!({
                "capability_id": "example/echo"
            }),
        })
        .await?;
    anyhow::ensure!(invoke_missing.output["kind"] == json!("tool_bridge_invocation_plan"), "invoke_tool wrong kind");
    anyhow::ensure!(invoke_missing.output["status"] == json!("rejected"), "missing provider should be rejected");
    anyhow::ensure!(invoke_missing.output["rejection_reason"] == json!("missing_provider"), "missing provider wrong reason");

    // invoke_tool: explicit third-party provider produces plan
    let invoke_explicit = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/capability-tool-bridge-lab/invoke_tool".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/capability-tool-bridge-lab".to_string()),
            version: None,
            input: json!({
                "capability_id": "example/echo",
                "provider_package_id": "thirdparty/my-tool"
            }),
        })
        .await?;
    anyhow::ensure!(invoke_explicit.output["status"] == json!("plan_ready"), "explicit third-party invoke should be plan_ready");
    anyhow::ensure!(invoke_explicit.output["method"] == json!("kernel.capability.invoke"), "invoke_tool method should be kernel.capability.invoke");
    anyhow::ensure!(invoke_explicit.output["requires_user_approval"] == json!(true), "invoke plan must require approval");

    // invoke_tool: explicit provider must match supplied candidates
    let invoke_bad_provider = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/capability-tool-bridge-lab/invoke_tool".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/capability-tool-bridge-lab".to_string()),
            version: None,
            input: json!({
                "capability_id": "example/echo",
                "provider_package_id": "thirdparty/not-a-provider",
                "providers": ["official/pkg-a", "thirdparty/pkg-b"]
            }),
        })
        .await?;
    anyhow::ensure!(invoke_bad_provider.output["status"] == json!("rejected"), "invoke explicit non-candidate provider should reject");
    anyhow::ensure!(invoke_bad_provider.output["rejection_reason"] == json!("provider_not_in_candidates"), "invoke explicit non-candidate wrong reason");

    // preview_tool_permissions: denied reports missing permission
    let preview_denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/capability-tool-bridge-lab/preview_tool_permissions".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/capability-tool-bridge-lab".to_string()),
            version: None,
            input: json!({
                "required_permissions": ["capabilities.invoke"],
                "grants": [],
                "provider_package_id": "official/echo"
            }),
        })
        .await?;
    anyhow::ensure!(preview_denied.output["kind"] == json!("tool_bridge_permission_preview"), "preview wrong kind");
    anyhow::ensure!(preview_denied.output["allowed"] == json!(false), "denied preview should not be allowed");
    let missing = preview_denied.output["missing_permissions"].as_array().unwrap();
    anyhow::ensure!(missing.len() == 1, "should report 1 missing permission");

    // preview_tool_permissions: granted with wildcard
    let preview_granted = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/capability-tool-bridge-lab/preview_tool_permissions".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/capability-tool-bridge-lab".to_string()),
            version: None,
            input: json!({
                "required_permissions": ["capabilities.invoke"],
                "grants": ["*"],
                "provider_package_id": "official/echo"
            }),
        })
        .await?;
    anyhow::ensure!(preview_granted.output["allowed"] == json!(true), "granted preview should be allowed");

    // raw secret payload: unsafe_blocked
    let raw_secret = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/capability-tool-bridge-lab/invoke_tool".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/capability-tool-bridge-lab".to_string()),
            version: None,
            input: json!({
                "capability_id": "example/echo",
                "provider_package_id": "official/pkg",
                "api_key": "sk-raw-secret-value-here"
            }),
        })
        .await?;
    anyhow::ensure!(raw_secret.output["redaction_state"] == json!("unsafe_blocked"), "raw secret should be unsafe_blocked");
    anyhow::ensure!(raw_secret.output["status"] == json!("rejected"), "raw secret invoke should be rejected");

    // stream_tool: missing provider rejected
    let stream_missing = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/capability-tool-bridge-lab/stream_tool".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/capability-tool-bridge-lab".to_string()),
            version: None,
            input: json!({
                "capability_id": "example/stream"
            }),
        })
        .await?;
    anyhow::ensure!(stream_missing.output["kind"] == json!("tool_bridge_stream_plan"), "stream_tool wrong kind");
    anyhow::ensure!(stream_missing.output["status"] == json!("rejected"), "missing provider stream should be rejected");
    anyhow::ensure!(stream_missing.output["method"] == json!("kernel.capability.stream"), "stream method should be kernel.capability.stream");

    // stream_tool: explicit provider must match supplied candidates
    let stream_bad_provider = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/capability-tool-bridge-lab/stream_tool".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/capability-tool-bridge-lab".to_string()),
            version: None,
            input: json!({
                "capability_id": "example/stream",
                "provider_package_id": "thirdparty/not-a-provider",
                "providers": ["official/pkg-a", "thirdparty/pkg-b"]
            }),
        })
        .await?;
    anyhow::ensure!(stream_bad_provider.output["status"] == json!("rejected"), "stream explicit non-candidate provider should reject");
    anyhow::ensure!(stream_bad_provider.output["rejection_reason"] == json!("provider_not_in_candidates"), "stream explicit non-candidate wrong reason");

    // explain_tool_call: audit-safe summary
    let explain = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/capability-tool-bridge-lab/explain_tool_call".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/capability-tool-bridge-lab".to_string()),
            version: None,
            input: json!({
                "capability_id": "example/echo",
                "provider_package_id": "thirdparty/my-tool",
                "method": "kernel.capability.invoke"
            }),
        })
        .await?;
    anyhow::ensure!(explain.output["kind"] == json!("tool_bridge_explanation"), "explain wrong kind");
    anyhow::ensure!(explain.output["redaction_state"] == json!("clean"), "explain with clean input should be clean");

    // surfaces discoverable: forge_panel, assistant_action, home_card
    let forge_surfaces = runtime.list_surface_contributions(Some("forge_panel".to_string())).await;
    let has_forge = forge_surfaces
        .as_array()
        .map(|items| items.iter().any(|r| r["package_id"] == json!("official/capability-tool-bridge-lab")))
        .unwrap_or(false);
    anyhow::ensure!(has_forge, "tool bridge forge_panel surface missing");

    let assistant_surfaces = runtime.list_surface_contributions(Some("assistant_action".to_string())).await;
    let has_assistant = assistant_surfaces
        .as_array()
        .map(|items| items.iter().any(|r| r["package_id"] == json!("official/capability-tool-bridge-lab")))
        .unwrap_or(false);
    anyhow::ensure!(has_assistant, "tool bridge assistant_action surface missing");

    let home_surfaces = runtime.list_surface_contributions(Some("home_card".to_string())).await;
    let has_home = home_surfaces
        .as_array()
        .map(|items| items.iter().any(|r| r["package_id"] == json!("official/capability-tool-bridge-lab")))
        .unwrap_or(false);
    anyhow::ensure!(has_home, "tool bridge home_card surface missing");

    Ok(())
}

pub(crate) async fn pi_agent_runtime_lab() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("packages/official/pi-agent-runtime-lab/manifest.yaml")).await?).await?;

    // run: deterministic no-inference no-network plan
    let run_result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/pi-agent-runtime-lab/run".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/pi-agent-runtime-lab".to_string()),
            version: None,
            input: json!({}),
        })
        .await?;
    anyhow::ensure!(run_result.output["kind"] == json!("pi_agent_run_plan"), "pi-agent run returned wrong kind");
    anyhow::ensure!(run_result.output["inference_performed"] == json!(false), "pi-agent run must not perform inference");
    anyhow::ensure!(run_result.output["network_performed"] == json!(false), "pi-agent run must not perform network");
    anyhow::ensure!(run_result.output["trace_events"].is_array(), "pi-agent run missing trace_events");
    anyhow::ensure!(run_result.output["stream_frames"].is_array(), "pi-agent run missing stream_frames");
    anyhow::ensure!(run_result.output["proposal_draft"].is_object(), "pi-agent run missing proposal_draft");
    anyhow::ensure!(run_result.output["provenance"]["package_id"] == json!("official/pi-agent-runtime-lab"), "pi-agent run provenance mismatch");

    // explain_run: no-inference explanation
    let explain = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/pi-agent-runtime-lab/explain_run".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/pi-agent-runtime-lab".to_string()),
            version: None,
            input: json!({"trace_events": [{"step": 1}, {"step": 2}]}),
        })
        .await?;
    anyhow::ensure!(explain.output["kind"] == json!("pi_agent_run_explanation"), "pi-agent explain_run wrong kind");
    anyhow::ensure!(explain.output["inference_performed"] == json!(false), "pi-agent explain_run must not claim inference");
    anyhow::ensure!(explain.output["network_performed"] == json!(false), "pi-agent explain_run must not claim network");
    anyhow::ensure!(explain.output["trace_event_count"] == json!(2), "pi-agent explain_run wrong event count");

    // draft_proposal: approval-gated
    let proposal = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/pi-agent-runtime-lab/draft_proposal".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/pi-agent-runtime-lab".to_string()),
            version: None,
            input: json!({"change": "agent-driven modification"}),
        })
        .await?;
    anyhow::ensure!(proposal.output["kind"] == json!("pi_agent_proposal"), "pi-agent draft_proposal wrong kind");
    anyhow::ensure!(proposal.output["requires_user_approval"] == json!(true), "pi-agent proposal must require approval");
    anyhow::ensure!(proposal.output["provenance"]["package_id"] == json!("official/pi-agent-runtime-lab"), "pi-agent proposal provenance mismatch");

    // summarize_trace: no-inference summary
    let trace = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/pi-agent-runtime-lab/summarize_trace".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/pi-agent-runtime-lab".to_string()),
            version: None,
            input: json!({"trace_events": [{"e": 1}, {"e": 2}, {"e": 3}]}),
        })
        .await?;
    anyhow::ensure!(trace.output["kind"] == json!("pi_agent_trace_summary"), "pi-agent summarize_trace wrong kind");
    anyhow::ensure!(trace.output["event_count"] == json!(3), "pi-agent summarize_trace wrong event count");
    anyhow::ensure!(trace.output["inference_performed"] == json!(false), "pi-agent summarize_trace must not claim inference");
    anyhow::ensure!(trace.output["network_performed"] == json!(false), "pi-agent summarize_trace must not claim network");

    // echo: passthrough
    let echo = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/pi-agent-runtime-lab/echo".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/pi-agent-runtime-lab".to_string()),
            version: None,
            input: json!({"hello": "agent"}),
        })
        .await?;
    anyhow::ensure!(echo.output["kind"] == json!("pi_agent_echo"), "pi-agent echo wrong kind");
    anyhow::ensure!(echo.output["input"]["hello"] == json!("agent"), "pi-agent echo did not pass through input");

    // surfaces discoverable: assistant_action, forge_panel, home_card
    let assistant_surfaces = runtime.list_surface_contributions(Some("assistant_action".to_string())).await;
    let has_assistant = assistant_surfaces
        .as_array()
        .map(|items| items.iter().any(|r| r["package_id"] == json!("official/pi-agent-runtime-lab")))
        .unwrap_or(false);
    anyhow::ensure!(has_assistant, "pi-agent assistant_action surface missing");

    let forge_surfaces = runtime.list_surface_contributions(Some("forge_panel".to_string())).await;
    let has_forge = forge_surfaces
        .as_array()
        .map(|items| items.iter().any(|r| r["package_id"] == json!("official/pi-agent-runtime-lab")))
        .unwrap_or(false);
    anyhow::ensure!(has_forge, "pi-agent forge_panel surface missing");

    let home_surfaces = runtime.list_surface_contributions(Some("home_card".to_string())).await;
    let has_home = home_surfaces
        .as_array()
        .map(|items| items.iter().any(|r| r["package_id"] == json!("official/pi-agent-runtime-lab")))
        .unwrap_or(false);
    anyhow::ensure!(has_home, "pi-agent home_card surface missing");

    Ok(())
}
