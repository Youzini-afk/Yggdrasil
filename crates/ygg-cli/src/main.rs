use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use serde_json::json;
use ygg_core::{
    CapabilityDescriptor, CapabilityPermissions, EventPermissions, KERNEL_PACKAGE_ID,
    PackageContributions, PackageEntry, PackageManifest, PermissionSet, SandboxPolicy,
};
use ygg_runtime::{
    AppendEventRequest, CapabilityInvocationRequest, EventStore, InMemoryEventStore,
    OpenSessionRequest, Runtime, RuntimeConfig, SqliteEventStore,
};

#[derive(Debug, Parser)]
#[command(name = "ygg")]
#[command(about = "Yggdrasil kernel CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run a content-free kernel event demo.
    Demo,
    /// Run a durable SQLite-backed kernel event demo.
    SqliteDemo { path: PathBuf },
    /// Run the headless kernel HTTP service.
    Serve {
        #[arg(long, default_value = "127.0.0.1:8787")]
        bind: SocketAddr,
    },
    /// Validate a package manifest file.
    Manifest {
        #[command(subcommand)]
        command: ManifestCommand,
    },
    /// Exercise the in-memory package registry.
    Package {
        #[command(subcommand)]
        command: PackageCommand,
    },
    /// Exercise capability discovery and invocation against a manifest.
    Capability {
        #[command(subcommand)]
        command: CapabilityCommand,
    },
    /// Generate package skeletons.
    InitPackage {
        path: PathBuf,
        #[arg(long, default_value = "example/new-package")]
        id: String,
        #[arg(long, default_value = "rust_inproc")]
        entry: String,
    },
    /// Run local kernel conformance checks.
    Conformance,
}

#[derive(Debug, Subcommand)]
enum ManifestCommand {
    Validate { path: PathBuf },
}

#[derive(Debug, Subcommand)]
enum PackageCommand {
    Load { path: PathBuf },
}

#[derive(Debug, Subcommand)]
enum CapabilityCommand {
    Invoke {
        manifest: PathBuf,
        capability_id: String,
        #[arg(long, default_value = "{}")]
        input: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Command::Demo => demo().await,
        Command::SqliteDemo { path } => sqlite_demo(path).await,
        Command::Serve { bind } => serve(bind).await,
        Command::Manifest { command } => match command {
            ManifestCommand::Validate { path } => validate_manifest(path).await,
        },
        Command::Package { command } => match command {
            PackageCommand::Load { path } => package_load(path).await,
        },
        Command::Capability { command } => match command {
            CapabilityCommand::Invoke { manifest, capability_id, input } => {
                capability_invoke(manifest, capability_id, input).await
            }
        },
        Command::InitPackage { path, id, entry } => init_package(path, id, entry).await,
        Command::Conformance => conformance().await,
    }
}

async fn read_manifest(path: PathBuf) -> anyhow::Result<PackageManifest> {
    let raw = fs::read_to_string(&path)?;
    let manifest = match path.extension().and_then(|ext| ext.to_str()) {
        Some("yaml") | Some("yml") => serde_yaml::from_str(&raw)?,
        _ => serde_json::from_str(&raw)?,
    };
    Ok(manifest)
}

async fn validate_manifest(path: PathBuf) -> anyhow::Result<()> {
    let manifest = read_manifest(path).await?;
    manifest.validate_basic()?;
    println!("valid manifest: {}@{}", manifest.id, manifest.version);
    Ok(())
}

async fn package_load(path: PathBuf) -> anyhow::Result<()> {
    let manifest = read_manifest(path).await?;
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store, RuntimeConfig::default());
    let record = runtime.load_package(manifest).await?;
    println!("loaded package: {}@{} ({:?})", record.id, record.version, record.state);
    Ok(())
}

async fn capability_invoke(manifest_path: PathBuf, capability_id: String, input: String) -> anyhow::Result<()> {
    let manifest = read_manifest(manifest_path).await?;
    let payload: serde_json::Value = serde_json::from_str(&input)?;
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store, RuntimeConfig::default());
    runtime.load_package(manifest).await?;
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest { capability_id, caller_package_id: None, input: payload })
        .await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

