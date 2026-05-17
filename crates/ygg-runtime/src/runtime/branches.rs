use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use ygg_core::{new_id, EventSequence, SessionId, EVENT_SESSION_FORKED};

use super::Runtime;
use crate::EventStore;

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

impl<S> Runtime<S>
where
    S: EventStore,
{
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
            .open_session(super::OpenSessionRequest {
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
}
