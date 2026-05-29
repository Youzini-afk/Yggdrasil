//! Local exec / port / proxy target primitives.
//!
//! Phase 1 is intentionally fail-closed and in-memory only: no OS process
//! launch, no host proxying, and no raw secret material is stored here.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

pub type ExecutionTargetId = String;
pub type ExecId = String;
pub type PortLeaseId = String;
pub type ProxyRouteId = String;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExecCommand {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecStatusKind {
    Pending,
    Running,
    Stopped,
    Exited,
    Failed,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecLifecyclePolicy {
    StopOnSessionClose,
    KeepAlive,
    StopOnIdle,
}

impl Default for ExecLifecyclePolicy {
    fn default() -> Self {
        Self::StopOnSessionClose
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExecResourceLimits {
    pub max_memory_mb: Option<u64>,
    pub max_cpu_millis: Option<u64>,
    pub max_duration_ms: Option<u64>,
    pub max_log_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessProbeKind {
    None,
    TcpPort,
    HttpGet,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReadinessProbe {
    pub kind: ReadinessProbeKind,
    pub port_name: Option<String>,
    pub path: Option<String>,
    pub initial_delay_ms: Option<u64>,
    pub timeout_ms: Option<u64>,
}

impl Default for ReadinessProbe {
    fn default() -> Self {
        Self {
            kind: ReadinessProbeKind::None,
            port_name: None,
            path: None,
            initial_delay_ms: None,
            timeout_ms: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExecStatus {
    pub exec_id: Option<ExecId>,
    pub target_id: Option<ExecutionTargetId>,
    pub kind: ExecStatusKind,
    pub exit_code: Option<i32>,
    pub message: Option<String>,
    pub ready: bool,
}

impl ExecStatus {
    fn denied(message: impl Into<String>) -> Self {
        Self {
            exec_id: None,
            target_id: None,
            kind: ExecStatusKind::Denied,
            exit_code: None,
            message: Some(message.into()),
            ready: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LocalExecStartRequest {
    pub target_id: ExecutionTargetId,
    pub command: ExecCommand,
    #[serde(default)]
    pub lifecycle: ExecLifecyclePolicy,
    #[serde(default)]
    pub resource_limits: ExecResourceLimits,
    #[serde(default)]
    pub readiness_probe: ReadinessProbe,
    #[serde(default)]
    pub port_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LocalExecStartResponse {
    pub exec_id: Option<ExecId>,
    pub status: ExecStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LocalExecStopRequest {
    pub exec_id: ExecId,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LocalExecStopResponse {
    pub exec_id: ExecId,
    pub status: ExecStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LocalExecStatusRequest {
    pub exec_id: ExecId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LocalExecStatusResponse {
    pub status: ExecStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LocalExecLogsRequest {
    pub exec_id: ExecId,
    pub since_seq: Option<u64>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalExecLogStream {
    Stdout,
    Stderr,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LocalExecLogLine {
    pub seq: u64,
    pub stream: LocalExecLogStream,
    pub message_redacted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LocalExecLogsResponse {
    pub exec_id: ExecId,
    pub lines: Vec<LocalExecLogLine>,
    pub next_seq: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LocalExecListResponse {
    pub executions: Vec<ExecStatus>,
}

#[derive(Default)]
pub struct ExecRegistry {
    executions: RwLock<HashMap<ExecId, ExecStatus>>,
}

impl ExecRegistry {
    pub fn new() -> Self {
        Self {
            executions: RwLock::new(HashMap::new()),
        }
    }

    pub async fn upsert_from_status(&self, status: &ExecStatus) {
        if let Some(exec_id) = &status.exec_id {
            self.executions
                .write()
                .await
                .insert(exec_id.clone(), status.clone());
        }
    }

    pub async fn status(&self, exec_id: &str) -> Option<ExecStatus> {
        self.executions.read().await.get(exec_id).cloned()
    }

    pub async fn list(&self) -> Vec<ExecStatus> {
        self.executions.read().await.values().cloned().collect()
    }
}

#[async_trait]
pub trait LocalExecExecutor: Send + Sync + 'static {
    async fn start(&self, request: LocalExecStartRequest) -> anyhow::Result<LocalExecStartResponse>;
    async fn stop(&self, request: LocalExecStopRequest) -> anyhow::Result<LocalExecStopResponse>;
    async fn status(
        &self,
        request: LocalExecStatusRequest,
    ) -> anyhow::Result<LocalExecStatusResponse>;
    async fn logs(&self, request: LocalExecLogsRequest) -> anyhow::Result<LocalExecLogsResponse>;
}

#[derive(Clone)]
pub enum LocalExecExecutorConfig {
    DenyAll,
    Custom(Arc<dyn LocalExecExecutor>),
    Fake,
}

impl Default for LocalExecExecutorConfig {
    fn default() -> Self {
        Self::DenyAll
    }
}

impl LocalExecExecutorConfig {
    pub fn executor(&self) -> Arc<dyn LocalExecExecutor> {
        match self {
            Self::DenyAll => Arc::new(DenyAllLocalExecExecutor),
            Self::Custom(executor) => executor.clone(),
            Self::Fake => Arc::new(FakeLocalExecExecutor),
        }
    }
}

pub struct DenyAllLocalExecExecutor;

#[async_trait]
impl LocalExecExecutor for DenyAllLocalExecExecutor {
    async fn start(
        &self,
        _request: LocalExecStartRequest,
    ) -> anyhow::Result<LocalExecStartResponse> {
        let error = "local exec denied by host policy".to_string();
        Ok(LocalExecStartResponse {
            exec_id: None,
            status: ExecStatus::denied(error.clone()),
            error: Some(error),
        })
    }

    async fn stop(&self, request: LocalExecStopRequest) -> anyhow::Result<LocalExecStopResponse> {
        let error = "local exec stop denied by host policy".to_string();
        let mut status = ExecStatus::denied(error.clone());
        status.exec_id = Some(request.exec_id.clone());
        Ok(LocalExecStopResponse {
            exec_id: request.exec_id,
            status,
            error: Some(error),
        })
    }

    async fn status(
        &self,
        request: LocalExecStatusRequest,
    ) -> anyhow::Result<LocalExecStatusResponse> {
        let error = "local exec status denied by host policy".to_string();
        let mut status = ExecStatus::denied(error.clone());
        status.exec_id = Some(request.exec_id);
        Ok(LocalExecStatusResponse {
            status,
            error: Some(error),
        })
    }

    async fn logs(&self, request: LocalExecLogsRequest) -> anyhow::Result<LocalExecLogsResponse> {
        Ok(LocalExecLogsResponse {
            exec_id: request.exec_id,
            lines: Vec::new(),
            next_seq: None,
            error: Some("local exec logs denied by host policy".to_string()),
        })
    }
}

pub struct FakeLocalExecExecutor;

#[async_trait]
impl LocalExecExecutor for FakeLocalExecExecutor {
    async fn start(&self, request: LocalExecStartRequest) -> anyhow::Result<LocalExecStartResponse> {
        let exec_id = fake_exec_id(&request.target_id, &request.command.program);
        let status = ExecStatus {
            exec_id: Some(exec_id.clone()),
            target_id: Some(request.target_id),
            kind: ExecStatusKind::Running,
            exit_code: None,
            message: Some("fake local exec running".to_string()),
            ready: true,
        };
        Ok(LocalExecStartResponse {
            exec_id: Some(exec_id),
            status,
            error: None,
        })
    }

    async fn stop(&self, request: LocalExecStopRequest) -> anyhow::Result<LocalExecStopResponse> {
        Ok(LocalExecStopResponse {
            exec_id: request.exec_id.clone(),
            status: ExecStatus {
                exec_id: Some(request.exec_id.clone()),
                target_id: None,
                kind: ExecStatusKind::Stopped,
                exit_code: Some(0),
                message: Some("fake local exec stopped".to_string()),
                ready: false,
            },
            error: None,
        })
    }

    async fn status(
        &self,
        request: LocalExecStatusRequest,
    ) -> anyhow::Result<LocalExecStatusResponse> {
        Ok(LocalExecStatusResponse {
            status: ExecStatus {
                exec_id: Some(request.exec_id),
                target_id: None,
                kind: ExecStatusKind::Running,
                exit_code: None,
                message: Some("fake local exec running".to_string()),
                ready: true,
            },
            error: None,
        })
    }

    async fn logs(&self, request: LocalExecLogsRequest) -> anyhow::Result<LocalExecLogsResponse> {
        let since = request.since_seq.unwrap_or(0);
        let limit = request.limit.unwrap_or(100) as usize;
        let lines: Vec<_> = [
            LocalExecLogLine {
                seq: 1,
                stream: LocalExecLogStream::System,
                message_redacted: "fake local exec started".to_string(),
            },
            LocalExecLogLine {
                seq: 2,
                stream: LocalExecLogStream::Stdout,
                message_redacted: "fake local exec output".to_string(),
            },
        ]
        .into_iter()
        .filter(|line| line.seq > since)
        .take(limit)
        .collect();
        let next_seq = lines.last().map(|line| line.seq + 1);
        Ok(LocalExecLogsResponse {
            exec_id: request.exec_id,
            lines,
            next_seq,
            error: None,
        })
    }
}

fn fake_exec_id(target_id: &str, program: &str) -> ExecId {
    format!("fake-exec-{}-{}", sanitize_id(target_id), sanitize_id(program))
}

fn sanitize_id(input: &str) -> String {
    let sanitized: String = input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = sanitized.trim_matches('-');
    if trimmed.is_empty() {
        "id".to_string()
    } else {
        trimmed.to_string()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionTargetReachability {
    LocalHost,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionTargetCapability {
    LocalExec,
    PortLease,
    HttpProxyUpstream,
    WebsocketProxyUpstream,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionTargetStatusKind {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExecutionTarget {
    pub id: ExecutionTargetId,
    pub name: String,
    pub reachability: ExecutionTargetReachability,
    #[serde(default)]
    pub capabilities: Vec<ExecutionTargetCapability>,
    pub status: ExecutionTargetStatusKind,
}

pub struct ExecutionTargetRegistry {
    targets: RwLock<HashMap<ExecutionTargetId, ExecutionTarget>>,
}

impl ExecutionTargetRegistry {
    pub fn new() -> Self {
        let target = ExecutionTarget {
            id: "local".to_string(),
            name: "local-host".to_string(),
            reachability: ExecutionTargetReachability::LocalHost,
            capabilities: vec![
                ExecutionTargetCapability::LocalExec,
                ExecutionTargetCapability::PortLease,
                ExecutionTargetCapability::HttpProxyUpstream,
                ExecutionTargetCapability::WebsocketProxyUpstream,
            ],
            status: ExecutionTargetStatusKind::Available,
        };
        let mut targets = HashMap::new();
        targets.insert(target.id.clone(), target);
        Self {
            targets: RwLock::new(targets),
        }
    }

    pub async fn list(&self) -> Vec<ExecutionTarget> {
        self.targets.read().await.values().cloned().collect()
    }

    pub async fn status(&self, target_id: &str) -> Option<ExecutionTarget> {
        self.targets.read().await.get(target_id).cloned()
    }

    pub async fn register(&self, target: ExecutionTarget) -> Option<ExecutionTarget> {
        self.targets
            .write()
            .await
            .insert(target.id.clone(), target)
    }

    pub async fn unregister(&self, target_id: &str) -> Option<ExecutionTarget> {
        self.targets.write().await.remove(target_id)
    }
}

impl Default for ExecutionTargetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortProtocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortBindScope {
    LoopbackOnly,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortLeaseStatusKind {
    Active,
    Released,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PortLeaseRequest {
    pub target_id: ExecutionTargetId,
    pub port_name: String,
    #[serde(default = "default_port_protocol")]
    pub protocol: PortProtocol,
    pub requested_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PortLeaseRecord {
    pub id: PortLeaseId,
    pub target_id: ExecutionTargetId,
    pub port_name: String,
    pub host: String,
    pub port: u16,
    pub protocol: PortProtocol,
    pub bind: PortBindScope,
    pub status: PortLeaseStatusKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PortLeaseResponse {
    pub lease: PortLeaseRecord,
}

pub struct PortLeaseRegistry {
    leases: RwLock<HashMap<PortLeaseId, PortLeaseRecord>>,
    next_id: AtomicU64,
}

impl PortLeaseRegistry {
    pub fn new() -> Self {
        Self {
            leases: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(0),
        }
    }

    pub async fn lease(&self, request: PortLeaseRequest) -> PortLeaseResponse {
        let sequence = self.next_id.fetch_add(1, Ordering::SeqCst);
        let record = PortLeaseRecord {
            id: format!("port-lease-{sequence:06}"),
            target_id: request.target_id,
            port_name: request.port_name,
            host: "127.0.0.1".to_string(),
            port: request.requested_port.unwrap_or(39_000 + sequence as u16),
            protocol: request.protocol,
            bind: PortBindScope::LoopbackOnly,
            status: PortLeaseStatusKind::Active,
        };
        self.leases
            .write()
            .await
            .insert(record.id.clone(), record.clone());
        PortLeaseResponse { lease: record }
    }

    pub async fn release(&self, lease_id: &str) -> Option<PortLeaseRecord> {
        let mut leases = self.leases.write().await;
        let lease = leases.get_mut(lease_id)?;
        lease.status = PortLeaseStatusKind::Released;
        Some(lease.clone())
    }

    pub async fn status(&self, lease_id: &str) -> Option<PortLeaseRecord> {
        self.leases.read().await.get(lease_id).cloned()
    }

    pub async fn list(&self) -> Vec<PortLeaseRecord> {
        self.leases.read().await.values().cloned().collect()
    }
}

impl Default for PortLeaseRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn default_port_protocol() -> PortProtocol {
    PortProtocol::Tcp
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProxyProtocol {
    Http,
    Websocket,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProxyRouteStatusKind {
    Active,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProxyRouteUpstream {
    pub port_lease_id: PortLeaseId,
    pub port_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProxyRouteRegisterRequest {
    pub route_id: Option<ProxyRouteId>,
    pub upstream: ProxyRouteUpstream,
    #[serde(default = "default_proxy_protocol")]
    pub protocol: ProxyProtocol,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProxyRouteRecord {
    pub id: ProxyRouteId,
    pub upstream: ProxyRouteUpstream,
    pub protocol: ProxyProtocol,
    pub public_url: String,
    pub iframe_url: String,
    pub status: ProxyRouteStatusKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProxyRouteRegisterResponse {
    pub route: ProxyRouteRecord,
}

pub struct ProxyRouteRegistry {
    routes: RwLock<HashMap<ProxyRouteId, ProxyRouteRecord>>,
    next_id: AtomicU64,
}

impl ProxyRouteRegistry {
    pub fn new() -> Self {
        Self {
            routes: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(0),
        }
    }

    pub async fn register(&self, request: ProxyRouteRegisterRequest) -> ProxyRouteRegisterResponse {
        let sequence = self.next_id.fetch_add(1, Ordering::SeqCst);
        let id = request
            .route_id
            .unwrap_or_else(|| format!("proxy-route-{sequence:06}"));
        let route = ProxyRouteRecord {
            public_url: format!("/_ygg/app/{id}/"),
            iframe_url: format!("/_ygg/app/{id}/"),
            id: id.clone(),
            upstream: request.upstream,
            protocol: request.protocol,
            status: ProxyRouteStatusKind::Active,
        };
        self.routes.write().await.insert(id, route.clone());
        ProxyRouteRegisterResponse { route }
    }

    pub async fn unregister(&self, route_id: &str) -> Option<ProxyRouteRecord> {
        let mut routes = self.routes.write().await;
        let route = routes.get_mut(route_id)?;
        route.status = ProxyRouteStatusKind::Removed;
        Some(route.clone())
    }

    pub async fn status(&self, route_id: &str) -> Option<ProxyRouteRecord> {
        self.routes.read().await.get(route_id).cloned()
    }

    pub async fn list(&self) -> Vec<ProxyRouteRecord> {
        self.routes.read().await.values().cloned().collect()
    }
}

impl Default for ProxyRouteRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn default_proxy_protocol() -> ProxyProtocol {
    ProxyProtocol::Http
}
