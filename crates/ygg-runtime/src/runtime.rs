use std::sync::Arc;

use chrono::Utc;
use serde_json::{json, Value};
use ygg_core::{
    new_id, EventEnvelope, EventKind, KernelSession, PackageId, SessionId, SessionStatus,
    EVENT_SESSION_CLOSED, EVENT_SESSION_OPENED, KERNEL_PACKAGE_ID,
};

use crate::EventStore;

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub default_labels: Vec<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self { default_labels: vec!["kernel".to_string()] }
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
    config: RuntimeConfig,
}

impl<S> Runtime<S>
where
    S: EventStore,
{
    pub fn new(store: Arc<S>, config: RuntimeConfig) -> Self {
        Self { store, config }
    }

    pub fn store(&self) -> Arc<S> {
        self.store.clone()
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
}
