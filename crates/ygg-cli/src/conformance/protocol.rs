use serde_json::json;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use ygg_runtime::{
    ContractOwnerLayer, ContractSelection, ContractVersionRequirement, DeploymentReconcileSource,
    EventStore, ExecStatus, ExecStatusKind, InMemoryEventStore, LocalExecExecutor,
    LocalExecExecutorConfig, LocalExecLogsRequest, LocalExecLogsResponse, LocalExecStartRequest,
    LocalExecStartResponse, LocalExecStatusRequest, LocalExecStatusResponse, LocalExecStopRequest,
    LocalExecStopResponse, ManagedContainerReport, PortLeaseStatusKind, ProtocolContext,
    ProtocolPrincipal, ProxyRouteStatusKind, Runtime, RuntimeConfig, SqliteEventStore,
    CONTRACT_LAYER_VERSION, DEFAULT_CONTRACT_PROFILE, SHELL_DEFAULT_PROFILE,
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

pub(crate) async fn alias_equivalent() -> anyhow::Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let mut config = RuntimeConfig::default();
    config
        .surface_dev_paths
        .insert("smoke".to_string(), ".".to_string());
    let runtime = Runtime::new(store.clone(), config);
    let context = ProtocolContext::host_dev("conformance");
    let canonical = runtime
        .call_protocol(&context, "host.info", json!({}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let legacy = runtime
        .call_protocol(&context, "kernel.v1.host.info", json!({}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(canonical == legacy, "canonical and legacy host.info differ");
    anyhow::ensure!(
        canonical["aliases"].as_array().is_some_and(|aliases| {
            aliases.iter().any(|alias| {
                alias["id"] == "kernel.v1.host.info" && alias["canonical_id"] == "host.info"
            })
        }),
        "host.info did not advertise its legacy alias"
    );

    for (canonical_id, legacy_id, params) in [
        ("host.project.list", "kernel.v1.project.list", json!({})),
        ("host.target.list", "kernel.v1.target.list", json!({})),
        ("host.exec.list", "kernel.v1.exec.list", json!({})),
        ("host.port.list", "kernel.v1.port.list", json!({})),
        ("host.proxy.list", "kernel.v1.proxy.list", json!({})),
        (
            "host.surface.bundle.resolve",
            "kernel.v1.surface.resolve_bundle",
            json!({"surface_id": "smoke/entry"}),
        ),
        (
            "shell.contribution.list",
            "kernel.v1.surface.contribution.list",
            json!({}),
        ),
        ("change.proposal.list", "kernel.v1.proposal.list", json!({})),
        ("projection.list", "kernel.v1.projection.list", json!({})),
    ] {
        let canonical = runtime
            .call_protocol(&context, canonical_id, params.clone())
            .await
            .map_err(|error| anyhow::anyhow!(error.message))?;
        let legacy = runtime
            .call_protocol(&context, legacy_id, params)
            .await
            .map_err(|error| anyhow::anyhow!(error.message))?;
        anyhow::ensure!(
            canonical == legacy,
            "canonical {canonical_id} and legacy {legacy_id} differ"
        );
    }

    let denied_context = ProtocolContext {
        principal: ProtocolPrincipal::Anonymous,
        transport: "conformance".to_string(),
        session_id: None,
        correlation_id: None,
        parent_invocation_id: None,
    };
    let canonical_error = runtime
        .call_protocol(&denied_context, "host.target.list", json!({}))
        .await
        .expect_err("canonical target.list must preserve the permission gate");
    let legacy_error = runtime
        .call_protocol(&denied_context, "kernel.v1.target.list", json!({}))
        .await
        .expect_err("legacy target.list must preserve the permission gate");
    anyhow::ensure!(
        canonical_error == legacy_error,
        "canonical and legacy permission/error mapping differ"
    );
    anyhow::ensure!(
        store.list_all().await?.is_empty(),
        "identity aliases must not create a distinct audit/event path"
    );
    Ok(())
}

pub(crate) async fn layered_namespace_smoke() -> anyhow::Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let mut config = RuntimeConfig::default();
    config
        .surface_dev_paths
        .insert("smoke".to_string(), ".".to_string());
    let runtime = Runtime::new(store.clone(), config);
    let context = ProtocolContext::host_dev("layered_namespace_smoke");
    let default_selection = ContractSelection {
        profile: DEFAULT_CONTRACT_PROFILE.to_string(),
        versions: Vec::new(),
    };

    for (method, params) in [
        ("host.info", json!({})),
        ("host.project.list", json!({})),
        ("host.target.list", json!({})),
        ("host.exec.list", json!({})),
        ("host.port.list", json!({})),
        ("host.proxy.list", json!({})),
        (
            "host.surface.bundle.resolve",
            json!({"surface_id": "smoke/entry"}),
        ),
        ("change.proposal.list", json!({})),
        ("projection.list", json!({})),
    ] {
        runtime
            .call_protocol_negotiated(&context, method, params, Some(&default_selection))
            .await
            .map_err(|error| anyhow::anyhow!("{method}: {}", error.message))?;
    }

    let shell_selection = ContractSelection {
        profile: SHELL_DEFAULT_PROFILE.to_string(),
        versions: Vec::new(),
    };
    runtime
        .call_protocol_negotiated(
            &context,
            "shell.contribution.list",
            json!({}),
            Some(&shell_selection),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;

    anyhow::ensure!(
        store.list_all().await?.is_empty(),
        "read-only layered namespace smoke created events"
    );
    Ok(())
}

pub(crate) async fn unsupported_version_rejected() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let selection = ContractSelection {
        profile: DEFAULT_CONTRACT_PROFILE.to_string(),
        versions: vec![ContractVersionRequirement {
            layer: ContractOwnerLayer::Host,
            version: "999.0.0".to_string(),
        }],
    };
    let error = runtime
        .call_protocol_negotiated(
            &ProtocolContext::host_dev("conformance"),
            "host.info",
            json!({}),
            Some(&selection),
        )
        .await
        .expect_err("unsupported contract version must fail");
    anyhow::ensure!(error.code == "kernel/v1/error/unsupported_contract");
    anyhow::ensure!(error.details["reason"] == "unsupported_version");
    anyhow::ensure!(
        error.details["details"]["supported_version"] == CONTRACT_LAYER_VERSION,
        "supported version missing from structured error"
    );
    Ok(())
}

pub(crate) async fn no_silent_downgrade() -> anyhow::Result<()> {
    let (store, runtime) = runtime();
    let selection = ContractSelection {
        profile: DEFAULT_CONTRACT_PROFILE.to_string(),
        versions: vec![ContractVersionRequirement {
            layer: ContractOwnerLayer::Substrate,
            version: "999.0.0".to_string(),
        }],
    };
    let error = runtime
        .call_protocol_negotiated(
            &ProtocolContext::host_dev("conformance"),
            "kernel.v1.session.open",
            json!({"labels": [], "metadata": {}, "active_package_set": []}),
            Some(&selection),
        )
        .await
        .expect_err("unsupported selection must not fall back to kernel.v1");
    anyhow::ensure!(error.code == "kernel/v1/error/unsupported_contract");
    anyhow::ensure!(
        store.list_all().await?.is_empty(),
        "rejected negotiation still reached the session handler"
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
    anyhow::ensure!(
        !restored_route.ready,
        "proxy route must rehydrate as not ready before reconcile"
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

pub(crate) async fn deployment_hub_exec_stop_receipt() -> anyhow::Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let mut config = RuntimeConfig::default();
    config.local_exec_executor = LocalExecExecutorConfig::Fake;
    let runtime = Runtime::new(store.clone(), config);
    let context = ProtocolContext::host_dev("conformance");
    let started = runtime
        .call_protocol(
            &context,
            "kernel.v1.exec.start",
            json!({"target_id":"local","command":{"program":"demo","args":[]}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let exec_id = started["exec_id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("exec id missing"))?;
    let stopped = runtime
        .call_protocol(
            &context,
            "kernel.v1.exec.stop",
            json!({"exec_id": exec_id, "reason": "conformance"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(
        stopped["status"]["kind"] == json!("stopped"),
        "fake exec did not stop"
    );
    let events = store.list_all().await?;
    let event = events
        .iter()
        .find(|event| event.kind == ygg_core::EVENT_EXEC_STOPPED)
        .ok_or_else(|| anyhow::anyhow!("exec stopped event missing"))?;
    let receipt_digest = event.payload["receipt"]["digest"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("exec stopped receipt missing"))?;
    let replay = runtime.replay_effect_receipt(receipt_digest).await?;
    anyhow::ensure!(
        replay.receipt.effect_kind == "exec.run"
            && replay.receipt.status == ygg_core::EffectTerminalStatus::Cancelled,
        "exec stopped receipt is incomplete"
    );
    Ok(())
}

struct AutoTerminalExecExecutor {
    status_polls: AtomicUsize,
}

#[async_trait]
impl LocalExecExecutor for AutoTerminalExecExecutor {
    fn supports_terminal_monitoring(&self) -> bool {
        true
    }

    async fn start(
        &self,
        request: LocalExecStartRequest,
    ) -> anyhow::Result<LocalExecStartResponse> {
        let exec_id = "auto-terminal-exec".to_string();
        let status = ExecStatus {
            exec_id: Some(exec_id.clone()),
            target_id: Some(request.target_id),
            kind: ExecStatusKind::Running,
            exit_code: None,
            message: Some("running".to_string()),
            ready: true,
        };
        Ok(LocalExecStartResponse {
            exec_id: Some(exec_id),
            status,
            error: None,
        })
    }

    async fn stop(&self, request: LocalExecStopRequest) -> anyhow::Result<LocalExecStopResponse> {
        Ok(LocalExecStopResponse {
            exec_id: request.exec_id.clone(),
            status: ExecStatus {
                exec_id: Some(request.exec_id),
                target_id: Some("local".to_string()),
                kind: ExecStatusKind::Stopped,
                exit_code: None,
                message: Some("stopped".to_string()),
                ready: false,
            },
            error: None,
        })
    }

    async fn status(
        &self,
        request: LocalExecStatusRequest,
    ) -> anyhow::Result<LocalExecStatusResponse> {
        self.status_polls.fetch_add(1, Ordering::SeqCst);
        Ok(LocalExecStatusResponse {
            status: ExecStatus {
                exec_id: Some(request.exec_id),
                target_id: Some("local".to_string()),
                kind: ExecStatusKind::Exited,
                exit_code: Some(0),
                message: Some("exited".to_string()),
                ready: false,
            },
            error: None,
        })
    }

    async fn logs(&self, request: LocalExecLogsRequest) -> anyhow::Result<LocalExecLogsResponse> {
        Ok(LocalExecLogsResponse {
            exec_id: request.exec_id,
            lines: Vec::new(),
            next_seq: None,
            error: None,
        })
    }
}

pub(crate) async fn deployment_hub_exec_terminal_is_observed_once() -> anyhow::Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let executor = Arc::new(AutoTerminalExecExecutor {
        status_polls: AtomicUsize::new(0),
    });
    let mut config = RuntimeConfig::default();
    let object_store = config.object_store.clone();
    config.local_exec_executor = LocalExecExecutorConfig::Custom(executor.clone());
    let runtime = Runtime::new(store.clone(), config);
    let context = ProtocolContext::host_dev("conformance");
    runtime
        .call_protocol(
            &context,
            "kernel.v1.exec.start",
            json!({"target_id":"local","command":{"program":"demo","args":[]}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;

    let mut completed = None;
    for _ in 0..100 {
        if let Some(event) = store
            .list_all()
            .await?
            .into_iter()
            .find(|event| event.kind == ygg_core::EVENT_EXEC_COMPLETED)
        {
            completed = Some(event);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let completed = completed.ok_or_else(|| {
        anyhow::anyhow!(
            "exec terminal monitor did not converge after {} status polls",
            executor.status_polls.load(Ordering::SeqCst)
        )
    })?;
    let receipt_digest = completed.payload["receipt"]["digest"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("natural exec completion receipt missing"))?;
    let replay = runtime.replay_effect_receipt(receipt_digest).await?;
    anyhow::ensure!(
        replay.receipt.effect_kind == "exec.run"
            && replay.receipt.status == ygg_core::EffectTerminalStatus::Succeeded,
        "natural exec completion receipt is incomplete"
    );
    anyhow::ensure!(
        store
            .list_all()
            .await?
            .iter()
            .filter(|event| event.kind == ygg_core::EVENT_EXEC_COMPLETED)
            .count()
            == 1,
        "natural exec completion emitted duplicate terminal events"
    );

    drop(runtime);
    let mut hydrated_config = RuntimeConfig::default();
    hydrated_config.object_store = object_store;
    let hydrated = Runtime::new(store.clone(), hydrated_config);
    hydrated.hydrate_deployment_from_events().await?;
    let status = hydrated
        .call_protocol(
            &context,
            "kernel.v1.exec.status",
            json!({"exec_id": "auto-terminal-exec"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(
        status["status"]["kind"] == json!("exited"),
        "terminal exec status was not restored after hydration"
    );
    anyhow::ensure!(
        store
            .list_all()
            .await?
            .iter()
            .filter(|event| event.kind == ygg_core::EVENT_EXEC_COMPLETED)
            .count()
            == 1,
        "status after hydration emitted a duplicate terminal receipt"
    );
    Ok(())
}

pub(crate) async fn deployment_hub_exec_denial_is_deduplicated() -> anyhow::Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store.clone(), RuntimeConfig::default());
    let context = ProtocolContext::host_dev("conformance");
    for _ in 0..2 {
        let response = runtime
            .call_protocol(
                &context,
                "kernel.v1.exec.status",
                json!({"exec_id": "denied-exec"}),
            )
            .await
            .map_err(|error| anyhow::anyhow!(error.message))?;
        anyhow::ensure!(
            response["status"]["kind"] == json!("denied"),
            "deny-all exec status did not return denied"
        );
    }
    anyhow::ensure!(
        store
            .list_all()
            .await?
            .iter()
            .filter(|event| event.kind == ygg_core::EVENT_EXEC_DENIED)
            .count()
            == 1,
        "repeated denied status created duplicate receipts"
    );

    drop(runtime);
    let hydrated = Runtime::new(store.clone(), RuntimeConfig::default());
    hydrated.hydrate_deployment_from_events().await?;
    hydrated
        .call_protocol(
            &context,
            "kernel.v1.exec.status",
            json!({"exec_id": "denied-exec"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(
        store
            .list_all()
            .await?
            .iter()
            .filter(|event| event.kind == ygg_core::EVENT_EXEC_DENIED)
            .count()
            == 1,
        "denied status after hydration created a duplicate receipt"
    );
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
    anyhow::ensure!(route.ready, "promoted proxy route must be ready");
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
