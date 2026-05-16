use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use serde_json::json;
use ygg_core::{
    EventPermissions, KERNEL_PACKAGE_ID, PackageContributions, PackageEntry, PackageManifest,
    PermissionSet, SandboxPolicy,
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
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store.clone(), RuntimeConfig::default());
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    runtime.load_package(demo_event_writer_manifest()).await?;
    runtime
        .append_event(AppendEventRequest {
            session_id: session.id.clone(),
            writer_package_id: "example/echo".to_string(),
            kind: "example/echo/conformance.event".to_string(),
            payload: json!({"conformance": true}),
            metadata: json!({}),
        })
        .await?;
    let events = store.list_session(&session.id).await?;
    anyhow::ensure!(events.len() == 2, "expected session open + conformance event");

    let manifest = read_manifest(PathBuf::from("examples/packages/echo-rust-inproc/manifest.yaml")).await?;
    runtime.load_package(manifest).await?;
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "example/echo-rust-inproc/echo".to_string(),
            caller_package_id: None,
            input: json!({"ok": true}),
        })
        .await?;
    anyhow::ensure!(result.output == json!({"ok": true}), "echo output mismatch");

    println!("conformance: ok");
    Ok(())
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
