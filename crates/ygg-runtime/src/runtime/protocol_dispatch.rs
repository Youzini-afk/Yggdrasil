use bytes::Bytes;
use reqwest::header::{HeaderName, HeaderValue};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::Instant;
use ygg_core::{
    project::{ProjectId, ProjectState},
    CapHandleId, PackageId, RedactionState,
};

use super::{OpenSessionRequest, Runtime};
use crate::{
    EventListRequest, EventStore, KernelMethod, OutboundFrameKind, OutboundStreamFrame,
    OutboundWebSocketFrame, ProtocolContext, ProtocolPrincipal, StreamEmitter, StreamRegistry,
    WebSocketEvent,
};

const WEBSOCKET_METHOD: &str = "WEBSOCKET";

#[derive(Debug, Serialize)]
struct ResolvedSurfaceBundle {
    surface_id: String,
    bundle_url: String,
    export_name: String,
    stylesheets: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wrapper_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_id: Option<String>,
    source: SurfaceBundleSource,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum SurfaceBundleSource {
    InstalledProject,
    DevPath,
}

fn surface_prefix_matches(surface_id: &str, prefix: &str) -> bool {
    surface_id == prefix
        || surface_id
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn default_surface_export_name(surface_id: &str) -> String {
    if surface_id.starts_with("ydltavern/") {
        match surface_id {
            "ydltavern/play" | "ydltavern/surface" => "mountTavernPlaySurface".to_string(),
            "ydltavern/settings" => "mountTavernSettingsSurface".to_string(),
            "ydltavern/extensions" => "mountTavernExtensionsSurface".to_string(),
            "ydltavern/character" => "mountTavernCharactersSurface".to_string(),
            "ydltavern/world-info" => "mountTavernWorldInfoSurface".to_string(),
            "ydltavern/persona" => "mountTavernPersonaSurface".to_string(),
            "ydltavern/ai-response-config" => "mountTavernAIResponseConfigSurface".to_string(),
            "ydltavern/user-settings" => "mountTavernUserSettingsSurface".to_string(),
            "ydltavern/backgrounds" => "mountTavernBackgroundsSurface".to_string(),
            _ => "mountTavernPlaySurface".to_string(),
        }
    } else {
        "mountSurface".to_string()
    }
}

fn default_surface_stylesheets(prefix: &str) -> Vec<String> {
    if prefix == "ydltavern" {
        vec![
            "/surface-bundles/ydltavern/styles/surface.css".to_string(),
            "/surface-bundles/ydltavern/styles/mobile.css".to_string(),
        ]
    } else {
        Vec::new()
    }
}

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

    pub async fn call_subprocess_protocol(
        &self,
        context: &ProtocolContext,
        method: &str,
        params: Value,
    ) -> Result<Value, crate::ProtocolError> {
        let kernel_method: KernelMethod = method.parse().map_err(|_| {
            crate::ProtocolError::invalid_request(format!(
                "protocol method '{}' is not a known kernel method",
                method
            ))
        })?;
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
        result.map_err(crate::ProtocolError::from_anyhow)
    }

    pub(crate) async fn call_protocol_inner(
        &self,
        context: &ProtocolContext,
        method: &str,
        params: Value,
    ) -> anyhow::Result<Value> {
        let kernel_method: KernelMethod = method.parse().map_err(|_| {
            anyhow::anyhow!("protocol method '{}' is not a known kernel method", method)
        })?;
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

// ---------------------------------------------------------------------------
// Domain dispatch helpers
// ---------------------------------------------------------------------------

impl<S> Runtime<S>
where
    S: EventStore,
{
    // --- Surface ---

    async fn dispatch_surface_resolve_bundle(
        &self,
        context: &ProtocolContext,
        params: &Value,
    ) -> anyhow::Result<Value> {
        if !matches!(
            context.principal,
            ProtocolPrincipal::HostAdmin | ProtocolPrincipal::HostDev
        ) {
            anyhow::bail!(
                "kernel.v1.surface.resolve_bundle permission denied: requires host admin/dev principal"
            );
        }

        let surface_id = params
            .get("surface_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("surface_id required"))?;

        for entry in self.config.project_registry.list() {
            if let Some(bundle) = self.try_resolve_via_project(&entry, surface_id)? {
                return Ok(serde_json::to_value(bundle)?);
            }
        }

        if let Some(bundle) = self.try_resolve_via_dev_path(surface_id)? {
            return Ok(serde_json::to_value(bundle)?);
        }

        anyhow::bail!("surface_not_found: {surface_id}")
    }

    fn try_resolve_via_project(
        &self,
        entry: &crate::ProjectEntry,
        surface_id: &str,
    ) -> anyhow::Result<Option<ResolvedSurfaceBundle>> {
        if entry.descriptor.project.entry_surface_id.as_deref() != Some(surface_id) {
            return Ok(None);
        }

        let project_id = entry.descriptor.project.id.as_str();
        Ok(Some(ResolvedSurfaceBundle {
            surface_id: surface_id.to_string(),
            bundle_url: format!("/surface-bundles/projects/{project_id}/bundle.mjs"),
            export_name: default_surface_export_name(surface_id),
            stylesheets: Vec::new(),
            wrapper_class: None,
            project_id: Some(project_id.to_string()),
            source: SurfaceBundleSource::InstalledProject,
        }))
    }

    fn try_resolve_via_dev_path(
        &self,
        surface_id: &str,
    ) -> anyhow::Result<Option<ResolvedSurfaceBundle>> {
        let Some((prefix, _path)) = self
            .config
            .surface_dev_paths
            .iter()
            .filter(|(prefix, _)| surface_prefix_matches(surface_id, prefix))
            .max_by_key(|(prefix, _)| prefix.len())
        else {
            return Ok(None);
        };

        Ok(Some(ResolvedSurfaceBundle {
            surface_id: surface_id.to_string(),
            bundle_url: format!("/surface-bundles/{prefix}/bundle.mjs"),
            export_name: default_surface_export_name(surface_id),
            stylesheets: default_surface_stylesheets(prefix),
            wrapper_class: Some(format!("{}-surface", prefix.replace(['/', '_'], "-"))),
            project_id: None,
            source: SurfaceBundleSource::DevPath,
        }))
    }

    async fn dispatch_surface_list(&self, params: &Value) -> anyhow::Result<Value> {
        let slot = params
            .get("slot")
            .and_then(Value::as_str)
            .map(str::to_string);
        Ok(self.list_surface_contributions(slot).await)
    }

    async fn dispatch_surface_describe(&self, params: &Value) -> anyhow::Result<Value> {
        let surface_id = params
            .get("surface_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow::anyhow!("kernel.v1.surface.contribution.describe requires surface_id")
            })?;
        self.describe_surface_contribution(surface_id).await
    }

    // --- Project ---

    fn ensure_project_admin(context: &ProtocolContext, method: &str) -> anyhow::Result<()> {
        if !matches!(
            context.principal,
            ProtocolPrincipal::HostAdmin | ProtocolPrincipal::HostDev
        ) {
            anyhow::bail!("{method} permission denied: requires host admin/dev principal");
        }
        Ok(())
    }

    fn project_id_param(params: &Value, method: &str) -> anyhow::Result<ProjectId> {
        let id = params
            .get("project_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("{method} requires project_id"))?;
        ProjectId::new(id)
    }

    fn project_summary(entry: &crate::ProjectEntry) -> anyhow::Result<Value> {
        let mut summary = json!({
            "id": entry.descriptor.project.id.as_str(),
            "title": entry.descriptor.project.title,
            "description": entry.descriptor.project.description,
            "type": serde_json::to_value(&entry.descriptor.project.project_type)?,
            "state": serde_json::to_value(entry.state)?,
            "icon": entry.descriptor.project.icon,
            "entry_surface_id": entry.descriptor.project.entry_surface_id,
        });
        if let Value::Object(map) = &mut summary {
            if let Some(session_id) = entry
                .descriptor
                .project
                .metadata
                .get("running_session_id")
                .and_then(Value::as_str)
            {
                map.insert("running_session_id".to_string(), json!(session_id));
            }
        }
        Ok(summary)
    }

    fn count_dir_entries(path: &std::path::Path) -> usize {
        fs::read_dir(path)
            .map(|entries| entries.filter_map(Result::ok).count())
            .unwrap_or(0)
    }

    fn project_paths_and_counts(id: &ProjectId) -> Value {
        let project_dir = ygg_core::paths::project_dir(id).ok();
        let sessions_count = project_dir
            .as_ref()
            .map(|dir| Self::count_dir_entries(&dir.join("sessions")))
            .unwrap_or(0);
        let secrets_path = ygg_core::paths::project_secret_store_path(id).ok();
        let secrets_exists = secrets_path.as_ref().is_some_and(|path| path.is_file());
        let secrets_count = if secrets_exists { 1 } else { 0 };
        json!({
            "project_dir": project_dir.map(|path| path.display().to_string()),
            "secrets_exists": secrets_exists,
            "sessions_count": sessions_count,
            "secrets_count": secrets_count,
        })
    }

    async fn dispatch_project_list(
        &self,
        context: &ProtocolContext,
        params: &Value,
    ) -> anyhow::Result<Value> {
        Self::ensure_project_admin(context, "kernel.v1.project.list")?;
        let filter_state = params
            .get("filter_state")
            .map(|value| serde_json::from_value::<ProjectState>(value.clone()))
            .transpose()?;
        let projects = self
            .config
            .project_registry
            .list()
            .into_iter()
            .filter(|entry| filter_state.map_or(true, |state| entry.state == state))
            .map(|entry| Self::project_summary(&entry))
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(json!({ "projects": projects }))
    }

    async fn dispatch_project_get(
        &self,
        context: &ProtocolContext,
        params: &Value,
    ) -> anyhow::Result<Value> {
        Self::ensure_project_admin(context, "kernel.v1.project.get")?;
        let id = Self::project_id_param(params, "kernel.v1.project.get")?;
        let entry = self
            .config
            .project_registry
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("project '{}' not found", id))?;
        let mut value = serde_json::to_value(&entry.descriptor)?;
        if let Value::Object(map) = &mut value {
            map.insert("state".to_string(), serde_json::to_value(entry.state)?);
            map.insert("paths".to_string(), Self::project_paths_and_counts(&id));
            if matches!(entry.state, ProjectState::Running | ProjectState::Starting) {
                if let Some(session_id) = self.find_session_for_project(&id).await {
                    map.insert("running_session_id".to_string(), json!(session_id));
                }
            }
        }
        Ok(value)
    }

    async fn dispatch_project_status(
        &self,
        context: &ProtocolContext,
        params: &Value,
    ) -> anyhow::Result<Value> {
        Self::ensure_project_admin(context, "kernel.v1.project.status")?;
        let id = Self::project_id_param(params, "kernel.v1.project.status")?;
        let entry = self
            .config
            .project_registry
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("project '{}' not found", id))?;
        let details = Self::project_paths_and_counts(&id);
        let mut value = json!({
            "project_id": id.as_str(),
            "state": serde_json::to_value(entry.state)?,
            "sessions_count": details.get("sessions_count").and_then(Value::as_u64).unwrap_or(0),
            "secrets_count": details.get("secrets_count").and_then(Value::as_u64).unwrap_or(0),
        });
        if matches!(entry.state, ProjectState::Running | ProjectState::Starting) {
            if let Some(session_id) = self.find_session_for_project(&id).await {
                if let Value::Object(map) = &mut value {
                    map.insert("running_session_id".to_string(), json!(session_id));
                }
            }
        }
        Ok(value)
    }

    async fn dispatch_project_start(
        &self,
        context: &ProtocolContext,
        params: &Value,
    ) -> anyhow::Result<Value> {
        Self::ensure_project_admin(context, "kernel.v1.project.start")?;
        let id = Self::project_id_param(params, "kernel.v1.project.start")?;
        let entry = self
            .config
            .project_registry
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("project '{}' not found", id))?;
        let previous_state = entry.state;

        if matches!(entry.state, ProjectState::Running | ProjectState::Starting) {
            if let Some(existing_session_id) = self.find_session_for_project(&id).await {
                return Ok(json!({
                    "project_id": id.as_str(),
                    "previous_state": serde_json::to_value(previous_state)?,
                    "new_state": serde_json::to_value(entry.state)?,
                    "session_id": existing_session_id,
                    "already_running": true,
                }));
            }
        }

        if matches!(entry.state, ProjectState::Archived) {
            anyhow::bail!("project '{}' is archived; restore before starting", id);
        }

        if !matches!(
            entry.state,
            ProjectState::Installed | ProjectState::Stopped | ProjectState::Failed
        ) {
            anyhow::bail!("project '{}' cannot start from state {:?}", id, entry.state);
        }

        self.config
            .project_registry
            .set_state(&id, ProjectState::Starting)?;

        let session = self
            .open_session(OpenSessionRequest {
                labels: vec![format!("project:{}", id.as_str())],
                metadata: json!({
                    "project_id": id.as_str(),
                    "project_title": entry.descriptor.project.title,
                    "project_type": serde_json::to_value(&entry.descriptor.project.project_type)?,
                }),
                ..OpenSessionRequest::default()
            })
            .await?;
        let session_id = session.id.clone();

        self.append_kernel_event(
            &session_id,
            ygg_core::PROJECT_STARTED,
            json!({
                "project_id": entry.descriptor.project.id.as_str(),
                "title": entry.descriptor.project.title,
                "type": serde_json::to_value(&entry.descriptor.project.project_type)?,
                "previous_state": serde_json::to_value(previous_state)?,
                "new_state": serde_json::to_value(ProjectState::Running)?,
                "session_id": session_id,
            }),
        )
        .await?;

        self.config
            .project_registry
            .set_state(&id, ProjectState::Running)?;
        Ok(json!({
            "project_id": id.as_str(),
            "previous_state": serde_json::to_value(previous_state)?,
            "new_state": serde_json::to_value(ProjectState::Running)?,
            "session_id": session_id,
            "already_running": false,
        }))
    }

    async fn dispatch_project_stop(
        &self,
        context: &ProtocolContext,
        params: &Value,
    ) -> anyhow::Result<Value> {
        Self::ensure_project_admin(context, "kernel.v1.project.stop")?;
        let id = Self::project_id_param(params, "kernel.v1.project.stop")?;
        let entry = self
            .config
            .project_registry
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("project '{}' not found", id))?;
        if !matches!(entry.state, ProjectState::Running | ProjectState::Starting) {
            anyhow::bail!("project '{}' cannot stop from state {:?}", id, entry.state);
        }
        let previous_state = entry.state;
        let session_id = self.find_session_for_project(&id).await;
        self.config
            .project_registry
            .set_state(&id, ProjectState::Stopping)?;

        if let Some(session_id) = &session_id {
            self.append_kernel_event(
                session_id,
                ygg_core::PROJECT_STOPPED,
                json!({
                    "project_id": entry.descriptor.project.id.as_str(),
                    "title": entry.descriptor.project.title,
                    "type": serde_json::to_value(&entry.descriptor.project.project_type)?,
                    "previous_state": serde_json::to_value(previous_state)?,
                    "new_state": serde_json::to_value(ProjectState::Stopped)?,
                    "session_id": session_id,
                }),
            )
            .await?;
            self.close_session(session_id.clone()).await?;
        }

        self.config
            .project_registry
            .set_state(&id, ProjectState::Stopped)?;
        Ok(json!({
            "project_id": id.as_str(),
            "previous_state": serde_json::to_value(previous_state)?,
            "new_state": serde_json::to_value(ProjectState::Stopped)?,
            "session_id": session_id,
        }))
    }

