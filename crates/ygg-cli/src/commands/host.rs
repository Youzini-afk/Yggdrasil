use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use ygg_runtime::{EventStore, InMemoryEventStore, ProtocolContext, Runtime, RuntimeConfig, SqliteEventStore};

use super::manifest::read_manifest;
use crate::cli::{HostEventStoreProfile, HostProfile};

pub(crate) async fn host_serve(http: SocketAddr, profile: Option<PathBuf>) -> Result<()> {
    if let Some(profile_path) = profile {
        let raw = fs::read_to_string(&profile_path)?;
        let profile: HostProfile = serde_yaml::from_str(&raw)?;
        match &profile.event_store {
            HostEventStoreProfile::Memory => {
                let runtime = Arc::new(Runtime::new(Arc::new(InMemoryEventStore::default()), RuntimeConfig::default()));
                load_profile_packages(runtime.clone(), profile, profile_path).await?;
                serve_runtime(http, runtime, "memory").await
            }
            HostEventStoreProfile::Sqlite { path } => {
                let resolved = resolve_profile_path(&profile_path, path.clone());
                let runtime = Arc::new(Runtime::new(Arc::new(SqliteEventStore::open(resolved)?), RuntimeConfig::default()));
                load_profile_packages(runtime.clone(), profile, profile_path).await?;
                serve_runtime(http, runtime, "sqlite").await
            }
            HostEventStoreProfile::Postgres { env } => {
                #[cfg(feature = "postgres")]
                {
                    let url = std::env::var(env).map_err(|_| anyhow::anyhow!("postgres event store env ref unavailable (details redacted)"))?;
                    let store = ygg_runtime::PostgresEventStore::connect(&url).await?;
                    let runtime = Arc::new(Runtime::new(Arc::new(store), RuntimeConfig::default()));
                    load_profile_packages(runtime.clone(), profile, profile_path).await?;
                    serve_runtime(http, runtime, "postgres").await
                }
                #[cfg(not(feature = "postgres"))]
                {
                    let _ = env;
                    anyhow::bail!("postgres event store requested but this binary was built without postgres support")
                }
            }
        }
    } else {
        let runtime = Arc::new(Runtime::new(Arc::new(InMemoryEventStore::default()), RuntimeConfig::default()));
        serve_runtime(http, runtime, "memory").await
    }
}

async fn serve_runtime<S>(http: SocketAddr, runtime: Arc<Runtime<S>>, backend_kind: &'static str) -> Result<()>
where
    S: EventStore,
{
    let listener = tokio::net::TcpListener::bind(http).await?;
    println!("Yggdrasil host serving http://{http}");
    println!("  event store: {backend_kind} (config redacted)");
    println!("  RPC: POST http://{http}/rpc");
    println!("  SSE: GET  http://{http}/kernel/event.subscribe/:session_id");
    let app = ygg_service::app_with_state(ygg_service::AppState { runtime });
    axum::serve(listener, app).await?;
    Ok(())
}

fn resolve_profile_path(profile_path: &std::path::Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        profile_path.parent().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")).join(path)
    }
}

pub(crate) async fn load_host_profile<S>(runtime: Arc<Runtime<S>>, profile_path: PathBuf) -> Result<()>
where
    S: EventStore,
{
    let raw = fs::read_to_string(&profile_path)?;
    let profile: HostProfile = serde_yaml::from_str(&raw)?;
    load_profile_packages(runtime, profile, profile_path).await
}

async fn load_profile_packages<S>(runtime: Arc<Runtime<S>>, profile: HostProfile, profile_path: PathBuf) -> Result<()>
where
    S: EventStore,
{
    if let Some(title) = &profile.title {
        println!("loading host profile: {title}");
    }
    let base = profile_path.parent().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    for manifest_path in profile.autoload {
        let resolved = if manifest_path.is_absolute() { manifest_path } else { base.join(manifest_path) };
        let manifest = read_manifest(resolved).await?;
        let record = runtime.load_package(manifest).await?;
        println!("autoloaded package: {}@{} ({:?})", record.id, record.version, record.state);
    }
    Ok(())
}

pub(crate) async fn host_stdio() -> Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store, RuntimeConfig::default());
    let context = ProtocolContext::host_dev("host_stdio");
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    let mut stdout = tokio::io::stdout();
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<ygg_runtime::ProtocolRequest>(&line) {
            Ok(request) => match runtime.call_protocol(&context, &request.method, request.params).await {
                Ok(result) => ygg_runtime::ProtocolResponse { id: request.id, result: Some(result), error: None },
                Err(error) => ygg_runtime::ProtocolResponse { id: request.id, result: None, error: Some(error) },
            },
            Err(error) => ygg_runtime::ProtocolResponse {
                id: "invalid".to_string(),
                result: None,
                error: Some(ygg_runtime::ProtocolError::invalid_request(error.to_string())),
            },
        };
        stdout.write_all(serde_json::to_string(&response)?.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }
    Ok(())
}
