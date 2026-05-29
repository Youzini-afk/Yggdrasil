use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::Infallible;
use std::path::{Component, Path as FsPath, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use anyhow::Context;
use axum::body::to_bytes;
use axum::extract::ws::{Message as AxumWsMessage, WebSocket, WebSocketUpgrade};
use axum::extract::{FromRequestParts, OriginalUri, Path, Query, Request, State};
use axum::http::{header, HeaderMap, HeaderValue, Method, StatusCode, Uri};
use axum::middleware::{self, Next};
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::{any, get, post};
use axum::{Json, Router};
use futures::{SinkExt, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout, Instant};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use ygg_core::{EventEnvelope, KernelSession, PackageId, PackageManifest, ProjectId, SessionId};
use ygg_runtime::{
    host_info as runtime_host_info, CapabilityInvocationRequest, CapabilityInvocationResult,
    EventListRequest, PackageRecord, ProtocolContext, ProtocolError, ProtocolRequest,
    ProtocolResponse, RegisteredCapability,
};
use ygg_runtime::{
    AppendEventRequest, EventStore, InMemoryEventStore, OpenSessionRequest, Runtime, RuntimeConfig,
};
use ygg_runtime::{PortBindScope, PortLeaseStatusKind, ProxyProtocol, ProxyRouteStatusKind};

const PROXY_REQUEST_BODY_LIMIT_BYTES: usize = 64 * 1024 * 1024;
const PROXY_RESPONSE_BODY_LIMIT_BYTES: usize = 64 * 1024 * 1024;
const PROXY_WEBSOCKET_FRAME_LIMIT_BYTES: usize = 16 * 1024 * 1024;
const DEPLOY_READINESS_TIMEOUT: Duration = Duration::from_secs(15);
const DEPLOY_READINESS_INTERVAL: Duration = Duration::from_millis(500);
const DEPLOY_READINESS_CONNECT_TIMEOUT: Duration = Duration::from_secs(1);
const HEALTH_POLL_INTERVAL: Duration = Duration::from_secs(5);
const HEALTH_PROBE_TIMEOUT: Duration = Duration::from_millis(1000);
const HEALTH_FAILURE_THRESHOLD: u32 = 3;
const HEALTH_RECOVERY_THRESHOLD: u32 = 2;

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
        .route("/host/v1/deploy", post(deploy_project::<S>))
        .route("/host/v1/deploy/stop", post(stop_project_deployment::<S>))
        .route("/rpc", post(rpc::<S>))
        .route("/p/:route_id", any(proxy_root::<S>))
        .route("/p/:route_id/*path", any(proxy_path::<S>))
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

pub fn spawn_health_supervisor<S>(state: AppState<S>) -> tokio::task::JoinHandle<()>
where
    S: EventStore,
{
    tokio::spawn(run_health_supervisor(state))
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostDeployRequest {
    pub image: String,
    pub container_port: u16,
    pub port_name: String,
    pub route_id: String,
    #[serde(default)]
    pub health_path: Option<String>,
    #[serde(default)]
    pub pull_if_missing: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostDeployStopRequest {
    pub route_id: String,
}

#[derive(Debug, Serialize)]
pub struct HostDeployResponse {
    pub route_id: String,
    pub public_url: String,
    pub port_lease_id: String,
    pub container_id: String,
    pub container_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HostDeployStopResponse {
    pub route_id: String,
    pub stopped: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectWorkspaceCloneRequest {
    pub project_id: ProjectId,
    pub source_url: String,
    pub ref_name: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProjectWorkspaceCloneResult {
    pub project_id: ProjectId,
    pub ref_name: String,
    pub commit_sha: String,
    pub tree_hash: Option<String>,
    pub files_written: Option<u64>,
    pub total_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GitFetchTreeInvocation {
    resolve_ref_params: Value,
    fetch_tree_params: Value,
    workspace_dir: PathBuf,
    staging_dir: PathBuf,
}

async fn deploy_project<S>(
    State(state): State<AppState<S>>,
    Json(request): Json<HostDeployRequest>,
) -> anyhow::Result<Json<HostDeployResponse>, ServiceError>
where
    S: EventStore,
{
    validate_host_deploy_request(&request)?;
    let context = ProtocolContext::host_dev("host_deploy");
    let mut container_id: Option<String> = None;

    let lease = match call_host_protocol(
        &state,
        &context,
        "kernel.v1.port.lease",
        serde_json::json!({
            "target_id": "local",
            "port_name": &request.port_name,
            "protocol": "tcp",
        }),
    )
    .await
    .and_then(|value| value_field(value, "lease", "kernel.v1.port.lease"))
    {
        Ok(lease) => lease,
        Err(error) => return Err(anyhow::anyhow!("deployment port lease failed: {error}").into()),
    };

    let lease_id = required_string(&lease, "id", "port lease")?;
    let lease_port = required_u16(&lease, "port", "port lease")?;
    let port_lease_id = lease_id.clone();

    let start_output = match invoke_docker_runtime_lab(
        &state,
        &context,
        "official/docker-runtime-lab/start_container",
        serde_json::json!({
            "image": &request.image,
            "container_port": request.container_port,
            "host_port": lease_port,
            "route_id": &request.route_id,
            "port_lease_id": &port_lease_id,
            "approved": true,
            "pull_if_missing": request.pull_if_missing,
        }),
    )
    .await
    {
        Ok(output) => output,
        Err(error) => {
            rollback_deploy(
                &state,
                &context,
                &request.route_id,
                false,
                container_id.as_deref(),
                Some(&lease_id),
            )
            .await;
            return Err(anyhow::anyhow!("docker container start failed: {error}").into());
        }
    };

    let parsed_container_id = match require_started_container(&start_output) {
        Ok(container_id) => container_id,
        Err(error) => {
            rollback_deploy(
                &state,
                &context,
                &request.route_id,
                false,
                container_id.as_deref(),
                Some(&lease_id),
            )
            .await;
            return Err(error.into());
        }
    };
    container_id = Some(parsed_container_id.clone());
    let container_name = optional_string(&start_output, "container_name");

    let route = match call_host_protocol(
        &state,
        &context,
        "kernel.v1.proxy.register",
        serde_json::json!({
            "route_id": &request.route_id,
            "protocol": "http",
            "upstream": {
                "port_lease_id": &port_lease_id,
                "port_name": &request.port_name,
            },
        }),
    )
    .await
    .and_then(|value| value_field(value, "route", "kernel.v1.proxy.register"))
    {
        Ok(route) => route,
        Err(error) => {
            rollback_deploy(
                &state,
                &context,
                &request.route_id,
                false,
                container_id.as_deref(),
                Some(&lease_id),
            )
            .await;
            return Err(anyhow::anyhow!("proxy registration failed: {error}").into());
        }
    };
    let route_id = required_string(&route, "id", "proxy route")?;
    let public_url = required_string(&route, "public_url", "proxy route")?;

    if let Err(error) =
        wait_for_deployment_readiness(lease_port, request.health_path.as_deref()).await
    {
        rollback_deploy(
            &state,
            &context,
            &route_id,
            true,
            container_id.as_deref(),
            Some(&lease_id),
        )
        .await;
        return Err(anyhow::anyhow!("deployment did not become ready in time: {error}").into());
    }

    if state
        .runtime
        .config()
        .proxy_route_registry
        .set_ready(&route_id, true)
        .await
        .is_none()
    {
        rollback_deploy(
            &state,
            &context,
            &route_id,
            true,
            container_id.as_deref(),
            Some(&lease_id),
        )
        .await;
        return Err(anyhow::anyhow!("proxy route disappeared before readiness promotion").into());
    }

    Ok(Json(HostDeployResponse {
        route_id,
        public_url,
        port_lease_id,
        container_id: parsed_container_id,
        container_name,
    }))
}

async fn stop_project_deployment<S>(
    State(state): State<AppState<S>>,
    Json(request): Json<HostDeployStopRequest>,
) -> Json<HostDeployStopResponse>
where
    S: EventStore,
{
    let context = ProtocolContext::host_dev("host_deploy");
    let mut warnings = Vec::new();
    let route_id = request.route_id.trim().to_string();
    if !is_safe_route_token(&route_id) {
        return Json(HostDeployStopResponse {
            route_id,
            stopped: false,
            warnings: vec!["route_id must be label-safe".to_string()],
        });
    }

    let mut port_lease_id = None;
    match call_host_protocol(
        &state,
        &context,
        "kernel.v1.proxy.status",
        serde_json::json!({ "route_id": route_id }),
    )
    .await
    {
        Ok(route) => {
            port_lease_id = route
                .get("upstream")
                .and_then(|upstream| upstream.get("port_lease_id"))
                .and_then(Value::as_str)
                .map(str::to_string);
        }
        Err(error) => warnings.push(format!("proxy status unavailable: {error}")),
    }

    let mut container_ref = None;
    match invoke_docker_runtime_lab(
        &state,
        &context,
        "official/docker-runtime-lab/list_managed",
        serde_json::json!({}),
    )
    .await
    {
        Ok(output) => container_ref = find_managed_container_for_route(&output, &route_id),
        Err(error) => warnings.push(format!("managed container list unavailable: {error}")),
    }

    let mut stopped = false;
    if let Some(container) = container_ref.as_ref() {
        match invoke_docker_runtime_lab(
            &state,
            &context,
            "official/docker-runtime-lab/stop_container",
            serde_json::json!({ "container_id": container, "timeout_secs": 10 }),
        )
        .await
        {
            Ok(output) => {
                stopped = output
                    .get("docker_performed")
                    .and_then(Value::as_bool)
                    .unwrap_or(true);
                if !stopped {
                    warnings.push(
                        output
                            .get("reason")
                            .and_then(Value::as_str)
                            .unwrap_or("docker-runtime-lab did not stop the container")
                            .to_string(),
                    );
                }
            }
            Err(error) => warnings.push(format!("container stop failed: {error}")),
        }
    } else {
        warnings.push("no managed container found for route".to_string());
    }

    if let Err(error) = call_host_protocol(
        &state,
        &context,
        "kernel.v1.proxy.unregister",
        serde_json::json!({ "route_id": route_id }),
    )
    .await
    {
        warnings.push(format!("proxy unregister failed: {error}"));
    }

    if let Some(lease_id) = port_lease_id.as_ref() {
        if let Err(error) = call_host_protocol(
            &state,
            &context,
            "kernel.v1.port.release",
            serde_json::json!({ "lease_id": lease_id }),
        )
        .await
        {
            warnings.push(format!("port release failed: {error}"));
        }
    }

    Json(HostDeployStopResponse {
        route_id,
        stopped,
        warnings,
    })
}

async fn call_host_protocol<S>(
    state: &AppState<S>,
    context: &ProtocolContext,
    method: &str,
    params: Value,
) -> anyhow::Result<Value>
where
    S: EventStore,
{
    state
        .runtime
        .call_protocol(context, method, params)
        .await
        .map_err(protocol_error_to_anyhow)
}

async fn invoke_docker_runtime_lab<S>(
    state: &AppState<S>,
    context: &ProtocolContext,
    capability_id: &str,
    input: Value,
) -> anyhow::Result<Value>
where
    S: EventStore,
{
    let value = call_host_protocol(
        state,
        context,
        "kernel.v1.capability.invoke",
        serde_json::json!({
            "capability_id": capability_id,
            "provider_package_id": "official/docker-runtime-lab",
            "input": input,
        }),
    )
    .await?;
    value_field(value, "output", "kernel.v1.capability.invoke")
}

pub async fn clone_project_workspace_from_git<S>(
    state: &AppState<S>,
    request: ProjectWorkspaceCloneRequest,
) -> anyhow::Result<ProjectWorkspaceCloneResult>
where
    S: EventStore,
{
    validate_workspace_clone_url(&request.source_url)?;
    validate_workspace_clone_ref(&request.ref_name)?;
    let invocation = build_project_workspace_clone_invocation(&request, None)?;
    let context = ProtocolContext::host_dev("project_workspace_clone");

    let resolved = invoke_git_tools_lab(
        state,
        &context,
        "official/git-tools-lab/resolve_ref",
        invocation.resolve_ref_params,
    )
    .await?;
    let commit_sha = required_string(&resolved, "commit_sha", "git resolve_ref")?;
    let resolved_ref_name = required_string(&resolved, "ref_name", "git resolve_ref")?;
    validate_git_commit_sha(&commit_sha)?;
    validate_workspace_clone_ref(&resolved_ref_name)?;

    if invocation.staging_dir.exists() {
        std::fs::remove_dir_all(&invocation.staging_dir).with_context(|| {
            format!(
                "failed to clear workspace staging dir {}",
                invocation.staging_dir.display()
            )
        })?;
    }
    let fetch_params = serde_json::json!({
        "remote_url": request.source_url,
        "commit_sha": commit_sha,
        "ref_name": resolved_ref_name.clone(),
        "dest_dir": invocation.staging_dir.to_string_lossy(),
    });
    let fetch_result = async {
        let output = invoke_git_tools_lab(
            state,
            &context,
            "official/git-tools-lab/fetch_tree",
            fetch_params,
        )
        .await?;
        validate_workspace_staging_containment(&invocation.workspace_dir, &invocation.staging_dir)?;
        replace_workspace_from_staging(&invocation.workspace_dir, &invocation.staging_dir)?;
        anyhow::Ok(output)
    }
    .await;

    if fetch_result.is_err() {
        std::fs::remove_dir_all(&invocation.staging_dir).ok();
    }
    let output = fetch_result?;

    Ok(ProjectWorkspaceCloneResult {
        project_id: request.project_id,
        ref_name: resolved_ref_name,
        commit_sha,
        tree_hash: output
            .get("tree_hash")
            .and_then(Value::as_str)
            .map(str::to_string),
        files_written: output.get("files_written").and_then(Value::as_u64),
        total_bytes: output.get("total_bytes").and_then(Value::as_u64),
    })
}

async fn invoke_git_tools_lab<S>(
    state: &AppState<S>,
    context: &ProtocolContext,
    capability_id: &str,
    input: Value,
) -> anyhow::Result<Value>
where
    S: EventStore,
{
    let value = call_host_protocol(
        state,
        context,
        "kernel.v1.capability.invoke",
        serde_json::json!({
            "capability_id": capability_id,
            "provider_package_id": "official/git-tools-lab",
            "input": input,
        }),
    )
    .await?;
    value_field(value, "output", "kernel.v1.capability.invoke")
}

fn build_project_workspace_clone_invocation(
    request: &ProjectWorkspaceCloneRequest,
    data_dir_override: Option<&FsPath>,
) -> anyhow::Result<GitFetchTreeInvocation> {
    validate_workspace_clone_url(&request.source_url)?;
    validate_workspace_clone_ref(&request.ref_name)?;
    let workspace_dir = match data_dir_override {
        Some(data_dir) => ygg_core::paths::project_workspace_dir_in(data_dir, &request.project_id),
        None => ygg_core::paths::project_workspace_dir(&request.project_id)?,
    };
    validate_workspace_destination(&request.project_id, data_dir_override, &workspace_dir)?;
    let staging_dir = workspace_dir.with_file_name("workspace.staging");
    validate_workspace_destination(&request.project_id, data_dir_override, &staging_dir)?;
    Ok(GitFetchTreeInvocation {
        resolve_ref_params: serde_json::json!({
            "remote_url": request.source_url,
            "ref": request.ref_name,
        }),
        fetch_tree_params: serde_json::json!({
            "remote_url": request.source_url,
            "ref_name": request.ref_name,
            "dest_dir": staging_dir.to_string_lossy(),
        }),
        workspace_dir,
        staging_dir,
    })
}

fn validate_workspace_clone_url(source_url: &str) -> anyhow::Result<()> {
    let parsed = url::Url::parse(source_url).context("source_url must be an absolute HTTPS URL")?;
    if parsed.scheme() != "https" {
        anyhow::bail!("source_url must use https");
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        anyhow::bail!("source_url must not contain userinfo");
    }
    if parsed.host_str().is_none() {
        anyhow::bail!("source_url must include a host");
    }
    if parsed.query().is_some() || parsed.fragment().is_some() {
        anyhow::bail!("source_url must not contain query or fragment");
    }
    Ok(())
}

fn validate_workspace_clone_ref(ref_name: &str) -> anyhow::Result<()> {
    let trimmed = ref_name.trim();
    if trimmed.is_empty() || trimmed.len() > 256 {
        anyhow::bail!("git ref must be non-empty and at most 256 bytes");
    }
    if trimmed != ref_name
        || trimmed.starts_with('-')
        || trimmed.contains("..")
        || trimmed.contains("//")
        || trimmed.contains('\\')
        || trimmed.bytes().any(|byte| byte.is_ascii_control())
    {
        anyhow::bail!("git ref contains unsupported characters");
    }
    Ok(())
}

fn validate_git_commit_sha(commit_sha: &str) -> anyhow::Result<()> {
    if commit_sha.len() == 40 && commit_sha.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        anyhow::bail!("resolved commit_sha must be a 40-character hex SHA")
    }
}

fn validate_workspace_destination(
    project_id: &ProjectId,
    data_dir_override: Option<&FsPath>,
    candidate: &FsPath,
) -> anyhow::Result<()> {
    if candidate
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        anyhow::bail!("workspace path must not contain parent components");
    }
    let project_dir = match data_dir_override {
        Some(data_dir) => data_dir.join("projects").join(project_id.as_str()),
        None => ygg_core::paths::project_dir(project_id)?,
    };
    if !candidate.starts_with(&project_dir) {
        anyhow::bail!("workspace path escaped project directory");
    }
    Ok(())
}

fn validate_workspace_staging_containment(
    workspace_dir: &FsPath,
    staging_dir: &FsPath,
) -> anyhow::Result<()> {
    let staging = std::fs::canonicalize(staging_dir).with_context(|| {
        format!(
            "failed to canonicalize workspace staging dir {}",
            staging_dir.display()
        )
    })?;
    let project_dir = workspace_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("workspace dir has no project parent"))?;
    let project_dir = std::fs::canonicalize(project_dir).with_context(|| {
        format!(
            "failed to canonicalize project dir {}",
            project_dir.display()
        )
    })?;
    if !staging.starts_with(&project_dir) {
        anyhow::bail!("workspace staging dir escaped project directory");
    }
    Ok(())
}

fn replace_workspace_from_staging(
    workspace_dir: &FsPath,
    staging_dir: &FsPath,
) -> anyhow::Result<()> {
    let project_dir = workspace_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("workspace dir has no project parent"))?;
    std::fs::create_dir_all(project_dir)?;
    let backup_dir = workspace_dir.with_file_name("workspace.previous");
    if backup_dir.exists() {
        std::fs::remove_dir_all(&backup_dir).with_context(|| {
            format!(
                "failed to remove old workspace backup {}",
                backup_dir.display()
            )
        })?;
    }
    if workspace_dir.exists() {
        std::fs::rename(workspace_dir, &backup_dir).with_context(|| {
            format!(
                "failed to move existing workspace {} to backup",
                workspace_dir.display()
            )
        })?;
    }
    let replace_result = std::fs::rename(staging_dir, workspace_dir).with_context(|| {
        format!(
            "failed to atomically install workspace {}",
            workspace_dir.display()
        )
    });
    if replace_result.is_err() {
        if backup_dir.exists() && !workspace_dir.exists() {
            std::fs::rename(&backup_dir, workspace_dir).ok();
        }
        return replace_result;
    }
    if backup_dir.exists() {
        std::fs::remove_dir_all(&backup_dir).ok();
    }
    Ok(())
}

async fn wait_for_deployment_readiness(port: u16, health_path: Option<&str>) -> anyhow::Result<()> {
    let deadline = Instant::now() + DEPLOY_READINESS_TIMEOUT;

    loop {
        if let Err(error) = probe_loopback_port(port, health_path).await {
            let now = Instant::now();
            if now >= deadline {
                return Err(error.context("readiness deadline expired"));
            }
            sleep(std::cmp::min(DEPLOY_READINESS_INTERVAL, deadline - now)).await;
        } else {
            return Ok(());
        }
    }
}

async fn probe_loopback_port(port: u16, health_path: Option<&str>) -> anyhow::Result<()> {
    timeout(
        DEPLOY_READINESS_CONNECT_TIMEOUT,
        TcpStream::connect(("127.0.0.1", port)),
    )
    .await
    .map_err(|_| anyhow::anyhow!("tcp readiness probe timed out"))?
    .map_err(|error| anyhow::anyhow!("tcp readiness probe failed: {error}"))?;

    if let Some(path) = health_path {
        let url = format!("http://127.0.0.1:{port}{path}");
        let response = hardened_proxy_client()
            .get(url)
            .send()
            .await
            .map_err(|error| anyhow::anyhow!("http readiness probe failed: {error}"))?;
        let status = response.status();
        if status.is_success() || status.is_redirection() || status.is_client_error() {
            return Ok(());
        }
        anyhow::bail!("http readiness probe returned {status}");
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct HealthRouteSnapshot {
    route_id: String,
    port_lease_id: String,
    port: u16,
    ready: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct HealthCounters {
    consecutive_failures: u32,
    consecutive_successes: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HealthProbeResult {
    Success,
    Failure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HealthTransition {
    ready: bool,
    reason: &'static str,
}

fn decide_health_transition(
    current_ready: bool,
    counters: &mut HealthCounters,
    probe_result: HealthProbeResult,
) -> Option<HealthTransition> {
    match probe_result {
        HealthProbeResult::Failure => {
            counters.consecutive_failures = counters.consecutive_failures.saturating_add(1);
            counters.consecutive_successes = 0;
            if current_ready && counters.consecutive_failures >= HEALTH_FAILURE_THRESHOLD {
                return Some(HealthTransition {
                    ready: false,
                    reason: "tcp_probe_failed",
                });
            }
        }
        HealthProbeResult::Success => {
            counters.consecutive_successes = counters.consecutive_successes.saturating_add(1);
            counters.consecutive_failures = 0;
            if !current_ready && counters.consecutive_successes >= HEALTH_RECOVERY_THRESHOLD {
                return Some(HealthTransition {
                    ready: true,
                    reason: "recovered",
                });
            }
        }
    }
    None
}

async fn run_health_supervisor<S>(state: AppState<S>)
where
    S: EventStore,
{
    let mut counters: HashMap<String, HealthCounters> = HashMap::new();
    let mut health_session_id: Option<SessionId> = None;
    loop {
        sleep(HEALTH_POLL_INTERVAL).await;
        let snapshots = health_route_snapshots(&state).await;
        let active_route_ids: HashSet<String> = snapshots
            .iter()
            .map(|snapshot| snapshot.route_id.clone())
            .collect();
        counters.retain(|route_id, _| active_route_ids.contains(route_id));

        for snapshot in snapshots {
            let probe_result = if probe_health_tcp(snapshot.port).await.is_ok() {
                HealthProbeResult::Success
            } else {
                HealthProbeResult::Failure
            };
            let route_counters = counters.entry(snapshot.route_id.clone()).or_default();
            let previous_ready = snapshot.ready;
            let Some(transition) =
                decide_health_transition(previous_ready, route_counters, probe_result)
            else {
                continue;
            };

            let updated = state
                .runtime
                .config()
                .proxy_route_registry
                .set_ready_if_active_with_lease(
                    &snapshot.route_id,
                    &snapshot.port_lease_id,
                    transition.ready,
                )
                .await;
            if updated.is_none() {
                counters.remove(&snapshot.route_id);
                continue;
            }

            let failure_count = route_counters.consecutive_failures;
            let payload = serde_json::json!({
                "route_id": snapshot.route_id,
                "port_lease_id": snapshot.port_lease_id,
                "previous_ready": previous_ready,
                "ready": transition.ready,
                "reason": transition.reason,
                "failure_count": failure_count,
                "probe": { "kind": "tcp" },
            });
            if let Err(error) = state
                .runtime
                .append_event_with_context(
                    &ProtocolContext::host_dev("health_supervisor"),
                    AppendEventRequest {
                        session_id: deployment_health_session(&state, &mut health_session_id).await,
                        writer_package_id: ygg_core::KERNEL_PACKAGE_ID.to_string(),
                        kind: ygg_core::EVENT_DEPLOYMENT_HEALTH.to_string(),
                        payload,
                        metadata: serde_json::json!({}),
                    },
                )
                .await
            {
                eprintln!("deployment health audit append failed: {error}");
            }
        }
    }
}

async fn deployment_health_session<S>(
    state: &AppState<S>,
    cached: &mut Option<SessionId>,
) -> SessionId
where
    S: EventStore,
{
    if let Some(session_id) = cached.as_ref() {
        return session_id.clone();
    }
    match state
        .runtime
        .open_session(OpenSessionRequest {
            labels: vec!["kernel:deployment-health".to_string()],
            metadata: serde_json::json!({"kind":"deployment_health"}),
            ..OpenSessionRequest::default()
        })
        .await
    {
        Ok(session) => {
            *cached = Some(session.id.clone());
            session.id
        }
        Err(_) => "kernel_deployment_health".to_string(),
    }
}

async fn health_route_snapshots<S>(state: &AppState<S>) -> Vec<HealthRouteSnapshot>
where
    S: EventStore,
{
    let routes = state.runtime.config().proxy_route_registry.list().await;
    let mut snapshots = Vec::new();
    for route in routes {
        if route.status != ProxyRouteStatusKind::Active {
            continue;
        }
        let port_lease_id = route.upstream.port_lease_id.clone();
        let Some(lease) = state
            .runtime
            .config()
            .port_lease_registry
            .status(&port_lease_id)
            .await
        else {
            continue;
        };
        if lease.status != PortLeaseStatusKind::Active {
            continue;
        }
        snapshots.push(HealthRouteSnapshot {
            route_id: route.id,
            port_lease_id,
            port: lease.port,
            ready: route.ready,
        });
    }
    snapshots
}

async fn probe_health_tcp(port: u16) -> anyhow::Result<()> {
    timeout(
        HEALTH_PROBE_TIMEOUT,
        TcpStream::connect(("127.0.0.1", port)),
    )
    .await
    .map_err(|_| anyhow::anyhow!("tcp health probe timed out"))?
    .map_err(|error| anyhow::anyhow!("tcp health probe failed: {error}"))?;
    Ok(())
}

async fn rollback_deploy<S>(
    state: &AppState<S>,
    context: &ProtocolContext,
    route_id: &str,
    proxy_registered: bool,
    container_id: Option<&str>,
    lease_id: Option<&str>,
) where
    S: EventStore,
{
    if proxy_registered {
        let _ = call_host_protocol(
            state,
            context,
            "kernel.v1.proxy.unregister",
            serde_json::json!({ "route_id": route_id }),
        )
        .await;
    }
    if let Some(container_id) = container_id {
        let _ = invoke_docker_runtime_lab(
            state,
            context,
            "official/docker-runtime-lab/stop_container",
            serde_json::json!({ "container_id": container_id, "timeout_secs": 5 }),
        )
        .await;
    }
    if let Some(lease_id) = lease_id {
        let _ = call_host_protocol(
            state,
            context,
            "kernel.v1.port.release",
            serde_json::json!({ "lease_id": lease_id }),
        )
        .await;
    }
}

fn validate_host_deploy_request(request: &HostDeployRequest) -> anyhow::Result<()> {
    let image = request.image.trim();
    if !is_safe_docker_image(image) {
        anyhow::bail!("image must be a safe Docker image reference");
    }
    if request.container_port == 0 {
        anyhow::bail!("container_port must be an integer in 1..=65535");
    }
    if !is_safe_route_token(&request.port_name) {
        anyhow::bail!("port_name must be label-safe");
    }
    if !is_safe_route_token(&request.route_id) {
        anyhow::bail!("route_id must be label-safe");
    }
    if let Some(health_path) = request.health_path.as_deref() {
        if !health_path.starts_with('/') || health_path.len() > 256 {
            anyhow::bail!("health_path must start with / and be at most 256 bytes");
        }
    }
    Ok(())
}

fn is_safe_docker_image(image: &str) -> bool {
    !image.is_empty()
        && image.len() <= 255
        && !image.starts_with('-')
        && !image.contains("..")
        && image
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '/' | ':' | '_' | '@' | '-'))
}

fn is_safe_route_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && !value.contains("..")
        && value
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric())
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

fn require_started_container(output: &Value) -> anyhow::Result<String> {
    if output
        .get("docker_performed")
        .and_then(Value::as_bool)
        .is_some_and(|performed| !performed)
        || output
            .get("container_started")
            .and_then(Value::as_bool)
            .is_some_and(|started| !started)
    {
        anyhow::bail!(
            "docker-runtime-lab did not start the container: {}",
            output
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("unknown reason")
        );
    }
    required_string(output, "container_id", "docker-runtime-lab start_container")
}

fn find_managed_container_for_route(output: &Value, route_id: &str) -> Option<String> {
    let managed = output.get("managed")?.as_array()?;
    for container in managed {
        if container.get("route_id").and_then(Value::as_str) != Some(route_id) {
            continue;
        }
        for field in ["container_id", "id", "container_name", "name"] {
            if let Some(value) = optional_string(container, field) {
                return Some(value);
            }
        }
    }
    None
}

fn value_field(value: Value, field: &str, context: &str) -> anyhow::Result<Value> {
    value
        .get(field)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("{context} response missing {field}"))
}

fn required_string(value: &Value, field: &str, context: &str) -> anyhow::Result<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("{context} missing {field}"))
}

fn optional_string(value: &Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
}

fn required_u16(value: &Value, field: &str, context: &str) -> anyhow::Result<u16> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| anyhow::anyhow!("{context} missing valid {field}"))
}

