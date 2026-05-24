use super::*;

impl<S> Runtime<S>
where
    S: EventStore,
{
    // --- Session ---

    pub(crate) async fn dispatch_session_close(&self, params: &Value) -> anyhow::Result<Value> {
        let session_id = params
            .get("session_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.session.close requires session_id"))?
            .to_string();
        Ok(serde_json::to_value(self.close_session(session_id).await?)?)
    }

    pub(crate) async fn dispatch_session_get(&self, params: &Value) -> anyhow::Result<Value> {
        let session_id = params
            .get("session_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.session.get requires session_id"))?;
        Ok(serde_json::to_value(
            self.get_session(session_id)
                .await
                .ok_or_else(|| anyhow::anyhow!("session '{session_id}' not found"))?,
        )?)
    }

    pub(crate) async fn dispatch_session_fork(&self, params: &Value) -> anyhow::Result<Value> {
        let parent_session_id = params
            .get("parent_session_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.session.fork requires parent_session_id"))?
            .to_string();
        let forked_from_sequence = params
            .get("forked_from_sequence")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                anyhow::anyhow!("kernel.v1.session.fork requires forked_from_sequence")
            })?;
        let metadata = params.get("metadata").cloned().unwrap_or_else(|| json!({}));
        Ok(serde_json::to_value(
            self.fork_session(parent_session_id, forked_from_sequence, metadata)
                .await?,
        )?)
    }

    pub(crate) async fn dispatch_session_branch_list(&self, params: &Value) -> anyhow::Result<Value> {
        let session_id = params
            .get("session_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.session.branch.list requires session_id"))?
            .to_string();
        Ok(serde_json::to_value(self.list_branches(&session_id).await)?)
    }

    // --- Event ---

    pub(crate) async fn dispatch_event_list(
        &self,
        context: &ProtocolContext,
        params: &Value,
    ) -> anyhow::Result<Value> {
        let request: EventListRequest = serde_json::from_value(params.clone())?;
        Ok(serde_json::to_value(
            self.list_events_range_with_context(context, &request)
                .await?,
        )?)
    }
}
