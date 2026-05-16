use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use serde_json::json;
use ygg_core::{KERNEL_PACKAGE_ID, PackageManifest};
use ygg_runtime::{
    AppendEventRequest, CapabilityInvocationRequest, EventStore, InMemoryEventStore,
    OpenSessionRequest, Runtime, RuntimeConfig,
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
        .invoke_capability(CapabilityInvocationRequest { capability_id, input: payload })
        .await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

async fn demo() -> anyhow::Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store.clone(), RuntimeConfig::default());

    let session = runtime.open_session(OpenSessionRequest::default()).await?;
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

async fn serve(bind: SocketAddr) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(bind).await?;
    println!("Yggdrasil kernel service listening on http://{bind}");
    axum::serve(listener, ygg_service::app()).await?;
    Ok(())
}
