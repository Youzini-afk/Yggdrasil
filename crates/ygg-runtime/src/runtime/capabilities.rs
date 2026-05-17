use serde_json::{json, Value};
use ygg_core::PackageEntry;

use super::Runtime;
use crate::{
    CapabilityInvocationRequest, CapabilityInvocationResult, EventStore,
    InprocInvocation, PackageState, ProtocolContext, ProtocolPrincipal, RegisteredCapability,
    validate_json_schema_subset,
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
        if let Some(caller) = &request.caller_package_id {
            let allowed = self
                .packages
                .permissions(caller)
                .await
                .map(|permissions| {
                    permissions.capabilities.invoke.iter().any(|pattern| {
                        pattern == "*" || pattern == &request.capability_id || request.capability_id.starts_with(pattern.trim_end_matches('*'))
                    })
                })
                .unwrap_or(false);
            if !allowed {
                self.audit_permission_denied(
                    &format!("kernel_capability_{}", request.capability_id.replace('/', "_")),
                    caller,
                    "capabilities.invoke",
                )
                .await?;
                anyhow::bail!("package '{caller}' is not allowed to invoke '{}'", request.capability_id);
            }
        }
        let before = self
            .dispatch_extension_handlers(
                "kernel/capability.before_invoke",
                json!({
                    "capability_id": request.capability_id,
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
                &request.capability_id,
                request.provider_package_id.as_ref(),
                request.version.as_deref(),
            )
            .await?;
        validate_json_schema_subset(&provider.descriptor.input_schema, &request.input)?;
        let output = self.execute_registered_capability(&provider, &request.capability_id, request.input).await?;
        let result = CapabilityInvocationResult {
            capability_id: provider.descriptor.id,
            provider_package_id: provider.provider_package_id,
            output,
        };
        let _ = self
            .dispatch_extension_handlers("kernel/capability.after_invoke", serde_json::to_value(&result).unwrap_or_else(|_| json!({})))
            .await;
        Ok(result)
    }

    pub(crate) async fn execute_registered_capability(
        &self,
        provider: &RegisteredCapability,
        capability_id: &str,
        input: Value,
    ) -> anyhow::Result<Value> {
        let output = match self.package_status(&provider.provider_package_id).await {
            Some(record) => match record.manifest.entry {
                PackageEntry::RustInproc { crate_ref, symbol, .. } => {
                    let package = self
                        .config
                        .inproc_packages
                        .lookup(&crate_ref, &symbol)
                        .ok_or_else(|| anyhow::anyhow!("rust_inproc entry '{crate_ref}::{symbol}' is not available"))?;
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
                            self.append_package_degraded_event(&record, &error.to_string()).await?;
                        }
                        return Err(error);
                    }
                },
                other => anyhow::bail!("entry kind '{}' cannot execute capabilities yet", crate::entry_kind(&other)),
            },
            None => anyhow::bail!("provider package '{}' is not loaded", provider.provider_package_id),
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
                self.invoke_capability(request).await
            }
            ProtocolPrincipal::Package { package_id } => {
                request.caller_package_id = Some(package_id.clone());
                self.invoke_capability(request).await
            }
            ProtocolPrincipal::Human { .. } | ProtocolPrincipal::Assistant { .. } | ProtocolPrincipal::Anonymous => {
                if self.principal_has_grant(&context.principal, "capabilities.invoke", Some(&request.capability_id)).await {
                    request.caller_package_id = None;
                    self.invoke_capability(request).await
                } else {
                    anyhow::bail!("principal is not allowed to invoke capabilities")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;
    use serde_json::Value;
    use ygg_core::{PackageContributions, PackageEntry, PermissionSet, SandboxPolicy, EVENT_PERMISSION_DENIED};

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
                entry: PackageEntry::RustInproc {
                    crate_ref: "example-echo-rust-inproc".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                },
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
                capability_id: "example/echo/echo".to_string(),
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
                entry: PackageEntry::RustInproc {
                    crate_ref: "example-echo-rust-inproc".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                },
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
                entry: PackageEntry::RustInproc {
                    crate_ref: "example-caller".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                },
                provides: Vec::new(),
                consumes: Vec::new(),
                contributes: PackageContributions::default(),
                permissions: PermissionSet::default(),
                sandbox_policy: SandboxPolicy::default(),
            })
            .await?;

        let denied = runtime
            .invoke_capability(CapabilityInvocationRequest {
                capability_id: "example/echo/echo".to_string(),
                caller_package_id: Some("example/caller".to_string()),
                provider_package_id: None,
                version: None,
                input: json!({}),
            })
            .await;
        assert!(denied.is_err());

        let events = store.list_session(&"kernel_capability_example_echo_echo".to_string()).await?;
        assert_eq!(events.last().expect("audit event").kind, EVENT_PERMISSION_DENIED);
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
                entry: PackageEntry::RustInproc {
                    crate_ref: "example-echo-rust-inproc".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                },
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
                entry: PackageEntry::RustInproc {
                    crate_ref: "example-caller".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                },
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
                    capability_id: "example/echo/echo".to_string(),
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
