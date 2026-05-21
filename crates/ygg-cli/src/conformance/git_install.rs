use std::sync::Arc;

use serde_json::json;
use ygg_core::{
    CapabilityDescriptor, GitFetchPermissions, PackageContributions, PackageEntry, PackageManifest,
    PermissionSet, SandboxPolicy, EVENT_GIT_FETCH_COMPLETED, EVENT_GIT_FETCH_DENIED,
};
use ygg_runtime::{
    EventStore, ExecutorKind, FakeGitOutboundExecutor, GitFetchKind, GitOutboundExecutorConfig,
    GitOutboundPolicyConfig, GitOutboundRequest, GitOutboundResponse, InMemoryEventStore,
    ProtocolContext, ProtocolPrincipal, Runtime, RuntimeConfig,
};

use crate::commands::package;

fn git_package(id: &str, hosts: Vec<String>) -> PackageManifest {
    PackageManifest {
        schema_version: 1,
        id: id.to_string(),
        version: "0.1.0".to_string(),
        display_name: None,
        description: None,
        author: None,
        license: None,
        entry: PackageEntry::RustInproc {
            crate_ref: "example-echo-rust-inproc".to_string(),
            symbol: "register".to_string(),
            abi_version: 1,
        },
        provides: vec![CapabilityDescriptor {
            id: format!("{id}/install"),
            version: "0.1.0".to_string(),
            input_schema: serde_json::Value::Null,
            output_schema: serde_json::Value::Null,
            streaming: false,
            side_effects: vec!["git_fetch".to_string()],
            description: None,
        }],
        consumes: Vec::new(),
        contributes: PackageContributions::default(),
        permissions: PermissionSet {
            git_fetch: GitFetchPermissions { hosts },
            ..PermissionSet::default()
        },
        sandbox_policy: SandboxPolicy::default(),
    }
}

fn runtime_with_git(
    enabled: bool,
    allowed_hosts: Vec<String>,
    executor: GitOutboundExecutorConfig,
) -> (Arc<InMemoryEventStore>, Runtime<InMemoryEventStore>) {
    let store = Arc::new(InMemoryEventStore::default());
    let config = RuntimeConfig {
        git_outbound_policy: GitOutboundPolicyConfig {
            enabled,
            allowed_hosts,
            ..GitOutboundPolicyConfig::default()
        },
        git_outbound_executor: executor,
        ..RuntimeConfig::default()
    };
    let runtime = Runtime::new(store.clone(), config);
    (store, runtime)
}

fn request(package_id: &str, remote_url: &str) -> GitOutboundRequest {
    GitOutboundRequest {
        package_id: package_id.to_string(),
        capability_id: format!("{package_id}/install"),
        remote_url: remote_url.to_string(),
        reference: "main".to_string(),
        fetch_kind: GitFetchKind::RefsOnly,
        destination_hint: None,
        secret_refs: Vec::new(),
        redaction_state: None,
        timeout_ms: None,
        metadata: serde_json::Value::Null,
    }
}

pub(crate) async fn git_fetch_deny_all_default() -> anyhow::Result<()> {
    let (store, runtime) = runtime_with_git(false, vec![], GitOutboundExecutorConfig::DenyAll);
    runtime
        .load_package(git_package(
            "example/git-deny",
            vec!["github.com".to_string()],
        ))
        .await?;
    let result = runtime
        .execute_git_outbound_with_policy(
            ProtocolPrincipal::Package {
                package_id: "example/git-deny".to_string(),
            },
            request("example/git-deny", "https://github.com/example/pkg"),
        )
        .await;
    anyhow::ensure!(
        result.is_err(),
        "deny-all git policy must reject by default"
    );
    let events = store
        .list_session(&"kernel_git_fetch_example_git-deny".to_string())
        .await?;
    anyhow::ensure!(
        events
            .iter()
            .any(|event| event.kind == EVENT_GIT_FETCH_DENIED),
        "expected git fetch denied audit event"
    );
    Ok(())
}

pub(crate) async fn git_fetch_requires_https() -> anyhow::Result<()> {
    let (_store, runtime) = runtime_with_git(
        true,
        vec!["github.com".to_string()],
        GitOutboundExecutorConfig::DenyAll,
    );
    runtime
        .load_package(git_package(
            "example/git-http",
            vec!["github.com".to_string()],
        ))
        .await?;
    let result = runtime
        .execute_git_outbound_with_policy(
            ProtocolPrincipal::Package {
                package_id: "example/git-http".to_string(),
            },
            request("example/git-http", "http://github.com/example/pkg"),
        )
        .await;
    anyhow::ensure!(result.is_err(), "http:// git URL must be rejected");
    Ok(())
}

pub(crate) async fn git_fetch_requires_host_allowlist() -> anyhow::Result<()> {
    let (_store, runtime) = runtime_with_git(
        true,
        vec!["gitlab.com".to_string()],
        GitOutboundExecutorConfig::DenyAll,
    );
    runtime
        .load_package(git_package(
            "example/git-host",
            vec!["github.com".to_string()],
        ))
        .await?;
    let result = runtime
        .execute_git_outbound_with_policy(
            ProtocolPrincipal::Package {
                package_id: "example/git-host".to_string(),
            },
            request("example/git-host", "https://github.com/example/pkg"),
        )
        .await;
    anyhow::ensure!(
        result.is_err(),
        "host not in policy allowlist must be rejected"
    );
    Ok(())
}

