use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::Value;
use tower_http::cors::CorsLayer;
use ygg_core::{EventEnvelope, KernelSession, PackageId, PackageManifest, SessionId};
use ygg_runtime::{
    host_info as runtime_host_info, CapabilityInvocationRequest, CapabilityInvocationResult,
    PackageRecord, ProtocolContext, ProtocolError, ProtocolRequest, ProtocolResponse, RegisteredCapability,
};
use ygg_runtime::{AppendEventRequest, InMemoryEventStore, OpenSessionRequest, Runtime, RuntimeConfig};

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
        .route("/kernel/host.info", get(host_info))
        .route("/rpc", post(rpc))
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
            .append_event_with_context(&ProtocolContext::host_dev("http_ad_hoc"), AppendEventRequest {
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
    Ok(Json(
        state
            .runtime
            .list_events_with_context(&ProtocolContext::host_dev("http_ad_hoc"), &session_id)
            .await?,
    ))
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
    Ok(Json(
        state
            .runtime
            .invoke_capability_with_context(&ProtocolContext::host_dev("http_ad_hoc"), request)
            .await?,
    ))
}

async fn host_info() -> Json<serde_json::Value> {
    Json(serde_json::to_value(runtime_host_info()).expect("host info serializes"))
}

async fn rpc(
    State(state): State<AppState>,
    Json(request): Json<ProtocolRequest>,
) -> Json<ProtocolResponse> {
    let context = ProtocolContext::host_dev("http_rpc");
    match state.runtime.call_protocol(&context, &request.method, request.params).await {
        Ok(result) => Json(ProtocolResponse { id: request.id, result: Some(result), error: None }),
        Err(error) => Json(ProtocolResponse { id: request.id, result: None, error: Some(error) }),
    }
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
        let error = ProtocolError::from_anyhow(self.0);
        let status = match error.code.as_str() {
            "kernel/error/permission_denied" => StatusCode::FORBIDDEN,
            "kernel/error/not_found" => StatusCode::NOT_FOUND,
            "kernel/error/schema_invalid" | "kernel/error/invalid_request" => StatusCode::BAD_REQUEST,
            "kernel/error/ambiguous_route" | "kernel/error/package_state" => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(serde_json::json!({ "error": error }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn rpc_host_info_returns_protocol_envelope() -> anyhow::Result<()> {
        let response = app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/rpc")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({"id": "1", "method": "kernel.host.info", "params": {}}).to_string(),
                    ))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)?;
        assert_eq!(value["id"], "1");
        assert!(value["result"]["supported_transports"].is_array());
        Ok(())
    }

    #[tokio::test]
    async fn rpc_returns_structured_error() -> anyhow::Result<()> {
        let response = app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/rpc")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({"id": "1", "method": "kernel.event.list", "params": {}}).to_string(),
                    ))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)?;
        assert_eq!(value["error"]["code"], "kernel/error/internal");
        Ok(())
    }
}
