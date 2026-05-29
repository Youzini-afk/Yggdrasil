use serde_json::json;
use ygg_runtime::{ProtocolContext, ProtocolPrincipal};

use super::fixtures::*;

pub(crate) async fn call_host_info() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let value = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.v1.host.info",
            json!({}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(
        value.get("supported_transports").is_some(),
        "host.info missing transports"
    );
    Ok(())
}

pub(crate) async fn call_capability_in_process() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime
        .load_package(echo_package("example/protocol", "example/protocol/echo"))
        .await?;
    let value = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.v1.capability.invoke",
            json!({"capability_id": "example/protocol/echo", "input": {"via": "protocol"}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(
        value.get("output") == Some(&json!({"via": "protocol"})),
        "protocol invoke mismatch"
    );
    Ok(())
}

pub(crate) async fn deployment_hub_requires_host_principal() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let context = ProtocolContext {
        principal: ProtocolPrincipal::Anonymous,
        transport: "conformance".to_string(),
        session_id: None,
        correlation_id: None,
        parent_invocation_id: None,
    };

    let result = runtime
        .call_protocol(
            &context,
            "kernel.v1.port.lease",
            json!({"target_id":"local","port_name":"web"}),
        )
        .await;
    let error = result.expect_err("anonymous deployment hub call must fail");
    anyhow::ensure!(
        error.code == "kernel/v1/error/permission_denied",
        "unexpected error code: {}",
        error.code
    );
    Ok(())
}

pub(crate) async fn deployment_hub_port_lease_loopback() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let value = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.v1.port.lease",
            json!({"target_id":"local","port_name":"web","requested_port":39123}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(value["lease"]["host"] == json!("127.0.0.1"));
    anyhow::ensure!(value["lease"]["bind"] == json!("loopback_only"));
    Ok(())
}

pub(crate) async fn deployment_hub_proxy_requires_matching_lease_port() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let context = ProtocolContext::host_dev("conformance");
    let lease = runtime
        .call_protocol(
            &context,
            "kernel.v1.port.lease",
            json!({"target_id":"local","port_name":"web"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let lease_id = lease["lease"]["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("lease id missing"))?;

    let mismatch = runtime
        .call_protocol(
            &context,
            "kernel.v1.proxy.register",
            json!({
                "upstream": {"port_lease_id": lease_id, "port_name": "admin"},
                "protocol": "http"
            }),
        )
        .await;
    anyhow::ensure!(mismatch.is_err(), "mismatched port_name must fail");
    Ok(())
}
