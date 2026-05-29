use serde_json::json;
use std::fs;
use std::sync::Arc;

use async_trait::async_trait;
use ygg_runtime::{
    DeploymentReconcileSource, ExecStatusKind, LocalExecExecutorConfig, ManagedContainerReport,
    PortLeaseStatusKind, ProtocolContext, ProtocolPrincipal, ProxyRouteStatusKind, Runtime,
    RuntimeConfig, SqliteEventStore,
};

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

pub(crate) async fn deployment_sqlite_rehydrate() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!(
        "ygg-deployment-rehydrate-{}.db",
        std::process::id()
    ));
    if path.exists() {
        fs::remove_file(&path)?;
    }

    let store = Arc::new(SqliteEventStore::open(&path)?);
    let mut config = RuntimeConfig::default();
    config.local_exec_executor = LocalExecExecutorConfig::Fake;
    let runtime = Runtime::new(store.clone(), config);
    let context = ProtocolContext::host_dev("conformance");

    let lease = runtime
        .call_protocol(
            &context,
            "kernel.v1.port.lease",
            json!({"target_id":"local","port_name":"web","requested_port":39201}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let lease_id = lease["lease"]["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("lease id missing"))?
        .to_string();

    let route = runtime
        .call_protocol(
            &context,
            "kernel.v1.proxy.register",
            json!({
                "upstream": {"port_lease_id": lease_id, "port_name": "web"},
                "protocol": "http"
            }),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let route_id = route["route"]["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("route id missing"))?
        .to_string();

    let exec = runtime
        .call_protocol(
            &context,
            "kernel.v1.exec.start",
            json!({"target_id":"local","command":{"program":"demo","args":[]}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let exec_id = exec["exec_id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("exec id missing"))?
        .to_string();
    drop(runtime);
    drop(store);

    let reopened = Arc::new(SqliteEventStore::open(&path)?);
    let config = RuntimeConfig::default();
    let port_lease_registry = config.port_lease_registry.clone();
    let proxy_route_registry = config.proxy_route_registry.clone();
    let exec_registry = config.exec_registry.clone();
    let hydrated = Runtime::new(reopened, config);
    hydrated.hydrate_deployment_from_events().await?;

    let restored_lease = port_lease_registry
        .status(&lease_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("port lease did not rehydrate"))?;
    anyhow::ensure!(
        restored_lease.status == PortLeaseStatusKind::Reserved,
        "port lease rehydrated as {:?}, expected Reserved",
        restored_lease.status
    );

    let restored_route = proxy_route_registry
        .status(&route_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("proxy route did not rehydrate"))?;
    anyhow::ensure!(
        restored_route.status == ProxyRouteStatusKind::Stale,
        "proxy route rehydrated as {:?}, expected Stale",
        restored_route.status
    );

    let restored_exec = exec_registry
        .status(&exec_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("exec did not rehydrate"))?;
    anyhow::ensure!(
        restored_exec.kind == ygg_runtime::ExecStatusKind::Unknown,
        "exec rehydrated as {:?}, expected Unknown",
        restored_exec.kind
    );

    let fresh = hydrated
        .call_protocol(
            &context,
            "kernel.v1.port.lease",
            json!({"target_id":"local","port_name":"admin"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let fresh_lease_id = fresh["lease"]["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("fresh lease id missing"))?;
    anyhow::ensure!(
        fresh_lease_id != lease_id,
        "fresh lease id collided with rehydrated id"
    );

    let _ = fs::remove_file(path);
    Ok(())
}

pub(crate) async fn deployment_reconcile_empty_cleans_stale() -> anyhow::Result<()> {
    let (
        runtime,
        port_lease_registry,
        proxy_route_registry,
        exec_registry,
        lease_id,
        route_id,
        exec_id,
    ) = hydrated_deployment_runtime(None).await?;

    let summary = runtime.reconcile_deployment().await?;
    anyhow::ensure!(summary.execs_failed == 1, "expected one failed exec");
    anyhow::ensure!(summary.routes_removed == 1, "expected one removed route");
    anyhow::ensure!(summary.leases_released == 1, "expected one released lease");

    let lease = port_lease_registry.status(&lease_id).await.unwrap();
    let route = proxy_route_registry.status(&route_id).await.unwrap();
    let exec = exec_registry.status(&exec_id).await.unwrap();
    anyhow::ensure!(lease.status == PortLeaseStatusKind::Released);
    anyhow::ensure!(route.status == ProxyRouteStatusKind::Removed);
    anyhow::ensure!(exec.kind == ExecStatusKind::Failed);
    Ok(())
}

pub(crate) async fn deployment_reconcile_promotes_live_container() -> anyhow::Result<()> {
    let report = ManagedContainerReport {
        route_id: "proxy-route-000000".to_string(),
        port_lease_id: "port-lease-000000".to_string(),
        running: true,
        host_port: Some(39201),
    };
    let (
        runtime,
        port_lease_registry,
        proxy_route_registry,
        _exec_registry,
        lease_id,
        route_id,
        _exec_id,
    ) = hydrated_deployment_runtime(Some(vec![report])).await?;

    let summary = runtime.reconcile_deployment().await?;
    anyhow::ensure!(summary.routes_promoted == 1, "expected one promoted route");
    anyhow::ensure!(summary.leases_promoted == 1, "expected one promoted lease");

    let lease = port_lease_registry.status(&lease_id).await.unwrap();
    let route = proxy_route_registry.status(&route_id).await.unwrap();
    anyhow::ensure!(lease.status == PortLeaseStatusKind::Active);
    anyhow::ensure!(route.status == ProxyRouteStatusKind::Active);
    Ok(())
}

pub(crate) async fn deployment_reconcile_exec_always_failed() -> anyhow::Result<()> {
    let report = ManagedContainerReport {
        route_id: "proxy-route-000000".to_string(),
        port_lease_id: "port-lease-000000".to_string(),
        running: true,
        host_port: Some(39201),
    };
    let (
        runtime,
        _port_lease_registry,
        _proxy_route_registry,
        exec_registry,
        _lease_id,
        _route_id,
        exec_id,
    ) = hydrated_deployment_runtime(Some(vec![report])).await?;

    let summary = runtime.reconcile_deployment().await?;
    anyhow::ensure!(summary.execs_failed == 1, "expected one failed exec");
    let exec = exec_registry.status(&exec_id).await.unwrap();
    anyhow::ensure!(exec.kind == ExecStatusKind::Failed);
    Ok(())
}

struct FakeReconcileSource {
    reports: Vec<ManagedContainerReport>,
}

#[async_trait]
impl DeploymentReconcileSource for FakeReconcileSource {
    async fn list_managed(&self) -> anyhow::Result<Vec<ManagedContainerReport>> {
        Ok(self.reports.clone())
    }
}

async fn hydrated_deployment_runtime(
    reports: Option<Vec<ManagedContainerReport>>,
) -> anyhow::Result<(
    Runtime<SqliteEventStore>,
    Arc<ygg_runtime::PortLeaseRegistry>,
    Arc<ygg_runtime::ProxyRouteRegistry>,
    Arc<ygg_runtime::ExecRegistry>,
    String,
    String,
    String,
)> {
    let path = std::env::temp_dir().join(format!(
        "ygg-deployment-reconcile-{}-{}.db",
        std::process::id(),
        reports.as_ref().map_or(0, Vec::len)
    ));
    if path.exists() {
        fs::remove_file(&path)?;
    }

    let store = Arc::new(SqliteEventStore::open(&path)?);
    let mut config = RuntimeConfig::default();
    config.local_exec_executor = LocalExecExecutorConfig::Fake;
    let runtime = Runtime::new(store.clone(), config);
    let context = ProtocolContext::host_dev("conformance");

    let lease = runtime
        .call_protocol(
            &context,
            "kernel.v1.port.lease",
            json!({"target_id":"local","port_name":"web","requested_port":39201}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let lease_id = lease["lease"]["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("lease id missing"))?
        .to_string();

    let route = runtime
        .call_protocol(
            &context,
            "kernel.v1.proxy.register",
            json!({"upstream":{"port_lease_id": lease_id, "port_name":"web"}, "protocol":"http"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let route_id = route["route"]["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("route id missing"))?
        .to_string();

    let exec = runtime
        .call_protocol(
            &context,
            "kernel.v1.exec.start",
            json!({"target_id":"local","command":{"program":"demo","args":[]}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let exec_id = exec["exec_id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("exec id missing"))?
        .to_string();
    drop(runtime);
    drop(store);

    let reopened = Arc::new(SqliteEventStore::open(&path)?);
    let mut config = RuntimeConfig::default();
    if let Some(reports) = reports {
        config.deployment_reconcile_source = Arc::new(FakeReconcileSource { reports });
    }
    let port_lease_registry = config.port_lease_registry.clone();
    let proxy_route_registry = config.proxy_route_registry.clone();
    let exec_registry = config.exec_registry.clone();
    let hydrated = Runtime::new(reopened, config);
    hydrated.hydrate_deployment_from_events().await?;

    Ok((
        hydrated,
        port_lease_registry,
        proxy_route_registry,
        exec_registry,
        lease_id,
        route_id,
        exec_id,
    ))
}
