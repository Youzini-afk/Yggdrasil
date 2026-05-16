use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use ygg_core::{CapabilityDescriptor, CapabilityId, HookSubscription, PackageId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredCapability {
    pub descriptor: CapabilityDescriptor,
    pub provider_package_id: PackageId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInvocationRequest {
    pub capability_id: CapabilityId,
    #[serde(default)]
    pub caller_package_id: Option<PackageId>,
    #[serde(default)]
    pub input: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInvocationResult {
    pub capability_id: CapabilityId,
    pub provider_package_id: PackageId,
    pub output: Value,
}

#[derive(Default)]
pub struct CapabilityFabric {
    providers: RwLock<HashMap<CapabilityId, Vec<RegisteredCapability>>>,
}

impl CapabilityFabric {
    pub async fn register_package(&self, package_id: &PackageId, descriptors: &[CapabilityDescriptor]) {
        let mut providers = self.providers.write().await;
        for descriptor in descriptors {
            providers.entry(descriptor.id.clone()).or_default().push(RegisteredCapability {
                descriptor: descriptor.clone(),
                provider_package_id: package_id.clone(),
            });
        }
    }

    pub async fn unregister_package(&self, package_id: &PackageId) {
        let mut providers = self.providers.write().await;
        providers.retain(|_, registered| {
            registered.retain(|capability| &capability.provider_package_id != package_id);
            !registered.is_empty()
        });
    }

    pub async fn discover(&self) -> Vec<RegisteredCapability> {
        let mut values: Vec<_> = self
            .providers
            .read()
            .await
            .values()
            .flat_map(|providers| providers.iter().cloned())
            .collect();
        values.sort_by(|a, b| a.descriptor.id.cmp(&b.descriptor.id).then(a.provider_package_id.cmp(&b.provider_package_id)));
        values
    }

    pub async fn describe(&self, capability_id: &CapabilityId) -> Vec<RegisteredCapability> {
        self.providers.read().await.get(capability_id).cloned().unwrap_or_default()
    }

    pub async fn resolve(&self, capability_id: &CapabilityId) -> anyhow::Result<RegisteredCapability> {
        let providers = self.describe(capability_id).await;
        match providers.as_slice() {
            [] => anyhow::bail!("capability '{}' has no provider", capability_id),
            [provider] => Ok(provider.clone()),
            _ => anyhow::bail!("capability '{}' has ambiguous providers", capability_id),
        }
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredHook {
    pub subscriber_package_id: PackageId,
    pub subscription: HookSubscription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionDispatchResult {
    pub extension_point: String,
    pub invoked: Vec<RegisteredHook>,
    pub vetoed_by: Option<PackageId>,
    pub payload: Value,
}

#[derive(Default)]
pub struct ExtensionRegistry {
    hooks: RwLock<HashMap<String, Vec<RegisteredHook>>>,
}

impl ExtensionRegistry {
    pub async fn register_package(&self, package_id: &PackageId, hooks: &[HookSubscription]) {
        let mut registry = self.hooks.write().await;
        for hook in hooks {
            registry.entry(hook.extension_point.clone()).or_default().push(RegisteredHook {
                subscriber_package_id: package_id.clone(),
                subscription: hook.clone(),
            });
        }
        for hooks in registry.values_mut() {
            hooks.sort_by_key(|hook| hook.subscription.precedence);
        }
    }

    pub async fn unregister_package(&self, package_id: &PackageId) {
        let mut registry = self.hooks.write().await;
        registry.retain(|_, hooks| {
            hooks.retain(|hook| &hook.subscriber_package_id != package_id);
            !hooks.is_empty()
        });
    }

    pub async fn list_hooks(&self, extension_point: &str) -> Vec<RegisteredHook> {
        self.hooks.read().await.get(extension_point).cloned().unwrap_or_default()
    }

    pub async fn dispatch(&self, extension_point: &str, payload: Value) -> ExtensionDispatchResult {
        let invoked = self.list_hooks(extension_point).await;
        let vetoed_by = invoked
            .iter()
            .find(|hook| hook.subscription.handler == "veto")
            .map(|hook| hook.subscriber_package_id.clone());
        ExtensionDispatchResult { extension_point: extension_point.to_string(), invoked, vetoed_by, payload }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use ygg_core::{CapabilityDescriptor, HookSubscription, HookTiming};

    use super::*;

    #[tokio::test]
    async fn ambiguous_capability_is_rejected() {
        let fabric = CapabilityFabric::default();
        let descriptor = CapabilityDescriptor {
            id: "example/echo/echo".to_string(),
            version: "0.1.0".to_string(),
            input_schema: Value::Null,
            output_schema: Value::Null,
            streaming: false,
            side_effects: Vec::new(),
            description: None,
        };
        fabric.register_package(&"example/a".to_string(), &[descriptor.clone()]).await;
        fabric.register_package(&"example/b".to_string(), &[descriptor]).await;

        let result = fabric.resolve(&"example/echo/echo".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn hook_dispatch_reports_veto() {
        let registry = ExtensionRegistry::default();
        registry
            .register_package(
                &"example/veto".to_string(),
                &[HookSubscription {
                    extension_point: "kernel/session.before_open".to_string(),
                    handler: "veto".to_string(),
                    timing: HookTiming::Sync,
                    precedence: 0,
                }],
            )
            .await;
        let result = registry.dispatch("kernel/session.before_open", json!({})).await;
        assert_eq!(result.vetoed_by, Some("example/veto".to_string()));
    }
}
