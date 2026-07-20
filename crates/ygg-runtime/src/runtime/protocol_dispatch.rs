use serde_json::{json, Value};

use super::Runtime;
use crate::{
    negotiate_contract, resolve_contract_method, ContractSelection, EventStore, KernelMethod,
    ProtocolContext, ProtocolPrincipal,
};

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
        self.call_protocol_negotiated(context, method, params, None)
            .await
    }

    pub async fn call_protocol_negotiated(
        &self,
        context: &ProtocolContext,
        method: &str,
        params: Value,
        selection: Option<&ContractSelection>,
    ) -> Result<Value, crate::ProtocolError> {
        negotiate_contract(selection)?;
        let resolved = resolve_contract_method(method).map_err(|_| {
            crate::ProtocolError::invalid_request(format!(
                "protocol method '{}' is not a known contract method",
                method
            ))
        })?;
        let params = resolved.adapt_request(params)?;
        let result = self
            .dispatch_protocol_method(context, resolved.method, params)
            .await
            .map_err(crate::ProtocolError::from_anyhow)?;
        resolved.adapt_response(result)
    }

    pub async fn call_subprocess_protocol(
        &self,
        context: &ProtocolContext,
        method: &str,
        params: Value,
    ) -> Result<Value, crate::ProtocolError> {
        self.call_subprocess_protocol_negotiated(context, method, params, None)
            .await
    }

    pub async fn call_subprocess_protocol_negotiated(
        &self,
        context: &ProtocolContext,
        method: &str,
        params: Value,
        selection: Option<&ContractSelection>,
    ) -> Result<Value, crate::ProtocolError> {
        negotiate_contract(selection)?;
        let resolved = resolve_contract_method(method).map_err(|_| {
            crate::ProtocolError::invalid_request(format!(
                "protocol method '{}' is not a known contract method",
                method
            ))
        })?;
        let kernel_method = resolved.method;
        let params = resolved.adapt_request(params)?;
        if is_deployment_hub_method(kernel_method) {
            ensure_deployment_hub_control_allowed(context)
                .map_err(crate::ProtocolError::from_anyhow)?;
        }
        let result: anyhow::Result<Value> = match kernel_method {
            KernelMethod::OutboundExecute => self.dispatch_outbound_execute(context, params).await,
            KernelMethod::OutboundStream => self.dispatch_outbound_stream(context, params).await,
            KernelMethod::OutboundWebSocketOpen => {
                self.dispatch_outbound_websocket_open(context, params).await
            }
            KernelMethod::OutboundWebSocketSend => {
                self.dispatch_outbound_websocket_send(&params).await
            }
            KernelMethod::OutboundWebSocketClose => {
                self.dispatch_outbound_websocket_close(&params).await
            }
            KernelMethod::TargetList => self.dispatch_target_list().await,
            KernelMethod::TargetStatus => self.dispatch_target_status(&params).await,
            KernelMethod::TargetRegister => self.dispatch_target_register(params).await,
            KernelMethod::TargetUnregister => self.dispatch_target_unregister(&params).await,
            KernelMethod::ExecStart => self.dispatch_exec_start(context, params).await,
            KernelMethod::ExecStop => self.dispatch_exec_stop(context, params).await,
            KernelMethod::ExecStatus => self.dispatch_exec_status(params).await,
            KernelMethod::ExecLogs => self.dispatch_exec_logs(params).await,
            KernelMethod::ExecList => self.dispatch_exec_list().await,
            KernelMethod::PortLease => self.dispatch_port_lease(context, params).await,
            KernelMethod::PortRelease => self.dispatch_port_release(context, &params).await,
            KernelMethod::PortStatus => self.dispatch_port_status(&params).await,
            KernelMethod::PortList => self.dispatch_port_list().await,
            KernelMethod::ProxyRegister => self.dispatch_proxy_register(context, params).await,
            KernelMethod::ProxyUnregister => self.dispatch_proxy_unregister(context, &params).await,
            KernelMethod::ProxyStatus => self.dispatch_proxy_status(&params).await,
            KernelMethod::ProxyList => self.dispatch_proxy_list().await,
            KernelMethod::CapabilityCancel => self.dispatch_capability_cancel(&params).await,
            KernelMethod::HostInfo => {
                serde_json::to_value(crate::host_info()).map_err(anyhow::Error::from)
            }
            KernelMethod::HostPing => Ok(json!({"ok": true})),
            KernelMethod::HostDiagnostics => Ok(self.host_diagnostics().await),
            KernelMethod::CapabilityDiscover => {
                serde_json::to_value(self.discover_capabilities().await)
                    .map_err(anyhow::Error::from)
            }
            KernelMethod::CapabilityInvoke => match serde_json::from_value(params) {
                Ok(request) => self
                    .invoke_capability_with_context(context, request)
                    .await
                    .and_then(|result| serde_json::to_value(result).map_err(anyhow::Error::from)),
                Err(error) => Err(anyhow::Error::from(error)),
            },
            other => Err(anyhow::anyhow!(
                "protocol method '{}' is not available over subprocess reverse stdio yet",
                other
            )),
        };
        let result = result.map_err(crate::ProtocolError::from_anyhow)?;
        resolved.adapt_response(result)
    }

    pub(crate) async fn dispatch_protocol_method(
        &self,
        context: &ProtocolContext,
        kernel_method: KernelMethod,
        params: Value,
    ) -> anyhow::Result<Value> {
        if is_deployment_hub_method(kernel_method) {
            ensure_deployment_hub_control_allowed(context)?;
        }
        match kernel_method {
            // Host domain
            KernelMethod::HostInfo => Ok(serde_json::to_value(crate::host_info())?),
            KernelMethod::HostPing => Ok(json!({"ok": true})),
            KernelMethod::HostDiagnostics => Ok(self.host_diagnostics().await),

            // Surface domain
            KernelMethod::SurfaceResolveBundle => {
                self.dispatch_surface_resolve_bundle(context, &params).await
            }
            KernelMethod::SurfaceContributionList => self.dispatch_surface_list(&params).await,
            KernelMethod::SurfaceContributionDescribe => {
                self.dispatch_surface_describe(&params).await
            }

            // Outbound domain
            KernelMethod::OutboundAudit => self.dispatch_outbound_audit(&params).await,
            KernelMethod::OutboundExecute => self.dispatch_outbound_execute(context, params).await,
            KernelMethod::OutboundStream => self.dispatch_outbound_stream(context, params).await,
            KernelMethod::OutboundWebSocketOpen => {
                self.dispatch_outbound_websocket_open(context, params).await
            }
            KernelMethod::OutboundWebSocketSend => {
                self.dispatch_outbound_websocket_send(&params).await
            }
            KernelMethod::OutboundWebSocketClose => {
                self.dispatch_outbound_websocket_close(&params).await
            }

            // Permission domain
            KernelMethod::PermissionGrant => self.dispatch_permission_grant(&params).await,
            KernelMethod::PermissionRevoke => self.dispatch_permission_revoke(&params).await,
            KernelMethod::PermissionList => self.dispatch_permission_list(&params).await,
            KernelMethod::PermissionAudit => self.dispatch_permission_audit().await,

            // Audit domain
            KernelMethod::AuditPackage => self.dispatch_audit_package(&params).await,

            // Proposal domain
            KernelMethod::ProposalCreate => self.dispatch_proposal_create(context, &params).await,
            KernelMethod::ProposalGet => self.dispatch_proposal_get(&params).await,
            KernelMethod::ProposalList => self.dispatch_proposal_list().await,
            KernelMethod::ProposalApprove => self.dispatch_proposal_approve(context, &params).await,
            KernelMethod::ProposalReject => self.dispatch_proposal_reject(context, &params).await,
            KernelMethod::ProposalApply => self.dispatch_proposal_apply(&params).await,

            // Session domain
            KernelMethod::SessionOpen => Ok(serde_json::to_value(
                self.open_session(serde_json::from_value(params)?).await?,
            )?),
            KernelMethod::SessionClose => self.dispatch_session_close(&params).await,
            KernelMethod::SessionFork => self.dispatch_session_fork(&params).await,
            KernelMethod::SessionBranchList => self.dispatch_session_branch_list(&params).await,
            KernelMethod::SessionGet => self.dispatch_session_get(&params).await,

            // Event domain
            KernelMethod::EventAppend => Ok(serde_json::to_value(
                self.append_event_with_context(context, serde_json::from_value(params)?)
                    .await?,
            )?),
            KernelMethod::EventList => self.dispatch_event_list(context, &params).await,

            // Package domain
            KernelMethod::PackageLoad => Ok(serde_json::to_value(
                self.load_package(serde_json::from_value(params)?).await?,
            )?),
            KernelMethod::PackageList => Ok(serde_json::to_value(self.list_packages().await)?),
            KernelMethod::PackageStatus => self.dispatch_package_status(&params).await,
            KernelMethod::PackageUnload => self.dispatch_package_unload(&params).await,
            KernelMethod::PackageRestart => self.dispatch_package_restart(&params).await,
            KernelMethod::PackageLogs => self.dispatch_package_logs(&params).await,

            // Project domain
            KernelMethod::ProjectList => self.dispatch_project_list(context, &params).await,
            KernelMethod::ProjectGet => self.dispatch_project_get(context, &params).await,
            KernelMethod::ProjectStart => self.dispatch_project_start(context, &params).await,
            KernelMethod::ProjectStop => self.dispatch_project_stop(context, &params).await,
            KernelMethod::ProjectStatus => self.dispatch_project_status(context, &params).await,

            // Deployment Hub Phase 1 primitives
            KernelMethod::TargetList => self.dispatch_target_list().await,
            KernelMethod::TargetStatus => self.dispatch_target_status(&params).await,
            KernelMethod::TargetRegister => self.dispatch_target_register(params).await,
            KernelMethod::TargetUnregister => self.dispatch_target_unregister(&params).await,
            KernelMethod::ExecStart => self.dispatch_exec_start(context, params).await,
            KernelMethod::ExecStop => self.dispatch_exec_stop(context, params).await,
            KernelMethod::ExecStatus => self.dispatch_exec_status(params).await,
            KernelMethod::ExecLogs => self.dispatch_exec_logs(params).await,
            KernelMethod::ExecList => self.dispatch_exec_list().await,
            KernelMethod::PortLease => self.dispatch_port_lease(context, params).await,
            KernelMethod::PortRelease => self.dispatch_port_release(context, &params).await,
            KernelMethod::PortStatus => self.dispatch_port_status(&params).await,
            KernelMethod::PortList => self.dispatch_port_list().await,
            KernelMethod::ProxyRegister => self.dispatch_proxy_register(context, params).await,
            KernelMethod::ProxyUnregister => self.dispatch_proxy_unregister(context, &params).await,
            KernelMethod::ProxyStatus => self.dispatch_proxy_status(&params).await,
            KernelMethod::ProxyList => self.dispatch_proxy_list().await,

            // Capability domain
            KernelMethod::CapabilityDiscover => {
                Ok(serde_json::to_value(self.discover_capabilities().await)?)
            }
            KernelMethod::CapabilityInvoke => Ok(serde_json::to_value(
                self.invoke_capability_with_context(context, serde_json::from_value(params)?)
                    .await?,
            )?),
            KernelMethod::CapabilityHandleAttenuate => self.dispatch_cap_attenuate(&params).await,
            KernelMethod::CapabilityHandleRevoke => self.dispatch_cap_revoke(&params).await,
            KernelMethod::CapabilityHandleListFor => self.dispatch_cap_list_for(&params).await,
            KernelMethod::CapabilityStream => self.dispatch_capability_stream(&params).await,
            KernelMethod::CapabilityCancel => self.dispatch_capability_cancel(&params).await,

            // Extension / hook domain
            KernelMethod::ExtensionPointList => Ok(json!([
                "kernel/v1/event.before_append",
                "kernel/v1/event.after_append",
                "kernel/v1/capability.before_invoke",
                "kernel/v1/capability.after_invoke",
                "kernel/v1/package.loaded",
                "kernel/v1/package.unloaded"
            ])),
            KernelMethod::HookList => Ok(serde_json::to_value(
                self.extensions.list_all_hooks().await,
            )?),

            // Asset domain
            KernelMethod::AssetPut => Ok(serde_json::to_value(
                self.put_asset(serde_json::from_value(params)?).await?,
            )?),
            KernelMethod::AssetGet => self.dispatch_asset_get(&params).await,
            KernelMethod::AssetList => Ok(serde_json::to_value(self.list_assets().await)?),

            // Projection domain
            KernelMethod::ProjectionRegister => Ok(serde_json::to_value(
                self.projection_register(serde_json::from_value(params)?)
                    .await?,
            )?),
            KernelMethod::ProjectionRebuild => self.dispatch_projection_rebuild(&params).await,
            KernelMethod::ProjectionGet => self.dispatch_projection_get(&params).await,
            KernelMethod::ProjectionList => Ok(serde_json::to_value(self.projection_list().await)?),

            // Planned methods — no dispatch yet
            KernelMethod::SessionList
            | KernelMethod::EventSubscribe
            | KernelMethod::PackageDescribe
            | KernelMethod::CapabilityDescribe
            | KernelMethod::ExtensionPointDescribe
            | KernelMethod::HostPrincipal => {
                anyhow::bail!("protocol method '{}' is not yet implemented", kernel_method)
            }
        }
    }
}

fn is_deployment_hub_method(method: KernelMethod) -> bool {
    matches!(
        method,
        KernelMethod::TargetList
            | KernelMethod::TargetStatus
            | KernelMethod::TargetRegister
            | KernelMethod::TargetUnregister
            | KernelMethod::ExecStart
            | KernelMethod::ExecStop
            | KernelMethod::ExecStatus
            | KernelMethod::ExecLogs
            | KernelMethod::ExecList
            | KernelMethod::PortLease
            | KernelMethod::PortRelease
            | KernelMethod::PortStatus
            | KernelMethod::PortList
            | KernelMethod::ProxyRegister
            | KernelMethod::ProxyUnregister
            | KernelMethod::ProxyStatus
            | KernelMethod::ProxyList
    )
}

fn ensure_deployment_hub_control_allowed(context: &ProtocolContext) -> anyhow::Result<()> {
    match &context.principal {
        ProtocolPrincipal::HostAdmin | ProtocolPrincipal::HostDev => Ok(()),
        _ => anyhow::bail!(
            "permission denied: deployment hub control methods require host_admin or host_dev principal"
        ),
    }
}
