use serde_json::json;
use ygg_runtime::{AppendEventRequest, CapabilityInvocationRequest, EventStore, OpenSessionRequest, ProtocolContext, RuntimeConfig};

use super::fixtures::*;

pub(crate) async fn session_open() -> anyhow::Result<()> {
    let (store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    let events = store.list_session(&session.id).await?;
    anyhow::ensure!(events.len() == 1, "expected one session-open event");
    Ok(())
}

pub(crate) async fn event_append_authorized() -> anyhow::Result<()> {
    let (store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/echo", true, true)).await?;
    runtime
        .append_event(AppendEventRequest {
            session_id: session.id.clone(),
            writer_package_id: "example/echo".to_string(),
            kind: "example/echo/conformance.event".to_string(),
            payload: json!({"conformance": true}),
            metadata: json!({}),
        })
        .await?;
    anyhow::ensure!(store.list_session(&session.id).await?.len() == 2, "expected append event");
    Ok(())
}

pub(crate) async fn event_append_without_permission_denied() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/noappend", true, false)).await?;
    let denied = runtime
        .append_event(AppendEventRequest {
            session_id: session.id,
            writer_package_id: "example/noappend".to_string(),
            kind: "example/noappend/event".to_string(),
            payload: json!({}),
            metadata: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "append without permission unexpectedly succeeded");
    Ok(())
}

pub(crate) async fn kernel_namespace_denied() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/writer", true, true)).await?;
    let denied = runtime
        .append_event(AppendEventRequest {
            session_id: session.id,
            writer_package_id: "example/writer".to_string(),
            kind: "kernel/v1/forged".to_string(),
            payload: json!({}),
            metadata: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "package wrote kernel namespace");
    Ok(())
}

pub(crate) async fn event_read_without_permission_denied() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/noread", false, false)).await?;
    let denied = runtime.list_events_for(&session.id, Some(&"example/noread".to_string())).await;
    anyhow::ensure!(denied.is_err(), "event read without permission unexpectedly succeeded");
    Ok(())
}

pub(crate) async fn closed_session_rejects_append() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/writer", true, true)).await?;
    runtime.close_session(session.id.clone()).await?;
    let denied = runtime
        .append_event(AppendEventRequest {
            session_id: session.id,
            writer_package_id: "example/writer".to_string(),
            kind: "example/writer/event".to_string(),
            payload: json!({}),
            metadata: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "append to closed session unexpectedly succeeded");
    Ok(())
}

pub(crate) async fn event_range_replay() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/range", true, true)).await?;
    for idx in 0..3 {
        runtime
            .append_event(AppendEventRequest {
                session_id: session.id.clone(),
                writer_package_id: "example/range".to_string(),
                kind: "example/range/event".to_string(),
                payload: json!({"idx": idx}),
                metadata: json!({}),
            })
            .await?;
    }
    let value = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.v1.event.list",
            json!({"session_id": session.id, "after_sequence": 1, "limit": 2, "kind_prefix": "example/range"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let events = value.as_array().ok_or_else(|| anyhow::anyhow!("event list did not return array"))?;
    anyhow::ensure!(events.len() == 2, "expected two ranged events, got {}", events.len());
    anyhow::ensure!(events[0]["sequence"] == json!(2), "range did not resume after sequence");
    Ok(())
}

pub(crate) async fn capability_invoke() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/echo-rust-inproc", "example/echo-rust-inproc/echo")).await?;
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/echo-rust-inproc/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({"ok": true}),
        })
        .await?;
    anyhow::ensure!(result.output == json!({"ok": true}), "echo output mismatch");
    Ok(())
}

pub(crate) async fn ambiguous_provider_denied() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/provider-a", "example/shared/echo")).await?;
    runtime.load_package(echo_package("example/provider-b", "example/shared/echo")).await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/shared/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "ambiguous route unexpectedly succeeded");
    Ok(())
}

pub(crate) async fn explicit_provider_selected() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/provider-a", "example/shared/selected")).await?;
    runtime.load_package(echo_package("example/provider-b", "example/shared/selected")).await?;
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/shared/selected".to_string(),
            caller_package_id: None,
            provider_package_id: Some("example/provider-b".to_string()),
            version: Some("^0.1".to_string()),
            input: json!({"selected": true}),
        })
        .await?;
    anyhow::ensure!(result.provider_package_id == "example/provider-b", "explicit provider was ignored");
    Ok(())
}

