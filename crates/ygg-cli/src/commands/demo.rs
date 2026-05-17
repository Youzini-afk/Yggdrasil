use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use serde_json::json;
use ygg_core::KERNEL_PACKAGE_ID;
use ygg_runtime::{
    AppendEventRequest, CapabilityInvocationRequest, EventStore, InMemoryEventStore,
    OpenSessionRequest, ProtocolContext, Runtime, RuntimeConfig, SqliteEventStore,
};

use super::manifest::read_manifest;

pub(crate) async fn demo() -> Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store.clone(), RuntimeConfig::default());

    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(demo_event_writer_manifest()).await?;
    runtime
        .append_event(AppendEventRequest {
            session_id: session.id.clone(),
            writer_package_id: "example/echo".to_string(),
            kind: "example/echo/event.demo".to_string(),
            payload: json!({"message": "content-free kernel event"}),
            metadata: json!({"created_by": "ygg-cli demo"}),
        })
        .await?;

    let events = store.list_session(&session.id).await?;

    println!("session_id: {}", session.id);
    println!("kernel_package_id: {KERNEL_PACKAGE_ID}");
    println!("\nevents:");
    for event in events {
        println!("- #{} {} {}", event.sequence, event.writer_package_id, event.kind);
    }

    Ok(())
}

pub(crate) fn demo_event_writer_manifest() -> ygg_core::PackageManifest {
    use ygg_core::{
        EventPermissions, PackageContributions, PackageEntry,
        PackageManifest, PermissionSet, SandboxPolicy,
    };

    PackageManifest {
        schema_version: 1,
        id: "example/echo".to_string(),
        version: "0.1.0".to_string(),
        display_name: Some("Demo Event Writer".to_string()),
        description: None,
        author: None,
        license: None,
        entry: PackageEntry::RustInproc {
            crate_ref: "example-echo".to_string(),
            symbol: "register".to_string(),
            abi_version: 1,
        },
        provides: Vec::new(),
        consumes: Vec::new(),
        contributes: PackageContributions::default(),
        permissions: PermissionSet {
            events: EventPermissions { read: false, append: true },
            ..PermissionSet::default()
        },
        sandbox_policy: SandboxPolicy::default(),
    }
}

pub(crate) async fn sqlite_demo(path: PathBuf) -> Result<()> {
    let store = Arc::new(SqliteEventStore::open(&path)?);
    let runtime = Runtime::new(store.clone(), RuntimeConfig::default());
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(sqlite_event_writer_manifest()).await?;
    runtime
        .append_event(AppendEventRequest {
            session_id: session.id.clone(),
            writer_package_id: "example/sqlite".to_string(),
            kind: "example/sqlite/event.demo".to_string(),
            payload: json!({"durable": true}),
            metadata: json!({}),
        })
        .await?;
    drop(runtime);
    drop(store);

    let reopened = SqliteEventStore::open(&path)?;
    let events = reopened.list_session(&session.id).await?;
    println!("sqlite_path: {}", path.display());
    println!("session_id: {}", session.id);
    for event in events {
        println!("- #{} {} {}", event.sequence, event.writer_package_id, event.kind);
    }
    Ok(())
}

pub(crate) fn sqlite_event_writer_manifest() -> ygg_core::PackageManifest {
    ygg_core::PackageManifest {
        id: "example/sqlite".to_string(),
        ..demo_event_writer_manifest()
    }
}

pub(crate) async fn serve(bind: std::net::SocketAddr) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(bind).await?;
    println!("Yggdrasil kernel service listening on http://{bind}");
    axum::serve(listener, ygg_service::app()).await?;
    Ok(())
}

#[derive(Debug)]
pub(crate) struct BlankLoopResult {
    pub(crate) session_id: String,
    pub(crate) branch_id: String,
    pub(crate) asset_id: String,
    pub(crate) projection_id: String,
}

pub(crate) async fn run_blank_play_creation_loop<S: EventStore>(runtime: &Runtime<S>) -> Result<BlankLoopResult> {
    for manifest in [
        "packages/official/assistant-lab/manifest.yaml",
        "packages/official/blank-experience/manifest.yaml",
    ] {
        runtime.load_package(read_manifest(PathBuf::from(manifest)).await?).await?;
    }
    let session = runtime
        .open_session(OpenSessionRequest {
            labels: vec!["play-create".to_string()],
            active_package_set: vec!["official/blank-experience".to_string(), "official/assistant-lab".to_string()],
            metadata: json!({"surface": "play"}),
        })
        .await?;
    let seed = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/blank-experience/create_seed".to_string(),
            caller_package_id: None,
            provider_package_id: None,
            version: None,
            input: json!({"title": "Blank Loop", "intent": "prove play-create substrate"}),
        })
        .await?;
    let assistant = json!({"kind": "assistant", "assistant_id": "assistant/blank-loop", "delegated_user_id": "user/demo"});
    runtime
        .call_protocol(
            &ProtocolContext::host_dev("demo"),
            "kernel.permission.grant",
            json!({"principal": assistant, "permission": "capabilities.invoke", "scope": "official/assistant-lab"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let assistant_context = ProtocolContext { principal: serde_json::from_value(assistant)?, transport: "demo".to_string() };
    let proposal = runtime
        .call_protocol(
            &assistant_context,
            "kernel.capability.invoke",
            json!({"capability_id": "official/assistant-lab/draft_branch_change", "input": {"seed": seed.output, "change": "try a first branch"}}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(proposal["output"]["requires_user_approval"] == json!(true), "assistant proposal must require approval");
    let branch = runtime.fork_session(session.id.clone(), 0, json!({"proposal": proposal["output"].clone()})).await?;
    let asset = runtime
        .put_asset(ygg_runtime::runtime::AssetPutRequest {
            origin_package_id: Some("official/blank-experience".to_string()),
            mime: "application/json".to_string(),
            content: serde_json::to_string(&json!({"seed": seed.output, "branch_id": branch.id}))?,
            metadata: json!({"kind": "blank_experience_seed"}),
        })
        .await?;
    let projection_id = "official/blank-experience/projection/demo".to_string();
    runtime
        .projection_register(ygg_runtime::runtime::ProjectionDefinition {
            id: projection_id.clone(),
            session_id: session.id.clone(),
            source_kind_prefix: Some("kernel/session".to_string()),
            state: json!({}),
        })
        .await?;
    runtime.projection_rebuild(&projection_id).await?;
    Ok(BlankLoopResult { session_id: session.id, branch_id: branch.id, asset_id: asset.id, projection_id })
}

pub(crate) async fn play_create_demo() -> Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store, RuntimeConfig::default());
    let result = run_blank_play_creation_loop(&runtime).await?;
    println!("blank play-creation loop ok");
    println!("session_id: {}", result.session_id);
    println!("branch_id: {}", result.branch_id);
    println!("asset_id: {}", result.asset_id);
    println!("projection_id: {}", result.projection_id);
    Ok(())
}
