use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use ygg_core::{
    new_id, AssetRecord, EventEnvelope, EventKind, EventSequence, KernelSession, PackageEntry, PackageId, PackageManifest, SessionId,
    SessionStatus, EVENT_PACKAGE_DEGRADED, EVENT_PACKAGE_LOADED, EVENT_PACKAGE_LOADING, EVENT_PACKAGE_LOG,
    EVENT_PACKAGE_READY, EVENT_PACKAGE_STARTING, EVENT_PACKAGE_STOPPED, EVENT_PACKAGE_STOPPING,
    EVENT_PACKAGE_UNLOADED, EVENT_PERMISSION_DENIED, EVENT_SESSION_CLOSED, EVENT_SESSION_FORKED,
    EVENT_SESSION_OPENED, KERNEL_PACKAGE_ID, EVENT_ASSET_PUT, EVENT_PROJECTION_UPDATED,
};

use crate::{
    CapabilityFabric, CapabilityInvocationRequest, CapabilityInvocationResult, EventStore,
    ExtensionDispatchResult, ExtensionRegistry, HostPolicy, InprocInvocation, InprocPackageCatalog,
    RegisteredCapability,
    PackageRecord, PackageRegistry, PackageState, ProtocolContext, ProtocolPrincipal, SubprocessSupervisor,
    validate_json_schema_subset,
};