fn protocol_error_to_anyhow(error: ProtocolError) -> anyhow::Error {
    anyhow::anyhow!("{}: {}", error.code, error.message)
}

async fn surface_bundle_file<S>(
    State(state): State<AppState<S>>,
    Path((prefix, file)): Path<(String, String)>,
) -> impl IntoResponse
where
    S: EventStore,
{
    let Some((root, path)) = surface_bundle_path(state, &prefix, &file) else {
        return (StatusCode::NOT_FOUND, "surface bundle path not found").into_response();
    };
    match read_static_file(&root, &path).await {
        StaticRead::Served(mut response) => {
            response.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("no-cache, must-revalidate"),
            );
            response
        }
        StaticRead::Missing | StaticRead::Forbidden => {
            (StatusCode::NOT_FOUND, "surface bundle file not found").into_response()
        }
    }
}

async fn proxy_root<S>(
    State(state): State<AppState<S>>,
    Path(route_id): Path<String>,
    OriginalUri(uri): OriginalUri,
    request: Request,
) -> impl IntoResponse
where
    S: EventStore,
{
    proxy_request(state, route_id, String::new(), uri, request).await
}

async fn proxy_path<S>(
    State(state): State<AppState<S>>,
    Path((route_id, path)): Path<(String, String)>,
    OriginalUri(uri): OriginalUri,
    request: Request,
) -> impl IntoResponse
where
    S: EventStore,
{
    proxy_request(state, route_id, path, uri, request).await
}

