use std::collections::VecDeque;
use std::convert::Infallible;
use std::path::{Component, PathBuf};
use std::sync::Arc;

use axum::extract::{OriginalUri, Path, Query, Request, State};
use axum::http::{header, HeaderMap, HeaderValue, Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::response::Response;
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
    pub static_dir: Option<PathBuf>,
    pub access_token: Option<String>,
}

impl<S> Clone for AppState<S>
where
    S: EventStore,
{
    fn clone(&self) -> Self {
        Self {
            runtime: self.runtime.clone(),
            static_dir: self.static_dir.clone(),
            access_token: self.access_token.clone(),
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
    app_with_state(AppState {
        runtime,
        static_dir: None,
        access_token: None,
    })
}

pub fn app_with_state<S>(state: AppState<S>) -> Router
where
    S: EventStore,
{
    let protected = Router::new()
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
        .route("/rpc", post(rpc::<S>))
        .route_layer(middleware::from_fn_with_state(
            state.access_token.clone(),
            require_access_token,
        ));

    Router::new()
        .route("/health", get(health))
        .route("/healthz", get(health))
        .route(
            "/surface-bundles/:prefix/*file",
            get(surface_bundle_file::<S>),
        )
        .merge(protected)
        .fallback(static_fallback::<S>)
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn require_access_token(
    State(access_token): State<Option<String>>,
    request: Request,
    next: Next,
) -> Response {
    let Some(expected) = access_token.as_deref().filter(|token| !token.is_empty()) else {
        return next.run(request).await;
    };

    if request_access_token_matches(&request, expected) {
        return next.run(request).await;
    }

    (StatusCode::UNAUTHORIZED, "missing or invalid access token").into_response()
}

fn request_access_token_matches(request: &Request, expected: &str) -> bool {
    if request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .is_some_and(|token| token == expected)
    {
        return true;
    }

    request
        .uri()
        .query()
        .and_then(|query| {
            url::form_urlencoded::parse(query.as_bytes())
                .find(|(key, _)| key == "access_token")
                .map(|(_, value)| value.into_owned())
        })
        .is_some_and(|token| token == expected)
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
            (public_static_headers(content_type), bytes).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "surface bundle file not found").into_response(),
    }
}

async fn static_fallback<S>(
    State(state): State<AppState<S>>,
    method: Method,
    OriginalUri(uri): OriginalUri,
) -> impl IntoResponse
where
    S: EventStore,
{
    if method != Method::GET && method != Method::HEAD {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }

    let Some(static_dir) = state.static_dir.as_ref() else {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    };

    let Ok(static_root) = std::fs::canonicalize(static_dir) else {
        return (StatusCode::NOT_FOUND, "static root not found").into_response();
    };

    let request_path = uri.path();
    if is_reserved_service_path(request_path) {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }

    let candidate = if request_path == "/" {
        static_root.join("index.html")
    } else {
        let Some(relative) = safe_relative_path(request_path.trim_start_matches('/')) else {
            return (StatusCode::NOT_FOUND, "not found").into_response();
        };
        static_root.join(relative)
    };

    let path = if candidate.is_dir() {
        candidate.join("index.html")
    } else {
        candidate
    };

    match read_static_file(&static_root, &path).await {
        StaticRead::Served(response) => return response,
        StaticRead::Forbidden => {
            return (StatusCode::NOT_FOUND, "static file not found").into_response()
        }
        StaticRead::Missing => {}
    }

    if is_static_asset_path(request_path) {
        return (StatusCode::NOT_FOUND, "static file not found").into_response();
    }

    let index = static_root.join("index.html");
    match read_static_file(&static_root, &index).await {
        StaticRead::Served(response) => response,
        StaticRead::Missing | StaticRead::Forbidden => {
            (StatusCode::NOT_FOUND, "static file not found").into_response()
        }
    }
}

fn is_reserved_service_path(path: &str) -> bool {
    path == "/rpc"
        || path.starts_with("/rpc/")
        || path.starts_with("/kernel")
        || path.starts_with("/surface-bundles")
}

fn is_static_asset_path(path: &str) -> bool {
    path == "/surface-frame.html"
        || path == "/surface-frame-bootstrap.js"
        || path.starts_with("/assets/")
        || std::path::Path::new(path.trim_start_matches('/'))
            .extension()
            .is_some()
}

enum StaticRead {
    Served(Response),
    Missing,
    Forbidden,
}

async fn read_static_file(static_root: &std::path::Path, path: &std::path::Path) -> StaticRead {
    let canonical = match std::fs::canonicalize(path) {
        Ok(path) => path,
        Err(_) => return StaticRead::Missing,
    };
    if !canonical.starts_with(static_root) || !canonical.is_file() {
        return StaticRead::Forbidden;
    }
    let bytes = match tokio::fs::read(&canonical).await {
        Ok(bytes) => bytes,
        Err(_) => return StaticRead::Missing,
    };
    StaticRead::Served((public_static_headers(content_type_for(&canonical)), bytes).into_response())
}

fn public_static_headers(content_type: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    // Surface frames are sandboxed without `allow-same-origin`; browser module
    // loading treats the frame as an opaque origin. Public browser assets must be
    // CORS-readable so `/surface-frame-bootstrap.js` and installed surface
    // bundles can load inside that sandbox while RPC/kernel routes remain token
    // gated separately.
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    headers
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
        Some("html") => "text/html; charset=utf-8",
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

    #[tokio::test]
    async fn healthz_returns_ok() -> anyhow::Result<()> {
        let response = app()
            .oneshot(Request::builder().uri("/healthz").body(Body::empty())?)
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn serves_static_files_when_configured() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        std::fs::write(dir.path().join("index.html"), "<main>Ygg web</main>")?;
        std::fs::create_dir_all(dir.path().join("assets"))?;
        std::fs::write(dir.path().join("assets/app.js"), "console.log('ygg');")?;

        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
        let app = app_with_state(AppState {
            runtime,
            static_dir: Some(dir.path().to_path_buf()),
            access_token: None,
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/assets/app.js")
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/javascript"
        );
        let bytes = to_bytes(response.into_body(), usize::MAX).await?;
        assert_eq!(&bytes[..], b"console.log('ygg');");
        Ok(())
    }

    #[tokio::test]
    async fn token_gate_protects_rpc_but_not_healthz_or_static() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        std::fs::write(dir.path().join("index.html"), "public")?;
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
        let app = app_with_state(AppState {
            runtime,
            static_dir: Some(dir.path().to_path_buf()),
            access_token: Some("secret-token".to_string()),
        });

        let health = app
            .clone()
            .oneshot(Request::builder().uri("/healthz").body(Body::empty())?)
            .await?;
        assert_eq!(health.status(), StatusCode::OK);

        let static_response = app
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty())?)
            .await?;
        assert_eq!(static_response.status(), StatusCode::OK);

        let denied = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/rpc")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({"id":"1","method":"kernel.v1.host.info","params":{}}).to_string(),
                    ))?,
            )
            .await?;
        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);

        let allowed = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/rpc")
                    .header("content-type", "application/json")
                    .header(header::AUTHORIZATION, "Bearer secret-token")
                    .body(Body::from(
                        json!({"id":"1","method":"kernel.v1.host.info","params":{}}).to_string(),
                    ))?,
            )
            .await?;
        assert_eq!(allowed.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn token_gate_accepts_query_token_for_sse() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
        let app = app_with_state(AppState {
            runtime,
            static_dir: None,
            access_token: Some("event-token".to_string()),
        });

        let denied = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/kernel/v1/event.subscribe/session-1")
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);

        let allowed = app
            .oneshot(
                Request::builder()
                    .uri("/kernel/v1/event.subscribe/session-1?access_token=event-token")
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(allowed.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn surface_bundles_are_public_static_artifacts_when_token_gate_enabled(
    ) -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let bundle_dir = dir.path().join("surface");
        std::fs::create_dir_all(&bundle_dir)?;
        std::fs::write(bundle_dir.join("main.mjs"), "export const ok = true;")?;

        let store = Arc::new(InMemoryEventStore::default());
        let mut config = RuntimeConfig::default();
        config
            .surface_dev_paths
            .insert("test".to_string(), bundle_dir.to_string_lossy().to_string());
        let runtime = Arc::new(Runtime::new(store, config));
        let app = app_with_state(AppState {
            runtime,
            static_dir: None,
            access_token: Some("bundle-token".to_string()),
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/surface-bundles/test/main.mjs")
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&HeaderValue::from_static("*"))
        );
        let bytes = to_bytes(response.into_body(), usize::MAX).await?;
        assert_eq!(&bytes[..], b"export const ok = true;");
        Ok(())
    }

    #[tokio::test]
    async fn static_js_is_cors_readable_for_sandboxed_surface_frame() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        std::fs::write(
            dir.path().join("surface-frame-bootstrap.js"),
            "export const ok = true;",
        )?;

        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
        let app = app_with_state(AppState {
            runtime,
            static_dir: Some(dir.path().to_path_buf()),
            access_token: Some("static-token".to_string()),
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/surface-frame-bootstrap.js")
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&HeaderValue::from_static("*"))
        );
        Ok(())
    }

    #[tokio::test]
    async fn reserved_paths_do_not_fallback_to_static_index() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        std::fs::write(dir.path().join("index.html"), "public")?;
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
        let app = app_with_state(AppState {
            runtime,
            static_dir: Some(dir.path().to_path_buf()),
            access_token: None,
        });

        for path in ["/rpc/anything", "/kernelx", "/surface-bundlesx"] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty())?)
                .await?;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "path {path}");
        }
        Ok(())
    }

    #[tokio::test]
    async fn static_serving_rejects_symlink_escape() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let outside = tempfile::tempdir()?;
        std::fs::write(dir.path().join("index.html"), "public")?;
        std::fs::write(outside.path().join("secret.txt"), "do-not-serve")?;

        #[cfg(unix)]
        std::os::unix::fs::symlink(
            outside.path().join("secret.txt"),
            dir.path().join("leak.txt"),
        )?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(
            outside.path().join("secret.txt"),
            dir.path().join("leak.txt"),
        )?;

        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
        let app = app_with_state(AppState {
            runtime,
            static_dir: Some(dir.path().to_path_buf()),
            access_token: None,
        });

        let response = app
            .oneshot(Request::builder().uri("/leak.txt").body(Body::empty())?)
            .await?;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        Ok(())
    }
}