    // --- Outbound ---

    async fn dispatch_outbound_audit(&self, params: &Value) -> anyhow::Result<Value> {
        let package_id = params
            .get("package_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.outbound.audit requires package_id"))?
            .to_string();
        Ok(serde_json::to_value(
            self.list_outbound_audit(&package_id).await?,
        )?)
    }

    // --- Audit ---

    async fn dispatch_audit_package(&self, params: &Value) -> anyhow::Result<Value> {
        let request: crate::AuditPackageParams = serde_json::from_value(params.clone())?;
        let (since, until) = request.window();
        Ok(serde_json::to_value(
            self.audit_package(&request.package_id, since, until)
                .await?,
        )?)
    }

    async fn dispatch_outbound_execute(
        &self,
        context: &ProtocolContext,
        params: Value,
    ) -> anyhow::Result<Value> {
        // --- L3: public outbound/secret boundary ---
        // Determine package_id from the protocol context principal.
        // The caller CANNOT self-assert a different package_id.
        let package_id = match &context.principal {
            ProtocolPrincipal::Package { package_id } => package_id.clone(),
            ProtocolPrincipal::HostAdmin | ProtocolPrincipal::HostDev => {
                // Host principals may supply package_id in params for testing.
                params
                    .get("package_id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| "host/test".to_string())
            }
            other => {
                anyhow::bail!(
                    "kernel.v1.outbound.execute requires package or host principal, got {:?}",
                    other
                )
            }
        };

        let capability_id = params
            .get("capability_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.outbound.execute requires capability_id"))?
            .to_string();
        if !capability_id.starts_with(&format!("{package_id}/")) {
            anyhow::bail!(
                "kernel.v1.outbound.execute capability_id must belong to the caller package namespace"
            );
        }
        let destination_host = params
            .get("destination_host")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.outbound.execute requires destination_host"))?
            .to_string();
        let method = params
            .get("method")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.outbound.execute requires method"))?
            .to_string();
        let path: Option<String> = params
            .get("path")
            .and_then(Value::as_str)
            .map(str::to_string);
        let purpose: Option<String> = params
            .get("purpose")
            .and_then(Value::as_str)
            .map(str::to_string);
        let secret_refs: Vec<String> = params
            .get("secret_refs")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let metadata = params.get("metadata").cloned().unwrap_or(Value::Null);
        let body_shape = params.get("body_shape").cloned();

        // L4: Parse secret_headers from params for host-side injection.
        // Format: { "Authorization": {"secret_ref": "...", "scheme": "bearer"} }
        let secret_headers_spec = parse_secret_headers(&params)?;

        // L5: Parse static_headers from params for safe non-secret header injection.
        // Format: { "anthropic-version": "2023-06-01" }
        // Only allowlisted header names are accepted; secret-bearing names are rejected.
        let static_headers = parse_static_headers(&params)?;

        // Collect secret_refs from both top-level and secret_headers
        let mut all_secret_refs = secret_refs.clone();
        for spec in &secret_headers_spec {
            if !all_secret_refs.contains(&spec.secret_ref) {
                all_secret_refs.push(spec.secret_ref.clone());
            }
        }

        // Y2: enforce manifest declarations for secret refs.
        // Any secret_ref used in top-level `secret_refs` or in
        // `secret_headers` must be declared in the caller package's
        // `permissions.secret_refs`. Undeclared refs → fail-closed.
        if !all_secret_refs.is_empty() && !self.is_contract_none_package(&package_id).await {
            let manifest = self.packages.manifest(&package_id).await.ok_or_else(|| {
                anyhow::anyhow!(
                    "kernel.v1.outbound.execute package '{}' is not loaded",
                    package_id
                )
            })?;
            let declared: std::collections::HashSet<&str> = manifest
                .permissions
                .secret_refs
                .iter()
                .map(|s| s.as_str())
                .collect();
            for secret_ref in &all_secret_refs {
                if !declared.contains(secret_ref.as_str()) {
                    anyhow::bail!(
                        "secret_ref '{}' is not declared in package manifest permissions.secret_refs",
                        secret_ref
                    );
                }
            }
        }

        let policy_request = super::OutboundRequest {
            principal: context.principal.clone(),
            package_id: package_id.clone(),
            capability_id: capability_id.clone(),
            destination_host: destination_host.clone(),
            method: method.clone(),
            purpose: purpose.clone(),
            secret_refs_used: all_secret_refs.clone(),
            correlation_id: context.correlation_id,
        };

        // L4: Resolve secret_headers into resolved_secret_headers
        let mut resolved_secret_headers = Vec::new();
        for spec in &secret_headers_spec {
            HeaderName::from_bytes(spec.header_name.as_bytes()).map_err(|_| {
                anyhow::anyhow!("kernel.v1.outbound.execute secret header name is invalid")
            })?;
            let raw_value = self
                .resolve_secret_ref_with_session(&spec.secret_ref, context.session_id.as_deref())
                .await
                .map_err(|_| {
                    anyhow::anyhow!("kernel.v1.outbound.execute secret header is unavailable")
                })?;
            let header_value = match spec.scheme.to_lowercase().as_str() {
                "bearer" => format!("Bearer {}", raw_value),
                "basic" => format!("Basic {}", raw_value),
                "raw" | "" => raw_value,
                other => format!("{} {}", other, raw_value),
            };
            HeaderValue::from_str(&header_value).map_err(|_| {
                anyhow::anyhow!("kernel.v1.outbound.execute secret header value is invalid")
            })?;
            resolved_secret_headers.push(super::outbound::ResolvedSecretHeader {
                header_name: spec.header_name.clone(),
                value: super::outbound::RedactedHeaderValue(header_value),
            });
        }

        let executor_request = super::OutboundExecutorRequest {
            package_id: package_id.clone(),
            capability_id: capability_id.clone(),
            destination_host: destination_host.clone(),
            method: method.clone(),
            path,
            purpose,
            secret_refs: all_secret_refs.clone(),
            redaction_state: None,
            timeout_ms: params.get("timeout_ms").and_then(Value::as_u64),
            metadata,
            body_shape,
            secret_headers: secret_headers_spec,
            resolved_secret_headers,
            static_headers,
        };

        let response = self
            .execute_outbound_with_policy(policy_request, executor_request)
            .await?;

        // Strip any raw-secret-like fields from the response before
        // returning it to the caller. The OutboundExecutorResponse
        // struct is already content-free by design, but we do an
        // extra sweep to ensure conformance: no raw_secret,
        // api_key, Bearer, sk- patterns in the serialized output.
        let mut response_value = serde_json::to_value(&response)?;
        strip_raw_secrets_from_value(&mut response_value);
        Ok(response_value)
    }

    /// Y3: Dispatch `kernel.v1.outbound.stream`.
    ///
    /// Performs the same permission checks as `kernel.v1.outbound.execute`,
    /// then starts a kernel stream and spawns the executor's `stream`
    /// method to emit frames asynchronously. Returns a stream_id
    /// that the caller subscribes to via the existing event stream.
    async fn dispatch_outbound_stream(
        &self,
        context: &ProtocolContext,
        params: Value,
    ) -> anyhow::Result<Value> {
        // --- Same auth/permission checks as dispatch_outbound_execute ---
        let package_id = match &context.principal {
            ProtocolPrincipal::Package { package_id } => package_id.clone(),
            ProtocolPrincipal::HostAdmin | ProtocolPrincipal::HostDev => params
                .get("package_id")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| "host/test".to_string()),
            other => {
                anyhow::bail!(
                    "kernel.v1.outbound.stream requires package or host principal, got {:?}",
                    other
                )
            }
        };

        let capability_id = params
            .get("capability_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.outbound.stream requires capability_id"))?
            .to_string();
        if !capability_id.starts_with(&format!("{package_id}/")) {
            anyhow::bail!(
                "kernel.v1.outbound.stream capability_id must belong to the caller package namespace"
            );
        }
        let destination_host = params
            .get("destination_host")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.outbound.stream requires destination_host"))?
            .to_string();
        let method = params
            .get("method")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.outbound.stream requires method"))?
            .to_string();
        let path: Option<String> = params
            .get("path")
            .and_then(Value::as_str)
            .map(str::to_string);
        let purpose: Option<String> = params
            .get("purpose")
            .and_then(Value::as_str)
            .map(str::to_string);
        let secret_refs: Vec<String> = params
            .get("secret_refs")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let metadata = params.get("metadata").cloned().unwrap_or(Value::Null);
        let body_shape = params.get("body_shape").cloned();

        // Y3: Host-level stream policy mirrors outbound.execute fail-closed
        // profile behavior. The default profile has this disabled, so stream
        // requests are rejected before any stream is registered or executor is
        // called.
        let stream_policy = &self.config.outbound_execute_policy;
        if !stream_policy.enabled {
            anyhow::bail!("host policy has not enabled outbound.stream");
        }
        if stream_policy.allow_redirects {
            anyhow::bail!("outbound.stream redirects are disabled");
        }
        if stream_policy.allowed_hosts.is_empty()
            || !stream_policy
                .allowed_hosts
                .iter()
                .any(|allowed| outbound_host_matches(allowed, &destination_host))
        {
            anyhow::bail!(
                "host policy does not allow outbound.stream host '{}'",
                destination_host
            );
        }
        if stream_policy.https_only {
            validate_outbound_stream_https_only(
                &destination_host,
                path.as_deref(),
                &metadata,
                stream_policy.allow_insecure_loopback_for_tests,
            )?;
        } else {
            anyhow::bail!("host policy attempted to disable HTTPS-only outbound.stream");
        }

        // Parse secret_headers (same as execute)
        let secret_headers_spec = parse_secret_headers(&params)?;

        // Parse static_headers (same as execute)
        let static_headers = parse_static_headers(&params)?;

        // Collect all secret_refs
        let mut all_secret_refs = secret_refs.clone();
        for spec in &secret_headers_spec {
            if !all_secret_refs.contains(&spec.secret_ref) {
                all_secret_refs.push(spec.secret_ref.clone());
            }
        }

        // Y2: enforce manifest declarations for secret refs
        if !all_secret_refs.is_empty() && !self.is_contract_none_package(&package_id).await {
            let manifest = self.packages.manifest(&package_id).await.ok_or_else(|| {
                anyhow::anyhow!(
                    "kernel.v1.outbound.stream package '{}' is not loaded",
                    package_id
                )
            })?;
            let declared: std::collections::HashSet<&str> = manifest
                .permissions
                .secret_refs
                .iter()
                .map(|s| s.as_str())
                .collect();
            for secret_ref in &all_secret_refs {
                if !declared.contains(secret_ref.as_str()) {
                    anyhow::bail!(
                        "secret_ref '{}' is not declared in package manifest permissions.secret_refs",
                        secret_ref
                    );
                }
            }
        }

        // Y3: Parse stream_format
        let stream_format_str = params
            .get("stream_format")
            .and_then(Value::as_str)
            .unwrap_or("sse");
        let stream_format = match stream_format_str {
            "sse" => super::outbound::StreamFormat::Sse,
            "ndjson" => super::outbound::StreamFormat::Ndjson,
            "raw" => super::outbound::StreamFormat::Raw,
            other => anyhow::bail!("kernel.v1.outbound.stream unknown stream_format '{other}'"),
        };
        let max_frame_bytes = params
            .get("max_frame_bytes")
            .and_then(Value::as_u64)
            .map(|v| v as usize);
        let max_total_bytes = params
            .get("max_total_bytes")
            .and_then(Value::as_u64)
            .map(|v| v as usize);
        let max_duration_ms = params.get("max_duration_ms").and_then(Value::as_u64);

        // Build the policy request (same checks as execute)
        let policy_request = super::OutboundRequest {
            principal: context.principal.clone(),
            package_id: package_id.clone(),
            capability_id: capability_id.clone(),
            destination_host: destination_host.clone(),
            method: method.clone(),
            purpose: purpose.clone(),
            secret_refs_used: all_secret_refs.clone(),
            correlation_id: context.correlation_id,
        };

        // Resolve secret headers
        let mut resolved_secret_headers = Vec::new();
        for spec in &secret_headers_spec {
            reqwest::header::HeaderName::from_bytes(spec.header_name.as_bytes()).map_err(|_| {
                anyhow::anyhow!("kernel.v1.outbound.stream secret header name is invalid")
            })?;
            let raw_value = self
                .resolve_secret_ref_with_session(&spec.secret_ref, context.session_id.as_deref())
                .await
                .map_err(|_| {
                    anyhow::anyhow!("kernel.v1.outbound.stream secret header is unavailable")
                })?;
            let header_value = match spec.scheme.to_lowercase().as_str() {
                "bearer" => format!("Bearer {}", raw_value),
                "basic" => format!("Basic {}", raw_value),
                "raw" | "" => raw_value,
                other => format!("{} {}", other, raw_value),
            };
            reqwest::header::HeaderValue::from_str(&header_value).map_err(|_| {
                anyhow::anyhow!("kernel.v1.outbound.stream secret header value is invalid")
            })?;
            resolved_secret_headers.push(super::outbound::ResolvedSecretHeader {
                header_name: spec.header_name.clone(),
                value: super::outbound::RedactedHeaderValue(header_value),
            });
        }

        // Run the policy check
        let _audit_record = self.check_and_audit_outbound(policy_request).await?;

        // Build the executor request
        let executor_request = super::OutboundExecutorRequest {
            package_id: package_id.clone(),
            capability_id: capability_id.clone(),
            destination_host: destination_host.clone(),
            method: method.clone(),
            path,
            purpose,
            secret_refs: all_secret_refs.clone(),
            redaction_state: None,
            timeout_ms: params.get("timeout_ms").and_then(Value::as_u64),
            metadata,
            body_shape,
            secret_headers: secret_headers_spec,
            resolved_secret_headers,
            static_headers,
        };

        // Start a kernel stream via the existing streaming infrastructure.
        // We create a synthetic "capability" for the outbound stream.
        let outbound_capability_id = capability_id.clone();
        let session_id = format!("kernel_outbound_stream_{}", package_id.replace('/', "_"));

        // Register the stream in the StreamRegistry
        let stream_record = self
            .streams
            .start_invocation(
                outbound_capability_id.clone(),
                package_id.clone(),
                session_id.clone(),
                serde_json::json!({
                    "destination_host": destination_host,
                    "method": method,
                    "stream_format": stream_format_str,
                }),
            )
            .await;

        // Emit kernel/v1/stream.started event
        let event_payload = json!({
            "invocation_id": stream_record.invocation_id,
            "stream_id": stream_record.stream_id,
            "capability_id": outbound_capability_id,
            "provider_package_id": package_id,
            "session_id": session_id,
        });
        self.append_kernel_event(&session_id, ygg_core::EVENT_STREAM_STARTED, event_payload)
            .await?;

        // Create cancel signal
        let (cancel_tx, cancel_rx) = super::outbound::CancelSignal::new();

        // Store the cancel sender so that kernel.v1.capability.cancel can set it
        let invocation_id = stream_record.invocation_id.clone();
        let stream_id = stream_record.stream_id.clone();

        // Determine executor kind for the response
        let executor = self.outbound_executor();
        let executor_kind = match &self.config.outbound_executor {
            super::outbound::OutboundExecutorConfig::DenyAll => {
                super::outbound::ExecutorKind::DenyAll
            }
            super::outbound::OutboundExecutorConfig::Custom(_) => {
                super::outbound::ExecutorKind::Fake
            }
            super::outbound::OutboundExecutorConfig::LiveHttp(_) => {
                super::outbound::ExecutorKind::Real
            }
        };
        let network_performed = matches!(executor_kind, super::outbound::ExecutorKind::Real);

        // Create a StreamEmitter that feeds into the kernel stream lifecycle
        let emitter = Arc::new(StreamEmitterAdapter {
            streams: self.streams.clone(),
            store: self.store.clone(),
            session_id: session_id.clone(),
            invocation_id: invocation_id.clone(),
            stream_id: stream_id.clone(),
            cancel_tx,
        });

        // Spawn the executor stream in a background task
        let executor_for_task = executor.clone();
        let invocation_id_for_end = invocation_id.clone();
        let session_id_for_end = session_id.clone();
        let streams_for_end = self.streams.clone();
        let store_for_end = self.store.clone();
        let stream_id_for_end = stream_id.clone();
        let context_principal = context.principal.clone();
        let pkg_id_for_end = package_id.clone();
        let cap_id_for_end = outbound_capability_id.clone();
        let host_for_end = destination_host.clone();
        let method_for_end = method.clone();
        let format_str_for_end = stream_format_str.to_string();
        let secret_refs_for_end = all_secret_refs.clone();
        let correlation_id_for_end = context.correlation_id;
        let completion_id_for_end = ygg_core::new_id("obc");
        let started_for_end = Instant::now();
        let executor_kind_for_error = executor_kind;
        let network_performed_for_error = network_performed;

        tokio::spawn(async move {
            let result = executor_for_task
                .stream(
                    executor_request,
                    stream_format,
                    emitter.clone(),
                    cancel_rx,
                    max_frame_bytes,
                    max_total_bytes,
                    max_duration_ms,
                )
                .await;

            // Helper closure for appending kernel events
            let append_event = |kind: &'static str, payload: Value| {
                let store = store_for_end.clone();
                let session_id = session_id_for_end.clone();
                async move {
                    use ygg_core::{new_id, EventEnvelope, KERNEL_PACKAGE_ID};
                    let seq = store.next_sequence(&session_id).await.unwrap_or(0);
                    let event = EventEnvelope {
                        id: new_id("evt"),
                        session_id,
                        sequence: seq,
                        timestamp: chrono::Utc::now(),
                        writer_package_id: KERNEL_PACKAGE_ID.to_string(),
                        kind: kind.to_string(),
                        schema_version: 1,
                        payload,
                        metadata: json!({}),
                    };
                    let _ = store.append(event).await;
                }
            };

            // On stream end, write the terminal state and audit
            match result {
                Ok(summary) => {
                    // End the invocation in the registry
                    let _ = streams_for_end.end_invocation(&invocation_id_for_end).await;

                    // Emit kernel/v1/stream.ended event
                    append_event(
                        ygg_core::EVENT_STREAM_ENDED,
                        json!({
                            "invocation_id": invocation_id_for_end,
                            "stream_id": stream_id_for_end,
                            "status": summary.status,
                            "frame_count": summary.frame_count,
                            "bytes_received": summary.bytes_received,
                            "executor_kind": summary.executor_kind,
                        }),
                    )
                    .await;

                    // Emit outbound audit record
                    append_event(ygg_core::EVENT_OUTBOUND_REQUEST, json!({
                        "principal": context_principal,
                        "package_id": pkg_id_for_end,
                        "capability_id": cap_id_for_end,
                        "destination_host": host_for_end,
                        "method": method_for_end,
                        "status": summary.status,
                        "stream_format": format_str_for_end,
                        "frame_count": summary.frame_count,
                        "bytes_received": summary.bytes_received,
                        "redaction_state": serde_json::to_value(summary.redaction_state).unwrap_or(Value::Null),
                        "executor_kind": summary.executor_kind,
                        "network_performed": summary.network_performed,
                    })).await;

                    let final_termination = match summary.status.as_str() {
                        "cancelled" => "cancelled",
                        "timeout" => "timeout",
                        "error" => "error",
                        _ => "ended",
                    };
                    append_event(
                        ygg_core::EVENT_OUTBOUND_STREAM_COMPLETED,
                        json!({
                            "id": completion_id_for_end,
                            "package_id": pkg_id_for_end,
                            "capability_id": cap_id_for_end,
                            "destination_host": host_for_end,
                            "method": method_for_end,
                            "stream_format": format_str_for_end,
                            "status": summary.status,
                            "total_chunks": summary.frame_count,
                            "total_bytes": summary.bytes_received,
                            "duration_ms": started_for_end.elapsed().as_millis() as u64,
                            "final_termination": final_termination,
                            "executor_kind": summary.executor_kind,
                            "network_performed": summary.network_performed,
                            "redaction_state": summary.redaction_state,
                            "secret_refs_used": secret_refs_for_end,
                            "correlation_id": correlation_id_for_end,
                        }),
                    )
                    .await;
                }
                Err(e) => {
                    // Error the invocation in the registry
                    let _ = streams_for_end
                        .error_invocation(&invocation_id_for_end, &e.to_string())
                        .await;

                    // Emit kernel/v1/stream.error event
                    append_event(
                        ygg_core::EVENT_STREAM_ERROR,
                        json!({
                            "invocation_id": invocation_id_for_end,
                            "stream_id": stream_id_for_end,
                            "error": e.to_string(),
                        }),
                    )
                    .await;
                    append_event(
                        ygg_core::EVENT_OUTBOUND_STREAM_COMPLETED,
                        json!({
                            "id": completion_id_for_end,
                            "package_id": pkg_id_for_end,
                            "capability_id": cap_id_for_end,
                            "destination_host": host_for_end,
                            "method": method_for_end,
                            "stream_format": format_str_for_end,
                            "status": "error",
                            "total_chunks": 0,
                            "total_bytes": 0,
                            "duration_ms": started_for_end.elapsed().as_millis() as u64,
                            "final_termination": "error",
                            "executor_kind": executor_kind_for_error,
                            "network_performed": network_performed_for_error,
                            "redaction_state": ygg_core::RedactionState::Redacted,
                            "secret_refs_used": secret_refs_for_end,
                            "correlation_id": correlation_id_for_end,
                        }),
                    )
                    .await;
                }
            }
        });

        // Return the stream response immediately
        let response = super::outbound::KernelOutboundStreamResponse {
            stream_id: stream_record.stream_id.clone(),
            status: super::outbound::StreamStartStatus::Ok,
            redaction_state: RedactionState::Redacted,
            network_performed,
            executor_kind,
        };

        let mut response_value = serde_json::to_value(&response)?;
        strip_raw_secrets_from_value(&mut response_value);
        Ok(response_value)
    }

    async fn dispatch_outbound_websocket_open(
        &self,
        context: &ProtocolContext,
        params: Value,
    ) -> anyhow::Result<Value> {
        let package_id = match &context.principal {
            ProtocolPrincipal::Package { package_id } => package_id.clone(),
            ProtocolPrincipal::HostAdmin | ProtocolPrincipal::HostDev => params
                .get("package_id")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| "host/test".to_string()),
            other => anyhow::bail!(
                "kernel.v1.outbound.websocket.open requires package or host principal, got {:?}",
                other
            ),
        };

        let capability_id = params
            .get("capability_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow::anyhow!("kernel.v1.outbound.websocket.open requires capability_id")
            })?
            .to_string();
        if !capability_id.starts_with(&format!("{package_id}/")) {
            anyhow::bail!(
                "kernel.v1.outbound.websocket.open capability_id must belong to the caller package namespace"
            );
        }
        let destination_host = params
            .get("destination_host")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow::anyhow!("kernel.v1.outbound.websocket.open requires destination_host")
            })?
            .to_string();
        let path = params
            .get("path")
            .and_then(Value::as_str)
            .map(str::to_string);
        let purpose = params
            .get("purpose")
            .and_then(Value::as_str)
            .map(str::to_string);
        let mut metadata = params.get("metadata").cloned().unwrap_or(Value::Null);
        let subprotocols: Vec<String> = params
            .get("subprotocols")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let secret_refs: Vec<String> = params
            .get("secret_refs")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let secret_headers_spec = parse_secret_headers(&params)?;
        let static_headers_spec = parse_static_headers(&params)?;
        let mut all_secret_refs = secret_refs.clone();
        for spec in &secret_headers_spec {
            if !all_secret_refs.contains(&spec.secret_ref) {
                all_secret_refs.push(spec.secret_ref.clone());
            }
        }

        if !all_secret_refs.is_empty() && !self.is_contract_none_package(&package_id).await {
            let manifest = self.packages.manifest(&package_id).await.ok_or_else(|| {
                anyhow::anyhow!(
                    "kernel.v1.outbound.websocket.open package '{}' is not loaded",
                    package_id
                )
            })?;
            let declared: std::collections::HashSet<&str> = manifest
                .permissions
                .secret_refs
                .iter()
                .map(|s| s.as_str())
                .collect();
            for secret_ref in &all_secret_refs {
                if !declared.contains(secret_ref.as_str()) {
                    anyhow::bail!(
                        "secret_ref '{}' is not declared in package manifest permissions.secret_refs",
                        secret_ref
                    );
                }
            }
        }

        let policy_request = super::OutboundRequest {
            principal: context.principal.clone(),
            package_id: package_id.clone(),
            capability_id: capability_id.clone(),
            destination_host: destination_host.clone(),
            method: WEBSOCKET_METHOD.to_string(),
            purpose: purpose.clone(),
            secret_refs_used: all_secret_refs.clone(),
            correlation_id: context.correlation_id,
        };
        let _audit_record = self.check_and_audit_outbound(policy_request).await?;

        let mut secret_headers = HashMap::new();
        for spec in &secret_headers_spec {
            HeaderName::from_bytes(spec.header_name.as_bytes()).map_err(|_| {
                anyhow::anyhow!("kernel.v1.outbound.websocket.open secret header name is invalid")
            })?;
            let raw_value = self
                .resolve_secret_ref_with_session(&spec.secret_ref, context.session_id.as_deref())
                .await
                .map_err(|_| {
                    anyhow::anyhow!(
                        "kernel.v1.outbound.websocket.open secret header is unavailable"
                    )
                })?;
            let header_value = match spec.scheme.to_lowercase().as_str() {
                "bearer" => format!("Bearer {}", raw_value),
                "basic" => format!("Basic {}", raw_value),
                "raw" | "" => raw_value,
                other => format!("{} {}", other, raw_value),
            };
            HeaderValue::from_str(&header_value).map_err(|_| {
                anyhow::anyhow!("kernel.v1.outbound.websocket.open secret header value is invalid")
            })?;
            secret_headers.insert(spec.header_name.clone(), header_value);
        }
        let static_headers = static_headers_spec
            .into_iter()
            .map(|hdr| (hdr.name, hdr.value))
            .collect::<HashMap<_, _>>();

        let session_id = format!("kernel_outbound_websocket_{}", package_id.replace('/', "_"));
        let stream_record = self
            .streams
            .start_invocation(
                capability_id.clone(),
                package_id.clone(),
                session_id.clone(),
                json!({
                    "destination_host": destination_host,
                    "method": WEBSOCKET_METHOD,
                }),
            )
            .await;
        self.append_kernel_event(
            &session_id,
            ygg_core::EVENT_STREAM_STARTED,
            json!({
                "invocation_id": stream_record.invocation_id,
                "stream_id": stream_record.stream_id,
                "capability_id": capability_id,
                "provider_package_id": package_id,
                "session_id": session_id,
            }),
        )
        .await?;

        metadata = match metadata {
            Value::Object(mut map) => {
                map.insert(
                    "connection_id".to_string(),
                    Value::String(stream_record.stream_id.clone()),
                );
                Value::Object(map)
            }
            other => json!({"connection_id": stream_record.stream_id, "request_metadata": other}),
        };

        let req = super::OutboundWebSocketOpenRequest {
            capability_id: capability_id.clone(),
            package_id: package_id.clone(),
            destination_host: destination_host.clone(),
            path,
            purpose,
            subprotocols,
            secret_refs: all_secret_refs.clone(),
            metadata,
            static_headers,
            secret_headers,
            max_frame_bytes: params
                .get("max_frame_bytes")
                .and_then(Value::as_u64)
                .unwrap_or(65_536) as usize,
            max_total_bytes_inbound: params
                .get("max_total_bytes_inbound")
                .and_then(Value::as_u64)
                .unwrap_or(10 * 1024 * 1024) as usize,
            max_total_bytes_outbound: params
                .get("max_total_bytes_outbound")
                .and_then(Value::as_u64)
                .unwrap_or(10 * 1024 * 1024) as usize,
            max_idle_ms: params
                .get("max_idle_ms")
                .and_then(Value::as_u64)
                .unwrap_or(60_000),
            max_duration_ms: params
                .get("max_duration_ms")
                .and_then(Value::as_u64)
                .unwrap_or(1_800_000),
        };
        let session = self.outbound_websocket_executor().open(req).await?;
        let response = json!({
            "connection_id": session.connection_id,
            "status": "ok",
            "subprotocol_negotiated": session.subprotocol_negotiated,
            "redaction_state": session.redaction_state,
            "network_performed": session.network_performed,
            "executor_kind": session.executor_kind,
        });
        let store = self.store.clone();
        let streams = self.streams.clone();
        let session_id_for_task = session_id.clone();
        let invocation_id_for_task = stream_record.invocation_id.clone();
        let stream_id_for_task = stream_record.stream_id.clone();
        let completion_id_for_task = ygg_core::new_id("obc");
        let correlation_id_for_task = context.correlation_id;
        let pkg_id_for_task = package_id.clone();
        let cap_id_for_task = capability_id.clone();
        let host_for_task = destination_host.clone();
        let executor_kind_for_task = session.executor_kind;
        let network_performed_for_task = session.network_performed;
        let redaction_state_for_task = session.redaction_state;
        let secret_refs_for_task = all_secret_refs.clone();
        let mut events = session.events;
        tokio::spawn(async move {
            while let Some(event) = events.recv().await {
                let (kind, mut payload, terminal) = websocket_event_to_kernel_event(event);
                if terminal {
                    if let Value::Object(map) = &mut payload {
                        map.insert(
                            "id".to_string(),
                            Value::String(completion_id_for_task.clone()),
                        );
                        map.insert(
                            "package_id".to_string(),
                            Value::String(pkg_id_for_task.clone()),
                        );
                        map.insert(
                            "capability_id".to_string(),
                            Value::String(cap_id_for_task.clone()),
                        );
                        map.insert(
                            "destination_host".to_string(),
                            Value::String(host_for_task.clone()),
                        );
                        map.insert("executor_kind".to_string(), json!(executor_kind_for_task));
                        map.insert(
                            "network_performed".to_string(),
                            Value::Bool(network_performed_for_task),
                        );
                        map.insert(
                            "redaction_state".to_string(),
                            json!(redaction_state_for_task),
                        );
                        map.insert("secret_refs_used".to_string(), json!(secret_refs_for_task));
                        map.insert("correlation_id".to_string(), json!(correlation_id_for_task));
                    }
                }
                use ygg_core::{new_id, EventEnvelope, KERNEL_PACKAGE_ID};
                let seq = store.next_sequence(&session_id_for_task).await.unwrap_or(0);
                let _ = store
                    .append(EventEnvelope {
                        id: new_id("evt"),
                        session_id: session_id_for_task.clone(),
                        sequence: seq,
                        timestamp: chrono::Utc::now(),
                        writer_package_id: KERNEL_PACKAGE_ID.to_string(),
                        kind: kind.to_string(),
                        schema_version: 1,
                        payload,
                        metadata: json!({}),
                    })
                    .await;
                if terminal {
                    let _ = streams.end_invocation(&invocation_id_for_task).await;
                    let seq = store.next_sequence(&session_id_for_task).await.unwrap_or(0);
                    let _ = store.append(EventEnvelope {
                        id: new_id("evt"),
                        session_id: session_id_for_task.clone(),
                        sequence: seq,
                        timestamp: chrono::Utc::now(),
                        writer_package_id: KERNEL_PACKAGE_ID.to_string(),
                        kind: ygg_core::EVENT_STREAM_ENDED.to_string(),
                        schema_version: 1,
                        payload: json!({"invocation_id": invocation_id_for_task, "stream_id": stream_id_for_task}),
                        metadata: json!({}),
                    }).await;
                    break;
                }
            }
        });
        let mut response_value = response;
        strip_raw_secrets_from_value(&mut response_value);
        Ok(response_value)
    }

    async fn dispatch_outbound_websocket_send(&self, params: &Value) -> anyhow::Result<Value> {
        let connection_id = params
            .get("connection_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow::anyhow!("kernel.v1.outbound.websocket.send requires connection_id")
            })?;
        let frame = parse_websocket_frame(params)?;
        let status = self
            .outbound_websocket_executor()
            .send(connection_id, frame)
            .await?;
        Ok(json!({"status": status}))
    }

    async fn dispatch_outbound_websocket_close(&self, params: &Value) -> anyhow::Result<Value> {
        let connection_id = params
            .get("connection_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow::anyhow!("kernel.v1.outbound.websocket.close requires connection_id")
            })?;
        let code = params.get("code").and_then(Value::as_u64).unwrap_or(1000) as u16;
        let reason = params
            .get("reason")
            .and_then(Value::as_str)
            .map(str::to_string);
        self.outbound_websocket_executor()
            .close(connection_id, code, reason)
            .await?;
        Ok(json!({"status": "ok"}))
    }

    // --- Permission ---

    async fn dispatch_permission_grant(&self, params: &Value) -> anyhow::Result<Value> {
        let principal = params
            .get("principal")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.permission.grant requires principal"))?;
        let principal: crate::ProtocolPrincipal = serde_json::from_value(principal)?;
        let permission = params
            .get("permission")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.permission.grant requires permission"))?
            .to_string();
        let scope = params
            .get("scope")
            .and_then(Value::as_str)
            .map(str::to_string);
        let reason = params
            .get("reason")
            .and_then(Value::as_str)
            .map(str::to_string);
        Ok(serde_json::to_value(
            self.grant_permission(principal, permission, scope, reason)
                .await?,
        )?)
    }

    async fn dispatch_permission_revoke(&self, params: &Value) -> anyhow::Result<Value> {
        let grant_id = params
            .get("grant_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.permission.revoke requires grant_id"))?;
        Ok(serde_json::to_value(
            self.revoke_permission(grant_id).await?,
        )?)
    }

    async fn dispatch_permission_list(&self, params: &Value) -> anyhow::Result<Value> {
        let principal = match params.get("principal") {
            Some(value) => Some(serde_json::from_value(value.clone())?),
            None => None,
        };
        Ok(serde_json::to_value(
            self.list_permission_grants(principal).await,
        )?)
    }

    async fn dispatch_permission_audit(&self) -> anyhow::Result<Value> {
        let events = self.store.list_kind_prefix("kernel/v1/permission").await?;
        Ok(serde_json::to_value(events)?)
    }

    // --- Proposal ---

    async fn dispatch_proposal_create(
        &self,
        context: &ProtocolContext,
        params: &Value,
    ) -> anyhow::Result<Value> {
        let proposal: super::ProposalRecord = serde_json::from_value(params.clone())?;
        Ok(serde_json::to_value(
            self.create_proposal(context, proposal).await?,
        )?)
    }

    async fn dispatch_proposal_get(&self, params: &Value) -> anyhow::Result<Value> {
        let proposal_id = params
            .get("proposal_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.proposal.get requires proposal_id"))?;
        Ok(serde_json::to_value(self.get_proposal(proposal_id).await?)?)
    }

    async fn dispatch_proposal_list(&self) -> anyhow::Result<Value> {
        Ok(serde_json::to_value(self.list_proposals().await)?)
    }

    async fn dispatch_proposal_approve(
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

    async fn dispatch_proposal_reject(
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

    async fn dispatch_proposal_apply(&self, params: &Value) -> anyhow::Result<Value> {
        let proposal_id = params
            .get("proposal_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.proposal.apply requires proposal_id"))?;
        Ok(serde_json::to_value(
            self.apply_proposal(proposal_id).await?,
        )?)
    }

    // --- Session ---

    async fn dispatch_session_close(&self, params: &Value) -> anyhow::Result<Value> {
        let session_id = params
            .get("session_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.session.close requires session_id"))?
            .to_string();
        Ok(serde_json::to_value(self.close_session(session_id).await?)?)
    }

    async fn dispatch_session_get(&self, params: &Value) -> anyhow::Result<Value> {
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

    async fn dispatch_session_fork(&self, params: &Value) -> anyhow::Result<Value> {
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

    async fn dispatch_session_branch_list(&self, params: &Value) -> anyhow::Result<Value> {
        let session_id = params
            .get("session_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.session.branch.list requires session_id"))?
            .to_string();
        Ok(serde_json::to_value(self.list_branches(&session_id).await)?)
    }

    // --- Event ---

    async fn dispatch_event_list(
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

    // --- Package ---

    async fn dispatch_package_status(&self, params: &Value) -> anyhow::Result<Value> {
        let package_id = params
            .get("package_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.package.status requires package_id"))?
            .to_string();
        Ok(serde_json::to_value(
            self.package_status(&package_id)
                .await
                .ok_or_else(|| anyhow::anyhow!("package '{package_id}' is not loaded"))?,
        )?)
    }

    async fn dispatch_package_unload(&self, params: &Value) -> anyhow::Result<Value> {
        let package_id = params
            .get("package_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.package.unload requires package_id"))?
            .to_string();
        Ok(serde_json::to_value(
            self.unload_package(&package_id).await?,
        )?)
    }

    async fn dispatch_package_restart(&self, params: &Value) -> anyhow::Result<Value> {
        let package_id = params
            .get("package_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.package.restart requires package_id"))?
            .to_string();
        Ok(serde_json::to_value(
            self.restart_package(&package_id).await?,
        )?)
    }

    async fn dispatch_package_logs(&self, params: &Value) -> anyhow::Result<Value> {
        let package_id = params
            .get("package_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.package.logs requires package_id"))?
            .to_string();
        Ok(serde_json::to_value(self.package_logs(&package_id).await)?)
    }

    // --- Capability ---

    async fn dispatch_cap_attenuate(&self, params: &Value) -> anyhow::Result<Value> {
        let parent_handle: CapHandleId =
            serde_json::from_value(params.get("parent_handle").cloned().ok_or_else(|| {
                anyhow::anyhow!("kernel.v1.cap.attenuate requires parent_handle")
            })?)?;
        let constraints = params.get("constraints").cloned().unwrap_or(Value::Null);
        let handle_id = self.handles.attenuate(parent_handle, constraints).await?;
        let handle = self
            .handles
            .lookup(handle_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("attenuated capability handle not found"))?;
        Ok(json!({ "handle": handle }))
    }

    async fn dispatch_cap_revoke(&self, params: &Value) -> anyhow::Result<Value> {
        let handle: CapHandleId = serde_json::from_value(
            params
                .get("handle")
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("kernel.v1.cap.revoke requires handle"))?,
        )?;
        self.handles.revoke(handle).await?;
        Ok(json!({}))
    }

    async fn dispatch_cap_list_for(&self, params: &Value) -> anyhow::Result<Value> {
        let package_id: PackageId = params
            .get("package_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.cap.list_for requires package_id"))?
            .to_string();
        Ok(json!({ "handles": self.handles.list_for(&package_id).await }))
    }

    async fn dispatch_capability_stream(&self, params: &Value) -> anyhow::Result<Value> {
        let (capability_id, handle_version) = if let Some(handle_value) = params.get("handle") {
            let handle_id: CapHandleId = serde_json::from_value(handle_value.clone())?;
            let handle = self
                .handles
                .lookup(handle_id)
                .await
                .ok_or_else(|| anyhow::anyhow!("capability handle not found"))?;
            if handle.revoked {
                anyhow::bail!("capability handle is revoked");
            }
            if let Some(expires_at) = handle.lease.expires_at {
                if expires_at <= chrono::Utc::now() {
                    anyhow::bail!("capability handle lease expired");
                }
            }
            if let Some(max_invocations) = handle.lease.max_invocations {
                if handle.lease.invocations_used >= max_invocations {
                    anyhow::bail!("capability handle lease exhausted");
                }
            }
            let version = if handle.cap_version == "1" {
                None
            } else {
                Some(handle.cap_version)
            };
            (handle.cap_type, version)
        } else {
            (
                params
                    .get("capability_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "kernel.v1.capability.stream requires capability_id or handle"
                        )
                    })?
                    .to_string(),
                None,
            )
        };
        let session_id = params
            .get("session_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.capability.stream requires session_id"))?
            .to_string();
        let provider_package_id: Option<String> = params
            .get("provider_package_id")
            .and_then(Value::as_str)
            .map(String::from);
        let version: Option<String> = handle_version.or_else(|| {
            params
                .get("version")
                .and_then(Value::as_str)
                .map(String::from)
        });
        let metadata = params
            .get("metadata")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let (frame, record) = self
            .stream_capability_start(
                &session_id,
                &capability_id,
                provider_package_id.as_ref().map(|x| x.as_str()),
                version.as_ref().map(|s| s.as_str()),
                metadata,
            )
            .await?;
        Ok(serde_json::json!({
            "frame": frame,
            "invocation": record,
        }))
    }

    async fn dispatch_capability_cancel(&self, params: &Value) -> anyhow::Result<Value> {
        let invocation_id = match params.get("invocation_id").and_then(Value::as_str) {
            Some(invocation_id) => invocation_id.to_string(),
            None => {
                let stream_id =
                    params
                        .get("stream_id")
                        .and_then(Value::as_str)
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "kernel.v1.capability.cancel requires invocation_id or stream_id"
                            )
                        })?;
                self.streams
                    .get_invocation_by_stream_id(stream_id)
                    .await
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "kernel.v1.capability.cancel stream_id '{}' not found",
                            stream_id
                        )
                    })?
                    .invocation_id
            }
        };
        let session_id = params
            .get("session_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.capability.cancel requires session_id"))?
            .to_string();
        let frame = self
            .stream_capability_cancel(&session_id, &invocation_id)
            .await?;
        if session_id.starts_with("kernel_outbound_websocket_") {
            self.outbound_websocket_executor()
                .close(&frame.stream_id, 1001, Some("cancelled".to_string()))
                .await?;
        }
        Ok(serde_json::to_value(frame)?)
    }

    // --- Asset ---

    async fn dispatch_asset_get(&self, params: &Value) -> anyhow::Result<Value> {
        let asset_id = params
            .get("asset_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.asset.get requires asset_id"))?;
        Ok(serde_json::to_value(self.get_asset(asset_id).await?)?)
    }

    // --- Projection ---

    async fn dispatch_projection_rebuild(&self, params: &Value) -> anyhow::Result<Value> {
        let projection_id = params
            .get("projection_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow::anyhow!("kernel.v1.projection.rebuild requires projection_id")
            })?;
        Ok(serde_json::to_value(
            self.projection_rebuild(projection_id).await?,
        )?)
    }

    async fn dispatch_projection_get(&self, params: &Value) -> anyhow::Result<Value> {
        let projection_id = params
            .get("projection_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.v1.projection.get requires projection_id"))?;
        Ok(serde_json::to_value(
            self.projection_get(projection_id).await?,
        )?)
    }
}

