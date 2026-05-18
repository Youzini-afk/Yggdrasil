use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::RwLock;
use ygg_core::{AssetRecord, EventEnvelope, KernelSession, SessionId, EVENT_ASSET_PUT, EVENT_PERMISSION_GRANTED, EVENT_PERMISSION_REVOKED, EVENT_PROJECTION_UPDATED, EVENT_SESSION_FORKED};

use crate::{EventStore, HostPolicy, InprocPackageCatalog, SecretResolverConfig};

mod session;
mod events;
mod packages;
mod capabilities;
mod hooks;
mod permissions;
mod assets;
mod branches;
mod projections;
mod proposals;
mod protocol_dispatch;
mod network;
mod streaming;
mod outbound;

// Re-export public types so old paths like ygg_runtime::runtime::AssetPutRequest keep working.
pub use self::assets::{AssetGetResponse, AssetPutRequest};
pub use self::branches::BranchRecord;
pub use self::events::{AppendEventRequest, EventListRequest};
pub use self::network::{NetworkPolicyDecision, OutboundRequest, check_network_policy};
pub use self::outbound::{
    DenyAllOutboundExecutor, ExecutorKind, FakeOutboundExecutor, OutboundExecutor,
    OutboundExecutorConfig, OutboundExecutorRequest, OutboundExecutorResponse,
};
pub use self::permissions::PermissionGrantRecord;
pub use self::projections::ProjectionDefinition;
pub use self::proposals::{ProposalApproval, ProposalOperation, ProposalRecord, ProposalStatus};
pub use self::session::OpenSessionRequest;
pub use self::streaming::StreamRegistry;

// ---------------------------------------------------------------------------
// RuntimeConfig
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct RuntimeConfig {
    pub default_labels: Vec<String>,
    pub host_policy: HostPolicy,
    pub inproc_packages: InprocPackageCatalog,
    pub secret_resolver: SecretResolverConfig,
    /// Outbound executor configuration. Defaults to `DenyAll` (fail-closed).
    pub outbound_executor: OutboundExecutorConfig,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            default_labels: vec!["kernel".to_string()],
            host_policy: HostPolicy::default(),
            inproc_packages: InprocPackageCatalog::with_default_examples(),
            secret_resolver: SecretResolverConfig::default(),
            outbound_executor: OutboundExecutorConfig::default(),
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

#[derive(Clone)]
pub struct Runtime<S>
where
    S: EventStore,
{
    pub(crate) store: Arc<S>,
    pub(crate) packages: Arc<crate::PackageRegistry>,
    pub(crate) capabilities: Arc<crate::CapabilityFabric>,
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

impl<S> Runtime<S>
where
    S: EventStore,
{
    pub fn new(store: Arc<S>, config: RuntimeConfig) -> Self {
        Self {
            store,
            packages: Arc::new(crate::PackageRegistry::default()),
            capabilities: Arc::new(crate::CapabilityFabric::default()),
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

    pub fn capabilities(&self) -> Arc<crate::CapabilityFabric> {
        self.capabilities.clone()
    }

    pub fn extensions(&self) -> Arc<crate::ExtensionRegistry> {
        self.extensions.clone()
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
        self.config.secret_resolver.resolver.resolve(ref_id).await
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
                    let content = event.metadata.get("content").and_then(Value::as_str).unwrap_or_default().to_string();
                    assets.insert(record.id.clone(), StoredAsset { record, content });
                }
                EVENT_SESSION_FORKED => {
                    let branch: BranchRecord = serde_json::from_value(event.payload.clone())?;
                    branches.insert(branch.id.clone(), branch);
                }
                EVENT_PROJECTION_UPDATED => {
                    let projection: ProjectionDefinition = serde_json::from_value(event.payload.clone())?;
                    projections.insert(projection.id.clone(), projection);
                }
                EVENT_PERMISSION_GRANTED => {
                    let record: PermissionGrantRecord = serde_json::from_value(event.payload.clone())?;
                    grants.insert(record.id.clone(), record);
                }
                EVENT_PERMISSION_REVOKED => {
                    let record: PermissionGrantRecord = serde_json::from_value(event.payload.clone())?;
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

    // Private helper used across submodules — event-appending via kernel identity.
    pub(crate) async fn append_kernel_event(
        &self,
        session_id: &SessionId,
        kind: &'static str,
        payload: Value,
    ) -> anyhow::Result<EventEnvelope> {
        self.append_kernel_event_with_metadata(session_id, kind, payload, json!({})).await
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
