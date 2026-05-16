use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::Value;
use tower_http::cors::CorsLayer;
use ygg_core::{EventEnvelope, KernelSession, PackageId, PackageManifest, SessionId};
use ygg_runtime::{CapabilityInvocationRequest, CapabilityInvocationResult, PackageRecord, RegisteredCapability};
use ygg_runtime::{AppendEventRequest, EventStore, InMemoryEventStore, OpenSessionRequest, Runtime, RuntimeConfig};

pub type AppRuntime = Runtime<InMemoryEventStore>;

#[derive(Clone)]
pub struct AppState {
    pub runtime: Arc<AppRuntime>,
}

#[derive(Debug, Deserialize)]
pub struct OpenSessionHttpRequest {
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub active_package_set: Vec<PackageId>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Deserialize)]
pub struct AppendEventHttpRequest {
    pub writer_package_id: PackageId,
    pub kind: String,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub metadata: Value,
}

pub fn app() -> Router {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
    app_with_state(AppState { runtime })
}

pub fn app_with_state(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/kernel/session.open", post(open_session))
        .route("/kernel/event.append/:session_id", post(append_event))
        .route("/kernel/event.list/:session_id", get(list_events))
        .route("/kernel/package.load", post(load_package))
        .route("/kernel/package.list", get(list_packages))
        .route("/kernel/package.status/:namespace/:name", get(package_status))
        .route("/kernel/package.unload/:namespace/:name", post(unload_package))
        .route("/kernel/capability.discover", get(discover_capabilities))
        .route("/kernel/capability.invoke", post(invoke_capability))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn open_session(
    State(state): State<AppState>,
    Json(request): Json<OpenSessionHttpRequest>,
) -> anyhow::Result<Json<KernelSession>, ServiceError> {
    Ok(Json(
        state
            .runtime
            .open_session(OpenSessionRequest {
                labels: request.labels,
                active_package_set: request.active_package_set,
                metadata: request.metadata,
            })
            .await?,
    ))
}

async fn append_event(
    State(state): State<AppState>,
    Path(session_id): Path<SessionId>,
    Json(request): Json<AppendEventHttpRequest>,
) -> anyhow::Result<Json<EventEnvelope>, ServiceError> {
    Ok(Json(
        state
            .runtime
            .append_event(AppendEventRequest {
                session_id,
                writer_package_id: request.writer_package_id,
                kind: request.kind,
                payload: request.payload,
                metadata: request.metadata,
            })
            .await?,
    ))
}

async fn list_events(
    State(state): State<AppState>,
    Path(session_id): Path<SessionId>,
) -> anyhow::Result<Json<Vec<EventEnvelope>>, ServiceError> {
    Ok(Json(state.runtime.store().list_session(&session_id).await?))
}

async fn load_package(
    State(state): State<AppState>,
    Json(manifest): Json<PackageManifest>,
) -> anyhow::Result<Json<PackageRecord>, ServiceError> {
    Ok(Json(state.runtime.load_package(manifest).await?))
}

async fn list_packages(State(state): State<AppState>) -> Json<Vec<PackageRecord>> {
    Json(state.runtime.list_packages().await)
}

async fn package_status(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> anyhow::Result<Json<PackageRecord>, ServiceError> {
    let package_id = format!("{namespace}/{name}");
    state
        .runtime
        .package_status(&package_id)
        .await
        .map(Json)
        .ok_or_else(|| anyhow::anyhow!("package '{package_id}' is not loaded").into())
}

async fn unload_package(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> anyhow::Result<Json<PackageRecord>, ServiceError> {
    let package_id = format!("{namespace}/{name}");
    Ok(Json(state.runtime.unload_package(&package_id).await?))
}

async fn discover_capabilities(State(state): State<AppState>) -> Json<Vec<RegisteredCapability>> {
    Json(state.runtime.discover_capabilities().await)
}

async fn invoke_capability(
    State(state): State<AppState>,
    Json(request): Json<CapabilityInvocationRequest>,
) -> anyhow::Result<Json<CapabilityInvocationResult>, ServiceError> {
    Ok(Json(state.runtime.invoke_capability(request).await?))
}

pub struct ServiceError(anyhow::Error);

impl<E> From<E> for ServiceError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

impl axum::response::IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}