async fn init_package(path: PathBuf, id: String, entry: String) -> anyhow::Result<()> {
    fs::create_dir_all(&path)?;
    let manifest = match entry.as_str() {
        "wasm" => format!(
            r#"schema_version: 1
id: {id}
version: 0.1.0
entry:
  kind: wasm
  module: package.wasm
  abi_version: 1
  memory_limit_mb: 64
provides: []
consumes: []
contributes:
  schemas: []
  hooks: []
  extension_points: []
permissions: {{}}
sandbox_policy:
  cpu_quota_ms_per_invoke: 5000
  memory_mb: 64
  wall_clock_ms: 30000
"#
        ),
        "remote" => format!(
            r#"schema_version: 1
id: {id}
version: 0.1.0
entry:
  kind: remote
  endpoint: https://example.invalid/ygg/package
  auth:
    scheme: none
    config: null
provides: []
consumes: []
contributes:
  schemas: []
  hooks: []
  extension_points: []
permissions: {{}}
sandbox_policy:
  cpu_quota_ms_per_invoke: 5000
  memory_mb: 128
  wall_clock_ms: 30000
"#
        ),
        "subprocess" => format!(
            r#"schema_version: 1
id: {id}
version: 0.1.0
entry:
  kind: subprocess
  command: ["./package"]
  transport: json_rpc_stdio
provides: []
consumes: []
contributes:
  schemas: []
  hooks: []
  extension_points: []
permissions: {{}}
sandbox_policy:
  cpu_quota_ms_per_invoke: 5000
  memory_mb: 128
  wall_clock_ms: 30000
"#
        ),
        _ => format!(
            r#"schema_version: 1
id: {id}
version: 0.1.0
entry:
  kind: rust_inproc
  crate_ref: package-crate
  symbol: register
  abi_version: 1
provides: []
consumes: []
contributes:
  schemas: []
  hooks: []
  extension_points: []
permissions: {{}}
sandbox_policy:
  cpu_quota_ms_per_invoke: 5000
  memory_mb: 128
  wall_clock_ms: 30000
"#
        ),
    };
    fs::write(path.join("manifest.yaml"), manifest)?;
    fs::write(
        path.join("README.md"),
        format!("# {id}\n\nYggdrasil capability package skeleton.\n"),
    )?;
    println!("initialized package skeleton at {}", path.display());
    Ok(())
}

async fn conformance() -> anyhow::Result<()> {
    let mut results = Vec::new();
    record_case(&mut results, "session.open_empty", conformance_session_open().await);
    record_case(&mut results, "event.append_authorized", conformance_event_append_authorized().await);
    record_case(
        &mut results,
        "event.append_without_permission_denied",
        conformance_event_append_without_permission_denied().await,
    );
    record_case(
        &mut results,
        "event.kernel_namespace_denied",
        conformance_kernel_namespace_denied().await,
    );
    record_case(
        &mut results,
        "event.read_without_permission_denied",
        conformance_event_read_without_permission_denied().await,
    );
    record_case(
        &mut results,
        "event.closed_session_rejects_append",
        conformance_closed_session_rejects_append().await,
    );
    record_case(&mut results, "capability.invoke_rust_inproc", conformance_capability_invoke().await);
    record_case(
        &mut results,
        "capability.ambiguous_provider_denied",
        conformance_ambiguous_provider_denied().await,
    );
    record_case(
        &mut results,
        "package.unload_removes_capability",
        conformance_unload_removes_capability().await,
    );
    record_case(&mut results, "official.no_privilege", conformance_official_no_privilege().await);
    record_case(
        &mut results,
        "schema.capability_input_rejects_invalid",
        conformance_capability_schema_rejects_invalid().await,
    );
    record_case(
        &mut results,
        "schema.event_payload_rejects_invalid",
        conformance_event_schema_rejects_invalid().await,
    );

    let mut failed = false;
    for (name, result) in &results {
        match result {
            Ok(()) => println!("{name:<45} PASS"),
            Err(err) => {
                failed = true;
                println!("{name:<45} FAIL {err}");
            }
        }
    }
    if failed {
        anyhow::bail!("conformance failed");
    }
    println!("conformance: ok ({} cases)", results.len());
    Ok(())
}

fn record_case(results: &mut Vec<(&'static str, anyhow::Result<()>)>, name: &'static str, result: anyhow::Result<()>) {
    results.push((name, result));
}

fn runtime() -> (Arc<InMemoryEventStore>, Runtime<InMemoryEventStore>) {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store.clone(), RuntimeConfig::default());
    (store, runtime)
}

