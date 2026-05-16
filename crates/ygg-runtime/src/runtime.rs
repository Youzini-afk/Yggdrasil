use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use ygg_core::{
    new_id, EventEnvelope, EventKind, KernelSession, PackageEntry, PackageId, PackageManifest, SessionId,
    SessionStatus, EVENT_PACKAGE_DEGRADED, EVENT_PACKAGE_LOADED, EVENT_PACKAGE_UNLOADED, EVENT_PERMISSION_DENIED,
    EVENT_SESSION_CLOSED, EVENT_SESSION_OPENED, KERNEL_PACKAGE_ID,
};

use crate::{
    CapabilityFabric, CapabilityInvocationRequest, CapabilityInvocationResult, EventStore,
    ExtensionDispatchResult, ExtensionRegistry, HostPolicy, InprocInvocation, InprocPackageCatalog,
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
            .extensions
            .dispatch(
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
            .extensions
            .dispatch("kernel/event.after_append", serde_json::to_value(&event).unwrap_or_else(|_| json!({})))
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
        if matches!(record.manifest.entry, PackageEntry::Subprocess { .. }) {
            record = self
                .packages
                .set_state(&record.id, PackageState::Starting)
                .await
                .unwrap_or(record);
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
        self.capabilities.register_package(&record.id, &record.manifest.provides).await;
        self.extensions.register_package(&record.id, &record.manifest.contributes.hooks).await;
        let session_id = format!("kernel_package_{}", record.id.replace('/', "_"));
        self.append_kernel_event(
            &session_id,
            EVENT_PACKAGE_LOADED,
            json!({
                "package_id": record.id,
                "version": record.version,
                "state": record.state,
                "entry_kind": record.entry_kind,
                "capability_count": record.capability_count,
                "hook_count": record.hook_count,
                "extension_point_count": record.extension_point_count,
            }),
        )
        .await?;
        Ok(record)
    }

    pub async fn unload_package(&self, package_id: &PackageId) -> anyhow::Result<PackageRecord> {
        self.subprocesses.stop(package_id).await;
        let record = self.packages.unload(package_id).await?;
        self.capabilities.unregister_package(package_id).await;
        self.extensions.unregister_package(package_id).await;
        let session_id = format!("kernel_package_{}", record.id.replace('/', "_"));
        self.append_kernel_event(
            &session_id,
            EVENT_PACKAGE_UNLOADED,
            json!({
                "package_id": record.id,
                "version": record.version,
                "state": record.state,
                "entry_kind": record.entry_kind,
            }),
        )
        .await?;
        Ok(record)
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
            .extensions
            .dispatch(
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

        let provider = self.capabilities.resolve(&request.capability_id).await?;
        validate_json_schema_subset(&provider.descriptor.input_schema, &request.input)?;
        let output = match &provider.descriptor.id {
            _ => match self.package_status(&provider.provider_package_id).await {
                Some(record) => match record.manifest.entry {
                    PackageEntry::RustInproc { crate_ref, symbol, .. } => {
                        let package = self
                            .config
                            .inproc_packages
                            .lookup(&crate_ref, &symbol)
                            .ok_or_else(|| anyhow::anyhow!("rust_inproc entry '{crate_ref}::{symbol}' is not available"))?;
                        package
                            .invoke(InprocInvocation {
                                capability_id: request.capability_id.clone(),
                                provider_package_id: provider.provider_package_id.clone(),
                                input: request.input,
                            })
                            .await?
                    }
                    PackageEntry::Subprocess { .. } => {
                        match self
                            .subprocesses
                            .invoke(&provider.provider_package_id, &request.capability_id, request.input)
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
                        }
                    }
                    other => anyhow::bail!("entry kind '{}' cannot execute capabilities yet", crate::entry_kind(&other)),
                },
                None => anyhow::bail!("provider package '{}' is not loaded", provider.provider_package_id),
            },
        };
        validate_json_schema_subset(&provider.descriptor.output_schema, &output)?;
        let result = CapabilityInvocationResult {
            capability_id: provider.descriptor.id,
            provider_package_id: provider.provider_package_id,
            output,
        };
        let _ = self
            .extensions
            .dispatch("kernel/capability.after_invoke", serde_json::to_value(&result).unwrap_or_else(|_| json!({})))
            .await;
        Ok(result)
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
            "kernel.session.open" => Ok(serde_json::to_value(
                self.open_session(serde_json::from_value(params)?).await?,
            )?),
            "kernel.event.append" => Ok(serde_json::to_value(
                self.append_event_with_context(context, serde_json::from_value(params)?).await?,
            )?),
            "kernel.event.list" => {
                let session_id = params
                    .get("session_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.event.list requires session_id"))?
                    .to_string();
                Ok(serde_json::to_value(self.list_events_with_context(context, &session_id).await?)?)
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
        let session_id = format!("kernel_package_{}", record.id.replace('/', "_"));
        self.append_kernel_event(
            &session_id,
            EVENT_PACKAGE_DEGRADED,
            json!({
                "package_id": record.id,
                "version": record.version,
                "state": record.state,
                "entry_kind": record.entry_kind,
                "reason": reason,
            }),
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
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EVENT_PACKAGE_LOADED);
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
                    input: json!({}),
                },
            )
            .await;

        assert!(denied.is_err());
        Ok(())
    }
}
