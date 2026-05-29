use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::RwLock;
use ygg_core::{
    project::ProjectId, AssetRecord, EventEnvelope, KernelSession, PackageId, SessionId,
    SessionStatus, EVENT_ASSET_PUT, EVENT_EXEC_DENIED, EVENT_EXEC_STARTED, EVENT_EXEC_STOPPED,
    EVENT_PERMISSION_GRANTED, EVENT_PERMISSION_REVOKED, EVENT_PORT_LEASED, EVENT_PORT_RELEASED,
    EVENT_PROJECTION_UPDATED, EVENT_PROXY_REGISTERED, EVENT_PROXY_UNREGISTERED,
    EVENT_SESSION_FORKED,
};

use crate::{
    EventStore, HostPolicy, InprocPackageCatalog, ProjectRegistry, ProjectScopeContext,
    SecretResolverConfig,
};

mod assets;
mod audit;
mod branches;
mod capabilities;
mod events;
mod handles;
mod hooks;
mod local_exec;
mod network;
mod outbound;
mod outbound_sse;
mod outbound_websocket;
mod packages;
mod permissions;
mod projections;
mod proposals;
mod protocol;
mod protocol_dispatch;
mod remote;
mod session;
mod streaming;
mod wasm;

// Re-export public types so old paths like ygg_runtime::runtime::AssetPutRequest keep working.
pub use self::assets::{
    content_address, standard_asset_metadata, AssetGetResponse, AssetPutRequest,
};
pub use self::audit::{
    AuditPackageParams, DeclaredAuthority, PackageAuditReport, TighteningSuggestion,
    UnusedAuthority, UsedAuthority,
};
pub use self::branches::BranchRecord;
pub use self::events::{AppendEventRequest, EventListRequest};
pub use self::handles::HandleTable;
pub use self::local_exec::{
    DenyAllLocalExecExecutor, ExecCommand, ExecId, ExecLifecyclePolicy, ExecRegistry,
    ExecResourceLimits, ExecStatus, ExecStatusKind, ExecutionTarget, ExecutionTargetCapability,
    ExecutionTargetId, ExecutionTargetReachability, ExecutionTargetRegistry,
    ExecutionTargetStatusKind, FakeLocalExecExecutor, LiveLocalExecExecutor,
    LiveLocalExecExecutorConfig, LocalExecExecutor, LocalExecExecutorConfig, LocalExecListResponse,
    LocalExecLogLine, LocalExecLogStream, LocalExecLogsRequest, LocalExecLogsResponse,
    LocalExecStartRequest, LocalExecStartResponse, LocalExecStatusRequest, LocalExecStatusResponse,
    LocalExecStopRequest, LocalExecStopResponse, PortBindScope, PortLeaseId, PortLeaseRecord,
    PortLeaseRegistry, PortLeaseRequest, PortLeaseResponse, PortLeaseStatusKind, PortProtocol,
    ProxyProtocol, ProxyRouteId, ProxyRouteRecord, ProxyRouteRegisterRequest,
    ProxyRouteRegisterResponse, ProxyRouteRegistry, ProxyRouteStatusKind, ProxyRouteUpstream,
    ReadinessProbe, ReadinessProbeKind,
};
pub use self::network::{
    check_network_policy, NetworkPolicyDecision, OutboundExecuteCompletion, OutboundRequest,
    OutboundStreamCompletion, OutboundWebSocketCompletion,
};
pub use self::outbound::{
    is_secret_header_name, is_static_header_allowed, CancelSignal, DenyAllOutboundExecutor,
    ExecutorKind, FakeOutboundExecutor, KernelOutboundStreamResponse, LiveHttpOutboundExecutor,
    LiveHttpOutboundExecutorConfig, OutboundExecutePolicyConfig, OutboundExecutor,
    OutboundExecutorConfig, OutboundExecutorRequest, OutboundExecutorResponse, OutboundFrameKind,
    OutboundSecretHeaderSpec, OutboundStaticHeader, OutboundStreamFrame, OutboundStreamSummary,
    RedactedHeaderValue, ResolvedSecretHeader, SecretHeaderSpec, StaticHeader, StreamEmitter,
    StreamFormat, StreamStartStatus, STATIC_HEADER_ALLOWLIST,
};
pub use self::outbound_sse::{SseEvent, SseParser};
pub use self::outbound_websocket::{
    DenyAllWebSocketExecutor, FakeWebSocketExecutor, FrameDirection, FrameKind,
    LiveWebSocketExecutor, LiveWebSocketExecutorConfig, LiveWebSocketProfile,
    OutboundWebSocketFrame, OutboundWebSocketOpenRequest, OutboundWebSocketSession, SendStatus,
    WebSocketEvent, WebSocketExecutor, WebSocketFramePayload,
};
pub use self::permissions::PermissionGrantRecord;
pub use self::projections::ProjectionDefinition;
pub use self::proposals::{ProposalApproval, ProposalOperation, ProposalRecord, ProposalStatus};
pub use self::session::OpenSessionRequest;
pub use self::streaming::StreamRegistry;

