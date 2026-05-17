//! Host secret resolver trait and configuration.
//!
//! The `HostSecretResolver` trait defines how the runtime resolves
//! `secret_ref` identifiers at execution time. Resolution is only
//! allowed during capability invocation; resolved raw secrets must
//! never be written back into events, proposals, logs, or audit records.
//!
//! This module provides the contract only. Production vault integrations
//! belong in host-level packages, not the kernel.

use async_trait::async_trait;

/// A host-level secret resolver that resolves `secret_ref` identifiers
/// to their raw values at runtime.
///
/// ## Contract
///
/// - Resolution is only permitted during capability invocation by the
///   runtime, not in event/proposal/audit paths.
/// - The resolved raw value must never be persisted or logged by the
///   kernel or any package.
/// - Packages reference secrets via `SecretRef` identifiers; they never
///   handle raw secret values.
#[async_trait]
pub trait HostSecretResolver: Send + Sync + 'static {
    /// Resolve a secret reference to its raw value.
    ///
    /// Returns the raw secret string if found, or an error if the
    /// reference cannot be resolved.
    async fn resolve(&self, ref_id: &str) -> anyhow::Result<String>;
}

/// A default resolver that denies all secret resolution.
///
/// Use this when no secret vault is configured. Any attempt to
/// resolve a secret reference will fail with a clear error.
pub struct DenyAllSecretResolver;

#[async_trait]
impl HostSecretResolver for DenyAllSecretResolver {
    async fn resolve(&self, ref_id: &str) -> anyhow::Result<String> {
        anyhow::bail!(
            "secret resolution denied: no secret resolver configured (ref_id='{}')",
            ref_id
        )
    }
}

/// Configuration for the host secret resolver.
#[derive(Clone)]
pub struct SecretResolverConfig {
    /// The resolver implementation. Defaults to `DenyAllSecretResolver`.
    pub resolver: std::sync::Arc<dyn HostSecretResolver>,
}

impl Default for SecretResolverConfig {
    fn default() -> Self {
        Self {
            resolver: std::sync::Arc::new(DenyAllSecretResolver),
        }
    }
}

impl SecretResolverConfig {
    /// Create a config with a custom resolver.
    pub fn with_resolver(resolver: std::sync::Arc<dyn HostSecretResolver>) -> Self {
        Self { resolver }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn deny_all_resolver_rejects_resolution() {
        let resolver = DenyAllSecretResolver;
        let result = resolver.resolve("secret_ref:env:MY_KEY").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no secret resolver configured"));
    }
}