// ---------------------------------------------------------------------------
// Y3: StreamEmitterAdapter — bridges OutboundStreamFrame to kernel stream lifecycle
// ---------------------------------------------------------------------------

/// Adapter that implements `StreamEmitter` and feeds frames into
/// the kernel stream registry.
///
/// Each emitted `OutboundStreamFrame` is converted to a chunk in
/// the `StreamRegistry` and records `kernel/v1/stream.chunk` events.
/// The spawned task's completion handler emits terminal events.
struct StreamEmitterAdapter<S: EventStore> {
    streams: Arc<StreamRegistry>,
    store: Arc<S>,
    session_id: String,
    invocation_id: String,
    stream_id: String,
    cancel_tx: tokio::sync::watch::Sender<bool>,
}

#[async_trait::async_trait]
impl<S> StreamEmitter for StreamEmitterAdapter<S>
where
    S: EventStore,
{
    async fn emit(&self, frame: OutboundStreamFrame) -> anyhow::Result<()> {
        // Check if the invocation is still active
        let record = self.streams.get_invocation(&self.invocation_id).await;
        if let Some(rec) = &record {
            if rec.is_terminal() {
                // Invocation is already terminal (cancelled/timed out/etc).
                // Signal the cancel so the executor stops.
                let _ = self.cancel_tx.send(true);
                return Ok(());
            }
        }

        // If this is a Done frame, end the invocation instead of appending a chunk
        if frame.kind == OutboundFrameKind::Done {
            // Signal cancel to stop any further emissions
            let _ = self.cancel_tx.send(true);
            return Ok(());
        }

        // Build the chunk payload from the outbound frame
        let payload = json!({
            "invocation_id": self.invocation_id,
            "stream_id": self.stream_id,
            "seq": frame.seq,
            "kind": frame.kind,
            "data_shape": frame.data_shape,
            "bytes_received": frame.bytes_received,
        });

        // Append a chunk frame to the kernel stream
        let _kernel_frame = self
            .streams
            .append_chunk(
                &self.invocation_id,
                payload.clone(),
                RedactionState::Redacted,
            )
            .await?;

        use ygg_core::{new_id, EventEnvelope, EVENT_STREAM_CHUNK, KERNEL_PACKAGE_ID};
        let seq = self.store.next_sequence(&self.session_id).await?;
        self.store
            .append(EventEnvelope {
                id: new_id("evt"),
                session_id: self.session_id.clone(),
                sequence: seq,
                timestamp: chrono::Utc::now(),
                writer_package_id: KERNEL_PACKAGE_ID.to_string(),
                kind: EVENT_STREAM_CHUNK.to_string(),
                schema_version: 1,
                payload: json!({
                    "invocation_id": self.invocation_id,
                    "stream_id": self.stream_id,
                    "outbound_seq": frame.seq,
                    "redaction_state": serde_json::to_value(RedactionState::Redacted)?,
                    "data": payload,
                }),
                metadata: json!({}),
            })
            .await?;

        Ok(())
    }
}

