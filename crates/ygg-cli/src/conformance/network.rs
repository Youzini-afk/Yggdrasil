//! Network permission conformance tests for Phase S2.
//!
//! Tests cover:
//! - Packages without network permission are denied outbound requests.
//! - Allowlisted host+method requests are allowed with redacted audit.
//! - Host/method mismatches are denied.
//! - Official packages have no network bypass.
//! - Audit records never contain raw secrets/bodies — only secret_ref and redaction_state.

use ygg_core::{
    CapabilityDescriptor, NetworkDeclaration, NetworkPermissions, PackageContributions,
    PackageEntry, PackageManifest, PermissionSet, RedactionState, SandboxPolicy,
    EVENT_OUTBOUND_DENIED, EVENT_OUTBOUND_REQUEST,
};
use ygg_runtime::{
    check_network_policy, EventStore, OutboundRequest, ProtocolPrincipal,
};

use super::fixtures::runtime;

fn network_package(
    id: &str,
    declarations: Vec<NetworkDeclaration>,
    hosts: Vec<String>,
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
            network: NetworkPermissions { declarations, hosts },
            ..PermissionSet::default()
        },
        sandbox_policy: SandboxPolicy::default(),
    }
}

/// Package with no network permission is denied and produces outbound denied audit.
pub(crate) async fn no_network_permission_denied() -> anyhow::Result<()> {
    let (store, runtime) = runtime();
    runtime.load_package(network_package("example/no-net", vec![], vec![])).await?;

    let result = runtime
        .check_and_audit_outbound(OutboundRequest {
            principal: ProtocolPrincipal::Package {
                package_id: "example/no-net".to_string(),
            },
            package_id: "example/no-net".to_string(),
            capability_id: "example/no-net/fetch".to_string(),
            destination_host: "api.example.com".to_string(),
            method: "GET".to_string(),
            purpose: None,
            secret_refs_used: vec![],
        })
        .await;

    anyhow::ensure!(result.is_err(), "outbound request should be denied without network permission");

    // Check that an outbound denied event was recorded
    let session_id = "kernel_outbound_example_no-net".to_string();
    let events = store.list_session(&session_id).await?;
    let denied_events: Vec<_> = events.iter().filter(|e| e.kind == EVENT_OUTBOUND_DENIED).collect();
    anyhow::ensure!(!denied_events.is_empty(), "expected kernel/outbound.denied audit event");

    // Verify audit record does not contain raw body/secret
    let payload = &denied_events[0].payload;
    anyhow::ensure!(
        payload.get("status").and_then(|v| v.as_str()) == Some("denied"),
        "audit record status should be 'denied'"
    );
    anyhow::ensure!(
        payload.get("redaction_state").and_then(|v| v.as_str()) == Some("not_captured"),
        "audit record redaction_state should be 'not_captured'"
    );
    Ok(())
}

/// Allowlisted host+method is allowed and produces redacted audit.
pub(crate) async fn allowlisted_host_method_allowed() -> anyhow::Result<()> {
    let (store, runtime) = runtime();
    runtime
        .load_package(network_package(
            "example/allowlisted",
            vec![NetworkDeclaration {
                host: "api.example.com".to_string(),
                methods: vec!["GET".to_string(), "POST".to_string()],
                purpose: Some("model inference".to_string()),
            }],
            vec![],
        ))
        .await?;

    let record = runtime
        .check_and_audit_outbound(OutboundRequest {
            principal: ProtocolPrincipal::Package {
                package_id: "example/allowlisted".to_string(),
            },
            package_id: "example/allowlisted".to_string(),
            capability_id: "example/allowlisted/fetch".to_string(),
            destination_host: "api.example.com".to_string(),
            method: "POST".to_string(),
            purpose: None,
            secret_refs_used: vec!["secret_ref:env:MY_KEY".to_string()],
        })
        .await?;

    anyhow::ensure!(record.status == "allowed", "allowlisted request should be allowed");
    anyhow::ensure!(
        record.redaction_state == RedactionState::Redacted,
        "audit record should have redacted redaction_state"
    );
    anyhow::ensure!(
        record.secret_refs_used.contains(&"secret_ref:env:MY_KEY".to_string()),
        "audit record should contain secret_ref"
    );
    anyhow::ensure!(
        record.purpose == Some("model inference".to_string()),
        "audit record should carry manifest purpose"
    );

    // Verify no raw secret/body in the audit event
    let session_id = "kernel_outbound_example_allowlisted".to_string();
    let events = store.list_session(&session_id).await?;
    let request_events: Vec<_> = events.iter().filter(|e| e.kind == EVENT_OUTBOUND_REQUEST).collect();
    anyhow::ensure!(!request_events.is_empty(), "expected kernel/outbound.request audit event");
    let payload_str = serde_json::to_string(&request_events[0].payload)?;
    anyhow::ensure!(
        !payload_str.contains("raw_body") && !payload_str.contains("raw_header"),
        "audit event must not contain raw body or header fields"
    );
    Ok(())
}