tokio::task_local! {
    pub static ACTIVE_PROJECT_SCOPE: ProjectScopeContext;
}

// ---------------------------------------------------------------------------
// RuntimeConfig
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct RuntimeConfig {
    pub default_labels: Vec<String>,
    pub host_policy: HostPolicy,
    pub inproc_packages: InprocPackageCatalog,
    pub secret_resolver: SecretResolverConfig,
    /// In-memory project registry. Default: empty.
    pub project_registry: Arc<ProjectRegistry>,
    /// Outbound executor configuration. Defaults to `DenyAll` (fail-closed).
    pub outbound_executor: OutboundExecutorConfig,
    /// Outbound execute host-level policy. Defaults disabled (fail-closed). (Y1)
    pub outbound_execute_policy: OutboundExecutePolicyConfig,
    /// Outbound WebSocket executor. Defaults to DenyAll (fail-closed).
    pub outbound_websocket_executor: Arc<dyn WebSocketExecutor>,
    /// Local exec executor. Defaults to DenyAll (fail-closed).
    pub local_exec_executor: LocalExecExecutorConfig,
    /// In-memory local exec status registry for Phase 1 fake/deny dispatch.
    pub exec_registry: Arc<ExecRegistry>,
    /// In-memory execution target registry. Defaults with local/local-host.
    pub target_registry: Arc<ExecutionTargetRegistry>,
    /// In-memory loopback-only port lease registry.
    pub port_lease_registry: Arc<PortLeaseRegistry>,
    /// In-memory placeholder proxy route registry.
    pub proxy_route_registry: Arc<ProxyRouteRegistry>,
    /// Development-mode surface bundle path overrides. Maps a surface_id prefix
    /// to a filesystem directory containing built bundles.
    pub surface_dev_paths: BTreeMap<String, String>,
    /// Host-local package root hints keyed by package id. Used only when a host
    /// profile loads manifests from disk so relative subprocess commands run
    /// from the manifest's package directory without adding local paths to
    /// protocol manifest payloads.
    pub package_roots: BTreeMap<PackageId, PathBuf>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            default_labels: vec!["kernel".to_string()],
            host_policy: HostPolicy::default(),
            inproc_packages: InprocPackageCatalog::with_default_examples(),
            secret_resolver: SecretResolverConfig::default(),
            project_registry: Arc::new(ProjectRegistry::new()),
            outbound_executor: OutboundExecutorConfig::default(),
            outbound_execute_policy: OutboundExecutePolicyConfig::default(),
            outbound_websocket_executor: Arc::new(DenyAllWebSocketExecutor),
            local_exec_executor: LocalExecExecutorConfig::default(),
            exec_registry: Arc::new(ExecRegistry::default()),
            target_registry: Arc::new(ExecutionTargetRegistry::default()),
            port_lease_registry: Arc::new(PortLeaseRegistry::default()),
            proxy_route_registry: Arc::new(ProxyRouteRegistry::default()),
            surface_dev_paths: BTreeMap::new(),
            package_roots: BTreeMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// StoredAsset (crate-private)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) struct StoredAsset {
    pub record: AssetRecord,
    pub content: String,
}

// ---------------------------------------------------------------------------
// Runtime<S>
// ---------------------------------------------------------------------------

pub struct Runtime<S>
where
    S: EventStore,
{
    pub(crate) store: Arc<S>,
    pub(crate) packages: Arc<crate::PackageRegistry>,
    pub(crate) capabilities: Arc<crate::CapabilityFabric>,
    pub(crate) handles: Arc<HandleTable>,
    pub(crate) extensions: Arc<crate::ExtensionRegistry>,
    pub(crate) subprocesses: Arc<crate::SubprocessSupervisor>,
    pub(crate) sessions: Arc<RwLock<HashMap<SessionId, KernelSession>>>,
    pub(crate) assets: Arc<RwLock<HashMap<String, StoredAsset>>>,
    pub(crate) projections: Arc<RwLock<HashMap<String, ProjectionDefinition>>>,
    pub(crate) branches: Arc<RwLock<HashMap<String, BranchRecord>>>,
    pub(crate) grants: Arc<RwLock<HashMap<String, PermissionGrantRecord>>>,
    pub(crate) proposals: Arc<RwLock<HashMap<String, ProposalRecord>>>,
    pub(crate) streams: Arc<StreamRegistry>,
    pub(crate) config: RuntimeConfig,
}