/// Recursively strip raw-secret-like field values from a JSON value
/// before returning it to a protocol caller.
///
/// This is a defense-in-depth sweep: replaces values of known secret
/// field names with `"[redacted]"`. Does not touch non-secret fields.
fn strip_raw_secrets_from_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                if ygg_core::is_secret_field_name(k) {
                    *v = Value::String("[redacted]".to_string());
                } else {
                    strip_raw_secrets_from_value(v);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                strip_raw_secrets_from_value(item);
            }
        }
        _ => {}
    }
}

fn outbound_host_matches(pattern: &str, destination: &str) -> bool {
    if pattern.eq_ignore_ascii_case(destination) {
        return true;
    }
    let pattern = pattern.to_ascii_lowercase();
    let destination = destination.to_ascii_lowercase();
    if let Some(suffix) = pattern.strip_prefix("*.") {
        return destination == suffix
            || destination
                .strip_suffix(suffix)
                .is_some_and(|prefix| prefix.ends_with('.') && prefix.len() > 1);
    }
    false
}

fn validate_outbound_stream_https_only(
    destination_host: &str,
    path: Option<&str>,
    metadata: &Value,
    allow_insecure_loopback_for_tests: bool,
) -> anyhow::Result<()> {
    let base_url = metadata.get("base_url").and_then(Value::as_str);
    let url_str = if let Some(base) = base_url {
        let mut url = base.trim_end_matches('/').to_string();
        if let Some(path) = path {
            if !path.starts_with('/') {
                url.push('/');
            }
            url.push_str(path);
        }
        url
    } else {
        let scheme = metadata
            .get("scheme")
            .and_then(Value::as_str)
            .unwrap_or("https");
        let raw_path = path.unwrap_or("/");
        let path = if raw_path.starts_with('/') {
            raw_path.to_string()
        } else {
            format!("/{raw_path}")
        };
        format!("{scheme}://{destination_host}{path}")
    };

    let url = reqwest::Url::parse(&url_str)
        .map_err(|e| anyhow::anyhow!("invalid outbound.stream URL '{}': {e}", url_str))?;
    let actual_host = url.host_str().unwrap_or("");
    if !actual_host.eq_ignore_ascii_case(destination_host) {
        anyhow::bail!(
            "outbound.stream URL host '{}' does not match destination_host '{}'",
            actual_host,
            destination_host
        );
    }
    if url.scheme() != "https" {
        let is_loopback =
            actual_host == "127.0.0.1" || actual_host == "localhost" || actual_host == "[::1]";
        if !allow_insecure_loopback_for_tests || !is_loopback {
            anyhow::bail!("outbound.stream rejects non-HTTPS URL: {}", url_str);
        }
    }
    Ok(())
}