/// Host/method mismatch is denied.
pub(crate) async fn host_method_mismatch_denied() -> anyhow::Result<()> {
    let (store, runtime) = runtime();
    runtime
        .load_package(network_package(
            "example/method-mismatch",
            vec![NetworkDeclaration {
                host: "api.example.com".to_string(),
                methods: vec!["GET".to_string()],
                purpose: None,
            }],
            vec![],
        ))
        .await?;

    // Wrong method
    let result = runtime
        .check_and_audit_outbound(OutboundRequest {
            principal: ProtocolPrincipal::Package {
                package_id: "example/method-mismatch".to_string(),
            },
            package_id: "example/method-mismatch".to_string(),
            capability_id: "example/method-mismatch/fetch".to_string(),
            destination_host: "api.example.com".to_string(),
            method: "DELETE".to_string(),
            purpose: None,
            secret_refs_used: vec![],
        })
        .await;
    anyhow::ensure!(result.is_err(), "wrong method should be denied");

    // Wrong host
    let result2 = runtime
        .check_and_audit_outbound(OutboundRequest {
            principal: ProtocolPrincipal::Package {
                package_id: "example/method-mismatch".to_string(),
            },
            package_id: "example/method-mismatch".to_string(),
            capability_id: "example/method-mismatch/fetch".to_string(),
            destination_host: "other.example.com".to_string(),
            method: "GET".to_string(),
            purpose: None,
            secret_refs_used: vec![],
        })
        .await;
    anyhow::ensure!(result2.is_err(), "wrong host should be denied");

    // Verify denied events
    let session_id = "kernel_outbound_example_method-mismatch".to_string();
    let events = store.list_session(&session_id).await?;
    let denied_events: Vec<_> = events.iter().filter(|e| e.kind == EVENT_OUTBOUND_DENIED).collect();
    anyhow::ensure!(denied_events.len() >= 2, "expected at least 2 outbound.denied events");
    Ok(())
}

/// Official package has no network bypass.
pub(crate) async fn official_no_network_bypass() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    // Load an "official" package with no network permission
    runtime
        .load_package(network_package("official/no-net-lab", vec![], vec![]))
        .await?;

    let result = runtime
        .check_and_audit_outbound(OutboundRequest {
            principal: ProtocolPrincipal::Package {
                package_id: "official/no-net-lab".to_string(),
            },
            package_id: "official/no-net-lab".to_string(),
            capability_id: "official/no-net-lab/fetch".to_string(),
            destination_host: "api.example.com".to_string(),
            method: "GET".to_string(),
            purpose: None,
            secret_refs_used: vec![],
        })
        .await;

    anyhow::ensure!(result.is_err(), "official package must not bypass network permission");
    Ok(())
}

/// Audit records never contain raw secrets/bodies, only secret_ref and redaction_state.
pub(crate) async fn audit_no_raw_secrets() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime
        .load_package(network_package(
            "example/audit-check",
            vec![NetworkDeclaration {
                host: "api.example.com".to_string(),
                methods: vec![],
                purpose: Some("audit test".to_string()),
            }],
            vec![],
        ))
        .await?;

    let record = runtime
        .check_and_audit_outbound(OutboundRequest {
            principal: ProtocolPrincipal::Package {
                package_id: "example/audit-check".to_string(),
            },
            package_id: "example/audit-check".to_string(),
            capability_id: "example/audit-check/fetch".to_string(),
            destination_host: "api.example.com".to_string(),
            method: "POST".to_string(),
            purpose: None,
            secret_refs_used: vec![
                "secret_ref:env:MY_API_KEY".to_string(),
                "host:internal_key".to_string(),
            ],
        })
        .await?;

    // Verify record has no raw body/header/prompt/response fields
    let record_json = serde_json::to_value(&record)?;
    let forbidden_fields = [
        "raw_body",
        "raw_header",
        "raw_prompt",
        "raw_response",
        "request_body",
        "response_body",
    ];
    for field in &forbidden_fields {
        anyhow::ensure!(
            record_json.get(field).is_none(),
            "audit record must not contain '{}' field",
            field
        );
    }

    // Verify secret_refs_used only contains references, not raw values
    for sr in &record.secret_refs_used {
        anyhow::ensure!(
            ygg_core::SecretRef::is_valid_ref(sr),
            "secret_refs_used entry '{}' must be a valid secret_ref",
            sr
        );
    }

    // Verify redaction_state is not explicitly_approved (it should be redacted)
    anyhow::ensure!(
        record.redaction_state == RedactionState::Redacted,
        "default outbound audit should have redaction_state=redacted, got {:?}",
        record.redaction_state
    );
    Ok(())
}

/// Pure function tests: check_network_policy works without runtime.
pub(crate) async fn network_policy_pure_function() -> anyhow::Result<()> {
    let perms = NetworkPermissions {
        declarations: vec![NetworkDeclaration {
            host: "api.openai.com".to_string(),
            methods: vec!["POST".to_string()],
            purpose: Some("chat completions".to_string()),
        }],
        hosts: vec!["cdn.example.com".to_string()],
    };

    // Allowed: structured declaration match
    let d = check_network_policy(&perms, "api.openai.com", "POST");
    anyhow::ensure!(d.allowed, "structured match should be allowed");

    // Allowed: flat host match
    let d = check_network_policy(&perms, "cdn.example.com", "GET");
    anyhow::ensure!(d.allowed, "flat host match should be allowed");

    // Denied: wrong host for structured
    let d = check_network_policy(&perms, "api.other.com", "POST");
    anyhow::ensure!(!d.allowed, "wrong host should be denied");

    // Denied: no permission at all
    let empty_perms = NetworkPermissions::default();
    let d = check_network_policy(&empty_perms, "anything.com", "GET");
    anyhow::ensure!(!d.allowed, "empty permissions should deny");
    Ok(())
}
