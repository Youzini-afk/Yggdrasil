use std::time::Instant;

use serde_json::{json, Value};
use uuid::Uuid;
use ygg_core::{
    CapHandle, CapHandleId, CapabilityId, HandleLease, HandleProvenance, HandleScope, PackageEntry,
    EVENT_CAPABILITY_COMPLETED, EVENT_CAPABILITY_FAILED, EVENT_CAPABILITY_INVOKED,
    KERNEL_PACKAGE_ID,
};

use super::Runtime;
use crate::{
    validate_json_schema_subset, CapabilityInvocationRequest, CapabilityInvocationResult,
    EventStore, InprocInvocation, PackageState, ProtocolContext, ProtocolPrincipal,
    RegisteredCapability,
};

impl<S> Runtime<S>
where
    S: EventStore,
{
    pub async fn discover_capabilities(&self) -> Vec<crate::RegisteredCapability> {
        self.capabilities.discover().await
    }

    pub async fn invoke_capability(
        &self,
        request: CapabilityInvocationRequest,
    ) -> anyhow::Result<CapabilityInvocationResult> {
        self.invoke_capability_authorized(request, Uuid::new_v4())
            .await
    }

    async fn invoke_capability_authorized(
        &self,
        request: CapabilityInvocationRequest,
        correlation_id: Uuid,
    ) -> anyhow::Result<CapabilityInvocationResult> {
        let started = Instant::now();
        let started_at = chrono::Utc::now();

        let prepared = self.prepare_capability_invocation(&request).await;
        let (capability_id, version, active_handle) = match prepared {
            Ok(prepared) => prepared,
            Err(error) => {
                if let Some(capability_id) = request.capability_id.as_ref() {
                    self.emit_capability_failed(
                        correlation_id,
                        capability_id,
                        None,
                        elapsed_ms(started),
                        "prepare_failed",
                        &error.to_string(),
                    )
                    .await?;
                }
                return Err(error);
            }
        };

        self.append_kernel_event(
            &capability_event_session_id(&capability_id),
            EVENT_CAPABILITY_INVOKED,
            json!({
                "correlation_id": correlation_id,
                "capability_id": capability_id,
                "caller_package_id": request.caller_package_id,
                "handle_id": active_handle,
                "started_at": started_at,
            }),
        )
        .await?;

        let invoke_result = self
            .invoke_capability_prepared(
                request,
                capability_id.clone(),
                version,
                active_handle,
                correlation_id,
                started,
            )
            .await;

        match invoke_result {
            Ok(result) => {
                self.append_kernel_event(
                    &capability_event_session_id(&capability_id),
                    EVENT_CAPABILITY_COMPLETED,
                    json!({
                        "correlation_id": correlation_id,
                        "capability_id": capability_id,
                        "duration_ms": result.duration_ms,
                        "completed_at": chrono::Utc::now(),
                    }),
                )
                .await?;
                Ok(result)
            }
            Err(error) => {
                self.emit_capability_failed(
                    correlation_id,
                    &capability_id,
                    Some(active_handle),
                    elapsed_ms(started),
                    "invoke_failed",
                    &error.to_string(),
                )
                .await?;
                Err(error)
            }
        }
    }

    async fn prepare_capability_invocation(
        &self,
        request: &CapabilityInvocationRequest,
    ) -> anyhow::Result<(CapabilityId, Option<String>, CapHandleId)> {
        if let Some(handle_id) = request.handle {
            let handle = self
                .handles
                .lookup(handle_id)
                .await
                .ok_or_else(|| anyhow::anyhow!("capability handle not found"))?;
            if let Some(caller) = &request.caller_package_id {
                if handle.scope.holder_package_id != *caller {
                    anyhow::bail!("capability handle is not held by caller package");
                }
            }
            validate_handle_lease(&handle)?;
            let version = if handle.cap_version == "1" {
                None
            } else {
                Some(handle.cap_version.clone())
            };
            return Ok((handle.cap_type, version, handle_id));
        }

        let capability_id = request
            .capability_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("capability invoke requires handle or capability_id"))?;
        if let Some(caller) = &request.caller_package_id {
            let allowed = if self.is_contract_none_package(caller).await {
                true
            } else {
                self.packages
                    .permissions(caller)
                    .await
                    .map(|permissions| {
                        permissions.capabilities.invoke.iter().any(|pattern| {
                            pattern == "*"
                                || pattern == &capability_id
                                || capability_id.starts_with(pattern.trim_end_matches('*'))
                        })
                    })
                    .unwrap_or(false)
            };
            if !allowed {
                self.audit_permission_denied(
                    &capability_event_session_id(&capability_id),
                    caller,
                    "capabilities.invoke",
                )
                .await?;
                anyhow::bail!(
                    "package '{caller}' is not allowed to invoke '{}'",
                    capability_id
                );
            }
        }
        let holder = request
            .caller_package_id
            .clone()
            .unwrap_or_else(|| KERNEL_PACKAGE_ID.to_string());
        let provider = self
            .capabilities
            .resolve(
                &capability_id,
                request.provider_package_id.as_ref(),
                request.version.as_deref(),
            )
            .await?;
        let handle_id = self
            .handles
            .mint(CapHandle {
                id: CapHandleId::new(),
                cap_type: capability_id.clone(),
                cap_version: provider.descriptor.version,
                scope: HandleScope {
                    holder_package_id: holder,
                    session_id: None,
                },
                constraints: json!({}),
                lease: HandleLease {
                    expires_at: None,
                    max_invocations: Some(1),
                    invocations_used: 0,
                },
                provenance: HandleProvenance {
                    granted_at: chrono::Utc::now(),
                    granted_by_package_id: KERNEL_PACKAGE_ID.to_string(),
                    via_method: "auto_mint".to_string(),
                },
                parent: None,
                revoked: false,
            })
            .await;
        Ok((capability_id, request.version.clone(), handle_id))
    }

    async fn invoke_capability_prepared(
        &self,
        request: CapabilityInvocationRequest,
        capability_id: CapabilityId,
        version: Option<String>,
        active_handle: CapHandleId,
        correlation_id: Uuid,
        started: Instant,
    ) -> anyhow::Result<CapabilityInvocationResult> {
        let before = self
            .dispatch_extension_handlers(
                "kernel/v1/capability.before_invoke",
                json!({
                    "capability_id": capability_id,
                    "caller_package_id": request.caller_package_id,
                    "input": request.input,
                }),
            )
            .await;
        if let Some(vetoed_by) = before.vetoed_by {
            anyhow::bail!("capability invoke vetoed by hook package '{vetoed_by}'");
        }

        let provider = self
            .capabilities
            .resolve(
                &capability_id,
                request.provider_package_id.as_ref(),
                version.as_deref(),
            )
            .await?;
        validate_json_schema_subset(&provider.descriptor.input_schema, &request.input)?;
        let output = self
            .execute_registered_capability(&provider, &capability_id, request.input)
            .await?;
        self.handles.record_invocation(active_handle).await?;
        let result = CapabilityInvocationResult {
            capability_id: provider.descriptor.id,
            provider_package_id: provider.provider_package_id,
            output,
            duration_ms: elapsed_ms(started),
            correlation_id,
        };
        let _ = self
            .dispatch_extension_handlers(
                "kernel/v1/capability.after_invoke",
                serde_json::to_value(&result).unwrap_or_else(|_| json!({})),
            )
            .await;
        Ok(result)
    }

    async fn emit_capability_failed(
        &self,
        correlation_id: Uuid,
        capability_id: &str,
        handle_id: Option<CapHandleId>,
        duration_ms: u64,
        error_kind: &str,
        error_message: &str,
    ) -> anyhow::Result<()> {
        self.append_kernel_event(
            &capability_event_session_id(capability_id),
            EVENT_CAPABILITY_FAILED,
            json!({
                "correlation_id": correlation_id,
                "capability_id": capability_id,
                "handle_id": handle_id,
                "duration_ms": duration_ms,
                "error_kind": error_kind,
                "error_message": error_message,
                "failed_at": chrono::Utc::now(),
            }),
        )
        .await?;
        Ok(())
    }

    pub(crate) async fn execute_registered_capability(
        &self,
        provider: &RegisteredCapability,
        capability_id: &str,
        input: Value,
    ) -> anyhow::Result<Value> {
        let output = match self.package_status(&provider.provider_package_id).await {
            Some(record) => match record.manifest.entry.kind {
                PackageEntry::RustInproc {
                    crate_ref, symbol, ..
                } => {
                    let package = self
                        .config
                        .inproc_packages
                        .lookup(&crate_ref, &symbol)
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "rust_inproc entry '{crate_ref}::{symbol}' is not available"
                            )
                        })?;
                    package
                        .invoke(InprocInvocation {
                            capability_id: capability_id.to_string(),
                            provider_package_id: provider.provider_package_id.clone(),
                            input,
                        })
                        .await?
                }
                PackageEntry::Subprocess { .. } => match self
                    .subprocesses
                    .invoke(&provider.provider_package_id, capability_id, input)
                    .await
                {
                    Ok(output) => output,
                    Err(error) => {
                        if let Some(record) = self
                            .packages
                            .set_state(&provider.provider_package_id, PackageState::Degraded)
                            .await
                        {
                            self.append_package_degraded_event(&record, &error.to_string())
                                .await?;
                        }
                        return Err(error);
                    }
                },
                other => anyhow::bail!(
                    "entry kind '{}' cannot execute capabilities yet",
                    crate::entry_kind(&other)
                ),
            },
            None => anyhow::bail!(
                "provider package '{}' is not loaded",
                provider.provider_package_id
            ),
        };
        validate_json_schema_subset(&provider.descriptor.output_schema, &output)?;
        Ok(output)
    }

    pub async fn invoke_capability_with_context(
        &self,
        context: &ProtocolContext,
        mut request: CapabilityInvocationRequest,
    ) -> anyhow::Result<CapabilityInvocationResult> {
        match &context.principal {
            ProtocolPrincipal::HostAdmin | ProtocolPrincipal::HostDev => {
                request.caller_package_id = None;
                self.invoke_capability_authorized(request, context.effective_correlation_id())
                    .await
            }
            ProtocolPrincipal::Package { package_id } => {
                request.caller_package_id = Some(package_id.clone());
                self.invoke_capability_authorized(request, context.effective_correlation_id())
                    .await
            }
            ProtocolPrincipal::Human { .. }
            | ProtocolPrincipal::Assistant { .. }
            | ProtocolPrincipal::Anonymous => {
                if self
                    .principal_has_grant(
                        &context.principal,
                        "capabilities.invoke",
                        request.capability_id.as_deref(),
                    )
                    .await
                {
                    request.caller_package_id = None;
                    self.invoke_capability_authorized(request, context.effective_correlation_id())
                        .await
                } else {
                    anyhow::bail!("principal is not allowed to invoke capabilities")
                }
            }
        }
    }
}