pub(crate) async fn unload_removes_capability() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/temp", "example/temp/echo")).await?;
    runtime.unload_package(&"example/temp".to_string()).await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/temp/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "unloaded capability remained invokable");
    Ok(())
}

pub(crate) async fn official_no_privilege() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("official/echo", "example/shared/echo")).await?;
    runtime.load_package(echo_package("thirdparty/echo", "example/shared/echo")).await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/shared/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "official-looking package won ambiguous route");
    Ok(())
}

pub(crate) async fn capability_schema_rejects_invalid() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime
        .load_package(schema_echo_package(
            "example/schema-echo",
            "example/schema-echo/echo",
            json!({"type": "object", "required": ["ok"]}),
            json!({"type": "object", "required": ["ok"]}),
        ))
        .await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/schema-echo/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "invalid capability input unexpectedly passed");
    Ok(())
}

pub(crate) async fn event_schema_rejects_invalid() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_schema_package()).await?;
    let denied = runtime
        .append_event(AppendEventRequest {
            session_id: session.id,
            writer_package_id: "example/schema-writer".to_string(),
            kind: "example/schema-writer/event.checked".to_string(),
            payload: json!({}),
            metadata: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "invalid event payload unexpectedly passed");
    Ok(())
}

pub(crate) async fn host_diagnostics() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/diag", "example/diag/echo")).await?;
    let diagnostics = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.v1.host.diagnostics", json!({}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(diagnostics["package_count"] == json!(1), "diagnostics package count mismatch");
    Ok(())
}

pub(crate) async fn host_profile_autoload() -> anyhow::Result<()> {
    use std::path::PathBuf;
    use std::sync::Arc;
    use ygg_runtime::{InMemoryEventStore, Runtime};
    use crate::commands::host;

    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
    host::load_host_profile(runtime.clone(), PathBuf::from("profiles/forge-alpha.yaml")).await?;
    let packages = runtime.list_packages().await;
    anyhow::ensure!(packages.iter().any(|package| package.id == "example/echo-rust-inproc"), "profile did not autoload rust package");
    anyhow::ensure!(packages.iter().any(|package| package.id == "example/echo-subprocess-python"), "profile did not autoload subprocess package");
    Ok(())
}

pub(crate) async fn asset_put_get_list() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let record_value = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.v1.asset.put",
            json!({"mime": "application/json", "content": "{\"hello\":true}", "metadata": {"purpose": "conformance"}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let asset_id = record_value["id"].as_str().ok_or_else(|| anyhow::anyhow!("asset put returned no id"))?;
    let get_value = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.v1.asset.get", json!({"asset_id": asset_id}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(get_value["content"] == json!("{\"hello\":true}"), "asset get content mismatch");
    let list_value = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.v1.asset.list", json!({}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(list_value.as_array().map(|items| items.len()).unwrap_or(0) == 1, "asset list missing record");
    Ok(())
}

pub(crate) async fn session_fork_branch() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    let branch_value = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.v1.session.fork",
            json!({"parent_session_id": session.id, "forked_from_sequence": 0, "metadata": {"why": "try"}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(branch_value["parent_session_id"] == json!(session.id), "branch parent mismatch");
    let branches = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.v1.session.branch.list", json!({"session_id": session.id}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(branches.as_array().map(|items| items.len()).unwrap_or(0) == 1, "branch list missing fork");
    Ok(())
}

pub(crate) async fn projection_rebuild() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/projection", true, true)).await?;
    runtime
        .append_event(AppendEventRequest {
            session_id: session.id.clone(),
            writer_package_id: "example/projection".to_string(),
            kind: "example/projection/event".to_string(),
            payload: json!({"ok": true}),
            metadata: json!({}),
        })
        .await?;
    runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.v1.projection.register",
            json!({"id": "example/projection/state", "session_id": session.id, "source_kind_prefix": "example/projection", "state": {}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let rebuilt = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.v1.projection.rebuild",
            json!({"projection_id": "example/projection/state"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(rebuilt["state"]["event_count"] == json!(1), "projection event count mismatch");
    Ok(())
}
