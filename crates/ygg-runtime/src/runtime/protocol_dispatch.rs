use serde_json::{json, Value};

use super::Runtime;
use crate::{EventStore, KernelMethod, ProtocolContext, EventListRequest};

impl<S> Runtime<S>
where
    S: EventStore,
{
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

    pub(crate) async fn call_protocol_inner(&self, context: &ProtocolContext, method: &str, params: Value) -> anyhow::Result<Value> {
        let kernel_method: KernelMethod = method.parse().map_err(|_| {
            anyhow::anyhow!("protocol method '{}' is not a known kernel method", method)
        })?;
        match kernel_method {
            KernelMethod::HostInfo => Ok(serde_json::to_value(crate::host_info())?),
            KernelMethod::HostPing => Ok(json!({"ok": true})),
            KernelMethod::HostDiagnostics => Ok(self.host_diagnostics().await),
            KernelMethod::SurfaceContributionList => {
                let slot = params.get("slot").and_then(Value::as_str).map(str::to_string);
                Ok(self.list_surface_contributions(slot).await)
            }
            KernelMethod::SurfaceContributionDescribe => {
                let surface_id = params
                    .get("surface_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.surface.contribution.describe requires surface_id"))?;
                self.describe_surface_contribution(surface_id).await
            }
            KernelMethod::OutboundAudit => {
                let package_id = params
                    .get("package_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.outbound.audit requires package_id"))?
                    .to_string();
                Ok(serde_json::to_value(self.list_outbound_audit(&package_id).await?)?)
            }
            KernelMethod::PermissionGrant => {
                let principal = params
                    .get("principal")
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("kernel.permission.grant requires principal"))?;
                let principal: crate::ProtocolPrincipal = serde_json::from_value(principal)?;
                let permission = params
                    .get("permission")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.permission.grant requires permission"))?
                    .to_string();
                let scope = params.get("scope").and_then(Value::as_str).map(str::to_string);
                let reason = params.get("reason").and_then(Value::as_str).map(str::to_string);
                Ok(serde_json::to_value(self.grant_permission(principal, permission, scope, reason).await?)?)
            }
            KernelMethod::PermissionRevoke => {
                let grant_id = params
                    .get("grant_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.permission.revoke requires grant_id"))?;
                Ok(serde_json::to_value(self.revoke_permission(grant_id).await?)?)
            }
            KernelMethod::PermissionList => {
                let principal = match params.get("principal") {
                    Some(value) => Some(serde_json::from_value(value.clone())?),
                    None => None,
                };
                Ok(serde_json::to_value(self.list_permission_grants(principal).await)?)
            }
            KernelMethod::PermissionAudit => {
                let events: Vec<_> = self
                    .store
                    .list_all()
                    .await?
                    .into_iter()
                    .filter(|event| event.kind.starts_with("kernel/permission"))
                    .collect();
                Ok(serde_json::to_value(events)?)
            }
            KernelMethod::ProposalCreate => {
                let proposal: super::ProposalRecord = serde_json::from_value(params)?;
                Ok(serde_json::to_value(self.create_proposal(context, proposal).await?)?)
            }
            KernelMethod::ProposalGet => {
                let proposal_id = params
                    .get("proposal_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.proposal.get requires proposal_id"))?;
                Ok(serde_json::to_value(self.get_proposal(proposal_id).await?)?)
            }
            KernelMethod::ProposalList => Ok(serde_json::to_value(self.list_proposals().await)?),
            KernelMethod::ProposalApprove => {
                let proposal_id = params
                    .get("proposal_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.proposal.approve requires proposal_id"))?;
                let reason = params.get("reason").and_then(Value::as_str).map(str::to_string);
                Ok(serde_json::to_value(self.approve_proposal(context, proposal_id, reason).await?)?)
            }
            KernelMethod::ProposalReject => {
                let proposal_id = params
                    .get("proposal_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.proposal.reject requires proposal_id"))?;
                let reason = params.get("reason").and_then(Value::as_str).map(str::to_string);
                Ok(serde_json::to_value(self.reject_proposal(context, proposal_id, reason).await?)?)
            }
            KernelMethod::ProposalApply => {
                let proposal_id = params
                    .get("proposal_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.proposal.apply requires proposal_id"))?;
                Ok(serde_json::to_value(self.apply_proposal(proposal_id).await?)?)
            }
            KernelMethod::SessionOpen => Ok(serde_json::to_value(
                self.open_session(serde_json::from_value(params)?).await?,
            )?),
            KernelMethod::SessionClose => {
                let session_id = params
                    .get("session_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.session.close requires session_id"))?
                    .to_string();
                Ok(serde_json::to_value(self.close_session(session_id).await?)?)
            }
            KernelMethod::SessionFork => {
                let parent_session_id = params
                    .get("parent_session_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.session.fork requires parent_session_id"))?
                    .to_string();
                let forked_from_sequence = params
                    .get("forked_from_sequence")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| anyhow::anyhow!("kernel.session.fork requires forked_from_sequence"))?;
                let metadata = params.get("metadata").cloned().unwrap_or_else(|| json!({}));
                Ok(serde_json::to_value(self.fork_session(parent_session_id, forked_from_sequence, metadata).await?)?)
            }
            KernelMethod::SessionBranchList => {
                let session_id = params
                    .get("session_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.session.branch.list requires session_id"))?
                    .to_string();
                Ok(serde_json::to_value(self.list_branches(&session_id).await)?)
            }
            KernelMethod::EventAppend => Ok(serde_json::to_value(
                self.append_event_with_context(context, serde_json::from_value(params)?).await?,
            )?),
            KernelMethod::EventList => {
                let request: EventListRequest = serde_json::from_value(params)?;
                Ok(serde_json::to_value(self.list_events_range_with_context(context, &request).await?)?)
            }
            KernelMethod::PackageLoad => Ok(serde_json::to_value(
                self.load_package(serde_json::from_value(params)?).await?,
            )?),
            KernelMethod::PackageList => Ok(serde_json::to_value(self.list_packages().await)?),
            KernelMethod::PackageStatus => {
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
            KernelMethod::PackageUnload => {
                let package_id = params
                    .get("package_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.package.unload requires package_id"))?
                    .to_string();
                Ok(serde_json::to_value(self.unload_package(&package_id).await?)?)
            }
            KernelMethod::PackageRestart => {
                let package_id = params
                    .get("package_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.package.restart requires package_id"))?
                    .to_string();
                Ok(serde_json::to_value(self.restart_package(&package_id).await?)?)
            }
            KernelMethod::PackageLogs => {
                let package_id = params
                    .get("package_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.package.logs requires package_id"))?
                    .to_string();
                Ok(serde_json::to_value(self.package_logs(&package_id).await)?)
            }
            KernelMethod::CapabilityDiscover => Ok(serde_json::to_value(self.discover_capabilities().await)?),
            KernelMethod::CapabilityInvoke => Ok(serde_json::to_value(
                self.invoke_capability_with_context(context, serde_json::from_value(params)?).await?,
            )?),
            KernelMethod::ExtensionPointList => Ok(json!([
                "kernel/event.before_append",
                "kernel/event.after_append",
                "kernel/capability.before_invoke",
                "kernel/capability.after_invoke",
                "kernel/package.loaded",
                "kernel/package.unloaded"
            ])),
            KernelMethod::HookList => Ok(serde_json::to_value(self.extensions.list_all_hooks().await)?),
            KernelMethod::AssetPut => Ok(serde_json::to_value(self.put_asset(serde_json::from_value(params)?).await?)?),
            KernelMethod::AssetGet => {
                let asset_id = params
                    .get("asset_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.asset.get requires asset_id"))?;
                Ok(serde_json::to_value(self.get_asset(asset_id).await?)?)
            }
            KernelMethod::AssetList => Ok(serde_json::to_value(self.list_assets().await)?),
            KernelMethod::ProjectionRegister => Ok(serde_json::to_value(self.projection_register(serde_json::from_value(params)?).await?)?),
            KernelMethod::ProjectionRebuild => {
                let projection_id = params
                    .get("projection_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.projection.rebuild requires projection_id"))?;
                Ok(serde_json::to_value(self.projection_rebuild(projection_id).await?)?)
            }
            KernelMethod::ProjectionGet => {
                let projection_id = params
                    .get("projection_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("kernel.projection.get requires projection_id"))?;
                Ok(serde_json::to_value(self.projection_get(projection_id).await?)?)
            }
            KernelMethod::ProjectionList => Ok(serde_json::to_value(self.projection_list().await)?),
            // Planned methods — no dispatch yet
            KernelMethod::SessionGet
            | KernelMethod::SessionList
            | KernelMethod::EventSubscribe
            | KernelMethod::PackageDescribe
            | KernelMethod::CapabilityDescribe
            | KernelMethod::CapabilityStream
            | KernelMethod::CapabilityCancel
            | KernelMethod::ExtensionPointDescribe
            | KernelMethod::HostPrincipal => {
                anyhow::bail!("protocol method '{}' is not yet implemented", kernel_method)
            }
        }
    }
}
