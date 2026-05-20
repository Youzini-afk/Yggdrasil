//! Network policy checker and outbound audit helpers.
//!
//! This module provides the generic network policy enforcement for
//! Ygg-provided network/request helpers. It does NOT intercept
//! arbitrary subprocess OS calls — it checks whether a package's
//! manifest and the host policy permit a given outbound request.
//!
//! The checker is called before the runtime records an outbound
//! audit event and before the request is forwarded.

use serde_json::json;
use ygg_core::{
    new_id, CapabilityId, NetworkDeclaration, NetworkPermissions, OutboundAuditRecord, PackageId,
    RedactionState, EVENT_OUTBOUND_DENIED, EVENT_OUTBOUND_REQUEST,
};

use super::Runtime;
use crate::{EventStore, ProtocolPrincipal};

/// Result of a network policy check.
#[derive(Debug, Clone)]
pub struct NetworkPolicyDecision {
    /// Whether the request is allowed.
    pub allowed: bool,
    /// Reason for denial, if not allowed.
    pub denial_reason: Option<String>,
    /// The matching declaration, if found.
    pub matched_declaration: Option<NetworkDeclaration>,
}

/// Check whether a package's manifest and host policy allow an outbound
/// request to the given destination.
///
/// This function:
/// 1. Looks up the package's `network` permissions from the manifest.
/// 2. Checks structured `declarations` first, falling back to the
///    flat `hosts` list for backward compatibility.
/// 3. Matches host (exact or glob-style with `*` prefix).
/// 4. Matches method if the declaration specifies methods.
/// 5. Official packages have no bypass — they must declare network
///    permissions like any other package.
///
/// This is designed for the Ygg-provided network/request helper path,
/// not for intercepting arbitrary subprocess OS calls.
pub fn check_network_policy(
    permissions: &NetworkPermissions,
    destination_host: &str,
    method: &str,
) -> NetworkPolicyDecision {
    // Check structured declarations first
    for decl in &permissions.declarations {
        if host_matches(&decl.host, destination_host) {
            if decl.methods.is_empty() || method_matches(&decl.methods, method) {
                return NetworkPolicyDecision {
                    allowed: true,
                    denial_reason: None,
                    matched_declaration: Some(decl.clone()),
                };
            }
        }
    }

    // Fall back to flat hosts list (backward compat)
    for host in &permissions.hosts {
        if host_matches(host, destination_host) {
            return NetworkPolicyDecision {
                allowed: true,
                denial_reason: None,
                matched_declaration: Some(NetworkDeclaration {
                    host: host.clone(),
                    methods: Vec::new(),
                    purpose: None,
                }),
            };
        }
    }

    // No matching declaration — denied
    NetworkPolicyDecision {
        allowed: false,
        denial_reason: Some(format!(
            "package has no network permission for host '{}' method '{}'",
            destination_host, method
        )),
        matched_declaration: None,
    }
}

/// Check whether a host pattern matches a destination host.
///
/// Supports:
/// - Exact match: `"api.example.com"` matches `"api.example.com"`
/// - Wildcard prefix: `"*.example.com"` matches `"api.example.com"`
/// - Port-specific: `"api.example.com:443"` matches `"api.example.com:443"`
fn host_matches(pattern: &str, destination: &str) -> bool {
    if pattern == destination {
        return true;
    }
    // Wildcard prefix: *.example.com matches sub.example.com
    if let Some(suffix) = pattern.strip_prefix("*.") {
        // destination must end with .suffix and have at least one char before it
        if let Some(dest_domain) = destination.strip_prefix("*.") {
            // destination is also a wildcard — exact match
            return dest_domain == suffix;
        }
        if destination.ends_with(&format!(".{}", suffix)) {
            // Ensure there's a subdomain part before the suffix
            let prefix = &destination[..destination.len() - suffix.len() - 1];
            return !prefix.is_empty();
        }
        // Also match the base domain itself for *.example.com
        if destination == suffix {
            return true;
        }
    }
    false
}

/// Check whether a method is in the allowed methods list (case-insensitive).
fn method_matches(allowed: &[String], method: &str) -> bool {
    allowed.iter().any(|m| m.eq_ignore_ascii_case(method))
}

