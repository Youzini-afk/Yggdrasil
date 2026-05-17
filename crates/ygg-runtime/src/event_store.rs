use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use rusqlite::{params, Connection};
use tokio::sync::Mutex;
use tokio::sync::{broadcast, RwLock};
use ygg_core::{EventEnvelope, EventSequence, PackageId, SessionId};

#[async_trait]
pub trait EventStore: Send + Sync + 'static {
    async fn append(&self, event: EventEnvelope) -> anyhow::Result<()>;
    async fn list_all(&self) -> anyhow::Result<Vec<EventEnvelope>>;
    async fn list_session(&self, session_id: &SessionId) -> anyhow::Result<Vec<EventEnvelope>>;
    async fn list_session_range(
        &self,
        session_id: &SessionId,
        after_sequence: Option<EventSequence>,
        limit: Option<usize>,
    ) -> anyhow::Result<Vec<EventEnvelope>>;
    async fn next_sequence(&self, session_id: &SessionId) -> anyhow::Result<EventSequence>;
    fn subscribe(&self) -> broadcast::Receiver<EventEnvelope>;
}

#[derive(Clone)]
pub struct SqliteEventStore {
    conn: Arc<Mutex<Connection>>,
    tx: broadcast::Sender<EventEnvelope>,
}

impl SqliteEventStore {
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        init_schema(&conn)?;
        let (tx, _) = broadcast::channel(256);
        Ok(Self { conn: Arc::new(Mutex::new(conn)), tx })
    }
}

fn init_schema(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS events (
          id TEXT PRIMARY KEY,
          session_id TEXT NOT NULL,
          sequence INTEGER NOT NULL,
          timestamp TEXT NOT NULL,
          writer_package_id TEXT NOT NULL,
          kind TEXT NOT NULL,
          schema_version INTEGER NOT NULL,
          payload_json TEXT NOT NULL,
          metadata_json TEXT NOT NULL,
          UNIQUE(session_id, sequence)
        );
        CREATE INDEX IF NOT EXISTS idx_events_session_sequence ON events(session_id, sequence);
        "#,
    )?;
    Ok(())
}

#[async_trait]
impl EventStore for SqliteEventStore {
    async fn append(&self, event: EventEnvelope) -> anyhow::Result<()> {
        let payload = serde_json::to_string(&event.payload)?;
        let metadata = serde_json::to_string(&event.metadata)?;
        self.conn.lock().await.execute(
            "INSERT INTO events (id, session_id, sequence, timestamp, writer_package_id, kind, schema_version, payload_json, metadata_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                event.id,
                event.session_id,
                event.sequence as i64,
                event.timestamp.to_rfc3339(),
                event.writer_package_id,
                event.kind,
                event.schema_version as i64,
                payload,
                metadata,
            ],
        )?;
        let _ = self.tx.send(event);
        Ok(())
    }

    async fn list_session(&self, session_id: &SessionId) -> anyhow::Result<Vec<EventEnvelope>> {
        self.list_session_range(session_id, None, None).await
    }

    async fn list_all(&self) -> anyhow::Result<Vec<EventEnvelope>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, sequence, timestamp, writer_package_id, kind, schema_version, payload_json, metadata_json
             FROM events ORDER BY timestamp ASC, session_id ASC, sequence ASC",
        )?;
        let rows = stmt.query_map([], row_to_event)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    async fn list_session_range(
        &self,
        session_id: &SessionId,
        after_sequence: Option<EventSequence>,
        limit: Option<usize>,
    ) -> anyhow::Result<Vec<EventEnvelope>> {
        let conn = self.conn.lock().await;
        let after_sequence = after_sequence.map(|sequence| sequence as i64).unwrap_or(-1);
        let limit = limit.unwrap_or(1_000).min(10_000) as i64;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, sequence, timestamp, writer_package_id, kind, schema_version, payload_json, metadata_json
             FROM events WHERE session_id = ?1 AND sequence > ?2 ORDER BY sequence ASC LIMIT ?3",
        )?;
        let rows = stmt.query_map(params![session_id, after_sequence, limit], row_to_event)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    async fn next_sequence(&self, session_id: &SessionId) -> anyhow::Result<EventSequence> {
        let next: i64 = self.conn.lock().await.query_row(
            "SELECT COALESCE(MAX(sequence) + 1, 0) FROM events WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;
        Ok(next as EventSequence)
    }

    fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.tx.subscribe()
    }
}

fn row_to_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<EventEnvelope> {
    let timestamp: String = row.get(3)?;
    let payload_json: String = row.get(7)?;
    let metadata_json: String = row.get(8)?;
    let sequence: i64 = row.get(2)?;
    let schema_version: i64 = row.get(6)?;
    Ok(EventEnvelope {
        id: row.get(0)?,
        session_id: row.get(1)?,
        sequence: sequence as EventSequence,
        timestamp: chrono::DateTime::parse_from_rfc3339(&timestamp)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|err| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(err)))?,
        writer_package_id: row.get::<_, PackageId>(4)?,
        kind: row.get(5)?,
        schema_version: schema_version as u16,
        payload: serde_json::from_str(&payload_json)
            .map_err(|err| rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(err)))?,
        metadata: serde_json::from_str(&metadata_json)
            .map_err(|err| rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Text, Box::new(err)))?,
    })
}

#[cfg(test)]
mod sqlite_tests {
    use serde_json::json;
    use ygg_core::{new_id, EventEnvelope, KERNEL_PACKAGE_ID};

    use super::*;

    #[tokio::test]
    async fn sqlite_store_persists_and_replays_events() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(format!("ygg-test-{}.db", new_id("sqlite")));
        let store = SqliteEventStore::open(&path)?;
        let session_id = "ses_test".to_string();
        store
            .append(EventEnvelope::new(
                new_id("evt"),
                session_id.clone(),
                0,
                KERNEL_PACKAGE_ID.to_string(),
                "kernel/session.opened",
                json!({"ok": true}),
            ))
            .await?;
        drop(store);

        let reopened = SqliteEventStore::open(&path)?;
        let events = reopened.list_session(&session_id).await?;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sequence, 0);
        assert_eq!(reopened.next_sequence(&session_id).await?, 1);
        let _ = std::fs::remove_file(path);
        Ok(())
    }
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
        self.list_session_range(session_id, None, None).await
    }

    async fn list_all(&self) -> anyhow::Result<Vec<EventEnvelope>> {
        let mut events: Vec<_> = self.events.read().await.values().flat_map(|events| events.clone()).collect();
        events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp).then(a.session_id.cmp(&b.session_id)).then(a.sequence.cmp(&b.sequence)));
        Ok(events)
    }

    async fn list_session_range(
        &self,
        session_id: &SessionId,
        after_sequence: Option<EventSequence>,
        limit: Option<usize>,
    ) -> anyhow::Result<Vec<EventEnvelope>> {
        let after_sequence = after_sequence;
        let mut events: Vec<_> = self
            .events
            .read()
            .await
            .get(session_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|event| after_sequence.map(|sequence| event.sequence > sequence).unwrap_or(true))
            .collect();
        if let Some(limit) = limit {
            events.truncate(limit);
        }
        Ok(events)
    }

    async fn next_sequence(&self, session_id: &SessionId) -> anyhow::Result<EventSequence> {
        Ok(self.events.read().await.get(session_id).map(|events| events.len() as EventSequence).unwrap_or(0))
    }

    fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.tx.subscribe()
    }
}
