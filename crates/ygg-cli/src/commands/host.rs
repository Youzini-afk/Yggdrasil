use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use ygg_runtime::{
    InMemoryEventStore, ProtocolContext, Runtime, RuntimeConfig,
};

use super::manifest::read_manifest;
use crate::cli::HostProfile;

pub(crate) async fn host_serve(http: SocketAddr, profile: Option<PathBuf>) -> Result<()> {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
    if let Some(profile_path) = profile {
        load_host_profile(runtime.clone(), profile_path).await?;
    }
    let listener = tokio::net::TcpListener::bind(http).await?;
    println!("Yggdrasil host serving http://{http}");
    println!("  RPC: POST http://{http}/rpc");
    println!("  SSE: GET  http://{http}/kernel/event.subscribe/:session_id");
    let app = ygg_service::app_with_state(ygg_service::AppState { runtime });
    axum::serve(listener, app).await?;
    Ok(())
}

pub(crate) async fn load_host_profile(runtime: Arc<Runtime<InMemoryEventStore>>, profile_path: PathBuf) -> Result<()> {
    let raw = fs::read_to_string(&profile_path)?;
    let profile: HostProfile = serde_yaml::from_str(&raw)?;
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