fn parse_websocket_frame(params: &Value) -> anyhow::Result<OutboundWebSocketFrame> {
    let kind = params.get("kind").and_then(Value::as_str).unwrap_or("text");
    match kind {
        "text" => Ok(OutboundWebSocketFrame::Text(
            params
                .get("text")
                .or_else(|| params.get("data"))
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    anyhow::anyhow!("kernel.v1.outbound.websocket.send requires text data")
                })?
                .to_string(),
        )),
        "binary" => {
            let arr = params
                .get("bytes")
                .or_else(|| params.get("data"))
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    anyhow::anyhow!("kernel.v1.outbound.websocket.send requires binary bytes array")
                })?;
            let mut bytes = Vec::with_capacity(arr.len());
            for value in arr {
                let byte = value.as_u64().ok_or_else(|| {
                    anyhow::anyhow!(
                        "kernel.v1.outbound.websocket.send binary bytes must be integers"
                    )
                })?;
                if byte > u8::MAX as u64 {
                    anyhow::bail!("kernel.v1.outbound.websocket.send binary byte out of range");
                }
                bytes.push(byte as u8);
            }
            Ok(OutboundWebSocketFrame::Binary(Bytes::from(bytes)))
        }
        other => anyhow::bail!("kernel.v1.outbound.websocket.send unknown frame kind '{other}'"),
    }
}