impl<S> Clone for Runtime<S>
where
    S: EventStore,
{
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            packages: self.packages.clone(),
            capabilities: self.capabilities.clone(),
            handles: self.handles.clone(),
            extensions: self.extensions.clone(),
            subprocesses: self.subprocesses.clone(),
            sessions: self.sessions.clone(),
            assets: self.assets.clone(),
            projections: self.projections.clone(),
            branches: self.branches.clone(),
            grants: self.grants.clone(),
            proposals: self.proposals.clone(),
            streams: self.streams.clone(),
            config: self.config.clone(),
        }
    }
}

impl<S> Runtime<S>
where
    S: EventStore,
{
    pub fn new(store: Arc<S>, config: RuntimeConfig) -> Self {
        Self {
            store,
            packages: Arc::new(crate::PackageRegistry::default()),
            capabilities: Arc::new(crate::CapabilityFabric::default()),
            handles: Arc::new(HandleTable::default()),
            extensions: Arc::new(crate::ExtensionRegistry::default()),
            subprocesses: Arc::new(crate::SubprocessSupervisor::default()),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            assets: Arc::new(RwLock::new(HashMap::new())),
            projections: Arc::new(RwLock::new(HashMap::new())),
            branches: Arc::new(RwLock::new(HashMap::new())),
            grants: Arc::new(RwLock::new(HashMap::new())),
            proposals: Arc::new(RwLock::new(HashMap::new())),
            streams: Arc::new(StreamRegistry::default()),
            config,
        }
    }

    pub fn store(&self) -> Arc<S> {
        self.store.clone()
    }

    pub fn packages(&self) -> Arc<crate::PackageRegistry> {
        self.packages.clone()
    }

    pub fn outbound_websocket_executor(&self) -> Arc<dyn WebSocketExecutor> {
        self.config.outbound_websocket_executor.clone()
    }

    pub fn capabilities(&self) -> Arc<crate::CapabilityFabric> {
        self.capabilities.clone()
    }

    pub fn handles(&self) -> Arc<HandleTable> {
        self.handles.clone()
    }

    pub fn extensions(&self) -> Arc<crate::ExtensionRegistry> {
        self.extensions.clone()
    }

    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Resolve a secret reference using the configured host secret resolver.
    ///
    /// This is a host-internal method (not a protocol method) for use by
    /// the host during capability invocation. It delegates to
    /// `self.config.secret_resolver.resolver.resolve(ref_id)`.
    ///
    /// Returns the raw secret string on success, or an error if the
    /// reference cannot be resolved. Raw values must never be written
    /// to events, proposals, logs, or audit records.
    pub async fn resolve_secret_ref(&self, ref_id: &str) -> anyhow::Result<String> {
        self.resolve_secret_ref_with_session(ref_id, None).await
    }

    /// Resolve a secret reference with optional session context.
    ///
    /// For `secret_ref:project:NAME`, the `session_id` is used to look up the
    /// session's `metadata.project_id`, which scopes the resolution.
    pub async fn resolve_secret_ref_with_session(
        &self,
        ref_id: &str,
        session_id: Option<&str>,
    ) -> anyhow::Result<String> {
        if ygg_core::secret_ref::is_project_backed_ref(ref_id) {
            let project_id = self.lookup_project_id_from_session(session_id).await?;
            let scope = self.build_project_scope(&project_id)?;
            return self
                .with_active_project_scope(scope, async {
                    self.config.secret_resolver.resolver.resolve(ref_id).await
                })
                .await;
        }

        self.config.secret_resolver.resolver.resolve(ref_id).await
    }

    async fn lookup_project_id_from_session(
        &self,
        session_id: Option<&str>,
    ) -> anyhow::Result<ProjectId> {
        let sid = session_id.ok_or_else(|| {
            anyhow::anyhow!("project secret resolution requires session_id in context")
        })?;
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(sid)
            .ok_or_else(|| anyhow::anyhow!("session '{}' not found", sid))?;
        if session.status != SessionStatus::Open {
            anyhow::bail!("session '{}' is closed", sid);
        }
        let pid_str = session
            .metadata
            .get("project_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("session '{}' has no metadata.project_id", sid))?;
        ProjectId::new(pid_str)
    }

    fn build_project_scope(&self, project_id: &ProjectId) -> anyhow::Result<ProjectScopeContext> {
        let entry = self
            .config
            .project_registry
            .get(project_id)
            .ok_or_else(|| anyhow::anyhow!("project '{}' not registered", project_id))?;
        let store_path = ygg_core::paths::project_secret_store_path(project_id)?;
        Ok(ProjectScopeContext {
            project_id: project_id.clone(),
            project_store_path: store_path,
            fallback_to_platform: entry.descriptor.project.secret_policy.fallback_to_platform,
            require_per_project: entry
                .descriptor
                .project
                .secret_policy
                .require_per_project
                .clone(),
        })
    }

    async fn with_active_project_scope<F, T>(&self, scope: ProjectScopeContext, future: F) -> T
    where
        F: Future<Output = T>,
    {
        ACTIVE_PROJECT_SCOPE.scope(scope, future).await
    }

    pub(crate) async fn find_session_for_project(&self, project_id: &ProjectId) -> Option<String> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .find(|session| {
                session.status == SessionStatus::Open
                    && session.metadata.get("project_id").and_then(Value::as_str)
                        == Some(project_id.as_str())
            })
            .map(|session| session.id.clone())
    }

    pub async fn get_session(&self, session_id: &str) -> Option<KernelSession> {
        self.sessions.read().await.get(session_id).cloned()
    }

    pub async fn hydrate_substrate_from_events(&self) -> anyhow::Result<()> {
        let events = self.store.list_all().await?;
        let mut assets = HashMap::new();
        let mut branches = HashMap::new();
        let mut projections = HashMap::new();
        let mut grants = HashMap::new();
        for event in events {
            match event.kind.as_str() {
                EVENT_ASSET_PUT => {
                    let record: AssetRecord = serde_json::from_value(event.payload.clone())?;
                    let content = event
                        .metadata
                        .get("content")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    assets.insert(record.id.clone(), StoredAsset { record, content });
                }
                EVENT_SESSION_FORKED => {
                    let branch: BranchRecord = serde_json::from_value(event.payload.clone())?;
                    branches.insert(branch.id.clone(), branch);
                }
                EVENT_PROJECTION_UPDATED => {
                    let projection: ProjectionDefinition =
                        serde_json::from_value(event.payload.clone())?;
                    projections.insert(projection.id.clone(), projection);
                }
                EVENT_PERMISSION_GRANTED => {
                    let record: PermissionGrantRecord =
                        serde_json::from_value(event.payload.clone())?;
                    grants.insert(record.id.clone(), record);
                }
                EVENT_PERMISSION_REVOKED => {
                    let record: PermissionGrantRecord =
                        serde_json::from_value(event.payload.clone())?;
                    // Overwrite with revoked version (revoked_at is set)
                    grants.insert(record.id.clone(), record);
                }
                _ => {}
            }
        }
        *self.assets.write().await = assets;
        *self.branches.write().await = branches;
        *self.projections.write().await = projections;
        *self.grants.write().await = grants;
        Ok(())
    }

    pub async fn hydrate_deployment_from_events(&self) -> anyhow::Result<()> {
        let events = self.store.list_all().await?;
        let mut port_leases: HashMap<PortLeaseId, PortLeaseRecord> = HashMap::new();
        let mut proxy_routes: HashMap<ProxyRouteId, ProxyRouteRecord> = HashMap::new();
        let mut executions: HashMap<ExecId, ExecStatus> = HashMap::new();

        for event in events {
            let payload = &event.payload;
            match event.kind.as_str() {
                EVENT_PORT_LEASED => {
                    if let Some(record) = port_lease_record_from_payload(payload) {
                        port_leases.insert(record.id.clone(), record);
                    }
                }
                EVENT_PORT_RELEASED => {
                    if let Some(lease_id) = payload_str(payload, "lease_id") {
                        if let Some(lease) = port_leases.get_mut(&lease_id) {
                            lease.status = PortLeaseStatusKind::Released;
                        }
                    }
                }
                EVENT_PROXY_REGISTERED => {
                    if let Some(record) = proxy_route_record_from_payload(payload) {
                        proxy_routes.insert(record.id.clone(), record);
                    }
                }
                EVENT_PROXY_UNREGISTERED => {
                    if let Some(route_id) = payload_str(payload, "route_id") {
                        if let Some(route) = proxy_routes.get_mut(&route_id) {
                            route.status = ProxyRouteStatusKind::Removed;
                        }
                    }
                }
                EVENT_EXEC_STARTED | EVENT_EXEC_DENIED => {
                    if let Some(status) = exec_status_from_payload(payload) {
                        if let Some(exec_id) = status.exec_id.clone() {
                            executions.insert(exec_id, status);
                        }
                    }
                }
                EVENT_EXEC_STOPPED => {
                    if let Some(exec_id) = payload_str(payload, "exec_id") {
                        let message = payload_str(payload, "error");
                        let status =
                            executions
                                .entry(exec_id.clone())
                                .or_insert_with(|| ExecStatus {
                                    exec_id: Some(exec_id),
                                    target_id: None,
                                    kind: ExecStatusKind::Stopped,
                                    exit_code: None,
                                    message: None,
                                    ready: false,
                                });
                        status.kind = ExecStatusKind::Stopped;
                        status.ready = false;
                        if message.is_some() {
                            status.message = message;
                        }
                    }
                }
                _ => {}
            }
        }

        for mut lease in port_leases.into_values() {
            if lease.status == PortLeaseStatusKind::Active {
                lease.status = PortLeaseStatusKind::Reserved;
            }
            self.config.port_lease_registry.restore(lease).await;
        }

        for mut route in proxy_routes.into_values() {
            if route.status == ProxyRouteStatusKind::Active {
                route.status = ProxyRouteStatusKind::Stale;
            }
            self.config.proxy_route_registry.restore(route).await;
        }

        for mut status in executions.into_values() {
            if matches!(
                status.kind,
                ExecStatusKind::Running | ExecStatusKind::Pending
            ) || status.ready
            {
                status.kind = ExecStatusKind::Unknown;
                status.ready = false;
            }
            self.config.exec_registry.restore(status).await;
        }

        Ok(())
    }

    // Private helper used across submodules — event-appending via kernel identity.
    pub(crate) async fn append_kernel_event(
        &self,
        session_id: &SessionId,
        kind: &'static str,
        payload: Value,
    ) -> anyhow::Result<EventEnvelope> {
        self.append_kernel_event_with_metadata(session_id, kind, payload, json!({}))
            .await
    }

    pub(crate) async fn append_kernel_event_with_metadata(
        &self,
        session_id: &SessionId,
        kind: &'static str,
        payload: Value,
        metadata: Value,
    ) -> anyhow::Result<EventEnvelope> {
        self.append_event_unchecked(AppendEventRequest {
            session_id: session_id.clone(),
            writer_package_id: ygg_core::KERNEL_PACKAGE_ID.to_string(),
            kind: kind.to_string(),
            payload,
            metadata,
        })
        .await
    }
}