async fn proxy_request<S>(
    state: AppState<S>,
    route_id: String,
    path: String,
    uri: Uri,
    request: Request,
) -> Response
where
    S: EventStore,
{
    if is_upgrade_request(request.headers()) {
        if !is_websocket_upgrade_request(request.headers()) {
            return (
                StatusCode::NOT_IMPLEMENTED,
                "upgrade proxy is not implemented",
            )
                .into_response();
        }
        let resolved = match resolve_proxy_upstream(&state, &route_id).await {
            Ok(resolved) => resolved,
            Err(response) => return response,
        };
        if resolved.protocol != ProxyProtocol::Websocket {
            return (
                StatusCode::NOT_IMPLEMENTED,
                "websocket proxy is not configured",
            )
                .into_response();
        }

        let target_url = loopback_websocket_url(resolved.port, &path, uri.query());
        let (mut parts, _) = request.into_parts();
        let upgrade = match WebSocketUpgrade::from_request_parts(&mut parts, &state).await {
            Ok(upgrade) => upgrade,
            Err(rejection) => return rejection.into_response(),
        };
        return upgrade
            .on_upgrade(move |socket| tunnel_websocket(socket, target_url))
            .into_response();
    }

    let resolved = match resolve_proxy_upstream(&state, &route_id).await {
        Ok(resolved) => resolved,
        Err(response) => return response,
    };
    if resolved.protocol != ProxyProtocol::Http {
        return (
            StatusCode::NOT_IMPLEMENTED,
            "websocket proxy is not implemented",
        )
            .into_response();
    }

    let method = request.method().clone();
    let request_headers = request.headers().clone();
    let body = match to_bytes(request.into_body(), PROXY_REQUEST_BODY_LIMIT_BYTES).await {
        Ok(body) => body,
        Err(_) => {
            return (
                StatusCode::PAYLOAD_TOO_LARGE,
                "proxy request body too large",
            )
                .into_response()
        }
    };

    let target_url = loopback_proxy_url(resolved.port, &path, uri.query());
    let client = hardened_proxy_client();
    let mut upstream = client.request(method, target_url).body(body);
    for (name, value) in request_headers.iter() {
        if should_forward_request_header(name) {
            upstream = upstream.header(name, value);
        }
    }

    let upstream_response = match upstream.send().await {
        Ok(response) => response,
        Err(_) => {
            return (StatusCode::BAD_GATEWAY, "proxy upstream request failed").into_response()
        }
    };
    let status = upstream_response.status();
    let headers = proxied_response_headers(upstream_response.headers());
    match read_limited_response_body(upstream_response, PROXY_RESPONSE_BODY_LIMIT_BYTES).await {
        Ok(body) => (status, headers, body).into_response(),
        Err(ProxyReadError::TooLarge) => {
            (StatusCode::BAD_GATEWAY, "proxy upstream response too large").into_response()
        }
        Err(_) => (StatusCode::BAD_GATEWAY, "proxy upstream response failed").into_response(),
    }
}

