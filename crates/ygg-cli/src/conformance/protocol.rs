use serde_json::json;
use ygg_runtime::ProtocolContext;

use super::fixtures::*;

pub(crate) async fn call_host_info() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let value = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.host.info", json!({}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(value.get("supported_transports").is_some(), "host.info missing transports");
    Ok(())
}

pub(crate) async fn call_capability_in_process() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/protocol", "example/protocol/echo")).await?;
    let value = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.capability.invoke",
            json!({"capability_id": "example/protocol/echo", "input": {"via": "protocol"}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(value.get("output") == Some(&json!({"via": "protocol"})), "protocol invoke mismatch");
    Ok(())
}