#[derive(Clone)]
pub struct RuntimeConfig {
    pub default_labels: Vec<String>,
    pub host_policy: HostPolicy,
    pub inproc_packages: InprocPackageCatalog,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            default_labels: vec!["kernel".to_string()],
            host_policy: HostPolicy::default(),
            inproc_packages: InprocPackageCatalog::with_default_examples(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenSessionRequest {
    pub labels: Vec<String>,
    pub active_package_set: Vec<PackageId>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEventRequest {
    pub session_id: SessionId,
    pub writer_package_id: PackageId,
    pub kind: EventKind,
    pub payload: Value,
    pub metadata: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventListRequest {
    pub session_id: SessionId,
    #[serde(default)]
    pub after_sequence: Option<EventSequence>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub kind_prefix: Option<String>,
    #[serde(default)]
    pub writer_package_id: Option<PackageId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPutRequest {
    #[serde(default)]
    pub origin_package_id: Option<PackageId>,
    pub mime: String,
    pub content: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetGetResponse {
    pub record: AssetRecord,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionDefinition {
    pub id: String,
    pub session_id: SessionId,
    #[serde(default)]
    pub source_kind_prefix: Option<String>,
    #[serde(default)]
    pub state: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchRecord {
    pub id: String,
    pub parent_session_id: SessionId,
    pub child_session_id: SessionId,
    pub forked_from_sequence: EventSequence,
    pub created_at: chrono::DateTime<Utc>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone)]
struct StoredAsset {
    record: AssetRecord,
    content: String,
}

#[derive(Clone)]
pub struct Runtime<S>
where
    S: EventStore,
{
    store: Arc<S>,
    packages: Arc<PackageRegistry>,
    capabilities: Arc<CapabilityFabric>,
    extensions: Arc<ExtensionRegistry>,
    subprocesses: Arc<SubprocessSupervisor>,
    sessions: Arc<RwLock<HashMap<SessionId, KernelSession>>>,
    assets: Arc<RwLock<HashMap<String, StoredAsset>>>,
    projections: Arc<RwLock<HashMap<String, ProjectionDefinition>>>,
    branches: Arc<RwLock<HashMap<String, BranchRecord>>>,
    config: RuntimeConfig,
}

impl<S> Runtime<S>
where
    S: EventStore,
{
    pub fn new(store: Arc<S>, config: RuntimeConfig) -> Self {
        Self {
            store,
            packages: Arc::new(PackageRegistry::default()),
            capabilities: Arc::new(CapabilityFabric::default()),
            extensions: Arc::new(ExtensionRegistry::default()),
            subprocesses: Arc::new(SubprocessSupervisor::default()),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            assets: Arc::new(RwLock::new(HashMap::new())),
            projections: Arc::new(RwLock::new(HashMap::new())),
            branches: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub fn store(&self) -> Arc<S> {
        self.store.clone()
    }

    pub fn packages(&self) -> Arc<PackageRegistry> {
        self.packages.clone()
    }

    pub fn capabilities(&self) -> Arc<CapabilityFabric> {
        self.capabilities.clone()
    }

    pub fn extensions(&self) -> Arc<ExtensionRegistry> {
        self.extensions.clone()
    }

    pub async fn open_session(&self, mut request: OpenSessionRequest) -> anyhow::Result<KernelSession> {
        if request.labels.is_empty() {
            request.labels = self.config.default_labels.clone();
        }

        let now = Utc::now();
        let session = KernelSession {
            id: new_id("ses"),
            labels: request.labels,
            active_package_set: request.active_package_set,
            principal_scope: None,
            status: SessionStatus::Open,
            created_at: now,
            updated_at: now,
            metadata: request.metadata,
        };

        self.sessions.write().await.insert(session.id.clone(), session.clone());

        self.append_kernel_event(
            &session.id,
            EVENT_SESSION_OPENED,
            json!({
                "labels": session.labels,
                "active_package_set": session.active_package_set,
                "principal_scope": session.principal_scope,
            }),
        )
        .await?;

        Ok(session)
    }

    pub async fn close_session(&self, session_id: SessionId) -> anyhow::Result<EventEnvelope> {
        let mut sessions = self.sessions.write().await;
        match sessions.get_mut(&session_id) {
            Some(session) if session.status == SessionStatus::Open => {
                session.status = SessionStatus::Closed;
                session.updated_at = Utc::now();
            }
            Some(_) => anyhow::bail!("session '{session_id}' is already closed"),
            None => anyhow::bail!("session '{session_id}' is not open"),
        }
        drop(sessions);
        self.append_kernel_event(&session_id, EVENT_SESSION_CLOSED, json!({})).await
    }

    pub async fn fork_session(
        &self,
        parent_session_id: SessionId,
        forked_from_sequence: EventSequence,
        metadata: Value,
    ) -> anyhow::Result<BranchRecord> {
        let parent = self
            .sessions
            .read()
            .await
            .get(&parent_session_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("parent session '{parent_session_id}' is not open"))?;
        let child = self
            .open_session(OpenSessionRequest {
                labels: parent.labels.clone(),
                active_package_set: parent.active_package_set.clone(),
                metadata: json!({"forked_from": parent_session_id, "forked_from_sequence": forked_from_sequence}),
            })
            .await?;
        let branch = BranchRecord {
            id: new_id("br"),
            parent_session_id: parent_session_id.clone(),
            child_session_id: child.id.clone(),
            forked_from_sequence,
            created_at: Utc::now(),
            metadata,
        };
        self.branches.write().await.insert(branch.id.clone(), branch.clone());
        self.append_kernel_event(&parent_session_id, EVENT_SESSION_FORKED, serde_json::to_value(&branch)?).await?;
        Ok(branch)
    }

    pub async fn list_branches(&self, session_id: &SessionId) -> Vec<BranchRecord> {
        let mut branches: Vec<_> = self
            .branches
            .read()
            .await
            .values()
            .filter(|branch| &branch.parent_session_id == session_id || &branch.child_session_id == session_id)
            .cloned()
            .collect();
        branches.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        branches
    }

    pub async fn append_event(&self, request: AppendEventRequest) -> anyhow::Result<EventEnvelope> {
        match self.sessions.read().await.get(&request.session_id) {
            Some(session) if session.status == SessionStatus::Open => {}
            Some(_) => anyhow::bail!("session '{}' is closed", request.session_id),
            None => anyhow::bail!("session '{}' is not open", request.session_id),
        }

        if request.writer_package_id != KERNEL_PACKAGE_ID {
            match self.packages.permissions(&request.writer_package_id).await {
                Some(permissions) if permissions.events.append => {}
                _ => {
                    self.audit_permission_denied(
                        &request.session_id,
                        &request.writer_package_id,
                        "events.append",
                    )
                    .await?;
                    anyhow::bail!("package '{}' is not allowed to append events", request.writer_package_id);
                }
            }
        }

        let mut request = request;
        let before = self
            .dispatch_extension_handlers(
                "kernel/event.before_append",
                json!({
                    "session_id": request.session_id,
                    "writer_package_id": request.writer_package_id,
                    "kind": request.kind,
                    "payload": request.payload,
                    "metadata": request.metadata,
                }),
            )
            .await;
        if let Some(vetoed_by) = before.vetoed_by {
            anyhow::bail!("event append vetoed by hook package '{vetoed_by}'");
        }
        request.metadata = before.payload.get("metadata").cloned().unwrap_or(request.metadata);
        let event = self.append_event_unchecked(request).await?;
        let _ = self
            .dispatch_extension_handlers("kernel/event.after_append", serde_json::to_value(&event).unwrap_or_else(|_| json!({})))
            .await;
        Ok(event)
    }

    pub async fn append_event_with_context(
        &self,
        context: &ProtocolContext,
        mut request: AppendEventRequest,
    ) -> anyhow::Result<EventEnvelope> {
        match &context.principal {
            ProtocolPrincipal::HostAdmin | ProtocolPrincipal::HostDev => self.append_event(request).await,
            ProtocolPrincipal::Package { package_id } => {
                request.writer_package_id = package_id.clone();
                self.append_event(request).await
            }
            ProtocolPrincipal::Anonymous => anyhow::bail!("anonymous principal is not allowed to append events"),
        }
    }

    async fn append_event_unchecked(&self, request: AppendEventRequest) -> anyhow::Result<EventEnvelope> {
        let sequence = self.store.next_sequence(&request.session_id).await?;
        let event = EventEnvelope::new(
            new_id("evt"),
            request.session_id,
            sequence,
            request.writer_package_id,
            request.kind,
            request.payload,
        );

        if !event.writer_owns_kind() {
            anyhow::bail!(
                "package '{}' cannot write event kind '{}'",
                event.writer_package_id,
                event.kind
            );
        }

        let mut event = event;
        event.metadata = request.metadata;
        if event.writer_package_id != KERNEL_PACKAGE_ID {
            if let Some(manifest) = self.packages.manifest(&event.writer_package_id).await {
                if let Some(schema) = manifest
                    .contributes
                    .schemas
                    .iter()
                    .find(|schema| schema.id == event.kind)
                    .map(|schema| &schema.schema)
                {
                    validate_json_schema_subset(schema, &event.payload)?;
                }
            }
        }
        self.store.append(event.clone()).await?;
        Ok(event)
    }

    pub async fn list_events(&self, session_id: &SessionId) -> anyhow::Result<Vec<EventEnvelope>> {
        self.store.list_session(session_id).await
    }

    pub async fn list_events_range(&self, request: &EventListRequest) -> anyhow::Result<Vec<EventEnvelope>> {
        let mut events = self
            .store
            .list_session_range(&request.session_id, request.after_sequence, request.limit)
            .await?;
        if let Some(kind_prefix) = &request.kind_prefix {
            events.retain(|event| event.kind.starts_with(kind_prefix));
        }
        if let Some(writer_package_id) = &request.writer_package_id {
            events.retain(|event| &event.writer_package_id == writer_package_id);
        }
        Ok(events)
    }

    pub async fn list_events_for(
        &self,
        session_id: &SessionId,
        caller_package_id: Option<&PackageId>,
    ) -> anyhow::Result<Vec<EventEnvelope>> {
        if let Some(caller) = caller_package_id {
            match self.packages.permissions(caller).await {
                Some(permissions) if permissions.events.read => {}
                _ => {
                    self.audit_permission_denied(session_id, caller, "events.read").await?;
                    anyhow::bail!("package '{caller}' is not allowed to read events");
                }
            }
        }
        self.list_events(session_id).await
    }

    pub async fn list_events_range_for(
        &self,
        request: &EventListRequest,
        caller_package_id: Option<&PackageId>,
    ) -> anyhow::Result<Vec<EventEnvelope>> {
        if let Some(caller) = caller_package_id {
            match self.packages.permissions(caller).await {
                Some(permissions) if permissions.events.read => {}
                _ => {
                    self.audit_permission_denied(&request.session_id, caller, "events.read").await?;
                    anyhow::bail!("package '{caller}' is not allowed to read events");
                }
            }
        }
        self.list_events_range(request).await
    }

    pub async fn list_events_with_context(
        &self,
        context: &ProtocolContext,
        session_id: &SessionId,
    ) -> anyhow::Result<Vec<EventEnvelope>> {
        match &context.principal {
            ProtocolPrincipal::HostAdmin | ProtocolPrincipal::HostDev => self.list_events(session_id).await,
            ProtocolPrincipal::Package { package_id } => self.list_events_for(session_id, Some(package_id)).await,
            ProtocolPrincipal::Anonymous => anyhow::bail!("anonymous principal is not allowed to read events"),
        }
    }

    pub async fn list_events_range_with_context(
        &self,
        context: &ProtocolContext,
        request: &EventListRequest,
    ) -> anyhow::Result<Vec<EventEnvelope>> {
        match &context.principal {
            ProtocolPrincipal::HostAdmin | ProtocolPrincipal::HostDev => self.list_events_range(request).await,
            ProtocolPrincipal::Package { package_id } => self.list_events_range_for(request, Some(package_id)).await,
            ProtocolPrincipal::Anonymous => anyhow::bail!("anonymous principal is not allowed to read events"),
        }
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<EventEnvelope> {
        self.store.subscribe()
    }

    pub async fn put_asset(&self, mut request: AssetPutRequest) -> anyhow::Result<AssetRecord> {
        let origin_package_id = request.origin_package_id.take().unwrap_or_else(|| KERNEL_PACKAGE_ID.to_string());
        let mut hasher = DefaultHasher::new();
        request.content.hash(&mut hasher);
        let record = AssetRecord {
            id: new_id("ast"),
            origin_package_id,
            mime: request.mime,
            hash: format!("{:016x}", hasher.finish()),
            size_bytes: request.content.len() as u64,
            created_at: Utc::now(),
            metadata: request.metadata,
        };
        self.assets.write().await.insert(record.id.clone(), StoredAsset { record: record.clone(), content: request.content });
        self.append_kernel_event(&format!("kernel_asset_{}", record.id), EVENT_ASSET_PUT, serde_json::to_value(&record)?).await?;
        Ok(record)
    }

    pub async fn get_asset(&self, asset_id: &str) -> anyhow::Result<AssetGetResponse> {
        self.assets
            .read()
            .await
            .get(asset_id)
            .cloned()
            .map(|stored| AssetGetResponse { record: stored.record, content: stored.content })
            .ok_or_else(|| anyhow::anyhow!("asset '{asset_id}' not found"))
    }

    pub async fn list_assets(&self) -> Vec<AssetRecord> {
        let mut assets: Vec<_> = self.assets.read().await.values().map(|stored| stored.record.clone()).collect();
        assets.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        assets
    }

    pub async fn projection_register(&self, definition: ProjectionDefinition) -> anyhow::Result<ProjectionDefinition> {
        self.projections.write().await.insert(definition.id.clone(), definition.clone());
        Ok(definition)
    }

    pub async fn projection_rebuild(&self, projection_id: &str) -> anyhow::Result<ProjectionDefinition> {
        let mut projections = self.projections.write().await;
        let projection = projections
            .get_mut(projection_id)
            .ok_or_else(|| anyhow::anyhow!("projection '{projection_id}' not found"))?;
        let events = self
            .list_events_range(&EventListRequest {
                session_id: projection.session_id.clone(),
                after_sequence: None,
                limit: None,
                kind_prefix: projection.source_kind_prefix.clone(),
                writer_package_id: None,
            })
            .await?;
        projection.state = json!({"event_count": events.len(), "last_sequence": events.last().map(|event| event.sequence)});
        let projection = projection.clone();
        drop(projections);
        self.append_kernel_event(
            &format!("kernel_projection_{}", projection.id.replace('/', "_")),
            EVENT_PROJECTION_UPDATED,
            serde_json::to_value(&projection)?,
        )
        .await?;
        Ok(projection)
    }

    pub async fn projection_get(&self, projection_id: &str) -> anyhow::Result<ProjectionDefinition> {
        self.projections
            .read()
            .await
            .get(projection_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("projection '{projection_id}' not found"))
    }

    pub async fn load_package(&self, manifest: PackageManifest) -> anyhow::Result<PackageRecord> {
        if let PackageEntry::RustInproc { crate_ref, symbol, .. } = &manifest.entry {
            if !manifest.provides.is_empty() && self.config.inproc_packages.lookup(crate_ref, symbol).is_none() {
                anyhow::bail!(
                    "rust_inproc entry '{}::{}' is not available in this host",
                    crate_ref,
                    symbol
                );
            }
        }
        let mut record = self.packages.load(manifest, &self.config.host_policy).await?;
        record = self.packages.set_state(&record.id, PackageState::Loading).await.unwrap_or(record);
        self.append_package_lifecycle_event(&record, EVENT_PACKAGE_LOADING, None).await?;
        if matches!(record.manifest.entry, PackageEntry::Subprocess { .. }) {
            record = self
                .packages
                .set_state(&record.id, PackageState::Starting)
                .await
                .unwrap_or(record);
            self.append_package_lifecycle_event(&record, EVENT_PACKAGE_STARTING, None).await?;
            if let Err(error) = self.subprocesses.start(&record.manifest).await {
                let degraded = self
                    .packages
                    .set_state(&record.id, PackageState::Degraded)
                    .await
                    .unwrap_or_else(|| record.clone());
                self.append_package_degraded_event(&degraded, &error.to_string()).await?;
                return Err(error);
            }
            record = self
                .packages
                .set_state(&record.id, PackageState::Ready)
                .await
                .unwrap_or(record);
        }
        if !matches!(record.state, PackageState::Ready) {
            record = self.packages.set_state(&record.id, PackageState::Ready).await.unwrap_or(record);
        }
        self.capabilities.register_package(&record.id, &record.manifest.provides).await;
        self.extensions.register_package(&record.id, &record.manifest.contributes.hooks).await;
        self.append_package_lifecycle_event(&record, EVENT_PACKAGE_READY, None).await?;
        self.append_package_lifecycle_event(&record, EVENT_PACKAGE_LOADED, None).await?;
        Ok(record)
    }

    pub async fn unload_package(&self, package_id: &PackageId) -> anyhow::Result<PackageRecord> {
        if let Some(stopping) = self.packages.set_state(package_id, PackageState::Stopping).await {
            self.append_package_lifecycle_event(&stopping, EVENT_PACKAGE_STOPPING, None).await?;
        }
        self.subprocesses.stop(package_id).await;
        if let Some(stopped) = self.packages.set_state(package_id, PackageState::Stopped).await {
            self.append_package_lifecycle_event(&stopped, EVENT_PACKAGE_STOPPED, None).await?;
        }
        let record = self.packages.unload(package_id).await?;
        self.capabilities.unregister_package(package_id).await;
        self.extensions.unregister_package(package_id).await;
        self.append_package_lifecycle_event(&record, EVENT_PACKAGE_UNLOADED, None).await?;
        Ok(record)
    }

    pub async fn restart_package(&self, package_id: &PackageId) -> anyhow::Result<PackageRecord> {
        let record = self
            .package_status(package_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("package '{package_id}' is not loaded"))?;
        if !matches!(record.manifest.entry, PackageEntry::Subprocess { .. }) {
            anyhow::bail!("package '{package_id}' entry kind '{}' cannot restart yet", record.entry_kind);
        }
        if let Some(stopping) = self.packages.set_state(package_id, PackageState::Stopping).await {
            self.append_package_lifecycle_event(&stopping, EVENT_PACKAGE_STOPPING, Some("restart")).await?;
        }
        self.subprocesses.restart(&record.manifest).await?;
        let ready = self
            .packages
            .set_state(package_id, PackageState::Ready)
            .await
            .ok_or_else(|| anyhow::anyhow!("package '{package_id}' disappeared during restart"))?;
        self.append_package_lifecycle_event(&ready, EVENT_PACKAGE_READY, Some("restart")).await?;
        Ok(ready)
    }

    pub async fn package_logs(&self, package_id: &PackageId) -> Vec<crate::SubprocessLogLine> {
        let logs = self.subprocesses.drain_logs(package_id).await;
        for log in &logs {
            let _ = self.append_package_log_event(package_id, &log.stream, &log.line).await;
        }
        logs
    }

    pub async fn host_diagnostics(&self) -> Value {
        let packages = self.list_packages().await;
        let capabilities = self.discover_capabilities().await;
        let hooks = self.extensions.list_all_hooks().await;
        json!({
            "package_count": packages.len(),
            "capability_provider_count": capabilities.len(),
            "hook_subscription_count": hooks.len(),
            "packages": packages,
        })
    }

    pub async fn list_packages(&self) -> Vec<PackageRecord> {
        self.packages.list().await
    }

    pub async fn package_status(&self, package_id: &PackageId) -> Option<PackageRecord> {
        self.packages.status(package_id).await
    }

    pub async fn discover_capabilities(&self) -> Vec<crate::RegisteredCapability> {
        self.capabilities.discover().await
    }

    pub async fn invoke_capability(
        &self,
        request: CapabilityInvocationRequest,
    ) -> anyhow::Result<CapabilityInvocationResult> {
        if let Some(caller) = &request.caller_package_id {
            let allowed = self
                .packages
                .permissions(caller)
                .await
                .map(|permissions| {
                    permissions.capabilities.invoke.iter().any(|pattern| {
                        pattern == "*" || pattern == &request.capability_id || request.capability_id.starts_with(pattern.trim_end_matches('*'))
                    })
                })
                .unwrap_or(false);
            if !allowed {
                self.audit_permission_denied(
                    &format!("kernel_capability_{}", request.capability_id.replace('/', "_")),
                    caller,
                    "capabilities.invoke",
                )
                .await?;
                anyhow::bail!("package '{caller}' is not allowed to invoke '{}'", request.capability_id);
            }
        }
        let before = self
            .dispatch_extension_handlers(
                "kernel/capability.before_invoke",
                json!({
                    "capability_id": request.capability_id,
                    "caller_package_id": request.caller_package_id,
                    "input": request.input,
                }),
            )
            .await;
        if let Some(vetoed_by) = before.vetoed_by {
            anyhow::bail!("capability invoke vetoed by hook package '{vetoed_by}'");
        }

        let provider = self
            .capabilities
            .resolve(
                &request.capability_id,
                request.provider_package_id.as_ref(),
                request.version.as_deref(),
            )
            .await?;
        validate_json_schema_subset(&provider.descriptor.input_schema, &request.input)?;
        let output = self.execute_registered_capability(&provider, &request.capability_id, request.input).await?;
        let result = CapabilityInvocationResult {
            capability_id: provider.descriptor.id,
            provider_package_id: provider.provider_package_id,
            output,
        };
        let _ = self
            .dispatch_extension_handlers("kernel/capability.after_invoke", serde_json::to_value(&result).unwrap_or_else(|_| json!({})))
            .await;
        Ok(result)
    }

    async fn execute_registered_capability(
        &self,
        provider: &RegisteredCapability,
        capability_id: &str,
        input: Value,
    ) -> anyhow::Result<Value> {
        let output = match self.package_status(&provider.provider_package_id).await {
            Some(record) => match record.manifest.entry {
                PackageEntry::RustInproc { crate_ref, symbol, .. } => {
                    let package = self
                        .config
                        .inproc_packages
                        .lookup(&crate_ref, &symbol)
                        .ok_or_else(|| anyhow::anyhow!("rust_inproc entry '{crate_ref}::{symbol}' is not available"))?;
                    package
                        .invoke(InprocInvocation {
                            capability_id: capability_id.to_string(),
                            provider_package_id: provider.provider_package_id.clone(),
                            input,
                        })
                        .await?
                }
                PackageEntry::Subprocess { .. } => match self
                    .subprocesses
                    .invoke(&provider.provider_package_id, capability_id, input)
                    .await
                {
                    Ok(output) => output,
                    Err(error) => {
                        if let Some(record) = self
                            .packages
                            .set_state(&provider.provider_package_id, PackageState::Degraded)
                            .await
                        {
                            self.append_package_degraded_event(&record, &error.to_string()).await?;
                        }
                        return Err(error);
                    }
                },
                other => anyhow::bail!("entry kind '{}' cannot execute capabilities yet", crate::entry_kind(&other)),
            },
            None => anyhow::bail!("provider package '{}' is not loaded", provider.provider_package_id),
        };
        validate_json_schema_subset(&provider.descriptor.output_schema, &output)?;
        Ok(output)
    }

    pub async fn invoke_capability_with_context(
        &self,
        context: &ProtocolContext,
        mut request: CapabilityInvocationRequest,
    ) -> anyhow::Result<CapabilityInvocationResult> {
        match &context.principal {
            ProtocolPrincipal::HostAdmin | ProtocolPrincipal::HostDev => {
                request.caller_package_id = None;
                self.invoke_capability(request).await
            }
            ProtocolPrincipal::Package { package_id } => {
                request.caller_package_id = Some(package_id.clone());
                self.invoke_capability(request).await
            }
            ProtocolPrincipal::Anonymous => anyhow::bail!("anonymous principal is not allowed to invoke capabilities"),
        }
    }

    pub async fn dispatch_extension(&self, extension_point: &str, payload: Value) -> ExtensionDispatchResult {
        self.extensions.dispatch(extension_point, payload).await
    }

    async fn dispatch_extension_handlers(&self, extension_point: &str, payload: Value) -> ExtensionDispatchResult {
        let invoked = self.extensions.list_hooks(extension_point).await;
        let mut payload = payload;
        let mut vetoed_by = None;
        for hook in &invoked {
            match hook.subscription.handler.as_str() {
                "veto" => {
                    vetoed_by = Some(hook.subscriber_package_id.clone());
                    break;
                }
                "metadata_trace" => merge_metadata_patch(&mut payload, json!({"hook_trace": hook.subscriber_package_id})),
                handler if handler.contains('/') => {
                    let handler_id = handler.to_string();
                    let provider = match self
                        .capabilities
                        .resolve(&handler_id, Some(&hook.subscriber_package_id), None)
                        .await {
                        Ok(provider) => provider,
                        Err(_) => {
                            vetoed_by = Some(hook.subscriber_package_id.clone());
                            break;
                        }
                    };
                    if validate_json_schema_subset(&provider.descriptor.input_schema, &payload).is_err() {
                        vetoed_by = Some(hook.subscriber_package_id.clone());
                        break;
                    }
                    let output = match self
                        .execute_registered_capability(&provider, &handler_id, payload.clone())
                        .await
                    {
                        Ok(output) => output,
                        Err(_) => {
                            vetoed_by = Some(hook.subscriber_package_id.clone());
                            break;
                        }
                    };
                    if output.get("decision").and_then(Value::as_str) == Some("veto") {
                        vetoed_by = Some(hook.subscriber_package_id.clone());
                        break;
                    }
                    if let Some(patch) = output.get("metadata_patch") {
                        merge_metadata_patch(&mut payload, patch.clone());
                    }
                }
                _ => {}
            }
        }
        ExtensionDispatchResult { extension_point: extension_point.to_string(), invoked, vetoed_by, payload }
    }

    pub async fn call_protocol(
        &self,
        context: &ProtocolContext,
        method: &str,
        params: Value,
    ) -> Result<Value, crate::ProtocolError> {
        self.call_protocol_inner(context, method, params)
            .await
            .map_err(crate::ProtocolError::from_anyhow)
    }

    async fn call_protocol_inner(&self, context: &ProtocolContext, method: &str, params: Value) -> anyhow::Result<Value> {
        match method {
            "kernel.host.info" => Ok(serde_json::to_value(crate::host_info())?),
            "kernel.host.ping" => Ok(json!({"ok": true})),
            "kernel.host.diagnostics" => Ok(self.host_diagnostics().await),
            "kernel.session.open" => Ok(serde_json::to_value(
                self.open_session(serde_json::from_value(params)?).await?,
            )?),
            "kernel.session.fork" => {
                let parent_session_id = params
                    .get("parent_session_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.session.fork requires parent_session_id"))?
                    .to_string();
                let forked_from_sequence = params
                    .get("forked_from_sequence")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| anyhow::anyhow!("kernel.session.fork requires forked_from_sequence"))?;
                let metadata = params.get("metadata").cloned().unwrap_or_else(|| json!({}));
                Ok(serde_json::to_value(self.fork_session(parent_session_id, forked_from_sequence, metadata).await?)?)
            }
            "kernel.session.branch.list" => {
                let session_id = params
                    .get("session_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.session.branch.list requires session_id"))?
                    .to_string();
                Ok(serde_json::to_value(self.list_branches(&session_id).await)?)
            }
            "kernel.event.append" => Ok(serde_json::to_value(
                self.append_event_with_context(context, serde_json::from_value(params)?).await?,
            )?),
            "kernel.event.list" => {
                let request: EventListRequest = serde_json::from_value(params)?;
                Ok(serde_json::to_value(self.list_events_range_with_context(context, &request).await?)?)
            }
            "kernel.package.load" => Ok(serde_json::to_value(
                self.load_package(serde_json::from_value(params)?).await?,
            )?),
            "kernel.package.list" => Ok(serde_json::to_value(self.list_packages().await)?),
            "kernel.package.status" => {
                let package_id = params
                    .get("package_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.package.status requires package_id"))?
                    .to_string();
                Ok(serde_json::to_value(
                    self.package_status(&package_id)
                        .await
                        .ok_or_else(|| anyhow::anyhow!("package '{package_id}' is not loaded"))?,
                )?)
            }
            "kernel.package.unload" => {
                let package_id = params
                    .get("package_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.package.unload requires package_id"))?
                    .to_string();
                Ok(serde_json::to_value(self.unload_package(&package_id).await?)?)
            }
            "kernel.package.restart" => {
                let package_id = params
                    .get("package_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.package.restart requires package_id"))?
                    .to_string();
                Ok(serde_json::to_value(self.restart_package(&package_id).await?)?)
            }
            "kernel.package.logs" => {
                let package_id = params
                    .get("package_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.package.logs requires package_id"))?
                    .to_string();
                Ok(serde_json::to_value(self.package_logs(&package_id).await)?)
            }
            "kernel.capability.discover" => Ok(serde_json::to_value(self.discover_capabilities().await)?),
            "kernel.capability.invoke" => Ok(serde_json::to_value(
                self.invoke_capability_with_context(context, serde_json::from_value(params)?).await?,
            )?),
            "kernel.extension_point.list" => Ok(json!([
                "kernel/event.before_append",
                "kernel/event.after_append",
                "kernel/capability.before_invoke",
                "kernel/capability.after_invoke",
                "kernel/package.loaded",
                "kernel/package.unloaded"
            ])),
            "kernel.hook.list" => Ok(serde_json::to_value(self.extensions.list_all_hooks().await)?),
            "kernel.asset.put" => Ok(serde_json::to_value(self.put_asset(serde_json::from_value(params)?).await?)?),
            "kernel.asset.get" => {
                let asset_id = params
                    .get("asset_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.asset.get requires asset_id"))?;
                Ok(serde_json::to_value(self.get_asset(asset_id).await?)?)
            }
            "kernel.asset.list" => Ok(serde_json::to_value(self.list_assets().await)?),
            "kernel.projection.register" => Ok(serde_json::to_value(self.projection_register(serde_json::from_value(params)?).await?)?),
            "kernel.projection.rebuild" => {
                let projection_id = params
                    .get("projection_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.projection.rebuild requires projection_id"))?;
                Ok(serde_json::to_value(self.projection_rebuild(projection_id).await?)?)
            }
            "kernel.projection.get" => {
                let projection_id = params
                    .get("projection_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.projection.get requires projection_id"))?;
                Ok(serde_json::to_value(self.projection_get(projection_id).await?)?)
            }
            other => anyhow::bail!("protocol method '{other}' is not implemented"),
        }
    }

