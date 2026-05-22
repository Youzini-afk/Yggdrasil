use reqwest::header::{HeaderName, HeaderValue};
use serde_json::{json, Value};

use super::Runtime;
use crate::{EventStore, KernelMethod, ProtocolContext, ProtocolPrincipal, EventListRequest};

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
            // Host domain
            KernelMethod::HostInfo => Ok(serde_json::to_value(crate::host_info())?),
            KernelMethod::HostPing => Ok(json!({"ok": true})),
            KernelMethod::HostDiagnostics => Ok(self.host_diagnostics().await),

            // Surface domain
            KernelMethod::SurfaceContributionList => self.dispatch_surface_list(&params).await,
            KernelMethod::SurfaceContributionDescribe => self.dispatch_surface_describe(&params).await,

            // Outbound domain
            KernelMethod::OutboundAudit => self.dispatch_outbound_audit(&params).await,
            KernelMethod::OutboundExecute => self.dispatch_outbound_execute(context, params).await,
            KernelMethod::OutboundGitFetch => self.dispatch_outbound_git_fetch(context, params).await,

            // Permission domain
            KernelMethod::PermissionGrant => self.dispatch_permission_grant(&params).await,
            KernelMethod::PermissionRevoke => self.dispatch_permission_revoke(&params).await,
            KernelMethod::PermissionList => self.dispatch_permission_list(&params).await,
            KernelMethod::PermissionAudit => self.dispatch_permission_audit().await,

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

            // Event domain
            KernelMethod::EventAppend => Ok(serde_json::to_value(
                self.append_event_with_context(context, serde_json::from_value(params)?).await?,
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

            // Capability domain
            KernelMethod::CapabilityDiscover => Ok(serde_json::to_value(self.discover_capabilities().await)?),
            KernelMethod::CapabilityInvoke => Ok(serde_json::to_value(
                self.invoke_capability_with_context(context, serde_json::from_value(params)?).await?,
            )?),
            KernelMethod::CapabilityStream => self.dispatch_capability_stream(&params).await,
            KernelMethod::CapabilityCancel => self.dispatch_capability_cancel(&params).await,

            // Extension / hook domain
            KernelMethod::ExtensionPointList => Ok(json!([
                "kernel/event.before_append",
                "kernel/event.after_append",
                "kernel/capability.before_invoke",
                "kernel/capability.after_invoke",
                "kernel/package.loaded",
                "kernel/package.unloaded"
            ])),
            KernelMethod::HookList => Ok(serde_json::to_value(self.extensions.list_all_hooks().await)?),

            // Asset domain
            KernelMethod::AssetPut => Ok(serde_json::to_value(self.put_asset(serde_json::from_value(params)?).await?)?),
            KernelMethod::AssetGet => self.dispatch_asset_get(&params).await,
            KernelMethod::AssetList => Ok(serde_json::to_value(self.list_assets().await)?),

            // Projection domain
            KernelMethod::ProjectionRegister => Ok(serde_json::to_value(self.projection_register(serde_json::from_value(params)?).await?)?),
            KernelMethod::ProjectionRebuild => self.dispatch_projection_rebuild(&params).await,
            KernelMethod::ProjectionGet => self.dispatch_projection_get(&params).await,
            KernelMethod::ProjectionList => Ok(serde_json::to_value(self.projection_list().await)?),

            // Planned methods — no dispatch yet
            KernelMethod::SessionGet
            | KernelMethod::SessionList
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

    async fn dispatch_surface_list(&self, params: &Value) -> anyhow::Result<Value> {
        let slot = params.get("slot").and_then(Value::as_str).map(str::to_string);
        Ok(self.list_surface_contributions(slot).await)
    }

    async fn dispatch_surface_describe(&self, params: &Value) -> anyhow::Result<Value> {
        let surface_id = params
            .get("surface_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.surface.contribution.describe requires surface_id"))?;
        self.describe_surface_contribution(surface_id).await
    }

    // --- Outbound ---

    async fn dispatch_outbound_audit(&self, params: &Value) -> anyhow::Result<Value> {
        let package_id = params
            .get("package_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.outbound.audit requires package_id"))?
            .to_string();
        Ok(serde_json::to_value(self.list_outbound_audit(&package_id).await?)?)
    }

    async fn dispatch_outbound_execute(&self, context: &ProtocolContext, params: Value) -> anyhow::Result<Value> {
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
                    "kernel.outbound.execute requires package or host principal, got {:?}",
                    other
                )
            }
        };

        let capability_id = params
            .get("capability_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.outbound.execute requires capability_id"))?
            .to_string();
        if !capability_id.starts_with(&format!("{package_id}/")) {
            anyhow::bail!(
                "kernel.outbound.execute capability_id must belong to the caller package namespace"
            );
        }
        let destination_host = params
            .get("destination_host")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.outbound.execute requires destination_host"))?
            .to_string();
        let method = params
            .get("method")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.outbound.execute requires method"))?
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
        let metadata = params
            .get("metadata")
            .cloned()
            .unwrap_or(Value::Null);
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
        if !all_secret_refs.is_empty() {
            let manifest = self.packages.manifest(&package_id).await.ok_or_else(|| {
                anyhow::anyhow!(
                    "kernel.outbound.execute package '{}' is not loaded",
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
        };

        // L4: Resolve secret_headers into resolved_secret_headers
        let mut resolved_secret_headers = Vec::new();
        for spec in &secret_headers_spec {
            HeaderName::from_bytes(spec.header_name.as_bytes()).map_err(|_| {
                anyhow::anyhow!("kernel.outbound.execute secret header name is invalid")
            })?;
            let raw_value = self.resolve_secret_ref(&spec.secret_ref).await.map_err(|_| {
                anyhow::anyhow!("kernel.outbound.execute secret header is unavailable")
            })?;
            let header_value = match spec.scheme.to_lowercase().as_str() {
                "bearer" => format!("Bearer {}", raw_value),
                "basic" => format!("Basic {}", raw_value),
                "raw" | "" => raw_value,
                other => format!("{} {}", other, raw_value),
            };
            HeaderValue::from_str(&header_value).map_err(|_| {
                anyhow::anyhow!("kernel.outbound.execute secret header value is invalid")
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
            secret_refs: all_secret_refs,
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

    async fn dispatch_outbound_git_fetch(&self, context: &ProtocolContext, params: Value) -> anyhow::Result<Value> {
        let package_id = match &context.principal {
            ProtocolPrincipal::Package { package_id } => package_id.clone(),
            ProtocolPrincipal::HostAdmin | ProtocolPrincipal::HostDev => params
                .get("package_id")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| "host/test".to_string()),
            other => anyhow::bail!(
                "kernel.outbound.git_fetch requires package or host principal, got {:?}",
                other
            ),
        };

        let capability_id = params
            .get("capability_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.outbound.git_fetch requires capability_id"))?
            .to_string();
        let remote_url = params
            .get("remote_url")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.outbound.git_fetch requires remote_url"))?
            .to_string();
        let reference = params
            .get("ref")
            .or_else(|| params.get("reference"))
            .and_then(Value::as_str)
            .unwrap_or("main")
            .to_string();
        let fetch_kind = match params.get("fetch_kind").and_then(Value::as_str).unwrap_or("refs_only") {
            "refs_only" => super::GitFetchKind::RefsOnly,
            "tree_only" => super::GitFetchKind::TreeOnly,
            "shallow_clone" => super::GitFetchKind::ShallowClone,
            other => anyhow::bail!("kernel.outbound.git_fetch unknown fetch_kind '{other}'"),
        };
        let secret_refs: Vec<String> = params
            .get("secret_refs")
            .and_then(Value::as_array)
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        let request = super::GitOutboundRequest {
            package_id,
            capability_id,
            remote_url,
            reference,
            fetch_kind,
            destination_hint: params.get("destination_hint").and_then(Value::as_str).map(str::to_string),
            secret_refs,
            redaction_state: None,
            timeout_ms: params.get("timeout_ms").and_then(Value::as_u64),
            metadata: params.get("metadata").cloned().unwrap_or(Value::Null),
        };

        let response = self.execute_git_outbound_with_policy(context.principal.clone(), request).await?;
        let mut response_value = serde_json::to_value(&response)?;
        strip_raw_secrets_from_value(&mut response_value);
        Ok(response_value)
    }

    // --- Permission ---

    async fn dispatch_permission_grant(&self, params: &Value) -> anyhow::Result<Value> {
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

    async fn dispatch_permission_revoke(&self, params: &Value) -> anyhow::Result<Value> {
        let grant_id = params
            .get("grant_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.permission.revoke requires grant_id"))?;
        Ok(serde_json::to_value(self.revoke_permission(grant_id).await?)?)
    }

    async fn dispatch_permission_list(&self, params: &Value) -> anyhow::Result<Value> {
        let principal = match params.get("principal") {
            Some(value) => Some(serde_json::from_value(value.clone())?),
            None => None,
        };
        Ok(serde_json::to_value(self.list_permission_grants(principal).await)?)
    }

    async fn dispatch_permission_audit(&self) -> anyhow::Result<Value> {
        let events = self
            .store
            .list_kind_prefix("kernel/permission")
            .await?;
        Ok(serde_json::to_value(events)?)
    }

    // --- Proposal ---

    async fn dispatch_proposal_create(&self, context: &ProtocolContext, params: &Value) -> anyhow::Result<Value> {
        let proposal: super::ProposalRecord = serde_json::from_value(params.clone())?;
        Ok(serde_json::to_value(self.create_proposal(context, proposal).await?)?)
    }

    async fn dispatch_proposal_get(&self, params: &Value) -> anyhow::Result<Value> {
        let proposal_id = params
            .get("proposal_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.proposal.get requires proposal_id"))?;
        Ok(serde_json::to_value(self.get_proposal(proposal_id).await?)?)
    }

    async fn dispatch_proposal_list(&self) -> anyhow::Result<Value> {
        Ok(serde_json::to_value(self.list_proposals().await)?)
    }

    async fn dispatch_proposal_approve(&self, context: &ProtocolContext, params: &Value) -> anyhow::Result<Value> {
        let proposal_id = params
            .get("proposal_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.proposal.approve requires proposal_id"))?;
        let reason = params.get("reason").and_then(Value::as_str).map(str::to_string);
        Ok(serde_json::to_value(self.approve_proposal(context, proposal_id, reason).await?)?)
    }

    async fn dispatch_proposal_reject(&self, context: &ProtocolContext, params: &Value) -> anyhow::Result<Value> {
        let proposal_id = params
            .get("proposal_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.proposal.reject requires proposal_id"))?;
        let reason = params.get("reason").and_then(Value::as_str).map(str::to_string);
        Ok(serde_json::to_value(self.reject_proposal(context, proposal_id, reason).await?)?)
    }

    async fn dispatch_proposal_apply(&self, params: &Value) -> anyhow::Result<Value> {
        let proposal_id = params
            .get("proposal_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.proposal.apply requires proposal_id"))?;
        Ok(serde_json::to_value(self.apply_proposal(proposal_id).await?)?)
    }

    // --- Session ---

    async fn dispatch_session_close(&self, params: &Value) -> anyhow::Result<Value> {
        let session_id = params
            .get("session_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.session.close requires session_id"))?
            .to_string();
        Ok(serde_json::to_value(self.close_session(session_id).await?)?)
    }

    async fn dispatch_session_fork(&self, params: &Value) -> anyhow::Result<Value> {
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

    async fn dispatch_session_branch_list(&self, params: &Value) -> anyhow::Result<Value> {
        let session_id = params
            .get("session_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.session.branch.list requires session_id"))?
            .to_string();
        Ok(serde_json::to_value(self.list_branches(&session_id).await)?)
    }

    // --- Event ---

    async fn dispatch_event_list(&self, context: &ProtocolContext, params: &Value) -> anyhow::Result<Value> {
        let request: EventListRequest = serde_json::from_value(params.clone())?;
        Ok(serde_json::to_value(self.list_events_range_with_context(context, &request).await?)?)
    }

    // --- Package ---

    async fn dispatch_package_status(&self, params: &Value) -> anyhow::Result<Value> {
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

    async fn dispatch_package_unload(&self, params: &Value) -> anyhow::Result<Value> {
        let package_id = params
            .get("package_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.package.unload requires package_id"))?
            .to_string();
        Ok(serde_json::to_value(self.unload_package(&package_id).await?)?)
    }

    async fn dispatch_package_restart(&self, params: &Value) -> anyhow::Result<Value> {
        let package_id = params
            .get("package_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.package.restart requires package_id"))?
            .to_string();
        Ok(serde_json::to_value(self.restart_package(&package_id).await?)?)
    }

    async fn dispatch_package_logs(&self, params: &Value) -> anyhow::Result<Value> {
        let package_id = params
            .get("package_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.package.logs requires package_id"))?
            .to_string();
        Ok(serde_json::to_value(self.package_logs(&package_id).await)?)
    }

    // --- Capability ---

    async fn dispatch_capability_stream(&self, params: &Value) -> anyhow::Result<Value> {
        let capability_id = params
            .get("capability_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.capability.stream requires capability_id"))?
            .to_string();
        let session_id = params
            .get("session_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.capability.stream requires session_id"))?
            .to_string();
        let provider_package_id: Option<String> = params.get("provider_package_id").and_then(Value::as_str).map(String::from);
        let version: Option<String> = params.get("version").and_then(Value::as_str).map(String::from);
        let metadata = params.get("metadata").cloned().unwrap_or_else(|| serde_json::json!({}));
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
        let invocation_id = params
            .get("invocation_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.capability.cancel requires invocation_id"))?
            .to_string();
        let session_id = params
            .get("session_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.capability.cancel requires session_id"))?
            .to_string();
        let frame = self.stream_capability_cancel(&session_id, &invocation_id).await?;
        Ok(serde_json::to_value(frame)?)
    }

    // --- Asset ---

    async fn dispatch_asset_get(&self, params: &Value) -> anyhow::Result<Value> {
        let asset_id = params
            .get("asset_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.asset.get requires asset_id"))?;
        Ok(serde_json::to_value(self.get_asset(asset_id).await?)?)
    }

    // --- Projection ---

    async fn dispatch_projection_rebuild(&self, params: &Value) -> anyhow::Result<Value> {
        let projection_id = params
            .get("projection_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.projection.rebuild requires projection_id"))?;
        Ok(serde_json::to_value(self.projection_rebuild(projection_id).await?)?)
    }

    async fn dispatch_projection_get(&self, params: &Value) -> anyhow::Result<Value> {
        let projection_id = params
            .get("projection_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.projection.get requires projection_id"))?;
        Ok(serde_json::to_value(self.projection_get(projection_id).await?)?)
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

