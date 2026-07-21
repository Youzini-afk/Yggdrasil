use super::*;

impl<S> Runtime<S>
where
    S: EventStore,
{
    // --- Proposal ---

    pub(crate) async fn dispatch_proposal_create(
        &self,
        context: &ProtocolContext,
        params: &Value,
    ) -> anyhow::Result<Value> {
        let proposal: crate::runtime::ProposalRecord = serde_json::from_value(params.clone())?;
        Ok(serde_json::to_value(
            self.create_proposal(context, proposal).await?,
        )?)
    }

    pub(crate) async fn dispatch_proposal_get(&self, params: &Value) -> anyhow::Result<Value> {
        let proposal_id = params
            .get("proposal_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.proposal.get requires proposal_id"))?;
        Ok(serde_json::to_value(self.get_proposal(proposal_id).await?)?)
    }

    pub(crate) async fn dispatch_proposal_list(&self) -> anyhow::Result<Value> {
        Ok(serde_json::to_value(self.list_proposals().await)?)
    }

    pub(crate) async fn dispatch_proposal_approve(
        &self,
        context: &ProtocolContext,
        params: &Value,
    ) -> anyhow::Result<Value> {
        let proposal_id = params
            .get("proposal_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.proposal.approve requires proposal_id"))?;
        let reason = params
            .get("reason")
            .and_then(Value::as_str)
            .map(str::to_string);
        Ok(serde_json::to_value(
            self.approve_proposal(context, proposal_id, reason).await?,
        )?)
    }

    pub(crate) async fn dispatch_proposal_reject(
        &self,
        context: &ProtocolContext,
        params: &Value,
    ) -> anyhow::Result<Value> {
        let proposal_id = params
            .get("proposal_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.proposal.reject requires proposal_id"))?;
        let reason = params
            .get("reason")
            .and_then(Value::as_str)
            .map(str::to_string);
        Ok(serde_json::to_value(
            self.reject_proposal(context, proposal_id, reason).await?,
        )?)
    }

    pub(crate) async fn dispatch_proposal_apply(
        &self,
        context: &ProtocolContext,
        params: &Value,
    ) -> anyhow::Result<Value> {
        let proposal_id = params
            .get("proposal_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.proposal.apply requires proposal_id"))?;
        Ok(serde_json::to_value(
            self.apply_proposal(context, proposal_id).await?,
        )?)
    }
}