struct ResolvedProxyUpstream {
    protocol: ProxyProtocol,
    port: u16,
}

async fn resolve_proxy_upstream<S>(
    state: &AppState<S>,
    route_id: &str,
) -> Result<ResolvedProxyUpstream, Response>
where
    S: EventStore,
{
    let route = match state
        .runtime
        .config()
        .proxy_route_registry
        .status(route_id)
        .await
    {
        Some(route) if route.status == ProxyRouteStatusKind::Active && route.ready => route,
        Some(route) if route.status == ProxyRouteStatusKind::Active => {
            return Err((StatusCode::SERVICE_UNAVAILABLE, "deployment not ready").into_response())
        }
        _ => return Err((StatusCode::NOT_FOUND, "proxy route not found").into_response()),
    };

    let lease = match state
        .runtime
        .config()
        .port_lease_registry
        .status(&route.upstream.port_lease_id)
        .await
    {
        Some(lease) if lease.status == PortLeaseStatusKind::Active => lease,
        _ => return Err((StatusCode::NOT_FOUND, "proxy upstream not found").into_response()),
    };

    if lease.host != "127.0.0.1" || lease.bind != PortBindScope::LoopbackOnly {
        return Err((
            StatusCode::BAD_GATEWAY,
            "proxy upstream is not loopback-only",
        )
            .into_response());
    }
    if lease.port_name != route.upstream.port_name {
        return Err((StatusCode::BAD_GATEWAY, "proxy upstream port name mismatch").into_response());
    }
    if !matches!(lease.protocol, ygg_runtime::PortProtocol::Tcp) {
        return Err((StatusCode::BAD_GATEWAY, "proxy upstream must be tcp").into_response());
    }

    Ok(ResolvedProxyUpstream {
        protocol: route.protocol,
        port: lease.port,
    })
}