async fn conformance_session_open() -> anyhow::Result<()> {
    let (store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    let events = store.list_session(&session.id).await?;
    anyhow::ensure!(events.len() == 1, "expected one session-open event");
    Ok(())
}

async fn conformance_event_append_authorized() -> anyhow::Result<()> {
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

async fn conformance_event_append_without_permission_denied() -> anyhow::Result<()> {
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

async fn conformance_kernel_namespace_denied() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/writer", true, true)).await?;
    let denied = runtime
        .append_event(AppendEventRequest {
            session_id: session.id,
            writer_package_id: "example/writer".to_string(),
            kind: "kernel/forged".to_string(),
            payload: json!({}),
            metadata: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "package wrote kernel namespace");
    Ok(())
}

async fn conformance_event_read_without_permission_denied() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(event_package("example/noread", false, false)).await?;
    let denied = runtime.list_events_for(&session.id, Some(&"example/noread".to_string())).await;
    anyhow::ensure!(denied.is_err(), "event read without permission unexpectedly succeeded");
    Ok(())
}

async fn conformance_closed_session_rejects_append() -> anyhow::Result<()> {
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

async fn conformance_capability_invoke() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/echo-rust-inproc", "example/echo-rust-inproc/echo")).await?;
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/echo-rust-inproc/echo".to_string(),
            caller_package_id: None,
            input: json!({"ok": true}),
        })
        .await?;
    anyhow::ensure!(result.output == json!({"ok": true}), "echo output mismatch");
    Ok(())
}

async fn conformance_ambiguous_provider_denied() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/provider-a", "example/shared/echo")).await?;
    runtime.load_package(echo_package("example/provider-b", "example/shared/echo")).await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/shared/echo".to_string(),
            caller_package_id: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "ambiguous route unexpectedly succeeded");
    Ok(())
}

async fn conformance_unload_removes_capability() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("example/temp", "example/temp/echo")).await?;
    runtime.unload_package(&"example/temp".to_string()).await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/temp/echo".to_string(),
            caller_package_id: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "unloaded capability remained invokable");
    Ok(())
}

async fn conformance_official_no_privilege() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.load_package(echo_package("official/echo", "example/shared/echo")).await?;
    runtime.load_package(echo_package("thirdparty/echo", "example/shared/echo")).await?;
    let denied = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/shared/echo".to_string(),
            caller_package_id: None,
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "official-looking package won ambiguous route");
    Ok(())
}

async fn conformance_capability_schema_rejects_invalid() -> anyhow::Result<()> {
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
            input: json!({}),
        })
        .await;
    anyhow::ensure!(denied.is_err(), "invalid capability input unexpectedly passed");
    Ok(())
}

async fn conformance_event_schema_rejects_invalid() -> anyhow::Result<()> {
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

fn event_package(id: &str, read: bool, append: bool) -> PackageManifest {
    PackageManifest {
        id: id.to_string(),
        permissions: PermissionSet { events: EventPermissions { read, append }, ..PermissionSet::default() },
        ..demo_event_writer_manifest()
    }
}

fn echo_package(id: &str, capability_id: &str) -> PackageManifest {
    schema_echo_package(id, capability_id, serde_json::Value::Null, serde_json::Value::Null)
}

fn schema_echo_package(
    id: &str,
    capability_id: &str,
    input_schema: serde_json::Value,
    output_schema: serde_json::Value,
) -> PackageManifest {
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
            id: capability_id.to_string(),
            version: "0.1.0".to_string(),
            input_schema,
            output_schema,
            streaming: false,
            side_effects: Vec::new(),
            description: None,
        }],
        consumes: Vec::new(),
        contributes: PackageContributions::default(),
        permissions: PermissionSet {
            capabilities: CapabilityPermissions { invoke: vec!["*".to_string()] },
            ..PermissionSet::default()
        },
        sandbox_policy: SandboxPolicy::default(),
    }
}

fn event_schema_package() -> PackageManifest {
    PackageManifest {
        id: "example/schema-writer".to_string(),
        contributes: PackageContributions {
            schemas: vec![ygg_core::SchemaContribution {
                id: "example/schema-writer/event.checked".to_string(),
                schema: json!({"type": "object", "required": ["ok"]}),
            }],
            hooks: Vec::new(),
            extension_points: Vec::new(),
        },
        permissions: PermissionSet {
            events: EventPermissions { read: false, append: true },
            ..PermissionSet::default()
        },
        ..demo_event_writer_manifest()
    }
}

async fn demo() -> anyhow::Result<()> {
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

fn demo_event_writer_manifest() -> PackageManifest {
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

async fn sqlite_demo(path: PathBuf) -> anyhow::Result<()> {
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

fn sqlite_event_writer_manifest() -> PackageManifest {
    PackageManifest {
        id: "example/sqlite".to_string(),
        ..demo_event_writer_manifest()
    }
}

async fn serve(bind: SocketAddr) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(bind).await?;
    println!("Yggdrasil kernel service listening on http://{bind}");
    axum::serve(listener, ygg_service::app()).await?;
    Ok(())
}