pub(crate) async fn git_fetch_fake_executor_returns_fixture() -> anyhow::Result<()> {
    let mut fake = FakeGitOutboundExecutor::new();
    fake.add_fixture(
        "https://github.com/example/pkg",
        "main",
        GitOutboundResponse {
            status: "ok".to_string(),
            resolved_commit_sha: Some("0123456789abcdef0123456789abcdef01234567".to_string()),
            resolved_content_hash: Some("sha256:fixture-tree".to_string()),
            resolved_path: Some("example-pkg-0123456".to_string()),
            redaction_state: ygg_core::RedactionState::Redacted,
            network_performed: false,
            executor_kind: ExecutorKind::Fake,
            metadata: json!({"fixture": true}),
        },
    );
    let (store, runtime) = runtime_with_git(
        true,
        vec!["github.com".to_string()],
        GitOutboundExecutorConfig::Custom(Arc::new(fake)),
    );
    runtime
        .load_package(git_package(
            "example/git-ok",
            vec!["github.com".to_string()],
        ))
        .await?;
    let response = runtime
        .execute_git_outbound_with_policy(
            ProtocolPrincipal::Package {
                package_id: "example/git-ok".to_string(),
            },
            request("example/git-ok", "https://github.com/example/pkg"),
        )
        .await?;
    anyhow::ensure!(
        response.status == "ok",
        "fake executor should return ok fixture"
    );
    anyhow::ensure!(
        response.executor_kind == ExecutorKind::Fake,
        "fixture must report fake executor"
    );
    let events = store
        .list_session(&"kernel_git_fetch_example_git-ok".to_string())
        .await?;
    anyhow::ensure!(
        events
            .iter()
            .any(|event| event.kind == EVENT_GIT_FETCH_COMPLETED),
        "expected git fetch completed audit event"
    );
    Ok(())
}

pub(crate) async fn git_fetch_audit_no_raw_secrets() -> anyhow::Result<()> {
    let (store, runtime) = runtime_with_git(
        true,
        vec!["github.com".to_string()],
        GitOutboundExecutorConfig::DenyAll,
    );
    runtime
        .load_package(git_package(
            "example/git-secret",
            vec!["github.com".to_string()],
        ))
        .await?;
    let result = runtime
        .call_protocol(
            &ProtocolContext::package("example/git-secret", "conformance"),
            "kernel.outbound.git_fetch",
            json!({
                "capability_id": "example/git-secret/install",
                "remote_url": "https://github.com/example/pkg?token=raw-secret-placeholder",
                "ref": "main"
            }),
        )
        .await;
    anyhow::ensure!(result.is_err(), "query-token URL must be rejected");
    let events = store
        .list_session(&"kernel_git_fetch_example_git-secret".to_string())
        .await?;
    let serialized = serde_json::to_string(&events)?;
    anyhow::ensure!(
        !serialized.contains("raw-secret-placeholder"),
        "git fetch audit must not include raw query token"
    );
    Ok(())
}

pub(crate) async fn installer_lockfile_round_trip() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let profile = dir.path().join("fixture-profile.yaml");
    std::fs::write(&profile, "title: fixture\n")?;
    let commit_a = "0123456789abcdef0123456789abcdef01234567".to_string();
    let commit_b = "abcdef0123456789abcdef0123456789abcdef01".to_string();

    package::package_install_git(
        profile.clone(),
        "https://github.com/example/pkg".to_string(),
        "thirdparty/pkg".to_string(),
        "main".to_string(),
        commit_a.clone(),
        "sha256:fixture-a".to_string(),
        "manifest.yaml".to_string(),
    )
    .await?;
    package::package_list_installed(profile.clone()).await?;
    package::package_inspect_lockfile(profile.clone()).await?;

    let lock_path = dir.path().join("fixture-profile.lock.yaml");
    let raw = std::fs::read_to_string(&lock_path)?;
    anyhow::ensure!(raw.contains("thirdparty/pkg"), "lockfile missing package id");
    anyhow::ensure!(raw.contains(&commit_a), "lockfile missing pinned commit");

    package::package_update_git(
        profile.clone(),
        "thirdparty/pkg".to_string(),
        None,
        "main".to_string(),
        commit_b.clone(),
        "sha256:fixture-b".to_string(),
        "manifest.yaml".to_string(),
    )
    .await?;
    let updated = std::fs::read_to_string(&lock_path)?;
    anyhow::ensure!(updated.contains(&commit_b), "lockfile missing updated commit");
    anyhow::ensure!(!updated.contains(&commit_a), "lockfile retained old commit after update");

    package::package_uninstall_git(profile.clone(), "thirdparty/pkg".to_string()).await?;
    let removed = std::fs::read_to_string(&lock_path)?;
    anyhow::ensure!(!removed.contains("thirdparty/pkg"), "lockfile retained package after uninstall");
    Ok(())
}

pub(crate) async fn installer_lockfile_rejects_unsafe_inputs() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let profile = dir.path().join("fixture-profile.yaml");
    std::fs::write(&profile, "title: fixture\n")?;
    let result = package::package_install_git(
        profile,
        "https://github.com/example/pkg?token=raw-secret-placeholder".to_string(),
        "thirdparty/pkg".to_string(),
        "main".to_string(),
        "0123456789abcdef0123456789abcdef01234567".to_string(),
        "sha256:fixture".to_string(),
        "manifest.yaml".to_string(),
    )
    .await;
    anyhow::ensure!(result.is_err(), "git install lockfile command must reject query tokens");
    Ok(())
}