    async fn append_kernel_event(
        &self,
        session_id: &SessionId,
        kind: &'static str,
        payload: Value,
    ) -> anyhow::Result<EventEnvelope> {
        self.append_event_unchecked(AppendEventRequest {
            session_id: session_id.clone(),
            writer_package_id: KERNEL_PACKAGE_ID.to_string(),
            kind: kind.to_string(),
            payload,
            metadata: json!({}),
        })
        .await
    }

    async fn append_package_degraded_event(&self, record: &PackageRecord, reason: &str) -> anyhow::Result<EventEnvelope> {
        self.append_package_lifecycle_event(record, EVENT_PACKAGE_DEGRADED, Some(reason)).await
    }

    async fn append_package_lifecycle_event(
        &self,
        record: &PackageRecord,
        kind: &'static str,
        reason: Option<&str>,
    ) -> anyhow::Result<EventEnvelope> {
        let session_id = format!("kernel_package_{}", record.id.replace('/', "_"));
        let mut payload = json!({
            "package_id": record.id,
            "version": record.version,
            "state": record.state,
            "entry_kind": record.entry_kind,
            "capability_count": record.capability_count,
            "hook_count": record.hook_count,
            "extension_point_count": record.extension_point_count,
        });
        if let Some(reason) = reason {
            payload["reason"] = json!(reason);
        }
        self.append_kernel_event(&session_id, kind, payload).await
    }

