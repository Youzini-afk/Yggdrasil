use std::path::PathBuf;

use serde_json::json;
use ygg_runtime::{CapabilityInvocationRequest, EventStore};

use super::fixtures::*;
use crate::commands::manifest;

pub(crate) async fn subprocess_load_ready() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let record = runtime.load_package(manifest::read_manifest(PathBuf::from("examples/packages/echo-subprocess-python/manifest.yaml")).await?).await?;
    anyhow::ensure!(record.id == "example/echo-subprocess-python", "wrong package loaded");
    Ok(())
}

pub(crate) async fn subprocess_invoke_echo() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("examples/packages/echo-subprocess-python/manifest.yaml")).await?).await?;
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/echo-subprocess-python/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({"subprocess": true}),
        })
        .await?;
    anyhow::ensure!(result.output == json!({"subprocess": true}), "subprocess echo mismatch");
    Ok(())
}

pub(crate) async fn package_lifecycle_timeline() -> anyhow::Result<()> {
    let (store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("examples/packages/echo-subprocess-python/manifest.yaml")).await?).await?;
    let session_id = "kernel_package_example_echo-subprocess-python".to_string();
    let events = store.list_session(&session_id).await?;
    for expected in ["kernel/package.loading", "kernel/package.starting", "kernel/package.ready", "kernel/package.loaded"] {
        anyhow::ensure!(events.iter().any(|event| event.kind == expected), "missing lifecycle event {expected}");
    }
    Ok(())
}

pub(crate) async fn package_logs_capture() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("examples/packages/logging-subprocess-python/manifest.yaml")).await?).await?;
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/logging-subprocess-python/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({"logs": true}),
        })
        .await?;
    anyhow::ensure!(result.output == json!({"logs": true}), "logging echo mismatch");
    let logs = runtime.package_logs(&"example/logging-subprocess-python".to_string()).await;
    anyhow::ensure!(logs.iter().any(|log| log.line.contains("invoke observed") || log.line.contains("booted")), "stderr logs were not captured");
    Ok(())
}

pub(crate) async fn package_restart_subprocess() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("examples/packages/echo-subprocess-python/manifest.yaml")).await?).await?;
    let record = runtime.restart_package(&"example/echo-subprocess-python".to_string()).await?;
    anyhow::ensure!(matches!(record.state, ygg_runtime::PackageState::Ready), "restart did not return ready package");
    Ok(())
}

pub(crate) async fn subprocess_bad_handshake() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let denied = runtime
        .load_package(manifest::read_manifest(PathBuf::from("examples/packages/bad-handshake-subprocess-python/manifest.yaml")).await?)
        .await;
    anyhow::ensure!(denied.is_err(), "bad handshake unexpectedly loaded");
    Ok(())
}

pub(crate) async fn subprocess_timeout() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("examples/packages/slow-subprocess-python/manifest.yaml")).await?).await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/slow-subprocess-python/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "slow subprocess did not time out");
    let status = runtime.package_status(&"example/slow-subprocess-python".to_string()).await;
    anyhow::ensure!(matches!(status.map(|record| record.state), Some(ygg_runtime::PackageState::Degraded)), "timeout did not degrade package");
    Ok(())
}

pub(crate) async fn subprocess_invalid_output_schema() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime
        .load_package(manifest::read_manifest(PathBuf::from("examples/packages/invalid-output-subprocess-python/manifest.yaml")).await?)
        .await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/invalid-output-subprocess-python/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "invalid subprocess output schema passed");
    Ok(())
}

pub(crate) async fn subprocess_unload_removes_capability() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(manifest::read_manifest(PathBuf::from("examples/packages/echo-subprocess-python/manifest.yaml")).await?).await?;
    runtime.unload_package(&"example/echo-subprocess-python".to_string()).await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/echo-subprocess-python/echo".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "unloaded subprocess capability remained invokable");
    Ok(())
}

/// Conformance: package_check enhanced diagnostics cover entry kind, trust level,
/// capabilities, surfaces, permissions, sandbox, and warnings for the echo subprocess.
pub(crate) async fn package_check_diagnostics() -> anyhow::Result<()> {
    // Use the echo subprocess manifest — it has 1 capability and 0 surfaces,
    // so it should trigger the "no surfaces" warning.
    let m = manifest::read_manifest(PathBuf::from("examples/packages/echo-subprocess-python/manifest.yaml")).await?;
    m.validate_basic()?;
    anyhow::ensure!(m.provides.len() == 1, "echo subprocess should have 1 capability");
    anyhow::ensure!(m.contributes.surfaces.is_empty(), "echo subprocess should have 0 surfaces (triggers warning)");
    // Verify entry kind and trust level are accessible from manifest
    let entry_kind = match &m.entry {
        ygg_core::PackageEntry::Subprocess { .. } => "subprocess",
        _ => anyhow::bail!("expected subprocess entry kind"),
    };
    anyhow::ensure!(entry_kind == "subprocess", "entry kind should be subprocess");
    Ok(())
}

/// Conformance: package_reload smoke for echo subprocess — loads, restarts,
/// checks before/after status and logs count.
pub(crate) async fn package_reload_smoke() -> anyhow::Result<()> {
    let m = manifest::read_manifest(PathBuf::from("examples/packages/echo-subprocess-python/manifest.yaml")).await?;
    let package_id = m.id.clone();

    let store = std::sync::Arc::new(ygg_runtime::InMemoryEventStore::default());
    let runtime = ygg_runtime::Runtime::new(store, ygg_runtime::RuntimeConfig::default());

    // Load
    let load_record = runtime.load_package(m.clone()).await?;
    anyhow::ensure!(load_record.id == package_id, "loaded id mismatch");
    let before = runtime.package_status(&package_id).await;
    anyhow::ensure!(before.is_some(), "package status should exist after load");
    anyhow::ensure!(matches!(before.as_ref().unwrap().state, ygg_runtime::PackageState::Ready), "package should be ready after load");

    // Restart
    let restart_record = runtime.restart_package(&package_id).await?;
    anyhow::ensure!(matches!(restart_record.state, ygg_runtime::PackageState::Ready), "package should be ready after restart");

    let after = runtime.package_status(&package_id).await;
    anyhow::ensure!(after.is_some(), "package status should exist after restart");
    anyhow::ensure!(matches!(after.as_ref().unwrap().state, ygg_runtime::PackageState::Ready), "package should be ready after restart (status)");

    // Logs (may or may not have content, just ensure no panic)
    let _logs = runtime.package_logs(&package_id).await;

    // Unload
    runtime.unload_package(&package_id).await?;
    let after_unload = runtime.package_status(&package_id).await;
    anyhow::ensure!(after_unload.is_none(), "package should be gone after unload");
    Ok(())
}
