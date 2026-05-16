use std::sync::Arc;

use chrono::Utc;
use serde_json::{json, Value};
use ygg_core::{
    new_id, EventEnvelope, EventKind, KernelSession, PackageId, PackageManifest, SessionId,
    SessionStatus, EVENT_PACKAGE_LOADED, EVENT_PACKAGE_UNLOADED, EVENT_SESSION_CLOSED,
    EVENT_SESSION_OPENED, KERNEL_PACKAGE_ID,
};

use crate::{
    CapabilityFabric, CapabilityInvocationRequest, CapabilityInvocationResult, EventStore,
    ExtensionDispatchResult, ExtensionRegistry, HostPolicy, PackageRecord, PackageRegistry,
};

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub default_labels: Vec<String>,
    pub host_policy: HostPolicy,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self { default_labels: vec!["kernel".to_string()], host_policy: HostPolicy::default() }
    }
}

#[derive(Debug, Clone, Default)]
pub struct OpenSessionRequest {
    pub labels: Vec<String>,
    pub active_package_set: Vec<PackageId>,
    pub metadata: Value,
}

#[derive(Debug, Clone)]
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
        self.append_kernel_event(&session_id, EVENT_SESSION_CLOSED, json!({})).await
    }

    pub async fn append_event(&self, request: AppendEventRequest) -> anyhow::Result<EventEnvelope> {
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
        self.store.append(event.clone()).await?;
        Ok(event)
    }

    pub async fn list_events(&self, session_id: &SessionId) -> anyhow::Result<Vec<EventEnvelope>> {
        self.store.list_session(session_id).await
    }

    pub async fn load_package(&self, manifest: PackageManifest) -> anyhow::Result<PackageRecord> {
        let record = self.packages.load(manifest, &self.config.host_policy).await?;
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
        self.capabilities.invoke(request).await
    }

    pub async fn dispatch_extension(&self, extension_point: &str, payload: Value) -> ExtensionDispatchResult {
        self.extensions.dispatch(extension_point, payload).await
    }

    async fn append_kernel_event(
        &self,
        session_id: &SessionId,
        kind: &'static str,
        payload: Value,
    ) -> anyhow::Result<EventEnvelope> {
        self.append_event(AppendEventRequest {
            session_id: session_id.clone(),
            writer_package_id: KERNEL_PACKAGE_ID.to_string(),
            kind: kind.to_string(),
            payload,
            metadata: json!({}),
        })
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
                    crate_ref: "example-echo".to_string(),
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
                input: json!({"ping": true}),
            })
            .await?;
        assert_eq!(result.output, json!({"ping": true}));
        Ok(())
    }
}
