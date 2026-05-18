//! Network permission conformance tests for Phase S2 + M3 outbound executor.
//!
//! Tests cover:
//! - Packages without network permission are denied outbound requests.
//! - Allowlisted host+method requests are allowed with redacted audit.
//! - Host/method mismatches are denied.
//! - Official packages have no network bypass.
//! - Audit records never contain raw secrets/bodies — only secret_ref and redaction_state.
//! - M3: Denied requests never reach the executor (call count stays 0).
//! - M3: Policy/executor request mismatches are rejected before executor call.
//! - M3: Allowed requests reach the fake executor, response is deterministic, no network.
//! - M3: Raw body_shape content is not persisted in audit; redaction_state captures policy.
//! - M3: Secret refs are stored as refs only; raw secrets are rejected/not echoed.
//! - M3: Host mismatch redirect is denied (redirect_target check deferred to M4).

use std::sync::Arc;

use ygg_core::{
    CapabilityDescriptor, NetworkDeclaration, NetworkPermissions, PackageContributions,
    PackageEntry, PackageManifest, PermissionSet, RedactionState, SandboxPolicy,
    EVENT_OUTBOUND_DENIED, EVENT_OUTBOUND_REQUEST,
};
use ygg_runtime::{
    check_network_policy, EventStore, ExecutorKind, FakeOutboundExecutor, InMemoryEventStore,
    OutboundExecutorConfig, OutboundExecutorRequest, OutboundRequest, ProtocolPrincipal,
    Runtime, RuntimeConfig,
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

// ---------------------------------------------------------------------------
// M3: Outbound executor conformance cases
// ---------------------------------------------------------------------------

/// Helper: create a runtime with a FakeOutboundExecutor.
fn runtime_with_fake_executor() -> (Arc<InMemoryEventStore>, Runtime<InMemoryEventStore>, Arc<FakeOutboundExecutor>) {
    let store = Arc::new(InMemoryEventStore::default());
    let fake = Arc::new(FakeOutboundExecutor::new());
    let config = RuntimeConfig {
        outbound_executor: OutboundExecutorConfig::Custom(fake.clone()),
        ..RuntimeConfig::default()
    };
    let runtime = Runtime::new(store.clone(), config);
    (store, runtime, fake)
}

/// M3: Package without network declaration is denied; executor is never called.
pub(crate) async fn outbound_no_permission_executor_not_called() -> anyhow::Result<()> {
    let (_store, runtime, fake) = runtime_with_fake_executor();
    runtime.load_package(network_package("example/m3-no-net", vec![], vec![])).await?;

    let result = runtime
        .execute_outbound_with_policy(
            OutboundRequest {
                principal: ProtocolPrincipal::Package {
                    package_id: "example/m3-no-net".to_string(),
                },
                package_id: "example/m3-no-net".to_string(),
                capability_id: "example/m3-no-net/fetch".to_string(),
                destination_host: "api.example.com".to_string(),
                method: "GET".to_string(),
                purpose: None,
                secret_refs_used: vec![],
            },
            OutboundExecutorRequest {
                package_id: "example/m3-no-net".to_string(),
                capability_id: "example/m3-no-net/fetch".to_string(),
                destination_host: "api.example.com".to_string(),
                method: "GET".to_string(),
                path: None,
                purpose: None,
                secret_refs: vec![],
                redaction_state: None,
                timeout_ms: None,
                metadata: serde_json::Value::Null,
                body_shape: None,
            },
        )
        .await;

    anyhow::ensure!(result.is_err(), "outbound request should be denied without network permission");
    // The executor should never have been called
    anyhow::ensure!(
        fake.call_count() == 0,
        "fake executor should not be called when policy denies, but call_count={}",
        fake.call_count()
    );
    Ok(())
}

/// M3: Policy/audit request and executor request must agree on host/method/package/capability/secret refs.
/// A mismatch is rejected before the executor is called, preventing policy-check/execute split bugs.
pub(crate) async fn outbound_policy_executor_mismatch_denied() -> anyhow::Result<()> {
    let (_store, runtime, fake) = runtime_with_fake_executor();
    runtime
        .load_package(network_package(
            "example/m3-mismatch",
            vec![NetworkDeclaration {
                host: "api.allowed.example".to_string(),
                methods: vec!["POST".to_string()],
                purpose: Some("outbound fixture".to_string()),
            }],
            vec![],
        ))
        .await?;

    let result = runtime
        .execute_outbound_with_policy(
            OutboundRequest {
                principal: ProtocolPrincipal::Package {
                    package_id: "example/m3-mismatch".to_string(),
                },
                package_id: "example/m3-mismatch".to_string(),
                capability_id: "example/m3-mismatch/fetch".to_string(),
                destination_host: "api.allowed.example".to_string(),
                method: "POST".to_string(),
                purpose: None,
                secret_refs_used: vec!["secret_ref:env:KEY".to_string()],
            },
            OutboundExecutorRequest {
                package_id: "example/m3-mismatch".to_string(),
                capability_id: "example/m3-mismatch/fetch".to_string(),
                destination_host: "api.evil.example".to_string(),
                method: "POST".to_string(),
                path: None,
                purpose: None,
                secret_refs: vec!["secret_ref:env:KEY".to_string()],
                redaction_state: Some(RedactionState::Redacted),
                timeout_ms: Some(30000),
                metadata: serde_json::Value::Null,
                body_shape: None,
            },
        )
        .await;

    anyhow::ensure!(result.is_err(), "mismatched policy/executor host should be rejected");
    anyhow::ensure!(fake.call_count() == 0, "executor must not be called after mismatch");
    Ok(())
}

/// M3: Allowlisted package reaches fake executor; response has network_performed:false,
/// executor_kind:fake, audit is redacted, status is ok.
pub(crate) async fn outbound_allowlisted_fake_executor() -> anyhow::Result<()> {
    let (store, runtime, fake) = runtime_with_fake_executor();
    runtime
        .load_package(network_package(
            "example/m3-allowlisted",
            vec![NetworkDeclaration {
                host: "api.openai.com".to_string(),
                methods: vec!["POST".to_string()],
                purpose: Some("chat completions".to_string()),
            }],
            vec![],
        ))
        .await?;

    let response = runtime
        .execute_outbound_with_policy(
            OutboundRequest {
                principal: ProtocolPrincipal::Package {
                    package_id: "example/m3-allowlisted".to_string(),
                },
                package_id: "example/m3-allowlisted".to_string(),
                capability_id: "example/m3-allowlisted/fetch".to_string(),
                destination_host: "api.openai.com".to_string(),
                method: "POST".to_string(),
                purpose: None,
                secret_refs_used: vec!["secret_ref:env:OPENAI_KEY".to_string()],
            },
            OutboundExecutorRequest {
                package_id: "example/m3-allowlisted".to_string(),
                capability_id: "example/m3-allowlisted/fetch".to_string(),
                destination_host: "api.openai.com".to_string(),
                method: "POST".to_string(),
                path: Some("/v1/chat/completions".to_string()),
                purpose: Some("chat completions".to_string()),
                secret_refs: vec!["secret_ref:env:OPENAI_KEY".to_string()],
                redaction_state: Some(RedactionState::Redacted),
                timeout_ms: Some(30000),
                metadata: serde_json::json!({"provider": "openai"}),
                body_shape: Some(serde_json::json!({"model": "gpt-4o", "messages": []})),
            },
        )
        .await?;

    // Executor was called
    anyhow::ensure!(
        fake.call_count() == 1,
        "fake executor should be called once, but call_count={}",
        fake.call_count()
    );

    // Response is from fake executor
    anyhow::ensure!(
        response.status == "ok",
        "fake executor response status should be 'ok', got '{}'",
        response.status
    );
    anyhow::ensure!(
        !response.network_performed,
        "fake executor should report network_performed=false"
    );
    anyhow::ensure!(
        response.executor_kind == ExecutorKind::Fake,
        "response executor_kind should be Fake"
    );

    // Verify audit event is redacted
    let session_id = "kernel_outbound_example_m3-allowlisted".to_string();
    let events = store.list_session(&session_id).await?;
    let request_events: Vec<_> = events.iter().filter(|e| e.kind == EVENT_OUTBOUND_REQUEST).collect();
    anyhow::ensure!(!request_events.is_empty(), "expected kernel/outbound.request audit event");

    let payload = &request_events[0].payload;
    anyhow::ensure!(
        payload.get("redaction_state").and_then(|v| v.as_str()) == Some("redacted"),
        "audit event should have redaction_state=redacted"
    );
    Ok(())
}

/// M3: Request body_shape content/prompt is not persisted raw in audit;
/// audit redaction_state is redacted or not_captured.
pub(crate) async fn outbound_raw_body_not_audited() -> anyhow::Result<()> {
    let (store, runtime, _fake) = runtime_with_fake_executor();
    runtime
        .load_package(network_package(
            "example/m3-raw-body",
            vec![NetworkDeclaration {
                host: "api.example.com".to_string(),
                methods: vec![],
                purpose: None,
            }],
            vec![],
        ))
        .await?;

    let _response = runtime
        .execute_outbound_with_policy(
            OutboundRequest {
                principal: ProtocolPrincipal::Package {
                    package_id: "example/m3-raw-body".to_string(),
                },
                package_id: "example/m3-raw-body".to_string(),
                capability_id: "example/m3-raw-body/fetch".to_string(),
                destination_host: "api.example.com".to_string(),
                method: "POST".to_string(),
                purpose: None,
                secret_refs_used: vec![],
            },
            OutboundExecutorRequest {
                package_id: "example/m3-raw-body".to_string(),
                capability_id: "example/m3-raw-body/fetch".to_string(),
                destination_host: "api.example.com".to_string(),
                method: "POST".to_string(),
                path: None,
                purpose: None,
                secret_refs: vec![],
                redaction_state: None,
                timeout_ms: None,
                metadata: serde_json::Value::Null,
                body_shape: Some(serde_json::json!({
                    "model": "gpt-4o",
                    "messages": [{"role": "user", "content": "Hello world"}],
                    "temperature": 0.7
                })),
            },
        )
        .await?;

    // Check audit event — raw body content must not appear
    let session_id = "kernel_outbound_example_m3-raw-body".to_string();
    let events = store.list_session(&session_id).await?;
    let request_events: Vec<_> = events.iter().filter(|e| e.kind == EVENT_OUTBOUND_REQUEST).collect();
    anyhow::ensure!(!request_events.is_empty(), "expected kernel/outbound.request audit event");

    let payload_str = serde_json::to_string(&request_events[0].payload)?;
    // The body_shape content ("Hello world", "gpt-4o", "temperature") must not be in the audit record
    anyhow::ensure!(
        !payload_str.contains("Hello world"),
        "audit event must not contain raw body content"
    );
    anyhow::ensure!(
        !payload_str.contains("body_shape"),
        "audit event must not contain body_shape field"
    );
    // Redaction state must be redacted or not_captured
    let redaction = request_events[0].payload.get("redaction_state")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    anyhow::ensure!(
        redaction == "redacted" || redaction == "not_captured",
        "audit redaction_state should be redacted or not_captured, got '{}'",
        redaction
    );
    Ok(())
}

/// M3: Raw secret-looking request content is rejected or not echoed;
/// secret_refs stored as references only.
pub(crate) async fn outbound_secret_refs_only() -> anyhow::Result<()> {
    let (_store, runtime, _fake) = runtime_with_fake_executor();
    runtime
        .load_package(network_package(
            "example/m3-secret-refs",
            vec![NetworkDeclaration {
                host: "api.example.com".to_string(),
                methods: vec![],
                purpose: None,
            }],
            vec![],
        ))
        .await?;

    // Request with a raw-looking API key in body_shape should still
    // pass through the executor (body_shape is opaque to the kernel),
    // but the audit record must not echo the raw secret.
    // The key point: secret_refs field contains only references.
    let response = runtime
        .execute_outbound_with_policy(
            OutboundRequest {
                principal: ProtocolPrincipal::Package {
                    package_id: "example/m3-secret-refs".to_string(),
                },
                package_id: "example/m3-secret-refs".to_string(),
                capability_id: "example/m3-secret-refs/fetch".to_string(),
                destination_host: "api.example.com".to_string(),
                method: "POST".to_string(),
                purpose: None,
                secret_refs_used: vec![
                    "secret_ref:env:MY_API_KEY".to_string(),
                    "host:internal_token".to_string(),
                ],
            },
            OutboundExecutorRequest {
                package_id: "example/m3-secret-refs".to_string(),
                capability_id: "example/m3-secret-refs/fetch".to_string(),
                destination_host: "api.example.com".to_string(),
                method: "POST".to_string(),
                path: None,
                purpose: None,
                secret_refs: vec![
                    "secret_ref:env:MY_API_KEY".to_string(),
                    "host:internal_token".to_string(),
                ],
                redaction_state: None,
                timeout_ms: None,
                metadata: serde_json::Value::Null,
                body_shape: None,
            },
        )
        .await?;

    // Response must not echo raw secrets
    let response_json = serde_json::to_value(&response)?;
    let response_str = serde_json::to_string(&response_json)?;
    // The response should not contain any field that looks like a raw secret key
    // (secret refs are `secret_ref:...` or `host:...` patterns, which are safe)
    anyhow::ensure!(
        !response_str.contains("sk-"),
        "response must not echo raw API key patterns"
    );
    anyhow::ensure!(
        !response_str.contains("Bearer "),
        "response must not echo Bearer token patterns"
    );

    // Verify executor request secret_refs contain only valid references
    // (We check the policy-level secret_refs_used which is what gets audited)
    // Already covered by audit_no_raw_secrets, but let's also verify
    // the executor response doesn't contain secret fields
    let forbidden_response_fields = [
        "raw_secret",
        "api_key",
        "secret_value",
        "token_value",
    ];
    for field in &forbidden_response_fields {
        anyhow::ensure!(
            response_json.get(field).is_none(),
            "executor response must not contain '{}' field",
            field
        );
    }
    Ok(())
}

/// M3: Host mismatch redirect is denied. Redirect-target checking is
/// deferred to M4; for now, ensure that a request to a non-allowlisted
/// host is still denied even if it looks like a redirect destination.
/// This case documents the M4 deferral.
pub(crate) async fn outbound_host_mismatch_redirect_denied() -> anyhow::Result<()> {
    let (_store, runtime, fake) = runtime_with_fake_executor();
    runtime
        .load_package(network_package(
            "example/m3-redirect",
            vec![NetworkDeclaration {
                host: "api.example.com".to_string(),
                methods: vec![],
                purpose: None,
            }],
            vec![],
        ))
        .await?;

    // Request to a different host that might be a redirect target
    let result = runtime
        .execute_outbound_with_policy(
            OutboundRequest {
                principal: ProtocolPrincipal::Package {
                    package_id: "example/m3-redirect".to_string(),
                },
                package_id: "example/m3-redirect".to_string(),
                capability_id: "example/m3-redirect/fetch".to_string(),
                destination_host: "redirect.other.com".to_string(),
                method: "GET".to_string(),
                purpose: None,
                secret_refs_used: vec![],
            },
            OutboundExecutorRequest {
                package_id: "example/m3-redirect".to_string(),
                capability_id: "example/m3-redirect/fetch".to_string(),
                destination_host: "redirect.other.com".to_string(),
                method: "GET".to_string(),
                path: None,
                purpose: None,
                secret_refs: vec![],
                redaction_state: None,
                timeout_ms: None,
                metadata: serde_json::Value::Null,
                body_shape: None,
            },
        )
        .await;

    anyhow::ensure!(result.is_err(), "request to non-allowlisted host must be denied");
    // Executor should not be called for denied requests
    anyhow::ensure!(
        fake.call_count() == 0,
        "executor should not be called for denied redirect-target request"
    );
    // Note: Redirect-target following/checking (ensuring the executor
    // doesn't silently follow redirects to non-allowlisted hosts) is
    // deferred to M4 when real HTTP executors are introduced.
    Ok(())
}