/// Request to check and potentially record an outbound network request.
#[derive(Debug, Clone)]
pub struct OutboundRequest {
    /// The principal that initiated the request.
    pub principal: ProtocolPrincipal,
    /// Package that owns the outbound request.
    pub package_id: PackageId,
    /// Capability through which the request was made.
    pub capability_id: CapabilityId,
    /// Destination host.
    pub destination_host: String,
    /// HTTP method (GET, POST, etc).
    pub method: String,
    /// Declared purpose (overrides manifest purpose if set).
    pub purpose: Option<String>,
    /// Secret references used (not raw secrets).
    pub secret_refs_used: Vec<String>,
}

impl<S> Runtime<S>
where
    S: EventStore,
{
    /// Check network policy for an outbound request and, if allowed,
    /// record an outbound audit event with redacted state.
    ///
    /// If denied, records an outbound denial audit event and returns an error.
    /// This is the primary entry point for Ygg-provided network helpers.
    pub async fn check_and_audit_outbound(
        &self,
        request: OutboundRequest,
    ) -> anyhow::Result<OutboundAuditRecord> {
        let permissions = self.packages.permissions(&request.package_id).await;
        let permissions = permissions.unwrap_or_default();

        let decision = check_network_policy(
            &permissions.network,
            &request.destination_host,
            &request.method,
        );

        let principal_str = serde_json::to_string(&request.principal)
            .unwrap_or_else(|_| "\"unknown\"".to_string());

        if !decision.allowed {
            let record = OutboundAuditRecord {
                id: new_id("ob"),
                principal: principal_str,
                package_id: request.package_id.clone(),
                capability_id: request.capability_id.clone(),
                destination_host: request.destination_host.clone(),
                method: request.method.clone(),
                purpose: request.purpose.clone(),
                redaction_state: RedactionState::NotCaptured,
                secret_refs_used: request.secret_refs_used.clone(),
                usage: json!({}),
                cost: json!({}),
                status: "denied".to_string(),
                error: decision.denial_reason.clone(),
            };
            let session_id =
                format!("kernel_outbound_{}", request.package_id.replace('/', "_"));
            self.append_kernel_event(
                &session_id,
                EVENT_OUTBOUND_DENIED,
                serde_json::to_value(&record)?,
            )
            .await?;
            anyhow::bail!(
                "outbound request denied: {}",
                decision.denial_reason.unwrap_or_default()
            );
        }

        let purpose = request
            .purpose
            .or_else(|| decision.matched_declaration.as_ref().and_then(|d| d.purpose.clone()));

        let record = OutboundAuditRecord {
            id: new_id("ob"),
            principal: principal_str,
            package_id: request.package_id.clone(),
            capability_id: request.capability_id.clone(),
            destination_host: request.destination_host.clone(),
            method: request.method.clone(),
            purpose,
            redaction_state: RedactionState::Redacted,
            secret_refs_used: request.secret_refs_used.clone(),
            usage: json!({}),
            cost: json!({}),
            status: "allowed".to_string(),
            error: None,
        };
        let session_id = format!("kernel_outbound_{}", request.package_id.replace('/', "_"));
        self.append_kernel_event(
            &session_id,
            EVENT_OUTBOUND_REQUEST,
            serde_json::to_value(&record)?,
        )
        .await?;

        Ok(record)
    }

    /// List outbound audit events for a given package (both allowed and denied).
    pub async fn list_outbound_audit(
        &self,
        package_id: &PackageId,
    ) -> anyhow::Result<Vec<ygg_core::EventEnvelope>> {
        let session_id = format!("kernel_outbound_{}", package_id.replace('/', "_"));
        // Use session+kind-prefix pushdown instead of list_session + full filter.
        let request_events = self
            .store
            .list_session_kind_prefix(&session_id, "kernel/outbound")
            .await?;
        Ok(request_events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ygg_core::NetworkPermissions;

    fn make_perms(declarations: Vec<NetworkDeclaration>, hosts: Vec<String>) -> NetworkPermissions {
        NetworkPermissions { hosts, declarations }
    }

    #[test]
    fn no_network_permission_denies() {
        let perms = make_perms(vec![], vec![]);
        let decision = check_network_policy(&perms, "api.example.com", "GET");
        assert!(!decision.allowed);
    }

    #[test]
    fn flat_hosts_allows_any_method() {
        let perms = make_perms(vec![], vec!["api.example.com".to_string()]);
        let decision = check_network_policy(&perms, "api.example.com", "POST");
        assert!(decision.allowed);
    }

    #[test]
    fn structured_declaration_allows_matching_host() {
        let perms = make_perms(
            vec![NetworkDeclaration {
                host: "api.example.com".to_string(),
                methods: vec!["GET".to_string(), "POST".to_string()],
                purpose: Some("model inference".to_string()),
            }],
            vec![],
        );
        let decision = check_network_policy(&perms, "api.example.com", "GET");
        assert!(decision.allowed);
        assert_eq!(
            decision.matched_declaration.unwrap().purpose,
            Some("model inference".to_string())
        );
    }

    #[test]
    fn structured_declaration_denies_non_matching_method() {
        let perms = make_perms(
            vec![NetworkDeclaration {
                host: "api.example.com".to_string(),
                methods: vec!["GET".to_string()],
                purpose: None,
            }],
            vec![],
        );
        let decision = check_network_policy(&perms, "api.example.com", "DELETE");
        assert!(!decision.allowed);
    }

    #[test]
    fn structured_declaration_empty_methods_allows_any() {
        let perms = make_perms(
            vec![NetworkDeclaration {
                host: "api.example.com".to_string(),
                methods: vec![],
                purpose: None,
            }],
            vec![],
        );
        let decision = check_network_policy(&perms, "api.example.com", "DELETE");
        assert!(decision.allowed);
    }

    #[test]
    fn wildcard_host_matches_subdomain() {
        let perms = make_perms(
            vec![NetworkDeclaration {
                host: "*.example.com".to_string(),
                methods: vec![],
                purpose: None,
            }],
            vec![],
        );
        let decision = check_network_policy(&perms, "api.example.com", "GET");
        assert!(decision.allowed);
    }

    #[test]
    fn wildcard_host_matches_base_domain() {
        let perms = make_perms(
            vec![NetworkDeclaration {
                host: "*.example.com".to_string(),
                methods: vec![],
                purpose: None,
            }],
            vec![],
        );
        let decision = check_network_policy(&perms, "example.com", "GET");
        assert!(decision.allowed);
    }

    #[test]
    fn wildcard_host_does_not_match_other_domain() {
        let perms = make_perms(
            vec![NetworkDeclaration {
                host: "*.example.com".to_string(),
                methods: vec![],
                purpose: None,
            }],
            vec![],
        );
        let decision = check_network_policy(&perms, "api.other.com", "GET");
        assert!(!decision.allowed);
    }

    #[test]
    fn structured_declarations_take_priority_over_flat_hosts() {
        let perms = make_perms(
            vec![NetworkDeclaration {
                host: "api.example.com".to_string(),
                methods: vec!["GET".to_string()],
                purpose: Some("structured".to_string()),
            }],
            vec!["api.example.com".to_string()],
        );
        // GET matches the structured declaration
        let decision = check_network_policy(&perms, "api.example.com", "GET");
        assert!(decision.allowed);
        assert_eq!(decision.matched_declaration.unwrap().purpose, Some("structured".to_string()));
    }

    #[test]
    fn method_matching_is_case_insensitive() {
        let perms = make_perms(
            vec![NetworkDeclaration {
                host: "api.example.com".to_string(),
                methods: vec!["get".to_string()],
                purpose: None,
            }],
            vec![],
        );
        let decision = check_network_policy(&perms, "api.example.com", "GET");
        assert!(decision.allowed);
    }

    #[test]
    fn exact_host_match() {
        assert!(host_matches("api.example.com", "api.example.com"));
        assert!(!host_matches("api.example.com", "other.example.com"));
    }

    #[test]
    fn port_specific_match() {
        assert!(host_matches("api.example.com:443", "api.example.com:443"));
        assert!(!host_matches("api.example.com:443", "api.example.com:80"));
    }
}
