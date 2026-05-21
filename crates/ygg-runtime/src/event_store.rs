use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use rusqlite::{params, Connection};
use tokio::sync::Mutex;
use tokio::sync::{broadcast, RwLock};
use ygg_core::{EventEnvelope, EventKind, EventSequence, PackageId, SessionId};

/// Backend-neutral event spine contract.
///
/// `EventStore` is the kernel's append-only event log abstraction. It is
/// **not** a database abstraction: no SQL, table, vector, DSN, connection,
/// transaction-isolation, or vendor-specific concept belongs here. Every
/// backend implementation (in-memory, SQLite, future PostgreSQL, etc.)
/// must produce the same observable behaviour for the same sequence of
/// calls.
///
/// # Ordering semantics
///
/// Events within a single session are ordered by the composite key
/// `(session_id, sequence)`. Each `(session_id, sequence)` pair is unique
/// within a store instance. Cross-session ordering is best-effort
/// (timestamp-based); the contract does not guarantee global total order.
///
/// # Append paths
///
/// - **`append_with_sequence`** — the runtime-recommended append path.
///   It atomically allocates the next sequence number and appends the
///   event under the same lock/transaction, guaranteeing no duplicate or
///   gap under concurrent access to the same session.
///
/// - **`append` + `next_sequence`** — low-level / test / admin path.
///   The caller must coordinate sequence assignment. Under concurrent
///   access, separate `next_sequence` + `append` calls can produce
///   duplicates or gaps. Prefer `append_with_sequence` unless you
///   explicitly need manual control.
///
/// # Kind-prefix queries
///
/// `list_kind_prefix` and `list_session_kind_prefix` are event-semantic
/// queries: "find events whose kind starts with this prefix." They are
/// **not** SQL `LIKE`, not index product API, and not vector search.
/// Backend implementations may use pushdown (SQLite `LIKE` / range scan)
/// or in-memory filtering; the observable result must be identical.
///
/// # No database concepts
///
/// This trait must never expose SQL, table, DSN, connection string,
/// credentials, file path, WAL mode, isolation level, vector dimension,
/// ANN index, embedding model, or any other concept specific to a
/// particular storage product. Such details belong to backend
/// constructors (`SqliteEventStore::open`, future
/// `PostgresEventStore::connect`, etc.), not to the event spine contract.
#[async_trait]
pub trait EventStore: Send + Sync + 'static {
    /// Low-level append: store a pre-constructed event envelope.
    /// Prefer `append_with_sequence` for runtime use; this method
    /// exists for replay, admin tooling, and test fixtures.
    async fn append(&self, event: EventEnvelope) -> anyhow::Result<()>;

    /// List all events across all sessions, ordered by
    /// `(timestamp, session_id, sequence)`.
    async fn list_all(&self) -> anyhow::Result<Vec<EventEnvelope>>;

    /// List all events within a session, ordered by sequence.
    async fn list_session(&self, session_id: &SessionId) -> anyhow::Result<Vec<EventEnvelope>>;

    /// List events within a session after a given sequence, with
    /// optional limit. This is the range-replay primitive.
    async fn list_session_range(
        &self,
        session_id: &SessionId,
        after_sequence: Option<EventSequence>,
        limit: Option<usize>,
    ) -> anyhow::Result<Vec<EventEnvelope>>;

    /// Return the next sequence number for a session.
    /// Low-level: prefer `append_with_sequence` for concurrent safety.
    async fn next_sequence(&self, session_id: &SessionId) -> anyhow::Result<EventSequence>;

    /// Subscribe to a broadcast channel of newly appended events.
    /// Backend-neutral: works identically for in-memory and durable stores.
    fn subscribe(&self) -> broadcast::Receiver<EventEnvelope>;

    /// **Runtime-recommended append path.** Atomically append an event,
    /// assigning the next sequence number within the same lock/transaction.
    /// This guarantees that concurrent appends to the same session receive
    /// contiguous, non-repeating sequence numbers without requiring the
    /// caller to call `next_sequence` separately.
    ///
    /// # Ordering guarantee
    ///
    /// Per-session `(session_id, sequence)` uniqueness is guaranteed.
    /// No two events in the same session will share a sequence number,
    /// and sequences are contiguous from 0.
    ///
    /// # When to use vs `append` + `next_sequence`
    ///
    /// - Use `append_with_sequence` in all runtime paths where
    ///   concurrent access is possible.
    /// - Use `append` + `next_sequence` only for single-writer
    ///   replay, admin tooling, or test fixtures where you control
    ///   all writers.
    ///
    /// The default implementation falls back to `next_sequence` + `append`,
    /// which is correct for single-writer or low-contention use but does
    /// not guarantee atomicity under concurrent access. Backend
    /// implementations that support atomic operations (e.g. SQLite with
    /// its connection mutex) override this with a truly atomic path.
    async fn append_with_sequence(
        &self,
        session_id: SessionId,
        writer_package_id: PackageId,
        kind: EventKind,
        schema_version: u16,
        payload_json: serde_json::Value,
        metadata_json: serde_json::Value,
    ) -> anyhow::Result<EventEnvelope> {
        let sequence = self.next_sequence(&session_id).await?;
        let event = EventEnvelope {
            id: ygg_core::new_id("evt"),
            session_id,
            sequence,
            timestamp: chrono::Utc::now(),
            writer_package_id,
            kind,
            schema_version,
            payload: payload_json,
            metadata: metadata_json,
        };
        self.append(event.clone()).await?;
        Ok(event)
    }

    /// **Event-semantic kind-prefix query.** List events whose `kind`
    /// starts with `prefix`, across all sessions. Results are ordered
    /// by `(timestamp, session_id, sequence)`.
    ///
    /// This is an event-level query: "find all events matching this
    /// kind prefix." It is **not** a SQL `LIKE` query, not an index
    /// product API, and not a vector/embedding search. Backend
    /// implementations may use pushdown (range scan, index, etc.)
    /// for performance, but the observable result set must match
    /// the in-memory baseline exactly.
    async fn list_kind_prefix(&self, prefix: &str) -> anyhow::Result<Vec<EventEnvelope>> {
        let all = self.list_all().await?;
        Ok(all
            .into_iter()
            .filter(|e| e.kind.starts_with(prefix))
            .collect())
    }

    /// **Event-semantic kind-prefix query within a session.** List events
    /// within a session whose `kind` starts with `prefix`. Results are
    /// ordered by sequence.
    ///
    /// Same contract as `list_kind_prefix` but scoped to a single session.
    /// Backend implementations may use session-scoped pushdown; the
    /// observable result set must match the in-memory baseline.
    async fn list_session_kind_prefix(
        &self,
        session_id: &SessionId,
        prefix: &str,
    ) -> anyhow::Result<Vec<EventEnvelope>> {
        let events = self.list_session(session_id).await?;
        Ok(events
            .into_iter()
            .filter(|e| e.kind.starts_with(prefix))
            .collect())
    }
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
        CREATE INDEX IF NOT EXISTS idx_events_kind ON events(kind);
        CREATE INDEX IF NOT EXISTS idx_events_session_kind_sequence ON events(session_id, kind, sequence);
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

    /// Atomically append: read max sequence and insert within the same
    /// connection mutex, guaranteeing no sequence gap or duplicate under
    /// concurrent access to the same session.
    async fn append_with_sequence(
        &self,
        session_id: SessionId,
        writer_package_id: PackageId,
        kind: EventKind,
        schema_version: u16,
        payload_json: serde_json::Value,
        metadata_json: serde_json::Value,
    ) -> anyhow::Result<EventEnvelope> {
        let conn = self.conn.lock().await;
        let next_seq: i64 = conn.query_row(
            "SELECT COALESCE(MAX(sequence) + 1, 0) FROM events WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;
        let event = EventEnvelope {
            id: ygg_core::new_id("evt"),
            session_id,
            sequence: next_seq as EventSequence,
            timestamp: chrono::Utc::now(),
            writer_package_id,
            kind,
            schema_version,
            payload: payload_json,
            metadata: metadata_json,
        };
        let payload = serde_json::to_string(&event.payload)?;
        let metadata = serde_json::to_string(&event.metadata)?;
        conn.execute(
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
        // Broadcast outside the lock is not strictly required for correctness,
        // but we hold the lock here to guarantee the event is visible before
        // subscribers are notified.
        let _ = self.tx.send(event.clone());
        Ok(event)
    }

    /// SQLite pushdown: uses `LIKE` with an upper bound to avoid a
    /// full-table scan when filtering by kind prefix.
    async fn list_kind_prefix(&self, prefix: &str) -> anyhow::Result<Vec<EventEnvelope>> {
        let conn = self.conn.lock().await;
        let prefix_end = kind_prefix_upper_bound(prefix);
        let mut stmt = if prefix_end.is_some() {
            conn.prepare(
                "SELECT id, session_id, sequence, timestamp, writer_package_id, kind, schema_version, payload_json, metadata_json
                 FROM events WHERE kind >= ?1 AND kind < ?2
                 ORDER BY timestamp ASC, session_id ASC, sequence ASC",
            )?
        } else {
            // Prefix with no upper bound (e.g. single-char or all-0xFF prefix);
            // fall back to LIKE-only.
            conn.prepare(
                "SELECT id, session_id, sequence, timestamp, writer_package_id, kind, schema_version, payload_json, metadata_json
                 FROM events WHERE kind LIKE ?1
                 ORDER BY timestamp ASC, session_id ASC, sequence ASC",
            )?
        };
        let rows = if let Some(end) = prefix_end {
            stmt.query_map(params![prefix, end], row_to_event)?
        } else {
            stmt.query_map(params![format!("{}%", prefix)], row_to_event)?
        };
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// SQLite pushdown: uses `LIKE` with session + kind within a single
    /// session to avoid full-session scan.
    async fn list_session_kind_prefix(
        &self,
        session_id: &SessionId,
        prefix: &str,
    ) -> anyhow::Result<Vec<EventEnvelope>> {
        let conn = self.conn.lock().await;
        let prefix_end = kind_prefix_upper_bound(prefix);
        let mut stmt = if prefix_end.is_some() {
            conn.prepare(
                "SELECT id, session_id, sequence, timestamp, writer_package_id, kind, schema_version, payload_json, metadata_json
                 FROM events WHERE session_id = ?1 AND kind >= ?2 AND kind < ?3
                 ORDER BY sequence ASC",
            )?
        } else {
            conn.prepare(
                "SELECT id, session_id, sequence, timestamp, writer_package_id, kind, schema_version, payload_json, metadata_json
                 FROM events WHERE session_id = ?1 AND kind LIKE ?2
                 ORDER BY sequence ASC",
            )?
        };
        let rows = if let Some(end) = prefix_end {
            stmt.query_map(params![session_id, prefix, end], row_to_event)?
        } else {
            stmt.query_map(params![session_id, format!("{}%", prefix)], row_to_event)?
        };
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
}

/// Compute an upper-bound string for a kind prefix so that a range
/// query `kind >= prefix AND kind < upper` is equivalent to a prefix
/// match without the `%` wild-card cost of LIKE.
///
/// Returns `None` if the upper bound cannot be computed (e.g. prefix
/// consists entirely of maximal chars `\xff`).
fn kind_prefix_upper_bound(prefix: &str) -> Option<String> {
    let mut bytes: Vec<u8> = prefix.as_bytes().to_vec();
    loop {
        match bytes.last_mut() {
            Some(b) if *b < 0xFF => {
                *b += 1;
                return Some(String::from_utf8(bytes).unwrap_or_default());
            }
            Some(_) => {
                bytes.pop();
            }
            None => return None,
        }
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

    #[tokio::test]
    async fn sqlite_concurrent_append_no_duplicate_sequences() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(format!("ygg-test-concurrent-{}.db", new_id("sqlite")));
        let store = SqliteEventStore::open(&path)?;
        let session_id = "ses_concurrent".to_string();

        // Open the session by appending a session.opened event
        store
            .append_with_sequence(
                session_id.clone(),
                KERNEL_PACKAGE_ID.to_string(),
                "kernel/session.opened".to_string(),
                1,
                json!({}),
                json!({}),
            )
            .await?;

        let n = 50;
        let mut handles = Vec::new();
        for i in 0..n {
            let s = store.clone();
            let sid = session_id.clone();
            handles.push(tokio::spawn(async move {
                s.append_with_sequence(
                    sid,
                    KERNEL_PACKAGE_ID.to_string(),
                    format!("kernel/test.concurrent.{}", i),
                    1,
                    json!({"i": i}),
                    json!({}),
                )
                .await
            }));
        }

        for h in handles {
            let _ = h.await.unwrap()?;
        }

        let events = store.list_session(&session_id).await?;
        let mut sequences: Vec<u64> = events.iter().map(|e| e.sequence).collect();
        sequences.sort();
        // No duplicates
        let dedup: Vec<u64> = sequences.iter().copied().collect::<std::collections::HashSet<_>>().into_iter().collect();
        assert_eq!(dedup.len(), sequences.len(), "duplicate sequences found: {:?}", sequences);
        // Contiguous from 0
        for (i, seq) in sequences.iter().enumerate() {
            assert_eq!(*seq, i as u64, "non-contiguous sequence at index {}: {}", i, seq);
        }

        let _ = std::fs::remove_file(path);
        Ok(())
    }

    #[tokio::test]
    async fn sqlite_kind_prefix_query_uses_pushdown() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(format!("ygg-test-prefix-{}.db", new_id("sqlite")));
        let store = SqliteEventStore::open(&path)?;
        let session_id = "ses_prefix".to_string();

        store
            .append_with_sequence(
                session_id.clone(),
                KERNEL_PACKAGE_ID.to_string(),
                "kernel/permission.granted".to_string(),
                1,
                json!({}),
                json!({}),
            )
            .await?;
        store
            .append_with_sequence(
                session_id.clone(),
                KERNEL_PACKAGE_ID.to_string(),
                "kernel/permission.denied".to_string(),
                1,
                json!({}),
                json!({}),
            )
            .await?;
        store
            .append_with_sequence(
                session_id.clone(),
                KERNEL_PACKAGE_ID.to_string(),
                "kernel/session.opened".to_string(),
                1,
                json!({}),
                json!({}),
            )
            .await?;

        let perm_events = store.list_kind_prefix("kernel/permission").await?;
        assert_eq!(perm_events.len(), 2);

        let session_perm = store.list_session_kind_prefix(&session_id, "kernel/permission").await?;
        assert_eq!(session_perm.len(), 2);

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

    /// In-memory atomic append: allocate sequence and push within the
    /// same write lock, guaranteeing no duplicate sequences under
    /// concurrent access.
    async fn append_with_sequence(
        &self,
        session_id: SessionId,
        writer_package_id: PackageId,
        kind: EventKind,
        schema_version: u16,
        payload_json: serde_json::Value,
        metadata_json: serde_json::Value,
    ) -> anyhow::Result<EventEnvelope> {
        let mut map = self.events.write().await;
        let seq = map.get(&session_id).map(|v| v.len() as EventSequence).unwrap_or(0);
        let event = EventEnvelope {
            id: ygg_core::new_id("evt"),
            session_id,
            sequence: seq,
            timestamp: chrono::Utc::now(),
            writer_package_id,
            kind,
            schema_version,
            payload: payload_json,
            metadata: metadata_json,
        };
        map.entry(event.session_id.clone())
            .or_default()
            .push(event.clone());
        let _ = self.tx.send(event.clone());
        Ok(event)
    }

    /// In-memory pushdown: filter by kind prefix within a single
    /// session read, avoiding full list_all().
    async fn list_session_kind_prefix(
        &self,
        session_id: &SessionId,
        prefix: &str,
    ) -> anyhow::Result<Vec<EventEnvelope>> {
        let events = self
            .events
            .read()
            .await
            .get(session_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|e| e.kind.starts_with(prefix))
            .collect();
        Ok(events)
    }
}