async fn tunnel_websocket(downstream: WebSocket, target_url: String) {
    let Ok(request) = target_url.as_str().into_client_request() else {
        return;
    };
    let Ok((upstream, _)) = tokio_tungstenite::connect_async(request).await else {
        return;
    };

    let (mut downstream_tx, mut downstream_rx) = downstream.split();
    let (mut upstream_tx, mut upstream_rx) = upstream.split();

    loop {
        tokio::select! {
            downstream_msg = downstream_rx.next() => {
                let Some(Ok(message)) = downstream_msg else { break; };
                if axum_ws_message_too_large(&message) { break; }
                match axum_to_tungstenite_message(message) {
                    Some((message, should_close)) => {
                        if upstream_tx.send(message).await.is_err() { break; }
                        if should_close { break; }
                    }
                    None => {}
                }
            }
            upstream_msg = upstream_rx.next() => {
                let Some(Ok(message)) = upstream_msg else { break; };
                if tungstenite_message_too_large(&message) { break; }
                match tungstenite_to_axum_message(message) {
                    Some((message, should_close)) => {
                        if downstream_tx.send(message).await.is_err() { break; }
                        if should_close { break; }
                    }
                    None => {}
                }
            }
        }
    }

    let _ = upstream_tx
        .send(tokio_tungstenite::tungstenite::Message::Close(None))
        .await;
    let _ = downstream_tx.send(AxumWsMessage::Close(None)).await;
}

fn axum_to_tungstenite_message(
    message: AxumWsMessage,
) -> Option<(tokio_tungstenite::tungstenite::Message, bool)> {
    match message {
        AxumWsMessage::Text(text) => {
            Some((tokio_tungstenite::tungstenite::Message::Text(text), false))
        }
        AxumWsMessage::Binary(bytes) => Some((
            tokio_tungstenite::tungstenite::Message::Binary(bytes),
            false,
        )),
        AxumWsMessage::Ping(bytes) => {
            Some((tokio_tungstenite::tungstenite::Message::Ping(bytes), false))
        }
        AxumWsMessage::Pong(bytes) => {
            Some((tokio_tungstenite::tungstenite::Message::Pong(bytes), false))
        }
        AxumWsMessage::Close(_) => {
            Some((tokio_tungstenite::tungstenite::Message::Close(None), true))
        }
    }
}

fn tungstenite_to_axum_message(
    message: tokio_tungstenite::tungstenite::Message,
) -> Option<(AxumWsMessage, bool)> {
    match message {
        tokio_tungstenite::tungstenite::Message::Text(text) => {
            Some((AxumWsMessage::Text(text.to_string()), false))
        }
        tokio_tungstenite::tungstenite::Message::Binary(bytes) => {
            Some((AxumWsMessage::Binary(bytes), false))
        }
        tokio_tungstenite::tungstenite::Message::Ping(bytes) => {
            Some((AxumWsMessage::Ping(bytes), false))
        }
        tokio_tungstenite::tungstenite::Message::Pong(bytes) => {
            Some((AxumWsMessage::Pong(bytes), false))
        }
        tokio_tungstenite::tungstenite::Message::Close(_) => {
            Some((AxumWsMessage::Close(None), true))
        }
        tokio_tungstenite::tungstenite::Message::Frame(_) => None,
    }
}

fn axum_ws_message_too_large(message: &AxumWsMessage) -> bool {
    (match message {
        AxumWsMessage::Text(text) => text.len(),
        AxumWsMessage::Binary(bytes) | AxumWsMessage::Ping(bytes) | AxumWsMessage::Pong(bytes) => {
            bytes.len()
        }
        AxumWsMessage::Close(_) => 0,
    }) > PROXY_WEBSOCKET_FRAME_LIMIT_BYTES
}

fn tungstenite_message_too_large(message: &tokio_tungstenite::tungstenite::Message) -> bool {
    (match message {
        tokio_tungstenite::tungstenite::Message::Text(text) => text.len(),
        tokio_tungstenite::tungstenite::Message::Binary(bytes)
        | tokio_tungstenite::tungstenite::Message::Ping(bytes)
        | tokio_tungstenite::tungstenite::Message::Pong(bytes) => bytes.len(),
        tokio_tungstenite::tungstenite::Message::Close(_)
        | tokio_tungstenite::tungstenite::Message::Frame(_) => 0,
    }) > PROXY_WEBSOCKET_FRAME_LIMIT_BYTES
}

fn hardened_proxy_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .expect("hardened proxy client builds")
    })
}

#[derive(Debug)]
enum ProxyReadError {
    Upstream,
    TooLarge,
}

async fn read_limited_response_body(
    response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, ProxyReadError> {
    let mut stream = response.bytes_stream();
    let mut body = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| ProxyReadError::Upstream)?;
        if body.len().saturating_add(chunk.len()) > limit {
            return Err(ProxyReadError::TooLarge);
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn loopback_proxy_url(port: u16, path: &str, query: Option<&str>) -> String {
    loopback_proxy_url_with_scheme("http", port, path, query)
}

fn loopback_websocket_url(port: u16, path: &str, query: Option<&str>) -> String {
    loopback_proxy_url_with_scheme("ws", port, path, query)
}

fn loopback_proxy_url_with_scheme(
    scheme: &str,
    port: u16,
    path: &str,
    query: Option<&str>,
) -> String {
    let path = if path.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", path.trim_start_matches('/'))
    };
    let mut url = format!("{scheme}://127.0.0.1:{port}{path}");
    if let Some(query) = sanitized_proxy_query(query) {
        url.push('?');
        url.push_str(&query);
    }
    url
}

fn sanitized_proxy_query(query: Option<&str>) -> Option<String> {
    let query = query?;
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    let mut any = false;
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        if key == "access_token" {
            continue;
        }
        serializer.append_pair(&key, &value);
        any = true;
    }
    any.then(|| serializer.finish())
}

fn is_upgrade_request(headers: &HeaderMap) -> bool {
    headers.contains_key(header::UPGRADE)
        || headers
            .get(header::CONNECTION)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| {
                value
                    .split(',')
                    .any(|part| part.trim().eq_ignore_ascii_case("upgrade"))
            })
}

fn is_websocket_upgrade_request(headers: &HeaderMap) -> bool {
    headers
        .get(header::UPGRADE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("websocket"))
        && headers
            .get(header::CONNECTION)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| {
                value
                    .split(',')
                    .any(|part| part.trim().eq_ignore_ascii_case("upgrade"))
            })
}