fn websocket_event_to_kernel_event(event: WebSocketEvent) -> (&'static str, Value, bool) {
    match event {
        WebSocketEvent::Opened {
            connection_id,
            subprotocol,
        } => (
            ygg_core::EVENT_OUTBOUND_WEBSOCKET_OPENED,
            json!({"connection_id": connection_id, "subprotocol": subprotocol}),
            false,
        ),
        WebSocketEvent::Frame {
            connection_id,
            direction,
            kind,
            bytes,
            seq,
            payload,
        } => {
            let payload_shape = match payload {
                super::WebSocketFramePayload::Text(text) => {
                    json!({"kind": "text", "bytes": text.len()})
                }
                super::WebSocketFramePayload::Binary(bytes) => {
                    json!({"kind": "binary", "bytes": bytes.len()})
                }
            };
            (
                ygg_core::EVENT_OUTBOUND_WEBSOCKET_FRAME,
                json!({
                    "connection_id": connection_id,
                    "direction": direction,
                    "frame_kind": kind,
                    "bytes": bytes,
                    "seq": seq,
                    "payload_shape": payload_shape,
                }),
                false,
            )
        }
        WebSocketEvent::Error {
            connection_id,
            code,
            message_redacted,
        } => (
            ygg_core::EVENT_OUTBOUND_WEBSOCKET_ERROR,
            json!({"connection_id": connection_id, "error_code": code, "message_redacted": message_redacted}),
            false,
        ),
        WebSocketEvent::Closed {
            connection_id,
            code,
            reason,
            total_frames_in,
            total_frames_out,
            total_bytes_in,
            total_bytes_out,
            duration_ms,
        } => (
            ygg_core::EVENT_OUTBOUND_WEBSOCKET_COMPLETED,
            json!({
                "connection_id": connection_id,
                "code": code,
                "reason": reason,
                "total_frames_in": total_frames_in,
                "total_frames_out": total_frames_out,
                "total_bytes_in": total_bytes_in,
                "total_bytes_out": total_bytes_out,
                "duration_ms": duration_ms,
            }),
            true,
        ),
    }
}

/// L4: Parse `secret_headers` from `kernel.v1.outbound.execute` params.
///
/// Expected format:
/// ```json
/// {
///   "Authorization": {"secret_ref": "secret_ref:env:DEEPSEEK_API_KEY", "scheme": "bearer"},
///   "x-api-key": {"secret_ref": "secret_ref:env:MY_KEY"}
/// }
/// ```
///
/// Each entry declares a header to be injected from a secret_ref, with
/// an optional scheme prefix (e.g. "bearer" → "Bearer <value>").
/// The host resolves these at execution time; raw values are never
/// returned to the caller, persisted in audit, or echoed in errors.
fn parse_secret_headers(params: &Value) -> anyhow::Result<Vec<super::outbound::SecretHeaderSpec>> {
    let secret_headers_value = match params.get("secret_headers") {
        Some(v) => v,
        None => return Ok(Vec::new()),
    };

    let headers_obj = secret_headers_value.as_object().ok_or_else(|| {
        anyhow::anyhow!("kernel.v1.outbound.execute secret_headers must be an object")
    })?;

    let mut specs = Vec::new();
    for (header_name, header_spec) in headers_obj {
        let secret_ref = header_spec
            .get("secret_ref")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow::anyhow!("kernel.v1.outbound.execute secret header requires secret_ref")
            })?
            .to_string();

        if !ygg_core::SecretRef::is_valid_ref(&secret_ref) {
            anyhow::bail!("kernel.v1.outbound.execute secret header secret_ref is invalid");
        }

        let scheme = header_spec
            .get("scheme")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        specs.push(super::outbound::SecretHeaderSpec {
            header_name: header_name.clone(),
            secret_ref,
            scheme,
        });
    }

    Ok(specs)
}

/// L5: Parse `static_headers` from `kernel.v1.outbound.execute` params.
///
/// Expected format:
/// ```json
/// {
///   "anthropic-version": "2023-06-01",
///   "accept": "application/json"
/// }
/// ```
///
/// Each entry declares a safe non-secret header to be injected into the
/// request. Only header names on the `STATIC_HEADER_ALLOWLIST` are
/// permitted; secret-bearing header names (Authorization, x-api-key,
/// Cookie, etc.) are rejected with an error. Values must be plain
/// strings that do not look like raw secrets.
///
/// This allows provider-specific version headers (e.g. Anthropic's
/// `anthropic-version`) without requiring secret resolution, while
/// preventing `static_headers` from becoming a secret bypass path.
fn parse_static_headers(params: &Value) -> anyhow::Result<Vec<super::outbound::StaticHeader>> {
    let static_headers_value = match params.get("static_headers") {
        Some(v) => v,
        None => return Ok(Vec::new()),
    };

    let headers_obj = static_headers_value.as_object().ok_or_else(|| {
        anyhow::anyhow!("kernel.v1.outbound.execute static_headers must be an object")
    })?;

    let mut headers = Vec::new();
    for (header_name, header_value) in headers_obj {
        // Defense-in-depth: reject known secret-bearing header names
        if super::outbound::is_secret_header_name(header_name) {
            anyhow::bail!(
                "kernel.v1.outbound.execute static_headers rejected: '{}' is a secret-bearing header; use secret_headers with secret_ref instead",
                header_name
            );
        }

        // Only allowlisted header names are permitted
        if !super::outbound::is_static_header_allowed(header_name) {
            anyhow::bail!(
                "kernel.v1.outbound.execute static_headers rejected: '{}' is not on the safe header allowlist",
                header_name
            );
        }

        let value = header_value
            .as_str()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "kernel.v1.outbound.execute static_headers value for '{}' must be a string",
                    header_name
                )
            })?
            .to_string();

        // Reject values that look like raw secrets
        if looks_like_raw_secret_value(&value) {
            anyhow::bail!(
                "kernel.v1.outbound.execute static_headers rejected: value for '{}' looks like a raw secret; use secret_headers with secret_ref instead",
                header_name
            );
        }

        headers.push(super::outbound::StaticHeader {
            name: header_name.clone(),
            value,
        });
    }

    Ok(headers)
}