/// L4: Parse `secret_headers` from `kernel.outbound.execute` params.
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

    let headers_obj = secret_headers_value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("kernel.outbound.execute secret_headers must be an object"))?;

    let mut specs = Vec::new();
    for (header_name, header_spec) in headers_obj {
        let secret_ref = header_spec
            .get("secret_ref")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("kernel.outbound.execute secret header requires secret_ref"))?
            .to_string();

        if !ygg_core::SecretRef::is_valid_ref(&secret_ref) {
            anyhow::bail!("kernel.outbound.execute secret header secret_ref is invalid");
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

/// L5: Parse `static_headers` from `kernel.outbound.execute` params.
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

    let headers_obj = static_headers_value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("kernel.outbound.execute static_headers must be an object"))?;

    let mut headers = Vec::new();
    for (header_name, header_value) in headers_obj {
        // Defense-in-depth: reject known secret-bearing header names
        if super::outbound::is_secret_header_name(header_name) {
            anyhow::bail!(
                "kernel.outbound.execute static_headers rejected: '{}' is a secret-bearing header; use secret_headers with secret_ref instead",
                header_name
            );
        }

        // Only allowlisted header names are permitted
        if !super::outbound::is_static_header_allowed(header_name) {
            anyhow::bail!(
                "kernel.outbound.execute static_headers rejected: '{}' is not on the safe header allowlist",
                header_name
            );
        }

        let value = header_value
            .as_str()
            .ok_or_else(|| anyhow::anyhow!(
                "kernel.outbound.execute static_headers value for '{}' must be a string",
                header_name
            ))?
            .to_string();

        // Reject values that look like raw secrets
        if looks_like_raw_secret_value(&value) {
            anyhow::bail!(
                "kernel.outbound.execute static_headers rejected: value for '{}' looks like a raw secret; use secret_headers with secret_ref instead",
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
    if value.len() >= 32 && value.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_') {
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

    use ygg_core::{
        CapabilityDescriptor, NetworkDeclaration, NetworkPermissions, PackageContributions,
        PackageEntry, PackageManifest, PermissionSet, SandboxPolicy,
    };
    use crate::{
        FakeOutboundExecutor, InMemoryEventStore, OutboundExecutorConfig, ProtocolContext,
        Runtime, RuntimeConfig,
    };

    /// Helper: create a runtime with a FakeOutboundExecutor.
    fn runtime_with_fake() -> (Arc<InMemoryEventStore>, Runtime<InMemoryEventStore>, Arc<FakeOutboundExecutor>) {
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
    fn package_with_secret_refs(
        id: &str,
        secret_refs: Vec<String>,
    ) -> PackageManifest {
        PackageManifest {
            schema_version: 1,
            id: id.to_string(),
            version: "0.1.0".to_string(),
            display_name: None,
            description: None,
            author: None,
            license: None,
            entry: PackageEntry::RustInproc {
                crate_ref: "example-echo-rust-inproc".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            },
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
                "kernel.outbound.execute",
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
            fake.call_count(), 0,
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
                "kernel.outbound.execute",
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
                "kernel.outbound.execute",
                serde_json::json!({
                    "capability_id": "example/y2-no-secret/fetch",
                    "destination_host": "api.openai.com",
                    "method": "POST",
                }),
            )
            .await;

        // Should succeed (fake executor returns ok)
        assert!(result.is_ok(), "request without secret_headers should succeed, got: {:?}", result.err());
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
                "kernel.outbound.execute",
                serde_json::json!({
                    "capability_id": "example/y2-multi/fetch",
                    "destination_host": "api.openai.com",
                    "method": "POST",
                    "secret_refs": ["secret_ref:env:KEY_A", "secret_ref:env:KEY_B"],
                }),
            )
            .await;

        assert!(result.is_err(), "undeclared second secret_ref should be denied");
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(
            err_msg.contains("not declared"),
            "error should mention undeclared secret_ref, got: {err_msg}"
        );
        assert_eq!(
            fake.call_count(), 0,
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
                "kernel.outbound.execute",
                serde_json::json!({
                    "capability_id": "example/y2-toplevel/fetch",
                    "destination_host": "api.openai.com",
                    "method": "POST",
                    "secret_refs": ["secret_ref:env:UNDECLARED"],
                }),
            )
            .await;

        assert!(result.is_err(), "top-level undeclared secret_ref should be denied");
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(
            err_msg.contains("not declared"),
            "error should mention undeclared, got: {err_msg}"
        );
        assert_eq!(fake.call_count(), 0, "executor should not be called");
    }
}
