use serde_json::json;
use ygg_runtime::{OpenSessionRequest, ProtocolContext};

use super::fixtures::*;

pub(crate) async fn lifecycle_apply() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime
        .projection_register(ygg_runtime::runtime::ProjectionDefinition {
            id: "proposal/test-projection".to_string(),
            session_id: session.id.clone(),
            source_kind_prefix: Some("kernel/session".to_string()),
            state: json!({}),
        })
        .await?;
    let created = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.proposal.create",
            json!({
                "target_session_id": session.id,
                "required_permissions": ["assets.write", "projections.rebuild"],
                "expected_effects": {"summary": "write asset and rebuild projection"},
                "operations": [
                    {"op": "asset.put", "payload": {"mime": "application/json", "content": "{\"proposal\":true}"}},
                    {"op": "projection.rebuild", "target": "proposal/test-projection"}
                ]
            }),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let proposal_id = created["id"].as_str().ok_or_else(|| anyhow::anyhow!("proposal missing id"))?.to_string();
    let denied = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.proposal.apply", json!({"proposal_id": proposal_id}))
        .await;
    anyhow::ensure!(denied.is_err(), "unapproved proposal should not apply");
    runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.proposal.approve", json!({"proposal_id": proposal_id, "reason": "conformance"}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let applied = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.proposal.apply", json!({"proposal_id": proposal_id}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(applied["status"] == json!("applied"), "proposal did not reach applied status");
    anyhow::ensure!(applied["result"]["operations"].as_array().map(|items| items.len()).unwrap_or(0) == 2, "proposal apply results missing");
    Ok(())
}

pub(crate) async fn reject_and_apply_denied() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let created = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.proposal.create",
            json!({"operations": [{"op": "asset.put", "payload": {"content": "{}"}}]}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let proposal_id = created["id"].as_str().ok_or_else(|| anyhow::anyhow!("proposal missing id"))?.to_string();
    runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.proposal.reject", json!({"proposal_id": proposal_id, "reason": "conformance"}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let denied = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.proposal.apply", json!({"proposal_id": proposal_id}))
        .await;
    anyhow::ensure!(denied.is_err(), "rejected proposal should not apply");
    Ok(())
}
