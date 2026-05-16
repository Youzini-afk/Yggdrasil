use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{broadcast, RwLock};
use ygg_core::{EventEnvelope, SessionId};

#[async_trait]
pub trait EventStore: Send + Sync + 'static {
    async fn append(&self, event: EventEnvelope) -> anyhow::Result<()>;
    async fn list_session(&self, session_id: &SessionId) -> anyhow::Result<Vec<EventEnvelope>>;
    fn subscribe(&self) -> broadcast::Receiver<EventEnvelope>;
}

#[derive(Clone)]
pub struct InMemoryEventStore {
    events: Arc<RwLock<HashMap<SessionId, Vec<EventEnvelope>>>>,
    tx: broadcast::Sender<EventEnvelope>,
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self { events: Arc::new(RwLock::new(HashMap::new())), tx }
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn append(&self, event: EventEnvelope) -> anyhow::Result<()> {
        self.events
            .write()
            .await
            .entry(event.session_id.clone())
            .or_default()
            .push(event.clone());
        let _ = self.tx.send(event);
        Ok(())
    }

    async fn list_session(&self, session_id: &SessionId) -> anyhow::Result<Vec<EventEnvelope>> {
        Ok(self.events.read().await.get(session_id).cloned().unwrap_or_default())
    }

    fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.tx.subscribe()
    }
}
