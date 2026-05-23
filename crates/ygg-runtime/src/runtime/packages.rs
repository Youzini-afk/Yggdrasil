use std::collections::HashMap;

use serde_json::{json, Value};
use ygg_core::{
    CapHandle, CapHandleId, ContractMode, HandleLease, HandleProvenance, HandleScope, PackageEntry,
    PackageId, PackageManifest, EVENT_PACKAGE_DEGRADED, EVENT_PACKAGE_LOADED,
    EVENT_PACKAGE_LOADING, EVENT_PACKAGE_LOG, EVENT_PACKAGE_READY, EVENT_PACKAGE_STARTING,
    EVENT_PACKAGE_STOPPED, EVENT_PACKAGE_STOPPING, EVENT_PACKAGE_UNLOADED, KERNEL_PACKAGE_ID,
};

use super::Runtime;
use crate::{EventStore, PackageRecord, PackageState};

impl<S> Runtime<S>
where
    S: EventStore,
{
    pub(crate) async fn is_contract_none_package(&self, package_id: &PackageId) -> bool {
        self.packages
            .manifest(package_id)
            .await
            .map(|manifest| manifest.entry.contract == ContractMode::None)
            .unwrap_or(false)
    }

    pub async fn load_package(&self, manifest: PackageManifest) -> anyhow::Result<PackageRecord> {
        if let PackageEntry::RustInproc {
            crate_ref, symbol, ..
        } = &manifest.entry.kind
        {
            if !manifest.provides.is_empty()
                && self
                    .config
                    .inproc_packages
                    .lookup(crate_ref, symbol)
                    .is_none()
            {
                anyhow::bail!(
                    "rust_inproc entry '{}::{}' is not available in this host",
                    crate_ref,
                    symbol
                );
            }
        }
        let mut record = self
            .packages
            .load(manifest, &self.config.host_policy)
            .await?;
        record = self
            .packages
            .set_state(&record.id, PackageState::Loading)
            .await
            .unwrap_or(record);
        self.append_package_lifecycle_event(&record, EVENT_PACKAGE_LOADING, None)
            .await?;
        self.capabilities
            .register_package(&record.id, &record.manifest.provides)
            .await;
        self.extensions
            .register_package(&record.id, &record.manifest.contributes.hooks)
            .await;
        let bindings = self.mint_package_bindings(&record.manifest).await;
        match &record.manifest.entry.kind {
            PackageEntry::Subprocess { .. } => {
                record = self
                    .packages
                    .set_state(&record.id, PackageState::Starting)
                    .await
                    .unwrap_or(record);
                self.append_package_lifecycle_event(&record, EVENT_PACKAGE_STARTING, None)
                    .await?;
                if let Err(error) = self
                    .subprocesses
                    .start(&record.manifest, (*self).clone(), bindings)
                    .await
                {
                    let degraded = self
                        .packages
                        .set_state(&record.id, PackageState::Degraded)
                        .await
                        .unwrap_or_else(|| record.clone());
                    self.capabilities.unregister_package(&record.id).await;
                    self.extensions.unregister_package(&record.id).await;
                    self.capabilities.unregister_package(&record.id).await;
                    self.extensions.unregister_package(&record.id).await;
                    self.capabilities.unregister_package(&record.id).await;
                    self.extensions.unregister_package(&record.id).await;
                    self.append_package_degraded_event(&degraded, &error.to_string())
                        .await?;
                    return Err(error);
                }
                record = self
                    .packages
                    .set_state(&record.id, PackageState::Ready)
                    .await
                    .unwrap_or(record);
            }
            PackageEntry::RustInproc {
                crate_ref, symbol, ..
            } => {
                if let Some(package) = self.config.inproc_packages.lookup(crate_ref, symbol) {
                    let env = crate::KernelEnv {
                        package_id: record.id.clone(),
                        bindings,
                        handles: self.handles.clone(),
                    };
                    package.init(&env);
                }
            }
            PackageEntry::Wasm { .. } => {
                if let Err(error) = super::wasm::load_wasm_placeholder() {
                    let degraded = self
                        .packages
                        .set_state(&record.id, PackageState::Degraded)
                        .await
                        .unwrap_or_else(|| record.clone());
                    self.append_package_degraded_event(&degraded, &error.to_string())
                        .await?;
                    return Err(error);
                }
            }
            PackageEntry::Remote { .. } => {
                if let Err(error) = super::remote::load_remote_placeholder() {
                    let degraded = self
                        .packages
                        .set_state(&record.id, PackageState::Degraded)
                        .await
                        .unwrap_or_else(|| record.clone());
                    self.append_package_degraded_event(&degraded, &error.to_string())
                        .await?;
                    return Err(error);
                }
            }
        }
        if !matches!(record.state, PackageState::Ready) {
            record = self
                .packages
                .set_state(&record.id, PackageState::Ready)
                .await
                .unwrap_or(record);
        }
        self.append_package_lifecycle_event(&record, EVENT_PACKAGE_READY, None)
            .await?;
        self.append_package_lifecycle_event(&record, EVENT_PACKAGE_LOADED, None)
            .await?;
        Ok(record)
    }

    pub(crate) async fn mint_package_bindings(
        &self,
        manifest: &PackageManifest,
    ) -> HashMap<String, CapHandleId> {
        let mut bindings = HashMap::new();
        for cap_id in &manifest.permissions.capabilities.invoke {
            let handle_id = self
                .handles
                .mint(package_load_handle(
                    manifest.id.clone(),
                    cap_id.clone(),
                    "1".to_string(),
                    json!({}),
                ))
                .await;
            bindings.insert(logical_binding_name(cap_id), handle_id);
        }

        for declaration in &manifest.permissions.network.declarations {
            let handle_id = self
                .handles
                .mint(package_load_handle(
                    manifest.id.clone(),
                    "kernel.outbound.execute".to_string(),
                    "1".to_string(),
                    json!({
                        "host": declaration.host,
                        "methods": declaration.methods,
                    }),
                ))
                .await;
            bindings.insert(network_binding_name(&declaration.host), handle_id);
        }

        for secret_ref in &manifest.permissions.secret_refs {
            let handle_id = self
                .handles
                .mint(package_load_handle(
                    manifest.id.clone(),
                    "kernel.secret.reveal".to_string(),
                    "1".to_string(),
                    json!({ "secret_ref": secret_ref }),
                ))
                .await;
            bindings.insert(secret_binding_name(secret_ref), handle_id);
        }

        bindings
    }

    pub async fn unload_package(&self, package_id: &PackageId) -> anyhow::Result<PackageRecord> {
        if let Some(stopping) = self
            .packages
            .set_state(package_id, PackageState::Stopping)
            .await
        {
            self.append_package_lifecycle_event(&stopping, EVENT_PACKAGE_STOPPING, None)
                .await?;
        }
        self.subprocesses.stop(package_id).await;
        if let Some(stopped) = self
            .packages
            .set_state(package_id, PackageState::Stopped)
            .await
        {
            self.append_package_lifecycle_event(&stopped, EVENT_PACKAGE_STOPPED, None)
                .await?;
        }
        let record = self.packages.unload(package_id).await?;
        self.capabilities.unregister_package(package_id).await;
        self.extensions.unregister_package(package_id).await;
        self.append_package_lifecycle_event(&record, EVENT_PACKAGE_UNLOADED, None)
            .await?;
        Ok(record)
    }

    pub async fn restart_package(&self, package_id: &PackageId) -> anyhow::Result<PackageRecord> {
        let record = self
            .package_status(package_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("package '{package_id}' is not loaded"))?;
        if !matches!(record.manifest.entry.kind, PackageEntry::Subprocess { .. }) {
            anyhow::bail!(
                "package '{package_id}' entry kind '{}' cannot restart yet",
                record.entry_kind
            );
        }
        if let Some(stopping) = self
            .packages
            .set_state(package_id, PackageState::Stopping)
            .await
        {
            self.append_package_lifecycle_event(&stopping, EVENT_PACKAGE_STOPPING, Some("restart"))
                .await?;
        }
        let bindings = self.mint_package_bindings(&record.manifest).await;
        self.subprocesses
            .restart(&record.manifest, (*self).clone(), bindings)
            .await?;
        let ready = self
            .packages
            .set_state(package_id, PackageState::Ready)
            .await
            .ok_or_else(|| anyhow::anyhow!("package '{package_id}' disappeared during restart"))?;
        self.append_package_lifecycle_event(&ready, EVENT_PACKAGE_READY, Some("restart"))
            .await?;
        Ok(ready)
    }

    pub async fn package_logs(&self, package_id: &PackageId) -> Vec<crate::SubprocessLogLine> {
        let logs = self.subprocesses.drain_logs(package_id).await;
        for log in &logs {
            let _ = self
                .append_package_log_event(package_id, &log.stream, &log.line)
                .await;
        }
        logs
    }

    pub async fn host_diagnostics(&self) -> Value {
        let packages = self.list_packages().await;
        let capabilities = self.discover_capabilities().await;
        let hooks = self.extensions.list_all_hooks().await;
        json!({
            "package_count": packages.len(),
            "capability_provider_count": capabilities.len(),
            "hook_subscription_count": hooks.len(),
            "packages": packages,
        })
    }

    pub async fn list_surface_contributions(&self, slot: Option<String>) -> Value {
        let packages = self.list_packages().await;
        let mut contributions = Vec::new();
        for package in packages {
            for contribution in &package.manifest.contributes.surfaces {
                let slot_name = serde_json::to_value(&contribution.slot)
                    .ok()
                    .and_then(|value| value.as_str().map(str::to_string))
                    .unwrap_or_else(|| "unknown".to_string());
                if slot.as_ref().map(|slot| slot == &slot_name).unwrap_or(true) {
                    contributions.push(json!({
                        "package_id": package.id,
                        "entry_kind": package.entry_kind,
                        "package_state": package.state,
                        "surface": contribution,
                    }));
                }
            }
        }
        json!(contributions)
    }

    pub async fn describe_surface_contribution(&self, surface_id: &str) -> anyhow::Result<Value> {
        let packages = self.list_packages().await;
        for package in packages {
            for contribution in &package.manifest.contributes.surfaces {
                if contribution.id == surface_id {
                    return Ok(json!({
                        "package_id": package.id,
                        "entry_kind": package.entry_kind,
                        "package_state": package.state,
                        "surface": contribution,
                    }));
                }
            }
        }
        anyhow::bail!("surface contribution '{surface_id}' not found")
    }

    pub async fn list_packages(&self) -> Vec<PackageRecord> {
        self.packages.list().await
    }

    pub async fn package_status(&self, package_id: &PackageId) -> Option<PackageRecord> {
        self.packages.status(package_id).await
    }

    pub(crate) async fn append_package_degraded_event(
        &self,
        record: &PackageRecord,
        reason: &str,
    ) -> anyhow::Result<ygg_core::EventEnvelope> {
        self.append_package_lifecycle_event(record, EVENT_PACKAGE_DEGRADED, Some(reason))
            .await
    }

    pub(crate) async fn append_package_lifecycle_event(
        &self,
        record: &PackageRecord,
        kind: &'static str,
        reason: Option<&str>,
    ) -> anyhow::Result<ygg_core::EventEnvelope> {
        let session_id = format!("kernel_package_{}", record.id.replace('/', "_"));
        let mut payload = json!({
            "package_id": record.id,
            "version": record.version,
            "state": record.state,
            "entry_kind": record.entry_kind,
            "contract_mode": match record.manifest.entry.contract {
                ContractMode::V1 => "v1",
                ContractMode::None => "none",
            },
            "capability_count": record.capability_count,
            "hook_count": record.hook_count,
            "extension_point_count": record.extension_point_count,
        });
        if let Some(reason) = reason {
            payload["reason"] = json!(reason);
        }
        self.append_kernel_event(&session_id, kind, payload).await
    }

    pub(crate) async fn append_package_log_event(
        &self,
        package_id: &PackageId,
        stream: &str,
        line: &str,
    ) -> anyhow::Result<ygg_core::EventEnvelope> {
        let session_id = format!("kernel_package_{}", package_id.replace('/', "_"));
        self.append_kernel_event(
            &session_id,
            EVENT_PACKAGE_LOG,
            json!({"package_id": package_id, "stream": stream, "line": line}),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::Value;
    use ygg_core::{
        EntryDescriptor, PackageContributions, PackageEntry, PermissionSet, SandboxPolicy,
        EVENT_PACKAGE_LOADED, EVENT_PACKAGE_LOADING, EVENT_PACKAGE_READY,
    };

    use super::*;
    use crate::{InMemoryEventStore, RuntimeConfig};

    #[tokio::test]
    async fn package_load_records_kernel_lifecycle_event() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(store.clone(), RuntimeConfig::default());

        let record = runtime
            .load_package(ygg_core::PackageManifest {
                schema_version: 1,
                id: "org/pkg".to_string(),
                version: "0.1.0".to_string(),
                display_name: None,
                description: None,
                author: None,
                license: None,
                entry: EntryDescriptor::v1(PackageEntry::RustInproc {
                    crate_ref: "org-pkg".to_string(),
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

        assert_eq!(record.id, "org/pkg");
        let events = store
            .list_session(&"kernel_package_org_pkg".to_string())
            .await?;
        assert!(events
            .iter()
            .any(|event| event.kind == EVENT_PACKAGE_LOADING));
        assert!(events.iter().any(|event| event.kind == EVENT_PACKAGE_READY));
        assert!(events
            .iter()
            .any(|event| event.kind == EVENT_PACKAGE_LOADED));
        Ok(())
    }

    #[tokio::test]
    async fn rust_inproc_provider_must_exist_in_host_catalog() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let runtime = Runtime::new(store, RuntimeConfig::default());

        let result = runtime
            .load_package(ygg_core::PackageManifest {
                schema_version: 1,
                id: "example/missing".to_string(),
                version: "0.1.0".to_string(),
                display_name: None,
                description: None,
                author: None,
                license: None,
                entry: EntryDescriptor::v1(PackageEntry::RustInproc {
                    crate_ref: "missing-crate".to_string(),
                    symbol: "register".to_string(),
                    abi_version: 1,
                }),
                provides: vec![ygg_core::CapabilityDescriptor {
                    id: "example/missing/echo".to_string(),
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
            .await;

        assert!(result.is_err());
        Ok(())
    }
}

fn package_load_handle(
    holder_package_id: PackageId,
    cap_type: String,
    cap_version: String,
    constraints: Value,
) -> CapHandle {
    CapHandle {
        id: CapHandleId::new(),
        cap_type,
        cap_version,
        scope: HandleScope {
            holder_package_id,
            session_id: None,
        },
        constraints,
        lease: HandleLease::default(),
        provenance: HandleProvenance {
            granted_at: chrono::Utc::now(),
            granted_by_package_id: KERNEL_PACKAGE_ID.to_string(),
            via_method: "package_load".to_string(),
        },
        parent: None,
        revoked: false,
    }
}

fn logical_binding_name(id: &str) -> String {
    camel_binding_name(id, "binding")
}

fn network_binding_name(host: &str) -> String {
    format!("network{}", pascal_binding_name(host))
}

fn secret_binding_name(secret_ref: &str) -> String {
    let tail = secret_ref.rsplit(':').next().unwrap_or(secret_ref);
    format!("secret{}", pascal_binding_name(tail))
}

fn pascal_binding_name(input: &str) -> String {
    let camel = camel_binding_name(input, "value");
    let mut chars = camel.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn camel_binding_name(input: &str, fallback: &str) -> String {
    let mut out = String::new();
    let mut uppercase_next = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            if out.is_empty() {
                out.push(ch.to_ascii_lowercase());
            } else if uppercase_next {
                out.push(ch.to_ascii_uppercase());
            } else {
                out.push(ch.to_ascii_lowercase());
            }
            uppercase_next = false;
        } else {
            uppercase_next = !out.is_empty();
        }
    }
    if out.is_empty() {
        fallback.to_string()
    } else {
        out
    }
}
