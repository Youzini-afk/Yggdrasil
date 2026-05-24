use std::collections::VecDeque;
use std::convert::Infallible;
use std::path::{Component, PathBuf};
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::Stream;
use serde::Deserialize;
use serde_json::Value;
use ygg_core::{EventEnvelope, KernelSession, PackageId, PackageManifest, SessionId};
use ygg_runtime::{
    host_info as runtime_host_info, CapabilityInvocationRequest, CapabilityInvocationResult,
    EventListRequest, PackageRecord, ProtocolContext, ProtocolError, ProtocolRequest,
    ProtocolResponse, RegisteredCapability,
};
use ygg_runtime::{
    AppendEventRequest, EventStore, InMemoryEventStore, OpenSessionRequest, Runtime, RuntimeConfig,
};

pub type AppRuntime = Runtime<InMemoryEventStore>;

pub struct AppState<S = InMemoryEventStore>
where
    S: EventStore,
{
    pub runtime: Arc<Runtime<S>>,
}

impl<S> Clone for AppState<S>
where
    S: EventStore,
{
    fn clone(&self) -> Self {
        Self {
            runtime: self.runtime.clone(),
        }
    }
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

#[derive(Debug, Default, Deserialize)]
pub struct EventListQuery {
    #[serde(default)]
    pub after_sequence: Option<u64>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub kind_prefix: Option<String>,
    #[serde(default)]
    pub writer_package_id: Option<PackageId>,
}

pub fn app() -> Router {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
    app_with_state(AppState { runtime })
}

pub fn app_with_state<S>(state: AppState<S>) -> Router
where
    S: EventStore,
{
    Router::new()
        .route("/health", get(health))
        .route("/kernel/v1/session.open", post(open_session::<S>))
        .route(
            "/kernel/v1/event.append/:session_id",
            post(append_event::<S>),
        )
        .route("/kernel/v1/event.list/:session_id", get(list_events::<S>))
        .route(
            "/kernel/v1/event.subscribe/:session_id",
            get(subscribe_events::<S>),
        )
        .route("/kernel/v1/package.load", post(load_package::<S>))
        .route("/kernel/v1/package.list", get(list_packages::<S>))
        .route(
            "/kernel/v1/package.status/:namespace/:name",
            get(package_status::<S>),
        )
        .route(
            "/kernel/v1/package.unload/:namespace/:name",
            post(unload_package::<S>),
        )
        .route(
            "/kernel/v1/capability.discover",
            get(discover_capabilities::<S>),
        )
        .route("/kernel/v1/capability.invoke", post(invoke_capability::<S>))
        .route("/kernel/v1/host.info", get(host_info))
        .route(
            "/surface-bundles/:prefix/*file",
            get(surface_bundle_file::<S>),
        )
        .route("/rpc", post(rpc::<S>))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn open_session<S>(
    State(state): State<AppState<S>>,
    Json(request): Json<OpenSessionHttpRequest>,
) -> anyhow::Result<Json<KernelSession>, ServiceError>
where
    S: EventStore,
{
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

async fn append_event<S>(
    State(state): State<AppState<S>>,
    Path(session_id): Path<SessionId>,
    Json(request): Json<AppendEventHttpRequest>,
) -> anyhow::Result<Json<EventEnvelope>, ServiceError>
where
    S: EventStore,
{
    Ok(Json(
        state
            .runtime
            .append_event_with_context(
                &ProtocolContext::host_dev("http_ad_hoc"),
                AppendEventRequest {
                    session_id,
                    writer_package_id: request.writer_package_id,
                    kind: request.kind,
                    payload: request.payload,
                    metadata: request.metadata,
                },
            )
            .await?,
    ))
}

async fn list_events<S>(
    State(state): State<AppState<S>>,
    Path(session_id): Path<SessionId>,
    Query(query): Query<EventListQuery>,
) -> anyhow::Result<Json<Vec<EventEnvelope>>, ServiceError>
where
    S: EventStore,
{
    let request = EventListRequest {
        session_id,
        after_sequence: query.after_sequence,
        limit: query.limit,
        kind_prefix: query.kind_prefix,
        writer_package_id: query.writer_package_id,
    };
    Ok(Json(
        state
            .runtime
            .list_events_range_with_context(&ProtocolContext::host_dev("http_ad_hoc"), &request)
            .await?,
    ))
}

async fn subscribe_events<S>(
    State(state): State<AppState<S>>,
    Path(session_id): Path<SessionId>,
    Query(query): Query<EventListQuery>,
) -> anyhow::Result<Sse<impl Stream<Item = Result<SseEvent, Infallible>>>, ServiceError>
where
    S: EventStore,
{
    let request = EventListRequest {
        session_id: session_id.clone(),
        after_sequence: query.after_sequence,
        limit: query.limit,
        kind_prefix: query.kind_prefix.clone(),
        writer_package_id: query.writer_package_id.clone(),
    };
    let replay = state
        .runtime
        .list_events_range_with_context(&ProtocolContext::host_dev("http_sse"), &request)
        .await?;
    let replay = VecDeque::from(replay);
    let rx = state.runtime.subscribe_events();
    let stream = futures::stream::unfold(
        (replay, rx, session_id, query),
        |(mut replay, mut rx, session_id, query)| async move {
            if let Some(event) = replay.pop_front() {
                let sse = SseEvent::default()
                    .event("kernel.v1.event")
                    .json_data(event)
                    .unwrap_or_else(|_| SseEvent::default().event("kernel.v1.error"));
                return Some((Ok(sse), (replay, rx, session_id, query)));
            }
            loop {
                match rx.recv().await {
                    Ok(event) if event_matches_query(&event, &session_id, &query) => {
                        let sse = SseEvent::default()
                            .event("kernel.v1.event")
                            .json_data(event)
                            .unwrap_or_else(|_| SseEvent::default().event("kernel.v1.error"));
                        return Some((Ok(sse), (replay, rx, session_id, query)));
                    }
                    Ok(_) => continue,
                    Err(_) => return None,
                }
            }
        },
    );
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

fn event_matches_query(event: &EventEnvelope, session_id: &str, query: &EventListQuery) -> bool {
    if event.session_id != session_id {
        return false;
    }
    if let Some(after_sequence) = query.after_sequence {
        if event.sequence <= after_sequence {
            return false;
        }
    }
    if let Some(kind_prefix) = &query.kind_prefix {
        if !event.kind.starts_with(kind_prefix) {
            return false;
        }
    }
    if let Some(writer_package_id) = &query.writer_package_id {
        if &event.writer_package_id != writer_package_id {
            return false;
        }
    }
    true
}

async fn load_package<S>(
    State(state): State<AppState<S>>,
    Json(manifest): Json<PackageManifest>,
) -> anyhow::Result<Json<PackageRecord>, ServiceError>
where
    S: EventStore,
{
    Ok(Json(state.runtime.load_package(manifest).await?))
}

async fn list_packages<S>(State(state): State<AppState<S>>) -> Json<Vec<PackageRecord>>
where
    S: EventStore,
{
    Json(state.runtime.list_packages().await)
}

async fn package_status<S>(
    State(state): State<AppState<S>>,
    Path((namespace, name)): Path<(String, String)>,
) -> anyhow::Result<Json<PackageRecord>, ServiceError>
where
    S: EventStore,
{
    let package_id = format!("{namespace}/{name}");
    state
        .runtime
        .package_status(&package_id)
        .await
        .map(Json)
        .ok_or_else(|| anyhow::anyhow!("package '{package_id}' is not loaded").into())
}

async fn unload_package<S>(
    State(state): State<AppState<S>>,
    Path((namespace, name)): Path<(String, String)>,
) -> anyhow::Result<Json<PackageRecord>, ServiceError>
where
    S: EventStore,
{
    let package_id = format!("{namespace}/{name}");
    Ok(Json(state.runtime.unload_package(&package_id).await?))
}

async fn discover_capabilities<S>(
    State(state): State<AppState<S>>,
) -> Json<Vec<RegisteredCapability>>
where
    S: EventStore,
{
    Json(state.runtime.discover_capabilities().await)
}

async fn invoke_capability<S>(
    State(state): State<AppState<S>>,
    Json(request): Json<CapabilityInvocationRequest>,
) -> anyhow::Result<Json<CapabilityInvocationResult>, ServiceError>
where
    S: EventStore,
{
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

async fn surface_bundle_file<S>(
    State(state): State<AppState<S>>,
    Path((prefix, file)): Path<(String, String)>,
) -> impl IntoResponse
where
    S: EventStore,
{
    let Some(path) = surface_bundle_path(state, &prefix, &file) else {
        return (StatusCode::NOT_FOUND, "surface bundle path not found").into_response();
    };
    match tokio::fs::read(&path).await {
        Ok(bytes) => {
            let content_type = content_type_for(&path);
            ([(header::CONTENT_TYPE, content_type)], bytes).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "surface bundle file not found").into_response(),
    }
}

fn surface_bundle_path<S>(state: AppState<S>, prefix: &str, file: &str) -> Option<PathBuf>
where
    S: EventStore,
{
    let safe_file = safe_relative_path(file)?;
    if prefix == "projects" {
        let mut parts = safe_file.components();
        let project_id = parts.next()?.as_os_str().to_str()?;
        let project_id = ygg_core::project::ProjectId::new(project_id).ok()?;
        let mut rest = PathBuf::new();
        for part in parts {
            rest.push(part.as_os_str());
        }
        if rest.as_os_str().is_empty() {
            return None;
        }
        return ygg_core::paths::project_dir(&project_id)
            .ok()
            .map(|dir| dir.join("dist").join(rest));
    }

    let Some(base) = state.runtime.config().surface_dev_paths.get(prefix) else {
        return None;
    };
    Some(PathBuf::from(base).join(safe_file))
}

fn safe_relative_path(path: &str) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for component in std::path::Path::new(path).components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            _ => return None,
        }
    }
    if out.as_os_str().is_empty() {
        None
    } else {
        Some(out)
    }
}

fn content_type_for(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("mjs") | Some("js") => "application/javascript",
        Some("css") => "text/css",
        Some("json") | Some("map") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

async fn rpc<S>(
    State(state): State<AppState<S>>,
    Json(request): Json<ProtocolRequest>,
) -> Json<ProtocolResponse>
where
    S: EventStore,
{
    let mut context = ProtocolContext::host_dev("http_rpc");
    context.session_id = request.session_id.clone();
    match state
        .runtime
        .call_protocol(&context, &request.method, request.params)
        .await
    {
        Ok(result) => Json(ProtocolResponse {
            id: request.id,
            result: Some(result),
            error: None,
        }),
        Err(error) => Json(ProtocolResponse {
            id: request.id,
            result: None,
            error: Some(error),
        }),
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
            "kernel/v1/error/permission_denied" => StatusCode::FORBIDDEN,
            "kernel/v1/error/not_found" => StatusCode::NOT_FOUND,
            "kernel/v1/error/schema_invalid" | "kernel/v1/error/invalid_request" => {
                StatusCode::BAD_REQUEST
            }
            "kernel/v1/error/ambiguous_route" | "kernel/v1/error/package_state" => {
                StatusCode::CONFLICT
            }
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
                        json!({"id": "1", "method": "kernel.v1.host.info", "params": {}})
                            .to_string(),
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
                        json!({"id": "1", "method": "kernel.v1.event.list", "params": {}})
                            .to_string(),
                    ))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)?;
        assert_eq!(value["error"]["code"], "kernel/v1/error/internal");
        Ok(())
    }
}