/// Check if a static header value looks like a raw secret.
/// This is a lightweight defense-in-depth check — not a full secret scanner.
fn looks_like_raw_secret_value(value: &str) -> bool {
    if value.starts_with("sk-") || value.starts_with("sk_") {
        return true;
    }
    if value.starts_with("key-") || value.starts_with("key_") {
        return true;
    }
    if value.starts_with("Bearer ") || value.starts_with("bearer ") {
        return true;
    }
    if value.starts_with("AIza") {
        return true;
    }
    // High-entropy alphanumeric strings of length >= 32
    if value.len() >= 32
        && value
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
    {
        let has_upper = value.chars().any(|c| c.is_uppercase());
        let has_lower = value.chars().any(|c| c.is_lowercase());
        let has_digit = value.chars().any(|c| c.is_ascii_digit());
        if has_upper && has_lower && has_digit {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Y2: Dispatch enforcement unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod y2_tests {
    use std::sync::Arc;

    use crate::{
        FakeOutboundExecutor, InMemoryEventStore, OutboundExecutorConfig, ProtocolContext, Runtime,
        RuntimeConfig,
    };
    use ygg_core::{
        CapabilityDescriptor, EntryDescriptor, NetworkDeclaration, NetworkPermissions,
        PackageContributions, PackageEntry, PackageManifest, PermissionSet, SandboxPolicy,
    };

    /// Helper: create a runtime with a FakeOutboundExecutor.
    fn runtime_with_fake() -> (
        Arc<InMemoryEventStore>,
        Runtime<InMemoryEventStore>,
        Arc<FakeOutboundExecutor>,
    ) {
        let store = Arc::new(InMemoryEventStore::default());
        let fake = Arc::new(FakeOutboundExecutor::new());
        let config = RuntimeConfig {
            outbound_executor: OutboundExecutorConfig::Custom(fake.clone()),
            ..RuntimeConfig::default()
        };
        let runtime = Runtime::new(store.clone(), config);
        (store, runtime, fake)
    }

    /// Helper: create a package manifest with network and secret_refs permissions.
    fn package_with_secret_refs(id: &str, secret_refs: Vec<String>) -> PackageManifest {
        PackageManifest {
            schema_version: 1,
            id: id.to_string(),
            version: "0.1.0".to_string(),
            display_name: None,
            description: None,
            author: None,
            license: None,
            entry: EntryDescriptor::v1(PackageEntry::RustInproc {
                crate_ref: "example-echo-rust-inproc".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            }),
            provides: vec![CapabilityDescriptor {
                id: format!("{id}/fetch"),
                version: "0.1.0".to_string(),
                input_schema: serde_json::Value::Null,
                output_schema: serde_json::Value::Null,
                streaming: false,
                side_effects: vec!["network".to_string()],
                description: None,
            }],
            consumes: Vec::new(),
            requires: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet {
                network: NetworkPermissions {
                    declarations: vec![NetworkDeclaration {
                        host: "api.openai.com".to_string(),
                        methods: vec!["POST".to_string()],
                        purpose: Some("test".to_string()),
                    }],
                    hosts: vec![],
                },
                secret_refs,
                ..PermissionSet::default()
            },
            sandbox_policy: SandboxPolicy::default(),
        }
    }

    /// Y2: Undeclared secret_ref in secret_headers is rejected.
    #[tokio::test]
    async fn outbound_execute_secret_ref_undeclared_fails() {
        let (_store, runtime, fake) = runtime_with_fake();
        // Package declares one secret_ref but request uses a different one
        runtime
            .load_package(package_with_secret_refs(
                "example/y2-undeclared",
                vec!["secret_ref:env:DECLARED_KEY".to_string()],
            ))
            .await
            .expect("load package");

        let context = ProtocolContext::package("example/y2-undeclared", "in_process");
        let result = runtime
            .call_protocol(
                &context,
                "kernel.v1.outbound.execute",
                serde_json::json!({
                    "capability_id": "example/y2-undeclared/fetch",
                    "destination_host": "api.openai.com",
                    "method": "POST",
                    "secret_headers": {
                        "Authorization": {
                            "secret_ref": "secret_ref:env:UNDECLARED_KEY",
                            "scheme": "bearer"
                        }
                    }
                }),
            )
            .await;

        assert!(result.is_err(), "undeclared secret_ref should be denied");
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(
            err_msg.contains("not declared"),
            "error should mention undeclared secret_ref, got: {err_msg}"
        );
        assert_eq!(
            fake.call_count(),
            0,
            "executor should not be called for undeclared secret_ref"
        );
    }

    /// Y2: Declared secret_ref is allowed to proceed.
    #[tokio::test]
    async fn outbound_execute_secret_ref_declared_resolves() {
        let (_store, runtime, _fake) = runtime_with_fake();
        runtime
            .load_package(package_with_secret_refs(
                "example/y2-declared",
                vec!["secret_ref:env:MY_API_KEY".to_string()],
            ))
            .await
            .expect("load package");

        let context = ProtocolContext::package("example/y2-declared", "in_process");

        // Note: secret resolution will fail (no resolver configured), but
        // the Y2 check happens BEFORE resolution. The error should be from
        // the resolver, not from the undeclared check.
        let result = runtime
            .call_protocol(
                &context,
                "kernel.v1.outbound.execute",
                serde_json::json!({
                    "capability_id": "example/y2-declared/fetch",
                    "destination_host": "api.openai.com",
                    "method": "POST",
                    "secret_headers": {
                        "Authorization": {
                            "secret_ref": "secret_ref:env:MY_API_KEY",
                            "scheme": "bearer"
                        }
                    }
                }),
            )
            .await;

        // The Y2 declaration check passes, but secret resolution may fail
        // (DenyAllSecretResolver is the default). The key point is we
        // should NOT get the "not declared" error.
        if let Err(e) = &result {
            let err_msg = format!("{:?}", e);
            assert!(
                !err_msg.contains("not declared"),
                "declared secret_ref should not produce 'not declared' error, got: {err_msg}"
            );
        }
        // Executor may or may not be called depending on resolver success,
        // but the Y2 check should not block it.
    }

    /// Y2: Request without secret_headers skips the manifest check.
    #[tokio::test]
    async fn outbound_execute_no_secret_headers_no_check_required() {
        let (_store, runtime, fake) = runtime_with_fake();
        // Package has no secret_refs declared, but also doesn't use any
        runtime
            .load_package(package_with_secret_refs("example/y2-no-secret", vec![]))
            .await
            .expect("load package");

        let context = ProtocolContext::package("example/y2-no-secret", "in_process");
        let result = runtime
            .call_protocol(
                &context,
                "kernel.v1.outbound.execute",
                serde_json::json!({
                    "capability_id": "example/y2-no-secret/fetch",
                    "destination_host": "api.openai.com",
                    "method": "POST",
                }),
            )
            .await;

        // Should succeed (fake executor returns ok)
        assert!(
            result.is_ok(),
            "request without secret_headers should succeed, got: {:?}",
            result.err()
        );
        assert_eq!(fake.call_count(), 1, "executor should be called");
    }

    /// Y2: Multiple secret_refs must all be declared.
    #[tokio::test]
    async fn outbound_execute_multiple_secret_refs_all_must_be_declared() {
        let (_store, runtime, fake) = runtime_with_fake();
        // Declare only one of two needed refs
        runtime
            .load_package(package_with_secret_refs(
                "example/y2-multi",
                vec!["secret_ref:env:KEY_A".to_string()],
            ))
            .await
            .expect("load package");

        let context = ProtocolContext::package("example/y2-multi", "in_process");
        let result = runtime
            .call_protocol(
                &context,
                "kernel.v1.outbound.execute",
                serde_json::json!({
                    "capability_id": "example/y2-multi/fetch",
                    "destination_host": "api.openai.com",
                    "method": "POST",
                    "secret_refs": ["secret_ref:env:KEY_A", "secret_ref:env:KEY_B"],
                }),
            )
            .await;

        assert!(
            result.is_err(),
            "undeclared second secret_ref should be denied"
        );
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(
            err_msg.contains("not declared"),
            "error should mention undeclared secret_ref, got: {err_msg}"
        );
        assert_eq!(
            fake.call_count(),
            0,
            "executor should not be called when any secret_ref is undeclared"
        );
    }

    /// Y2: Top-level secret_refs also require manifest declaration.
    #[tokio::test]
    async fn outbound_execute_top_level_secret_ref_undeclared_fails() {
        let (_store, runtime, fake) = runtime_with_fake();
        runtime
            .load_package(package_with_secret_refs("example/y2-toplevel", vec![]))
            .await
            .expect("load package");

        let context = ProtocolContext::package("example/y2-toplevel", "in_process");
        let result = runtime
            .call_protocol(
                &context,
                "kernel.v1.outbound.execute",
                serde_json::json!({
                    "capability_id": "example/y2-toplevel/fetch",
                    "destination_host": "api.openai.com",
                    "method": "POST",
                    "secret_refs": ["secret_ref:env:UNDECLARED"],
                }),
            )
            .await;

        assert!(
            result.is_err(),
            "top-level undeclared secret_ref should be denied"
        );
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(
            err_msg.contains("not declared"),
            "error should mention undeclared, got: {err_msg}"
        );
        assert_eq!(fake.call_count(), 0, "executor should not be called");
    }
}

#[cfg(test)]
mod z_websocket_tests {
    use std::sync::Arc;

    use crate::{
        EventStore, FakeOutboundExecutor, FakeWebSocketExecutor, InMemoryEventStore,
        OutboundExecutePolicyConfig, OutboundExecutorConfig, OutboundExecutorResponse,
        ProtocolContext, Runtime, RuntimeConfig,
    };
    use ygg_core::{
        CapabilityDescriptor, EntryDescriptor, NetworkDeclaration, NetworkPermissions,
        PackageContributions, PackageEntry, PackageManifest, PermissionSet, SandboxPolicy,
    };

    fn runtime_with_fake_ws() -> (
        Arc<InMemoryEventStore>,
        Runtime<InMemoryEventStore>,
        Arc<FakeWebSocketExecutor>,
    ) {
        let store = Arc::new(InMemoryEventStore::default());
        let fake = Arc::new(FakeWebSocketExecutor::new());
        let config = RuntimeConfig {
            outbound_websocket_executor: fake.clone(),
            ..RuntimeConfig::default()
        };
        let runtime = Runtime::new(store.clone(), config);
        (store, runtime, fake)
    }

    fn package_ws(id: &str, secret_refs: Vec<String>) -> PackageManifest {
        PackageManifest {
            schema_version: 1,
            id: id.to_string(),
            version: "0.1.0".to_string(),
            display_name: None,
            description: None,
            author: None,
            license: None,
            entry: EntryDescriptor::v1(PackageEntry::RustInproc {
                crate_ref: "example-echo-rust-inproc".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            }),
            provides: vec![CapabilityDescriptor {
                id: format!("{id}/ws"),
                version: "0.1.0".to_string(),
                input_schema: serde_json::Value::Null,
                output_schema: serde_json::Value::Null,
                streaming: true,
                side_effects: vec!["network".to_string()],
                description: None,
            }],
            consumes: Vec::new(),
            requires: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet {
                network: NetworkPermissions {
                    declarations: vec![NetworkDeclaration {
                        host: "api.example.com".to_string(),
                        methods: vec!["WEBSOCKET".to_string()],
                        purpose: Some("test websocket".to_string()),
                    }],
                    hosts: vec![],
                },
                secret_refs,
                ..PermissionSet::default()
            },
            sandbox_policy: SandboxPolicy::default(),
        }
    }

    #[tokio::test]
    async fn dispatch_outbound_websocket_open_namespace_enforced() {
        let (_store, runtime, _fake) = runtime_with_fake_ws();
        runtime
            .load_package(package_ws("example/ws-ns", vec![]))
            .await
            .expect("load package");
        let context = ProtocolContext::package("example/ws-ns", "in_process");
        let result = runtime
            .call_protocol(
                &context,
                "kernel.v1.outbound.websocket.open",
                serde_json::json!({
                    "capability_id": "other/pkg/ws",
                    "destination_host": "api.example.com"
                }),
            )
            .await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("namespace"));
    }

    #[tokio::test]
    async fn dispatch_outbound_websocket_open_secret_ref_undeclared_fails() {
        let (_store, runtime, _fake) = runtime_with_fake_ws();
        runtime
            .load_package(package_ws("example/ws-secret", vec![]))
            .await
            .expect("load package");
        let context = ProtocolContext::package("example/ws-secret", "in_process");
        let result = runtime
            .call_protocol(
                &context,
                "kernel.v1.outbound.websocket.open",
                serde_json::json!({
                    "capability_id": "example/ws-secret/ws",
                    "destination_host": "api.example.com",
                    "secret_refs": ["secret_ref:env:MISSING"]
                }),
            )
            .await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("not declared"));
    }

    #[tokio::test]
    async fn dispatch_outbound_websocket_open_with_fake_executor_emits_opened() {
        let (store, runtime, _fake) = runtime_with_fake_ws();
        runtime
            .load_package(package_ws("example/ws-ok", vec![]))
            .await
            .expect("load package");
        let context = ProtocolContext::package("example/ws-ok", "in_process");
        let result = runtime
            .call_protocol(
                &context,
                "kernel.v1.outbound.websocket.open",
                serde_json::json!({
                    "capability_id": "example/ws-ok/ws",
                    "destination_host": "api.example.com",
                    "subprotocols": ["json"]
                }),
            )
            .await
            .expect("open websocket");
        let connection_id = result
            .get("connection_id")
            .and_then(serde_json::Value::as_str)
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        let events = store
            .list_kind_prefix(ygg_core::EVENT_OUTBOUND_WEBSOCKET_OPENED)
            .await
            .unwrap();
        assert!(events.iter().any(|event| event
            .payload
            .get("connection_id")
            .and_then(serde_json::Value::as_str)
            == Some(connection_id)));
    }

    fn runtime_with_fake_execute(
        fake: Arc<FakeOutboundExecutor>,
    ) -> (Arc<InMemoryEventStore>, Runtime<InMemoryEventStore>) {
        let store = Arc::new(InMemoryEventStore::default());
        let config = RuntimeConfig {
            outbound_executor: OutboundExecutorConfig::Custom(fake),
            outbound_execute_policy: OutboundExecutePolicyConfig {
                enabled: true,
                allowed_hosts: vec!["api.example.com".to_string()],
                https_only: true,
                timeout_ms: 30_000,
                allow_redirects: false,
                allow_insecure_loopback_for_tests: false,
            },
            ..RuntimeConfig::default()
        };
        (store.clone(), Runtime::new(store, config))
    }

    async fn wait_for_event(store: &InMemoryEventStore, kind: &str) -> serde_json::Value {
        for _ in 0..40 {
            let events = store.list_kind_prefix(kind).await.unwrap();
            if let Some(event) = events.last() {
                return event.payload.clone();
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        panic!("event {kind} not found");
    }

    #[tokio::test]
    async fn outbound_execute_emits_completed_event_on_success() {
        let fake = Arc::new(FakeOutboundExecutor::new());
        let (store, runtime) = runtime_with_fake_execute(fake);
        runtime
            .load_package(package_ws("example/z6-exec-ok", vec![]))
            .await
            .unwrap();
        let context = ProtocolContext::package("example/z6-exec-ok", "in_process");
        let _ = runtime
            .call_protocol(
                &context,
                "kernel.v1.outbound.execute",
                serde_json::json!({
                    "capability_id": "example/z6-exec-ok/ws",
                    "destination_host": "api.example.com",
                    "method": "WEBSOCKET"
                }),
            )
            .await
            .unwrap();
        let payload = wait_for_event(&store, ygg_core::EVENT_OUTBOUND_EXECUTE_COMPLETED).await;
        assert_eq!(payload["status"], "ok");
        assert_eq!(payload["executor_kind"], "fake");
    }

    #[tokio::test]
    async fn outbound_execute_emits_completed_event_on_error() {
        let fake = Arc::new(FakeOutboundExecutor::with_fixture(
            "api.example.com",
            "WEBSOCKET",
            None,
            OutboundExecutorResponse {
                status: "error".to_string(),
                status_code: Some(500),
                headers_shape: None,
                body_shape: None,
                provider_request_id: None,
                usage: serde_json::Value::Null,
                cost: serde_json::Value::Null,
                redaction_state: ygg_core::RedactionState::Redacted,
                network_performed: false,
                executor_kind: crate::ExecutorKind::Fake,
            },
        ));
        let (store, runtime) = runtime_with_fake_execute(fake);
        runtime
            .load_package(package_ws("example/z6-exec-error", vec![]))
            .await
            .unwrap();
        let context = ProtocolContext::package("example/z6-exec-error", "in_process");
        let _ = runtime
            .call_protocol(
                &context,
                "kernel.v1.outbound.execute",
                serde_json::json!({
                    "capability_id": "example/z6-exec-error/ws",
                    "destination_host": "api.example.com",
                    "method": "WEBSOCKET"
                }),
            )
            .await
            .unwrap();
        let payload = wait_for_event(&store, ygg_core::EVENT_OUTBOUND_EXECUTE_COMPLETED).await;
        assert_eq!(payload["status"], "error");
    }

    #[tokio::test]
    async fn outbound_execute_emits_completed_event_on_denied() {
        let fake = Arc::new(FakeOutboundExecutor::new());
        let (store, runtime) = runtime_with_fake_execute(fake);
        runtime
            .load_package(package_ws("example/z6-exec-denied", vec![]))
            .await
            .unwrap();
        let context = ProtocolContext::package("example/z6-exec-denied", "in_process");
        let result = runtime
            .call_protocol(
                &context,
                "kernel.v1.outbound.execute",
                serde_json::json!({
                    "capability_id": "example/z6-exec-denied/ws",
                    "destination_host": "denied.example.com",
                    "method": "WEBSOCKET"
                }),
            )
            .await;
        assert!(result.is_err());
        let payload = wait_for_event(&store, ygg_core::EVENT_OUTBOUND_EXECUTE_COMPLETED).await;
        assert_eq!(payload["status"], "denied");
    }

    #[tokio::test]
    async fn outbound_stream_emits_completed_event_on_ended() {
        let fake = Arc::new(FakeOutboundExecutor::new());
        let (store, runtime) = runtime_with_fake_execute(fake);
        runtime
            .load_package(package_ws("example/z6-stream-ended", vec![]))
            .await
            .unwrap();
        let context = ProtocolContext::package("example/z6-stream-ended", "in_process");
        let _ = runtime
            .call_protocol(
                &context,
                "kernel.v1.outbound.stream",
                serde_json::json!({
                    "capability_id": "example/z6-stream-ended/ws",
                    "destination_host": "api.example.com",
                    "method": "WEBSOCKET",
                    "stream_format": "sse"
                }),
            )
            .await
            .unwrap();
        let payload = wait_for_event(&store, ygg_core::EVENT_OUTBOUND_STREAM_COMPLETED).await;
        assert_eq!(payload["status"], "ok");
        assert_eq!(payload["final_termination"], "ended");
    }

    #[tokio::test]
    async fn outbound_stream_emits_completed_event_on_cancelled() {
        let fake = Arc::new(FakeOutboundExecutor::new());
        let (store, runtime) = runtime_with_fake_execute(fake);
        runtime
            .load_package(package_ws("example/z6-stream-cancel", vec![]))
            .await
            .unwrap();
        let context = ProtocolContext::package("example/z6-stream-cancel", "in_process");
        let response = runtime
            .call_protocol(
                &context,
                "kernel.v1.outbound.stream",
                serde_json::json!({
                    "capability_id": "example/z6-stream-cancel/ws",
                    "destination_host": "api.example.com",
                    "method": "WEBSOCKET",
                    "stream_format": "sse"
                }),
            )
            .await
            .unwrap();
        let stream_id = response["stream_id"].as_str().unwrap();
        runtime
            .call_protocol(
                &context,
                "kernel.v1.capability.cancel",
                serde_json::json!({
                    "stream_id": stream_id,
                    "session_id": "kernel_outbound_stream_example_z6-stream-cancel"
                }),
            )
            .await
            .unwrap();
        let payload = wait_for_event(&store, ygg_core::EVENT_OUTBOUND_STREAM_COMPLETED).await;
        assert_eq!(payload["status"], "cancelled");
        assert_eq!(payload["final_termination"], "cancelled");
    }

    #[tokio::test]
    async fn outbound_websocket_emits_completed_event_on_close() {
        let fake = Arc::new(FakeWebSocketExecutor::with_canned_inbound_frames(vec![
            crate::OutboundWebSocketFrame::Text("hello".to_string()),
        ]));
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(
            store.clone(),
            RuntimeConfig {
                outbound_websocket_executor: fake,
                ..RuntimeConfig::default()
            },
        );
        runtime
            .load_package(package_ws("example/z6-ws-close", vec![]))
            .await
            .unwrap();
        let context = ProtocolContext::package("example/z6-ws-close", "in_process");
        let _ = runtime
            .call_protocol(
                &context,
                "kernel.v1.outbound.websocket.open",
                serde_json::json!({
                    "capability_id": "example/z6-ws-close/ws",
                    "destination_host": "api.example.com"
                }),
            )
            .await
            .unwrap();
        let payload = wait_for_event(&store, ygg_core::EVENT_OUTBOUND_WEBSOCKET_COMPLETED).await;
        assert_eq!(payload["package_id"], "example/z6-ws-close");
        assert_eq!(payload["total_frames_in"], 1);
    }

    #[tokio::test]
    async fn outbound_completion_event_has_no_secrets_and_redaction_state_set() {
        let env_name = format!("YGG_Z6_SECRET_{}", std::process::id());
        std::env::set_var(&env_name, "super-secret-value");
        struct Guard(String);
        impl Drop for Guard {
            fn drop(&mut self) {
                std::env::remove_var(&self.0);
            }
        }
        let _guard = Guard(env_name.clone());
        let fake = Arc::new(FakeOutboundExecutor::new());
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(
            store.clone(),
            RuntimeConfig {
                outbound_executor: OutboundExecutorConfig::Custom(fake),
                outbound_execute_policy: OutboundExecutePolicyConfig {
                    enabled: true,
                    allowed_hosts: vec!["api.example.com".to_string()],
                    https_only: true,
                    timeout_ms: 30_000,
                    allow_redirects: false,
                    allow_insecure_loopback_for_tests: false,
                },
                secret_resolver: crate::SecretResolverConfig::with_resolver(Arc::new(
                    crate::EnvSecretResolver::from_iter(vec![env_name.clone()]),
                )),
                ..RuntimeConfig::default()
            },
        );
        let secret_ref = format!("secret_ref:env:{env_name}");
        runtime
            .load_package(package_ws("example/z6-secret", vec![secret_ref.clone()]))
            .await
            .unwrap();
        let context = ProtocolContext::package("example/z6-secret", "in_process");
        let _ = runtime.call_protocol(&context, "kernel.v1.outbound.execute", serde_json::json!({
            "capability_id": "example/z6-secret/ws",
            "destination_host": "api.example.com",
            "method": "WEBSOCKET",
            "secret_headers": {"Authorization": {"secret_ref": secret_ref, "scheme": "bearer"}}
        })).await.unwrap();
        let payload = wait_for_event(&store, ygg_core::EVENT_OUTBOUND_EXECUTE_COMPLETED).await;
        let text = serde_json::to_string(&payload).unwrap();
        assert!(text.contains("secret_ref:env:"));
        assert!(!text.contains("super-secret-value"));
        assert_eq!(payload["redaction_state"], "redacted");
    }

    #[tokio::test]
    async fn outbound_completion_event_no_payload_in_websocket() {
        let fake = Arc::new(FakeWebSocketExecutor::with_canned_inbound_frames(vec![
            crate::OutboundWebSocketFrame::Text("raw-frame-payload".to_string()),
        ]));
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(
            store.clone(),
            RuntimeConfig {
                outbound_websocket_executor: fake,
                ..RuntimeConfig::default()
            },
        );
        runtime
            .load_package(package_ws("example/z6-ws-scrub", vec![]))
            .await
            .unwrap();
        let context = ProtocolContext::package("example/z6-ws-scrub", "in_process");
        let _ = runtime
            .call_protocol(
                &context,
                "kernel.v1.outbound.websocket.open",
                serde_json::json!({
                    "capability_id": "example/z6-ws-scrub/ws",
                    "destination_host": "api.example.com"
                }),
            )
            .await
            .unwrap();
        let payload = wait_for_event(&store, ygg_core::EVENT_OUTBOUND_WEBSOCKET_COMPLETED).await;
        assert!(payload.get("payload").is_none());
        assert!(payload.get("body").is_none());
        assert!(payload.get("data").is_none());
        assert!(!serde_json::to_string(&payload)
            .unwrap()
            .contains("raw-frame-payload"));
    }
}