    async fn append_package_log_event(&self, package_id: &PackageId, stream: &str, line: &str) -> anyhow::Result<EventEnvelope> {
        let session_id = format!("kernel_package_{}", package_id.replace('/', "_"));
        self.append_kernel_event(
            &session_id,
            EVENT_PACKAGE_LOG,
            json!({"package_id": package_id, "stream": stream, "line": line}),
        )
        .await
    }

    async fn audit_permission_denied(
        &self,
        session_id: &SessionId,
        package_id: &PackageId,
        operation: &str,
    ) -> anyhow::Result<EventEnvelope> {
        self.append_kernel_event(
            session_id,
            EVENT_PERMISSION_DENIED,
            json!({
                "package_id": package_id,
                "operation": operation,
            }),
        )
        .await
    }
}

fn merge_metadata_patch(payload: &mut Value, patch: Value) {
    let Some(patch) = patch.as_object() else { return };
    let Some(object) = payload.as_object_mut() else { return };
    let metadata = object.entry("metadata").or_insert_with(|| Value::Object(Default::default()));
    let Some(metadata) = metadata.as_object_mut() else { return };
    for (key, value) in patch {
        metadata.insert(key.clone(), value.clone());
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;
    use ygg_core::{PackageContributions, PackageEntry, PermissionSet, SandboxPolicy};

    use super::*;
    use crate::InMemoryEventStore;

    #[tokio::test]
    async fn session_open_records_kernel_event() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(store.clone(), RuntimeConfig::default());

        let session = runtime.open_session(OpenSessionRequest::default()).await?;
        let events = store.list_session(&session.id).await?;

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sequence, 0);
        assert_eq!(events[0].writer_package_id, KERNEL_PACKAGE_ID);
        assert_eq!(events[0].kind, EVENT_SESSION_OPENED);
        assert!(events[0].is_kernel_event());

        Ok(())
    }

    #[tokio::test]
    async fn package_cannot_write_another_namespace() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(store, RuntimeConfig::default());
        let session = runtime.open_session(OpenSessionRequest::default()).await?;

        let result = runtime
            .append_event(AppendEventRequest {
                session_id: session.id,
                writer_package_id: "org/a".to_string(),
                kind: "org/b/event".to_string(),
                payload: json!({}),
                metadata: json!({}),
            })
            .await;

        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn package_can_write_its_own_namespace() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(store.clone(), RuntimeConfig::default());
        let session = runtime.open_session(OpenSessionRequest::default()).await?;
        runtime
            .load_package(PackageManifest {
                schema_version: 1,
                id: "org/a".to_string(),
                version: "0.1.0".to_string(),
                display_name: None,
                description: None,
                author: None,
                license: None,
                entry: PackageEntry::RustInproc {
                    crate_ref: "org-a".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                },
                provides: Vec::new(),
                consumes: Vec::new(),
                contributes: PackageContributions::default(),
                permissions: PermissionSet {
                    events: ygg_core::EventPermissions { read: false, append: true },
                    ..PermissionSet::default()
                },
                sandbox_policy: SandboxPolicy::default(),
            })
            .await?;

        let event = runtime
            .append_event(AppendEventRequest {
                session_id: session.id.clone(),
                writer_package_id: "org/a".to_string(),
                kind: "org/a/event".to_string(),
                payload: json!({"ok": true}),
                metadata: json!({}),
            })
            .await?;

        assert_eq!(event.sequence, 1);
        let events = store.list_session(&session.id).await?;
        assert_eq!(events.len(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn package_load_records_kernel_lifecycle_event() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(store.clone(), RuntimeConfig::default());

        let record = runtime
            .load_package(PackageManifest {
                schema_version: 1,
                id: "org/pkg".to_string(),
                version: "0.1.0".to_string(),
                display_name: None,
                description: None,
                author: None,
                license: None,
                entry: PackageEntry::RustInproc {
                    crate_ref: "org-pkg".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                },
                provides: Vec::new(),
                consumes: Vec::new(),
                contributes: PackageContributions::default(),
                permissions: PermissionSet::default(),
                sandbox_policy: SandboxPolicy::default(),
            })
            .await?;

        assert_eq!(record.id, "org/pkg");
        let events = store.list_session(&"kernel_package_org_pkg".to_string()).await?;
        assert!(events.iter().any(|event| event.kind == EVENT_PACKAGE_LOADING));
        assert!(events.iter().any(|event| event.kind == EVENT_PACKAGE_READY));
        assert!(events.iter().any(|event| event.kind == EVENT_PACKAGE_LOADED));
        Ok(())
    }

    #[tokio::test]
    async fn loaded_package_registers_capability() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(store, RuntimeConfig::default());
        runtime
            .load_package(PackageManifest {
                schema_version: 1,
                id: "example/echo".to_string(),
                version: "0.1.0".to_string(),
                display_name: None,
                description: None,
                author: None,
                license: None,
                entry: PackageEntry::RustInproc {
                    crate_ref: "example-echo-rust-inproc".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                },
                provides: vec![ygg_core::CapabilityDescriptor {
                    id: "example/echo/echo".to_string(),
                    version: "0.1.0".to_string(),
                    input_schema: Value::Null,
                    output_schema: Value::Null,
                    streaming: false,
                    side_effects: Vec::new(),
                    description: None,
                }],
                consumes: Vec::new(),
                contributes: PackageContributions::default(),
                permissions: PermissionSet::default(),
                sandbox_policy: SandboxPolicy::default(),
            })
            .await?;

        let result = runtime
            .invoke_capability(CapabilityInvocationRequest {
                capability_id: "example/echo/echo".to_string(),
                caller_package_id: None,
                provider_package_id: None,
                version: None,
                input: json!({"ping": true}),
            })
            .await?;
        assert_eq!(result.output, json!({"ping": true}));
        Ok(())
    }

    #[tokio::test]
    async fn rust_inproc_provider_must_exist_in_host_catalog() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(store, RuntimeConfig::default());

        let result = runtime
            .load_package(PackageManifest {
                schema_version: 1,
                id: "example/missing".to_string(),
                version: "0.1.0".to_string(),
                display_name: None,
                description: None,
                author: None,
                license: None,
                entry: PackageEntry::RustInproc {
                    crate_ref: "missing-crate".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                },
                provides: vec![ygg_core::CapabilityDescriptor {
                    id: "example/missing/echo".to_string(),
                    version: "0.1.0".to_string(),
                    input_schema: Value::Null,
                    output_schema: Value::Null,
                    streaming: false,
                    side_effects: Vec::new(),
                    description: None,
                }],
                consumes: Vec::new(),
                contributes: PackageContributions::default(),
                permissions: PermissionSet::default(),
                sandbox_policy: SandboxPolicy::default(),
            })
            .await;

        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn denied_event_append_records_audit_event() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(store.clone(), RuntimeConfig::default());
        let session = runtime.open_session(OpenSessionRequest::default()).await?;

        let denied = runtime
            .append_event(AppendEventRequest {
                session_id: session.id.clone(),
                writer_package_id: "org/unauthorized".to_string(),
                kind: "org/unauthorized/event".to_string(),
                payload: json!({}),
                metadata: json!({}),
            })
            .await;
        assert!(denied.is_err());

        let events = store.list_session(&session.id).await?;
        assert_eq!(events.last().expect("audit event").kind, EVENT_PERMISSION_DENIED);
        Ok(())
    }

    #[tokio::test]
    async fn denied_capability_invoke_records_audit_event() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(store.clone(), RuntimeConfig::default());
        runtime
            .load_package(PackageManifest {
                schema_version: 1,
                id: "example/echo".to_string(),
                version: "0.1.0".to_string(),
                display_name: None,
                description: None,
                author: None,
                license: None,
                entry: PackageEntry::RustInproc {
                    crate_ref: "example-echo-rust-inproc".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                },
                provides: vec![ygg_core::CapabilityDescriptor {
                    id: "example/echo/echo".to_string(),
                    version: "0.1.0".to_string(),
                    input_schema: Value::Null,
                    output_schema: Value::Null,
                    streaming: false,
                    side_effects: Vec::new(),
                    description: None,
                }],
                consumes: Vec::new(),
                contributes: PackageContributions::default(),
                permissions: PermissionSet::default(),
                sandbox_policy: SandboxPolicy::default(),
            })
            .await?;
        runtime
            .load_package(PackageManifest {
                schema_version: 1,
                id: "example/caller".to_string(),
                version: "0.1.0".to_string(),
                display_name: None,
                description: None,
                author: None,
                license: None,
                entry: PackageEntry::RustInproc {
                    crate_ref: "example-caller".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                },
                provides: Vec::new(),
                consumes: Vec::new(),
                contributes: PackageContributions::default(),
                permissions: PermissionSet::default(),
                sandbox_policy: SandboxPolicy::default(),
            })
            .await?;

        let denied = runtime
            .invoke_capability(CapabilityInvocationRequest {
                capability_id: "example/echo/echo".to_string(),
                caller_package_id: Some("example/caller".to_string()),
                provider_package_id: None,
                version: None,
                input: json!({}),
            })
            .await;
        assert!(denied.is_err());

        let events = store.list_session(&"kernel_capability_example_echo_echo".to_string()).await?;
        assert_eq!(events.last().expect("audit event").kind, EVENT_PERMISSION_DENIED);
        Ok(())
    }

    #[tokio::test]
    async fn package_context_overrides_spoofed_event_writer() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(store, RuntimeConfig::default());
        let session = runtime.open_session(OpenSessionRequest::default()).await?;
        runtime
            .load_package(PackageManifest {
                schema_version: 1,
                id: "example/caller".to_string(),
                version: "0.1.0".to_string(),
                display_name: None,
                description: None,
                author: None,
                license: None,
                entry: PackageEntry::RustInproc {
                    crate_ref: "example-caller".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                },
                provides: Vec::new(),
                consumes: Vec::new(),
                contributes: PackageContributions::default(),
                permissions: PermissionSet {
                    events: ygg_core::EventPermissions { read: false, append: true },
                    ..PermissionSet::default()
                },
                sandbox_policy: SandboxPolicy::default(),
            })
            .await?;

        let event = runtime
            .append_event_with_context(
                &ProtocolContext::package("example/caller", "test"),
                AppendEventRequest {
                    session_id: session.id,
                    writer_package_id: "example/spoofed".to_string(),
                    kind: "example/caller/event".to_string(),
                    payload: json!({}),
                    metadata: json!({}),
                },
            )
            .await?;

        assert_eq!(event.writer_package_id, "example/caller");
        Ok(())
    }

    #[tokio::test]
    async fn package_context_overrides_spoofed_capability_caller() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(store, RuntimeConfig::default());
        runtime
            .load_package(PackageManifest {
                schema_version: 1,
                id: "example/echo".to_string(),
                version: "0.1.0".to_string(),
                display_name: None,
                description: None,
                author: None,
                license: None,
                entry: PackageEntry::RustInproc {
                    crate_ref: "example-echo-rust-inproc".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                },
                provides: vec![ygg_core::CapabilityDescriptor {
                    id: "example/echo/echo".to_string(),
                    version: "0.1.0".to_string(),
                    input_schema: Value::Null,
                    output_schema: Value::Null,
                    streaming: false,
                    side_effects: Vec::new(),
                    description: None,
                }],
                consumes: Vec::new(),
                contributes: PackageContributions::default(),
                permissions: PermissionSet::default(),
                sandbox_policy: SandboxPolicy::default(),
            })
            .await?;
        runtime
            .load_package(PackageManifest {
                schema_version: 1,
                id: "example/caller".to_string(),
                version: "0.1.0".to_string(),
                display_name: None,
                description: None,
                author: None,
                license: None,
                entry: PackageEntry::RustInproc {
                    crate_ref: "example-caller".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                },
                provides: Vec::new(),
                consumes: Vec::new(),
                contributes: PackageContributions::default(),
                permissions: PermissionSet::default(),
                sandbox_policy: SandboxPolicy::default(),
            })
            .await?;

        let denied = runtime
            .invoke_capability_with_context(
                &ProtocolContext::package("example/caller", "test"),
                CapabilityInvocationRequest {
                    capability_id: "example/echo/echo".to_string(),
                    caller_package_id: None,
                    provider_package_id: None,
                    version: None,
                    input: json!({}),
                },
            )
            .await;

        assert!(denied.is_err());
        Ok(())
    }
}