fn payload_str(payload: &Value, field: &str) -> Option<String> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn payload_u16(payload: &Value, field: &str) -> Option<u16> {
    payload
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
}

fn payload_bool(payload: &Value, field: &str) -> bool {
    payload.get(field).and_then(Value::as_bool).unwrap_or(false)
}

fn enum_from_payload<T>(payload: &Value, field: &str) -> Option<T>
where
    T: serde::de::DeserializeOwned,
{
    payload
        .get(field)
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
}

fn port_lease_record_from_payload(payload: &Value) -> Option<PortLeaseRecord> {
    Some(PortLeaseRecord {
        id: payload_str(payload, "lease_id")?,
        target_id: payload_str(payload, "target_id")?,
        port_name: payload_str(payload, "port_name")?,
        host: payload_str(payload, "host")?,
        port: payload_u16(payload, "port")?,
        protocol: enum_from_payload(payload, "protocol").unwrap_or(PortProtocol::Tcp),
        bind: enum_from_payload(payload, "bind").unwrap_or(PortBindScope::LoopbackOnly),
        status: enum_from_payload(payload, "status").unwrap_or(PortLeaseStatusKind::Active),
    })
}

fn proxy_route_record_from_payload(payload: &Value) -> Option<ProxyRouteRecord> {
    Some(ProxyRouteRecord {
        id: payload_str(payload, "route_id")?,
        upstream: ProxyRouteUpstream {
            port_lease_id: payload_str(payload, "port_lease_id")?,
            port_name: payload_str(payload, "port_name")?,
        },
        protocol: enum_from_payload(payload, "protocol").unwrap_or(ProxyProtocol::Http),
        public_url: payload_str(payload, "public_url")?,
        iframe_url: payload_str(payload, "iframe_url")?,
        status: enum_from_payload(payload, "status").unwrap_or(ProxyRouteStatusKind::Active),
    })
}

fn exec_status_from_payload(payload: &Value) -> Option<ExecStatus> {
    Some(ExecStatus {
        exec_id: Some(payload_str(payload, "exec_id")?),
        target_id: payload_str(payload, "target_id"),
        kind: enum_from_payload(payload, "status").unwrap_or(ExecStatusKind::Unknown),
        exit_code: None,
        message: payload_str(payload, "error"),
        ready: payload_bool(payload, "ready"),
    })
}
