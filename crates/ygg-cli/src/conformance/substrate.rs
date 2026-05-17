use std::fs;
use std::sync::Arc;

use serde_json::json;
use ygg_runtime::{OpenSessionRequest, Runtime, RuntimeConfig, SqliteEventStore};

pub(crate) async fn sqlite_rehydrate() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("ygg-substrate-{}.db", std::process::id()));
    if path.exists() {
        fs::remove_file(&path)?;
    }
    let store = Arc::new(SqliteEventStore::open(&path)?);
    let runtime = Runtime::new(store.clone(), RuntimeConfig::default());
    let session = runtime.open_session(OpenSessionRequest::default()).await?;
    let asset = runtime
        .put_asset(ygg_runtime::runtime::AssetPutRequest {
            origin_package_id: None,
            mime: "text/plain".to_string(),
            content: "durable".to_string(),
            metadata: json!({"phase": "A"}),
        })
        .await?;
    let branch = runtime.fork_session(session.id.clone(), 0, json!({"durable": true})).await?;
    runtime
        .projection_register(ygg_runtime::runtime::ProjectionDefinition {
            id: "example/durable/projection".to_string(),
            session_id: session.id.clone(),
            source_kind_prefix: Some("kernel/session".to_string()),
            state: json!({}),
        })
        .await?;
    runtime.projection_rebuild("example/durable/projection").await?;
    drop(runtime);
    drop(store);

    let reopened = Arc::new(SqliteEventStore::open(&path)?);
    let hydrated = Runtime::new(reopened, RuntimeConfig::default());
    hydrated.hydrate_substrate_from_events().await?;
    anyhow::ensure!(hydrated.get_asset(&asset.id).await?.content == "durable", "asset did not rehydrate");
    anyhow::ensure!(hydrated.list_branches(&session.id).await.iter().any(|item| item.id == branch.id), "branch did not rehydrate");
    let projection = hydrated.projection_get("example/durable/projection").await?;
    anyhow::ensure!(projection.state["event_count"].as_u64().unwrap_or(0) >= 1, "projection did not rehydrate");
    let _ = fs::remove_file(path);
    Ok(())
}