fn capability_event_session_id(capability_id: &str) -> String {
    format!("kernel_capability_{}", capability_id.replace('/', "_"))
}

fn validate_handle_lease(handle: &CapHandle) -> anyhow::Result<()> {
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
    Ok(())
}

fn elapsed_ms(started: Instant) -> u64 {
    (started.elapsed().as_millis() as u64).max(1)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;
    use serde_json::Value;
    use ygg_core::{
        EntryDescriptor, PackageContributions, PackageEntry, PermissionSet, SandboxPolicy,
        EVENT_PERMISSION_DENIED,
    };

    use super::*;
    use crate::{CapabilityInvocationRequest, InMemoryEventStore, RuntimeConfig};

    #[tokio::test]
    async fn loaded_package_registers_capability() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(store, RuntimeConfig::default());
        runtime
            .load_package(ygg_core::PackageManifest {
                schema_version: 1,
                id: "example/echo".to_string(),
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
                provides: vec![ygg_core::CapabilityDescriptor {
                    id: "example/echo/echo".to_string(),
                    version: "0.1.0".to_string(),
                    input_schema: Value::Null,
                    output_schema: Value::Null,
                    streaming: false,
                    side_effects: Vec::new(),
                    description: None,
                }],
                consumes: Vec::new(),
                contributes: PackageContributions::default(),
                permissions: PermissionSet::default(),
                sandbox_policy: SandboxPolicy::default(),
            })
            .await?;

        let result = runtime
            .invoke_capability(CapabilityInvocationRequest {
                handle: None,
                capability_id: Some("example/echo/echo".to_string()),
                caller_package_id: None,
                provider_package_id: None,
                version: None,
                input: json!({"ping": true}),
            })
            .await?;
        assert_eq!(result.output, json!({"ping": true}));
        Ok(())
    }

    #[tokio::test]
    async fn denied_capability_invoke_records_audit_event() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(store.clone(), RuntimeConfig::default());
        runtime
            .load_package(ygg_core::PackageManifest {
                schema_version: 1,
                id: "example/echo".to_string(),
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
                provides: vec![ygg_core::CapabilityDescriptor {
                    id: "example/echo/echo".to_string(),
                    version: "0.1.0".to_string(),
                    input_schema: Value::Null,
                    output_schema: Value::Null,
                    streaming: false,
                    side_effects: Vec::new(),
                    description: None,
                }],
                consumes: Vec::new(),
                contributes: PackageContributions::default(),
                permissions: PermissionSet::default(),
                sandbox_policy: SandboxPolicy::default(),
            })
            .await?;
        runtime
            .load_package(ygg_core::PackageManifest {
                schema_version: 1,
                id: "example/caller".to_string(),
                version: "0.1.0".to_string(),
                display_name: None,
                description: None,
                author: None,
                license: None,
                entry: EntryDescriptor::v1(PackageEntry::RustInproc {
                    crate_ref: "example-caller".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                }),
                provides: Vec::new(),
                consumes: Vec::new(),
                contributes: PackageContributions::default(),
                permissions: PermissionSet::default(),
                sandbox_policy: SandboxPolicy::default(),
            })
            .await?;

        let denied = runtime
            .invoke_capability(CapabilityInvocationRequest {
                handle: None,
                capability_id: Some("example/echo/echo".to_string()),
                caller_package_id: Some("example/caller".to_string()),
                provider_package_id: None,
                version: None,
                input: json!({}),
            })
            .await;
        assert!(denied.is_err());

        let events = store
            .list_session(&"kernel_capability_example_echo_echo".to_string())
            .await?;
        assert!(events
            .iter()
            .any(|event| event.kind == EVENT_PERMISSION_DENIED));
        Ok(())
    }

    #[tokio::test]
    async fn package_context_overrides_spoofed_capability_caller() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(store, RuntimeConfig::default());
        runtime
            .load_package(ygg_core::PackageManifest {
                schema_version: 1,
                id: "example/echo".to_string(),
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
                provides: vec![ygg_core::CapabilityDescriptor {
                    id: "example/echo/echo".to_string(),
                    version: "0.1.0".to_string(),
                    input_schema: Value::Null,
                    output_schema: Value::Null,
                    streaming: false,
                    side_effects: Vec::new(),
                    description: None,
                }],
                consumes: Vec::new(),
                contributes: PackageContributions::default(),
                permissions: PermissionSet::default(),
                sandbox_policy: SandboxPolicy::default(),
            })
            .await?;
        runtime
            .load_package(ygg_core::PackageManifest {
                schema_version: 1,
                id: "example/caller".to_string(),
                version: "0.1.0".to_string(),
                display_name: None,
                description: None,
                author: None,
                license: None,
                entry: EntryDescriptor::v1(PackageEntry::RustInproc {
                    crate_ref: "example-caller".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                }),
                provides: Vec::new(),
                consumes: Vec::new(),
                contributes: PackageContributions::default(),
                permissions: PermissionSet::default(),
                sandbox_policy: SandboxPolicy::default(),
            })
            .await?;

        let denied = runtime
            .invoke_capability_with_context(
                &ProtocolContext::package("example/caller", "test"),
                CapabilityInvocationRequest {
                    handle: None,
                    capability_id: Some("example/echo/echo".to_string()),
                    caller_package_id: None,
                    provider_package_id: None,
                    version: None,
                    input: json!({}),
                },
            )
            .await;

        assert!(denied.is_err());
        Ok(())
    }
}
