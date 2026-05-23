use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use schemars::JsonSchema;
use ygg_core::{
    new_id, SessionId, KERNEL_PACKAGE_ID,
    EVENT_PROPOSAL_APPLIED, EVENT_PROPOSAL_APPROVED, EVENT_PROPOSAL_CREATED, EVENT_PROPOSAL_FAILED,
    EVENT_PROPOSAL_REJECTED,
};

use super::Runtime;
use crate::{EventStore, ProtocolContext, ProtocolPrincipal, redaction};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Created,
    Approved,
    Rejected,
    Applied,
    Failed,
}

impl Default for ProposalStatus {
    fn default() -> Self {
        Self::Created
    }
}

fn anonymous_principal() -> ProtocolPrincipal {
    ProtocolPrincipal::Anonymous
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProposalRecord {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub status: ProposalStatus,
    #[serde(default = "anonymous_principal")]
    pub created_by: ProtocolPrincipal,
    #[serde(default = "Utc::now")]
    pub created_at: chrono::DateTime<Utc>,
    #[serde(default)]
    pub target_session_id: Option<SessionId>,
    #[serde(default)]
    pub target_branch_id: Option<String>,
    #[serde(default)]
    pub operations: Vec<ProposalOperation>,
    #[serde(default)]
    pub required_permissions: Vec<String>,
    #[serde(default)]
    pub expected_effects: Value,
    #[serde(default)]
    pub approval: Option<ProposalApproval>,
    #[serde(default)]
    pub result: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProposalOperation {
    pub op: String,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProposalApproval {
    pub principal: ProtocolPrincipal,
    pub decided_at: chrono::DateTime<Utc>,
    #[serde(default)]
    pub reason: Option<String>,
}

impl<S> Runtime<S>
where
    S: EventStore,
{
    pub async fn create_proposal(&self, context: &ProtocolContext, mut proposal: ProposalRecord) -> anyhow::Result<ProposalRecord> {
        // Scan for raw secrets in proposal operations and expected_effects
        let proposal_value = serde_json::to_value(&proposal)?;
        let scan = redaction::scan_value_for_raw_secrets(&proposal_value, "");
        if scan.has_findings() {
            let findings: Vec<String> = scan.findings.iter()
                .map(|f| format!("{} ({:?})", f.path, f.detection))
                .collect();
            anyhow::bail!(
                "proposal contains raw secret(s) in field(s): {}; use secret_ref references instead",
                findings.join(", ")
            );
        }

        proposal.id = if proposal.id.trim().is_empty() { new_id("prp") } else { proposal.id };
        proposal.status = ProposalStatus::Created;
        proposal.created_by = context.principal.clone();
        proposal.created_at = Utc::now();
        self.proposals.write().await.insert(proposal.id.clone(), proposal.clone());
        self.append_kernel_event(&format!("kernel_proposal_{}", proposal.id), EVENT_PROPOSAL_CREATED, serde_json::to_value(&proposal)?).await?;
        Ok(proposal)
    }

    pub async fn get_proposal(&self, proposal_id: &str) -> anyhow::Result<ProposalRecord> {
        self.proposals.read().await.get(proposal_id).cloned().ok_or_else(|| anyhow::anyhow!("proposal '{proposal_id}' not found"))
    }

    pub async fn list_proposals(&self) -> Vec<ProposalRecord> {
        let mut proposals: Vec<_> = self.proposals.read().await.values().cloned().collect();
        proposals.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        proposals
    }

    pub async fn approve_proposal(&self, context: &ProtocolContext, proposal_id: &str, reason: Option<String>) -> anyhow::Result<ProposalRecord> {
        let mut proposals = self.proposals.write().await;
        let proposal = proposals.get_mut(proposal_id).ok_or_else(|| anyhow::anyhow!("proposal '{proposal_id}' not found"))?;
        if proposal.status != ProposalStatus::Created {
            anyhow::bail!("proposal '{proposal_id}' is not awaiting approval");
        }
        proposal.status = ProposalStatus::Approved;
        proposal.approval = Some(ProposalApproval { principal: context.principal.clone(), decided_at: Utc::now(), reason });
        let proposal = proposal.clone();
        drop(proposals);
        self.append_kernel_event(&format!("kernel_proposal_{}", proposal.id), EVENT_PROPOSAL_APPROVED, serde_json::to_value(&proposal)?).await?;
        Ok(proposal)
    }

    pub async fn reject_proposal(&self, context: &ProtocolContext, proposal_id: &str, reason: Option<String>) -> anyhow::Result<ProposalRecord> {
        let mut proposals = self.proposals.write().await;
        let proposal = proposals.get_mut(proposal_id).ok_or_else(|| anyhow::anyhow!("proposal '{proposal_id}' not found"))?;
        if proposal.status != ProposalStatus::Created {
            anyhow::bail!("proposal '{proposal_id}' is not awaiting review");
        }
        proposal.status = ProposalStatus::Rejected;
        proposal.approval = Some(ProposalApproval { principal: context.principal.clone(), decided_at: Utc::now(), reason });
        let proposal = proposal.clone();
        drop(proposals);
        self.append_kernel_event(&format!("kernel_proposal_{}", proposal.id), EVENT_PROPOSAL_REJECTED, serde_json::to_value(&proposal)?).await?;
        Ok(proposal)
    }

    pub async fn apply_proposal(&self, proposal_id: &str) -> anyhow::Result<ProposalRecord> {
        let mut proposal = self.get_proposal(proposal_id).await?;
        if proposal.status != ProposalStatus::Approved {
            anyhow::bail!("proposal '{proposal_id}' must be approved before apply");
        }
        let mut results = Vec::new();
        for operation in &proposal.operations {
            match operation.op.as_str() {
                "asset.put" => {
                    let content = operation.payload.get("content").and_then(Value::as_str).unwrap_or("{}").to_string();
                    let mime = operation.payload.get("mime").and_then(Value::as_str).unwrap_or("application/json").to_string();
                    let asset = self.put_asset(super::AssetPutRequest { origin_package_id: Some(KERNEL_PACKAGE_ID.to_string()), mime, content, metadata: json!({"proposal_id": proposal.id}) }).await?;
                    results.push(json!({"op": operation.op, "asset_id": asset.id}));
                }
                "projection.rebuild" => {
                    let projection_id = operation.target.as_deref().ok_or_else(|| anyhow::anyhow!("projection.rebuild operation requires target"))?;
                    let projection = self.projection_rebuild(projection_id).await?;
                    results.push(json!({"op": operation.op, "projection_id": projection.id}));
                }
                other => anyhow::bail!("unsupported proposal operation '{other}'"),
            }
        }
        proposal.status = ProposalStatus::Applied;
        proposal.result = Some(json!({"operations": results}));
        self.proposals.write().await.insert(proposal.id.clone(), proposal.clone());
        self.append_kernel_event(&format!("kernel_proposal_{}", proposal.id), EVENT_PROPOSAL_APPLIED, serde_json::to_value(&proposal)?).await?;
        Ok(proposal)
    }

    pub async fn fail_proposal(&self, proposal_id: &str, error: String) -> anyhow::Result<ProposalRecord> {
        let mut proposals = self.proposals.write().await;
        let proposal = proposals.get_mut(proposal_id).ok_or_else(|| anyhow::anyhow!("proposal '{proposal_id}' not found"))?;
        proposal.status = ProposalStatus::Failed;
        proposal.result = Some(json!({"error": error}));
        let proposal = proposal.clone();
        drop(proposals);
        self.append_kernel_event(&format!("kernel_proposal_{}", proposal.id), EVENT_PROPOSAL_FAILED, serde_json::to_value(&proposal)?).await?;
        Ok(proposal)
    }
}