fn should_forward_request_header(name: &header::HeaderName) -> bool {
    matches!(
        name,
        &header::ACCEPT
            | &header::ACCEPT_LANGUAGE
            | &header::CACHE_CONTROL
            | &header::CONTENT_TYPE
            | &header::IF_MATCH
            | &header::IF_MODIFIED_SINCE
            | &header::IF_NONE_MATCH
            | &header::IF_UNMODIFIED_SINCE
            | &header::ORIGIN
            | &header::RANGE
            | &header::USER_AGENT
    )
}

fn proxied_response_headers(headers: &HeaderMap) -> HeaderMap {
    let mut out = HeaderMap::new();
    for (name, value) in headers.iter() {
        if should_forward_response_header(name) {
            out.append(name, value.clone());
        }
    }
    out
}

fn should_forward_response_header(name: &header::HeaderName) -> bool {
    !matches!(
        name,
        &header::CONNECTION
            | &header::PROXY_AUTHENTICATE
            | &header::PROXY_AUTHORIZATION
            | &header::TE
            | &header::TRAILER
            | &header::TRANSFER_ENCODING
            | &header::UPGRADE
            | &header::SET_COOKIE
            | &header::LOCATION
    ) && !name.as_str().eq_ignore_ascii_case("keep-alive")
        && !name
            .as_str()
            .to_ascii_lowercase()
            .starts_with("access-control-")
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

    if !is_spa_fallback_path(request_path) {
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
        || path == "/kernel"
        || path.starts_with("/kernel/")
        || path == "/p"
        || path.starts_with("/p/")
        || path == "/host"
        || path.starts_with("/host/")
        || path == "/surface-bundles"
        || path.starts_with("/surface-bundles/")
}

fn is_spa_fallback_path(path: &str) -> bool {
    path == "/"
        || path
            .strip_prefix("/project/")
            .is_some_and(is_valid_project_path_segment)
}

fn is_valid_project_path_segment(segment: &str) -> bool {
    !segment.is_empty()
        && !segment.contains('/')
        && segment.len() <= 128
        && url::form_urlencoded::parse(format!("id={segment}").as_bytes())
            .next()
            .is_some_and(|(_, decoded)| {
                !decoded.contains('/')
                    && decoded.as_ref() != "."
                    && decoded.as_ref() != ".."
                    && !decoded.starts_with('.')
                    && !decoded.contains("..")
                    && decoded
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_alphanumeric())
                    && decoded
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
            })
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

async fn read_static_file(static_root: &FsPath, path: &FsPath) -> StaticRead {
    let static_root = match std::fs::canonicalize(static_root) {
        Ok(path) => path,
        Err(_) => return StaticRead::Missing,
    };
    let canonical = match std::fs::canonicalize(path) {
        Ok(path) => path,
        Err(_) => return StaticRead::Missing,
    };
    if !canonical.starts_with(&static_root) || !canonical.is_file() {
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

fn surface_bundle_path<S>(
    state: AppState<S>,
    prefix: &str,
    file: &str,
) -> Option<(PathBuf, PathBuf)>
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
        let root = recoverable_project_dist_dir(&project_id)?;
        let path = root.join(rest);
        return Some((root, path));
    }

    let Some(base) = state.runtime.config().surface_dev_paths.get(prefix) else {
        return None;
    };
    let root = PathBuf::from(base);
    let path = root.join(safe_file);
    Some((root, path))
}

fn recoverable_project_dist_dir(project_id: &ygg_core::project::ProjectId) -> Option<PathBuf> {
    let project_dir = ygg_core::paths::project_dir(project_id).ok()?;
    let dist = project_dir.join("dist");
    if dist.is_dir() {
        return Some(dist);
    }
    latest_dist_backup(&project_dir)
}

fn latest_dist_backup(project_dir: &FsPath) -> Option<PathBuf> {
    let mut candidates = std::fs::read_dir(project_dir)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_name()?.to_str()?;
            if !name.starts_with(".dist.bak-") || !path.is_dir() {
                return None;
            }
            let modified = entry.metadata().and_then(|meta| meta.modified()).ok();
            Some((modified, path))
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(modified, _)| *modified);
    candidates.pop().map(|(_, path)| path)
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
    use std::net::SocketAddr;

    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use tokio::net::TcpStream;
    use tokio::sync::Mutex;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tokio_tungstenite::tungstenite::Message;
    use tower::ServiceExt;
    use ygg_runtime::{
        ExecutionTargetId, PortLeaseRequest, PortProtocol, ProxyProtocol,
        ProxyRouteRegisterRequest, ProxyRouteUpstream,
    };

    use super::*;

    #[test]
    fn workspace_clone_url_policy_rejects_unsafe_sources() {
        for url in [
            "file:///tmp/repo",
            "ssh://git@example.com/repo.git",
            "git@example.com:repo.git",
            "https://user@example.com/repo.git",
            "https://user:token@example.com/repo.git",
            "https://example.com/repo.git?token=secret",
            "https://example.com/repo.git#token",
            "/tmp/local-repo",
        ] {
            assert!(validate_workspace_clone_url(url).is_err(), "accepted {url}");
        }
        assert!(validate_workspace_clone_url("https://example.com/org/repo.git").is_ok());
    }

    #[test]
    fn workspace_clone_invocation_uses_expected_git_tools_shape() -> anyhow::Result<()> {
        let project_id = ProjectId::new("clone__abc123")?;
        let request = ProjectWorkspaceCloneRequest {
            project_id: project_id.clone(),
            source_url: "https://example.com/org/repo.git".to_string(),
            ref_name: "refs/heads/main".to_string(),
        };
        let data_dir = tempfile::tempdir()?;
        let invocation = build_project_workspace_clone_invocation(&request, Some(data_dir.path()))?;

        assert_eq!(
            invocation.workspace_dir,
            data_dir.path().join("projects/clone__abc123/workspace")
        );
        assert_eq!(
            invocation.staging_dir,
            data_dir
                .path()
                .join("projects/clone__abc123/workspace.staging")
        );
        assert_eq!(
            invocation.resolve_ref_params,
            json!({"remote_url":"https://example.com/org/repo.git","ref":"refs/heads/main"})
        );
        assert_eq!(
            invocation.fetch_tree_params,
            json!({
                "remote_url":"https://example.com/org/repo.git",
                "ref_name":"refs/heads/main",
                "dest_dir": invocation.staging_dir.to_string_lossy(),
            })
        );
        Ok(())
    }

    #[test]
    fn workspace_destination_rejects_escape() -> anyhow::Result<()> {
        let project_id = ProjectId::new("clone__abc123")?;
        let data_dir = tempfile::tempdir()?;
        assert!(validate_workspace_destination(
            &project_id,
            Some(data_dir.path()),
            &data_dir.path().join("projects/clone__abc123/workspace")
        )
        .is_ok());
        assert!(validate_workspace_destination(
            &project_id,
            Some(data_dir.path()),
            &data_dir.path().join("projects/other__abc123/workspace")
        )
        .is_err());
        assert!(validate_workspace_destination(
            &project_id,
            Some(data_dir.path()),
            &data_dir
                .path()
                .join("projects/clone__abc123/../other/workspace")
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn health_transition_decision_respects_thresholds() {
        let mut counters = HealthCounters::default();
        assert!(
            decide_health_transition(true, &mut counters, HealthProbeResult::Failure).is_none()
        );
        assert!(
            decide_health_transition(true, &mut counters, HealthProbeResult::Failure).is_none()
        );
        assert_eq!(
            decide_health_transition(true, &mut counters, HealthProbeResult::Failure),
            Some(HealthTransition {
                ready: false,
                reason: "tcp_probe_failed"
            })
        );
        assert_eq!(counters.consecutive_failures, HEALTH_FAILURE_THRESHOLD);
        assert_eq!(counters.consecutive_successes, 0);

        assert!(
            decide_health_transition(false, &mut counters, HealthProbeResult::Success).is_none()
        );
        assert_eq!(counters.consecutive_failures, 0);
        assert_eq!(counters.consecutive_successes, 1);
        assert_eq!(
            decide_health_transition(false, &mut counters, HealthProbeResult::Success),
            Some(HealthTransition {
                ready: true,
                reason: "recovered"
            })
        );
        assert_eq!(counters.consecutive_successes, HEALTH_RECOVERY_THRESHOLD);
    }

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
            .clone()
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

        let project_route = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/project/demo-project")
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(project_route.status(), StatusCode::OK);

        let random_route = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/unknown-route")
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(random_route.status(), StatusCode::NOT_FOUND);
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
    async fn token_gate_protects_host_deploy() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
        let app = app_with_state(AppState {
            runtime,
            static_dir: None,
            access_token: Some("deploy-token".to_string()),
        });

        let denied = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/host/v1/deploy")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "image": "example/app:latest",
                            "container_port": 8080,
                            "port_name": "web",
                            "route_id": "route-1",
                            "pull_if_missing": false,
                        })
                        .to_string(),
                    ))?,
            )
            .await?;
        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
        Ok(())
    }

    #[tokio::test]
    async fn host_deploy_rolls_back_port_lease_when_docker_start_fails() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
        let app = app_with_state(AppState {
            runtime: runtime.clone(),
            static_dir: None,
            access_token: None,
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/host/v1/deploy")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "image": "example/app:latest",
                            "container_port": 8080,
                            "port_name": "web",
                            "route_id": "rollback-route",
                            "pull_if_missing": false,
                        })
                        .to_string(),
                    ))?,
            )
            .await?;
        assert!(!response.status().is_success());

        let context = ProtocolContext::host_dev("host_deploy_test");
        let leases = runtime
            .call_protocol(&context, "kernel.v1.port.list", json!({}))
            .await
            .map_err(protocol_error_to_anyhow)?;
        assert!(leases
            .as_array()
            .expect("port.list returns array")
            .iter()
            .all(|lease| lease["status"] != "active"));

        let routes = runtime
            .call_protocol(&context, "kernel.v1.proxy.list", json!({}))
            .await
            .map_err(protocol_error_to_anyhow)?;
        assert!(routes
            .as_array()
            .expect("proxy.list returns array")
            .iter()
            .all(|route| route["status"] != "active"));
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
    async fn proxy_route_is_token_protected() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
        let app = app_with_state(AppState {
            runtime,
            static_dir: None,
            access_token: Some("proxy-token".to_string()),
        });

        let denied = app
            .oneshot(
                Request::builder()
                    .uri("/p/missing/app")
                    .body(Body::empty())?,
            )
            .await?;

        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
        Ok(())
    }

    #[tokio::test]
    async fn proxy_route_not_found_returns_404() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
        let app = app_with_state(AppState {
            runtime,
            static_dir: None,
            access_token: Some("proxy-token".to_string()),
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/p/missing/app")
                    .header(header::AUTHORIZATION, "Bearer proxy-token")
                    .body(Body::empty())?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        Ok(())
    }

    #[tokio::test]
    async fn proxy_route_requires_ready_before_forwarding() -> anyhow::Result<()> {
        let upstream = Router::new().fallback(any(|| async { "ready upstream" }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let upstream_addr: SocketAddr = listener.local_addr()?;
        tokio::spawn(async move {
            axum::serve(listener, upstream)
                .await
                .expect("test upstream serves");
        });

        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
        let lease = runtime
            .config()
            .port_lease_registry
            .lease(PortLeaseRequest {
                target_id: ExecutionTargetId::from("local"),
                port_name: "web".to_string(),
                protocol: PortProtocol::Tcp,
                requested_port: Some(upstream_addr.port()),
            })
            .await;
        runtime
            .config()
            .proxy_route_registry
            .register(ProxyRouteRegisterRequest {
                route_id: Some("readiness-route".to_string()),
                upstream: ProxyRouteUpstream {
                    port_lease_id: lease.lease.id.clone(),
                    port_name: "web".to_string(),
                },
                protocol: ProxyProtocol::Http,
            })
            .await;
        let app = app_with_state(AppState {
            runtime: runtime.clone(),
            static_dir: None,
            access_token: None,
        });

        let not_ready = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/p/readiness-route/app")
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(not_ready.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = to_bytes(not_ready.into_body(), usize::MAX).await?;
        assert_eq!(&body[..], b"deployment not ready");

        runtime
            .config()
            .proxy_route_registry
            .set_ready("readiness-route", true)
            .await;

        let ready = app
            .oneshot(
                Request::builder()
                    .uri("/p/readiness-route/app")
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(ready.status(), StatusCode::OK);
        let body = to_bytes(ready.into_body(), usize::MAX).await?;
        assert_eq!(&body[..], b"ready upstream");
        Ok(())
    }

    #[derive(Debug, Default)]
    struct ObservedProxyRequest {
        method: String,
        path: String,
        query: Option<String>,
        authorization: Option<String>,
        body: Vec<u8>,
    }

    #[tokio::test]
    async fn proxy_forwards_request_and_strips_ygg_credentials() -> anyhow::Result<()> {
        let observed = Arc::new(Mutex::new(None::<ObservedProxyRequest>));
        let upstream_observed = observed.clone();
        let upstream = Router::new()
            .fallback(any(
                move |State(observed): State<Arc<Mutex<Option<ObservedProxyRequest>>>>,
                      OriginalUri(uri): OriginalUri,
                      request: axum::extract::Request| async move {
                    let method = request.method().to_string();
                    let authorization = request
                        .headers()
                        .get(header::AUTHORIZATION)
                        .and_then(|value| value.to_str().ok())
                        .map(str::to_string);
                    let body = to_bytes(request.into_body(), usize::MAX)
                        .await
                        .expect("read upstream body")
                        .to_vec();
                    *observed.lock().await = Some(ObservedProxyRequest {
                        method,
                        path: uri.path().to_string(),
                        query: uri.query().map(str::to_string),
                        authorization,
                        body,
                    });
                    (
                        StatusCode::CREATED,
                        [("x-upstream-safe", "yes")],
                        "proxied response",
                    )
                },
            ))
            .with_state(upstream_observed);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let upstream_addr: SocketAddr = listener.local_addr()?;
        tokio::spawn(async move {
            axum::serve(listener, upstream)
                .await
                .expect("test upstream serves");
        });

        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
        let lease = runtime
            .config()
            .port_lease_registry
            .lease(PortLeaseRequest {
                target_id: ExecutionTargetId::from("local"),
                port_name: "web".to_string(),
                protocol: PortProtocol::Tcp,
                requested_port: Some(upstream_addr.port()),
            })
            .await;
        runtime
            .config()
            .proxy_route_registry
            .register(ProxyRouteRegisterRequest {
                route_id: Some("route-1".to_string()),
                upstream: ProxyRouteUpstream {
                    port_lease_id: lease.lease.id.clone(),
                    port_name: "web".to_string(),
                },
                protocol: ProxyProtocol::Http,
            })
            .await;
        runtime
            .config()
            .proxy_route_registry
            .set_ready("route-1", true)
            .await;
        let app = app_with_state(AppState {
            runtime,
            static_dir: None,
            access_token: Some("proxy-token".to_string()),
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/p/route-1/api/widgets?keep=1&access_token=proxy-token")
                    .header(header::AUTHORIZATION, "Bearer proxy-token")
                    .header(header::CONTENT_TYPE, "text/plain")
                    .body(Body::from("hello upstream"))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            response.headers().get("x-upstream-safe"),
            Some(&HeaderValue::from_static("yes"))
        );
        let response_body = to_bytes(response.into_body(), usize::MAX).await?;
        assert_eq!(&response_body[..], b"proxied response");

        let observed = observed.lock().await.take().expect("upstream request");
        assert_eq!(observed.method, "POST");
        assert_eq!(observed.path, "/api/widgets");
        assert_eq!(observed.query.as_deref(), Some("keep=1"));
        assert_eq!(observed.body, b"hello upstream");
        assert!(observed.authorization.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn proxy_does_not_forward_referer_or_follow_redirects_and_strips_dangerous_response_headers(
    ) -> anyhow::Result<()> {
        let observed_referer = Arc::new(Mutex::new(None::<String>));
        let upstream_referer = observed_referer.clone();
        let upstream = Router::new()
            .fallback(any(
                move |State(observed_referer): State<Arc<Mutex<Option<String>>>>,
                      headers: HeaderMap| async move {
                    *observed_referer.lock().await = headers
                        .get(header::REFERER)
                        .and_then(|value| value.to_str().ok())
                        .map(str::to_string);
                    (
                        StatusCode::FOUND,
                        [
                            (header::LOCATION, "http://169.254.169.254/latest/meta-data"),
                            (header::SET_COOKIE, "session=leak; Path=/"),
                            (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
                        ],
                        "redirect body",
                    )
                },
            ))
            .with_state(upstream_referer);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let upstream_addr: SocketAddr = listener.local_addr()?;
        tokio::spawn(async move {
            axum::serve(listener, upstream)
                .await
                .expect("test upstream serves");
        });

        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
        let lease = runtime
            .config()
            .port_lease_registry
            .lease(PortLeaseRequest {
                target_id: ExecutionTargetId::from("local"),
                port_name: "web".to_string(),
                protocol: PortProtocol::Tcp,
                requested_port: Some(upstream_addr.port()),
            })
            .await;
        runtime
            .config()
            .proxy_route_registry
            .register(ProxyRouteRegisterRequest {
                route_id: Some("redirect-route".to_string()),
                upstream: ProxyRouteUpstream {
                    port_lease_id: lease.lease.id.clone(),
                    port_name: "web".to_string(),
                },
                protocol: ProxyProtocol::Http,
            })
            .await;
        runtime
            .config()
            .proxy_route_registry
            .set_ready("redirect-route", true)
            .await;
        let app = app_with_state(AppState {
            runtime,
            static_dir: None,
            access_token: Some("proxy-token".to_string()),
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/p/redirect-route/redirect?access_token=proxy-token")
                    .header(header::AUTHORIZATION, "Bearer proxy-token")
                    .header(
                        header::REFERER,
                        "http://localhost/p/redirect-route/redirect?access_token=proxy-token",
                    )
                    .body(Body::empty())?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::FOUND);
        assert!(response.headers().get(header::LOCATION).is_none());
        assert!(response.headers().get(header::SET_COOKIE).is_none());
        assert!(response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none());
        let body = to_bytes(response.into_body(), usize::MAX).await?;
        assert_eq!(&body[..], b"redirect body");
        assert!(observed_referer.lock().await.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn websocket_proxy_route_is_token_protected() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
        let app = app_with_state(AppState {
            runtime,
            static_dir: None,
            access_token: Some("ws-token".to_string()),
        });

        let denied = app
            .oneshot(
                Request::builder()
                    .uri("/p/missing/ws")
                    .header(header::CONNECTION, "upgrade")
                    .header(header::UPGRADE, "websocket")
                    .header("sec-websocket-version", "13")
                    .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
                    .body(Body::empty())?,
            )
            .await?;

        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
        Ok(())
    }

    #[tokio::test]
    async fn websocket_proxy_requires_websocket_route() -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let upstream_addr: SocketAddr = listener.local_addr()?;
        drop(listener);

        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
        let lease = runtime
            .config()
            .port_lease_registry
            .lease(PortLeaseRequest {
                target_id: ExecutionTargetId::from("local"),
                port_name: "web".to_string(),
                protocol: PortProtocol::Tcp,
                requested_port: Some(upstream_addr.port()),
            })
            .await;
        runtime
            .config()
            .proxy_route_registry
            .register(ProxyRouteRegisterRequest {
                route_id: Some("http-only".to_string()),
                upstream: ProxyRouteUpstream {
                    port_lease_id: lease.lease.id.clone(),
                    port_name: "web".to_string(),
                },
                protocol: ProxyProtocol::Http,
            })
            .await;
        runtime
            .config()
            .proxy_route_registry
            .set_ready("http-only", true)
            .await;
        let app = app_with_state(AppState {
            runtime,
            static_dir: None,
            access_token: None,
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/p/http-only/ws")
                    .header(header::CONNECTION, "upgrade")
                    .header(header::UPGRADE, "websocket")
                    .header("sec-websocket-version", "13")
                    .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
                    .body(Body::empty())?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
        Ok(())
    }

    #[tokio::test]
    async fn websocket_proxy_echoes_text_and_strips_access_token() -> anyhow::Result<()> {
        let observed = Arc::new(Mutex::new(
            None::<(String, Option<String>, Option<String>, Option<String>)>,
        ));
        let upstream_observed = observed.clone();
        let upstream = Router::new()
            .fallback(any(
                move |State(observed): State<
                    Arc<Mutex<Option<(String, Option<String>, Option<String>, Option<String>)>>>,
                >,
                      ws: WebSocketUpgrade,
                      OriginalUri(uri): OriginalUri,
                      headers: HeaderMap| async move {
                    let authorization = headers
                        .get(header::AUTHORIZATION)
                        .and_then(|value| value.to_str().ok())
                        .map(str::to_string);
                    let cookie = headers
                        .get(header::COOKIE)
                        .and_then(|value| value.to_str().ok())
                        .map(str::to_string);
                    *observed.lock().await = Some((
                        uri.path().to_string(),
                        uri.query().map(str::to_string),
                        authorization,
                        cookie,
                    ));
                    ws.on_upgrade(|mut socket| async move {
                        if let Some(Ok(message)) = socket.recv().await {
                            let _ = socket.send(message).await;
                        }
                    })
                },
            ))
            .with_state(upstream_observed);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let upstream_addr: SocketAddr = listener.local_addr()?;
        tokio::spawn(async move {
            axum::serve(listener, upstream)
                .await
                .expect("test upstream serves");
        });

        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Arc::new(Runtime::new(store, RuntimeConfig::default()));
        let lease = runtime
            .config()
            .port_lease_registry
            .lease(PortLeaseRequest {
                target_id: ExecutionTargetId::from("local"),
                port_name: "ws".to_string(),
                protocol: PortProtocol::Tcp,
                requested_port: Some(upstream_addr.port()),
            })
            .await;
        runtime
            .config()
            .proxy_route_registry
            .register(ProxyRouteRegisterRequest {
                route_id: Some("ws-route".to_string()),
                upstream: ProxyRouteUpstream {
                    port_lease_id: lease.lease.id.clone(),
                    port_name: "ws".to_string(),
                },
                protocol: ProxyProtocol::Websocket,
            })
            .await;
        runtime
            .config()
            .proxy_route_registry
            .set_ready("ws-route", true)
            .await;
        let app = app_with_state(AppState {
            runtime,
            static_dir: None,
            access_token: Some("ws-token".to_string()),
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let proxy_addr = listener.local_addr()?;
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("test proxy serves");
        });

        let mut request =
            format!("ws://{proxy_addr}/p/ws-route/socket/echo?keep=1&access_token=ws-token")
                .into_client_request()?;
        request.headers_mut().insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer ws-token"),
        );
        request
            .headers_mut()
            .insert(header::COOKIE, HeaderValue::from_static("session=secret"));
        let stream = TcpStream::connect(proxy_addr).await?;
        let (mut socket, _) = tokio_tungstenite::client_async(request, stream).await?;
        socket.send(Message::Text("hello ws".into())).await?;

        match socket.next().await.transpose()? {
            Some(Message::Text(text)) => assert_eq!(text, "hello ws"),
            other => panic!("unexpected websocket response: {other:?}"),
        }

        let observed = observed
            .lock()
            .await
            .take()
            .expect("upstream websocket request");
        assert_eq!(observed.0, "/socket/echo");
        assert_eq!(observed.1.as_deref(), Some("keep=1"));
        assert!(observed.2.is_none());
        assert!(observed.3.is_none());
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
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL),
            Some(&HeaderValue::from_static("no-cache, must-revalidate"))
        );
        let bytes = to_bytes(response.into_body(), usize::MAX).await?;
        assert_eq!(&bytes[..], b"export const ok = true;");
        Ok(())
    }

    #[tokio::test]
    async fn surface_bundle_serving_rejects_symlink_escape() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let outside = tempfile::tempdir()?;
        let bundle_dir = dir.path().join("surface");
        std::fs::create_dir_all(&bundle_dir)?;
        std::fs::write(outside.path().join("secret.mjs"), "do-not-serve")?;

        #[cfg(unix)]
        std::os::unix::fs::symlink(
            outside.path().join("secret.mjs"),
            bundle_dir.join("leak.mjs"),
        )?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(
            outside.path().join("secret.mjs"),
            bundle_dir.join("leak.mjs"),
        )?;

        let store = Arc::new(InMemoryEventStore::default());
        let mut config = RuntimeConfig::default();
        config
            .surface_dev_paths
            .insert("test".to_string(), bundle_dir.to_string_lossy().to_string());
        let runtime = Arc::new(Runtime::new(store, config));
        let app = app_with_state(AppState {
            runtime,
            static_dir: None,
            access_token: None,
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/surface-bundles/test/leak.mjs")
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
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
        assert!(
            response.headers().get(header::CACHE_CONTROL).is_none(),
            "generic static assets should not inherit surface bundle cache policy"
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

        for path in [
            "/rpc/anything",
            "/kernel/anything",
            "/p/anything",
            "/surface-bundles/anything",
            "/surface-bundlesx",
        ] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty())?)
                .await?;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "path {path}");
        }
        for path in ["/kernelx", "/project/bad/id", "/project/bad%2Fid"] {
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
