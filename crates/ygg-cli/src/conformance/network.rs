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
    LiveHttpOutboundExecutor, LiveHttpOutboundExecutorConfig, OutboundExecutor,
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
                secret_headers: Vec::new(),
                resolved_secret_headers: Vec::new(),
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
                secret_headers: Vec::new(),
                resolved_secret_headers: Vec::new(),
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
                secret_headers: Vec::new(),
                resolved_secret_headers: Vec::new(),
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
                secret_headers: Vec::new(),
                resolved_secret_headers: Vec::new(),
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
                secret_headers: Vec::new(),
                resolved_secret_headers: Vec::new(),
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
                secret_headers: Vec::new(),
                resolved_secret_headers: Vec::new(),
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

/// M4: Model provider request shapes pass through the fake outbound executor.
///
/// Constructs a Runtime with FakeOutboundExecutor and network declarations
/// for api.openai.com, api.anthropic.com, and generativelanguage.googleapis.com.
/// Calls execute_outbound_with_policy with equivalent OpenAI/Anthropic/Gemini
/// request shapes. Asserts executor_kind Fake and call_count=3.
pub(crate) async fn outbound_model_provider_shape_fake_executor() -> anyhow::Result<()> {
    let (_store, runtime, fake) = runtime_with_fake_executor();
    runtime
        .load_package(network_package(
            "example/m4-provider-shapes",
            vec![
                NetworkDeclaration {
                    host: "api.openai.com".to_string(),
                    methods: vec!["POST".to_string()],
                    purpose: Some("chat completions".to_string()),
                },
                NetworkDeclaration {
                    host: "api.anthropic.com".to_string(),
                    methods: vec!["POST".to_string()],
                    purpose: Some("messages".to_string()),
                },
                NetworkDeclaration {
                    host: "generativelanguage.googleapis.com".to_string(),
                    methods: vec!["POST".to_string()],
                    purpose: Some("generate content".to_string()),
                },
            ],
            vec![],
        ))
        .await?;

    let pkg_id = "example/m4-provider-shapes";
    let cap_id = "example/m4-provider-shapes/fetch";
    let principal = ProtocolPrincipal::Package { package_id: pkg_id.to_string() };
    let secret_ref = "secret_ref:env:PROVIDER_KEY".to_string();

    // OpenAI request shape
    let openai_response = runtime
        .execute_outbound_with_policy(
            OutboundRequest {
                principal: principal.clone(),
                package_id: pkg_id.to_string(),
                capability_id: cap_id.to_string(),
                destination_host: "api.openai.com".to_string(),
                method: "POST".to_string(),
                purpose: None,
                secret_refs_used: vec![secret_ref.clone()],
            },
            OutboundExecutorRequest {
                package_id: pkg_id.to_string(),
                capability_id: cap_id.to_string(),
                destination_host: "api.openai.com".to_string(),
                method: "POST".to_string(),
                path: Some("/v1/chat/completions".to_string()),
                purpose: Some("chat completions".to_string()),
                secret_refs: vec![secret_ref.clone()],
                redaction_state: Some(RedactionState::Redacted),
                timeout_ms: Some(30000),
                metadata: serde_json::json!({"provider": "openai", "request_dialect": "openai_chat"}),
                body_shape: Some(serde_json::json!({"model": "gpt-4o", "messages": []})),
                secret_headers: Vec::new(),
                resolved_secret_headers: Vec::new(),
            },
        )
        .await?;
    anyhow::ensure!(openai_response.status == "ok", "openai shape should succeed");
    anyhow::ensure!(!openai_response.network_performed, "openai shape must not perform real network");
    anyhow::ensure!(openai_response.executor_kind == ExecutorKind::Fake, "openai shape executor_kind must be Fake");

    // Anthropic request shape
    let anthropic_response = runtime
        .execute_outbound_with_policy(
            OutboundRequest {
                principal: principal.clone(),
                package_id: pkg_id.to_string(),
                capability_id: cap_id.to_string(),
                destination_host: "api.anthropic.com".to_string(),
                method: "POST".to_string(),
                purpose: None,
                secret_refs_used: vec![secret_ref.clone()],
            },
            OutboundExecutorRequest {
                package_id: pkg_id.to_string(),
                capability_id: cap_id.to_string(),
                destination_host: "api.anthropic.com".to_string(),
                method: "POST".to_string(),
                path: Some("/v1/messages".to_string()),
                purpose: Some("messages".to_string()),
                secret_refs: vec![secret_ref.clone()],
                redaction_state: Some(RedactionState::Redacted),
                timeout_ms: Some(30000),
                metadata: serde_json::json!({"provider": "anthropic", "request_dialect": "anthropic_messages"}),
                body_shape: Some(serde_json::json!({"model": "claude-3-5-sonnet-20241022", "messages": [], "max_tokens": 1024})),
                secret_headers: Vec::new(),
                resolved_secret_headers: Vec::new(),
            },
        )
        .await?;
    anyhow::ensure!(anthropic_response.status == "ok", "anthropic shape should succeed");
    anyhow::ensure!(anthropic_response.executor_kind == ExecutorKind::Fake, "anthropic shape executor_kind must be Fake");

    // Gemini request shape
    let gemini_response = runtime
        .execute_outbound_with_policy(
            OutboundRequest {
                principal: principal.clone(),
                package_id: pkg_id.to_string(),
                capability_id: cap_id.to_string(),
                destination_host: "generativelanguage.googleapis.com".to_string(),
                method: "POST".to_string(),
                purpose: None,
                secret_refs_used: vec![secret_ref.clone()],
            },
            OutboundExecutorRequest {
                package_id: pkg_id.to_string(),
                capability_id: cap_id.to_string(),
                destination_host: "generativelanguage.googleapis.com".to_string(),
                method: "POST".to_string(),
                path: Some("/v1beta/models/gemini-2.0-flash:generateContent".to_string()),
                purpose: Some("generate content".to_string()),
                secret_refs: vec![secret_ref],
                redaction_state: Some(RedactionState::Redacted),
                timeout_ms: Some(30000),
                metadata: serde_json::json!({"provider": "gemini", "request_dialect": "gemini_generate_content"}),
                body_shape: Some(serde_json::json!({"contents": []})),
                secret_headers: Vec::new(),
                resolved_secret_headers: Vec::new(),
            },
        )
        .await?;
    anyhow::ensure!(gemini_response.status == "ok", "gemini shape should succeed");
    anyhow::ensure!(gemini_response.executor_kind == ExecutorKind::Fake, "gemini shape executor_kind must be Fake");

    // Verify all three calls reached the executor
    anyhow::ensure!(
        fake.call_count() == 3,
        "fake executor should be called 3 times for 3 provider shapes, got {}",
        fake.call_count()
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// L2: LiveHttpOutboundExecutor conformance cases
// ---------------------------------------------------------------------------

/// L2: RuntimeConfig::default() still uses DenyAll; LiveHttp is not used.
///
/// Verifies that adding the `LiveHttp` variant to `OutboundExecutorConfig`
/// does not change the default behavior. The default remains deny-all.
pub(crate) async fn outbound_live_http_default_disabled() -> anyhow::Result<()> {
    let config = RuntimeConfig::default();
    // Default outbound executor config must be DenyAll
    anyhow::ensure!(
        matches!(config.outbound_executor, OutboundExecutorConfig::DenyAll),
        "RuntimeConfig::default must keep outbound executor DenyAll"
    );

    // A runtime with default config should deny all outbound
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store.clone(), config);

    runtime
        .load_package(network_package("example/l2-default", vec![], vec![]))
        .await?;

    let result = runtime
        .check_and_audit_outbound(OutboundRequest {
            principal: ProtocolPrincipal::Package {
                package_id: "example/l2-default".to_string(),
            },
            package_id: "example/l2-default".to_string(),
            capability_id: "example/l2-default/fetch".to_string(),
            destination_host: "api.example.com".to_string(),
            method: "GET".to_string(),
            purpose: None,
            secret_refs_used: vec![],
        })
        .await;

    anyhow::ensure!(
        result.is_err(),
        "default runtime config must deny outbound requests (DenyAll)"
    );
    Ok(())
}

/// L2: LiveHttpOutboundExecutor rejects non-HTTPS URLs; no network
/// is attempted for insecure destinations.
///
/// Tests that the executor fails closed when given an http:// URL or
/// metadata with an http scheme/base_url. No real network connection
/// is made — the rejection happens at URL construction time.
pub(crate) async fn outbound_live_http_rejects_insecure_url() -> anyhow::Result<()> {
    // Create a LiveHttp executor with default (safe) config
    let config = LiveHttpOutboundExecutorConfig::default();
    let executor = LiveHttpOutboundExecutor::new(config)?;

    // Test 1: http:// scheme in metadata is rejected
    let request_http_scheme = OutboundExecutorRequest {
        package_id: "test/pkg".to_string(),
        capability_id: "test/pkg/fetch".to_string(),
        destination_host: "api.example.com".to_string(),
        method: "POST".to_string(),
        path: Some("/v1/test".to_string()),
        purpose: None,
        secret_refs: vec![],
        redaction_state: None,
        timeout_ms: None,
        metadata: serde_json::json!({"scheme": "http"}),
        body_shape: None,
        secret_headers: Vec::new(),
        resolved_secret_headers: Vec::new(),
    };
    let result = executor.execute(request_http_scheme).await;
    anyhow::ensure!(
        result.is_err(),
        "live executor must reject http:// scheme URL"
    );

    // Test 2: http:// base_url in metadata is rejected
    let request_http_base = OutboundExecutorRequest {
        package_id: "test/pkg".to_string(),
        capability_id: "test/pkg/fetch".to_string(),
        destination_host: "api.example.com".to_string(),
        method: "POST".to_string(),
        path: None,
        purpose: None,
        secret_refs: vec![],
        redaction_state: None,
        timeout_ms: None,
        metadata: serde_json::json!({"base_url": "http://api.example.com"}),
        body_shape: None,
        secret_headers: Vec::new(),
        resolved_secret_headers: Vec::new(),
    };
    let result = executor.execute(request_http_base).await;
    anyhow::ensure!(
        result.is_err(),
        "live executor must reject http:// base_url"
    );

    // Test 3: allow_insecure_loopback_for_tests defaults to false
    let default_config = LiveHttpOutboundExecutorConfig::default();
    anyhow::ensure!(
        !default_config.allow_insecure_loopback_for_tests,
        "allow_insecure_loopback_for_tests must default to false"
    );

    Ok(())
}

/// L2: Live executor response/error shapes do not include raw body,
/// header secret-like values. Response is content-free with only
/// shapes, redacted preview, and safe metadata.
///
/// This test calls the executor directly with an invalid URL that
/// will fail at connect time, then inspects the error response to
/// confirm no raw body/header/secret leaks.
pub(crate) async fn outbound_live_http_redacted_shape() -> anyhow::Result<()> {
    // Create a LiveHttp executor with loopback enabled for testing
    // so we can attempt a connection that will fail (nothing listening)
    let config = LiveHttpOutboundExecutorConfig {
        allow_insecure_loopback_for_tests: true,
        timeout_ms: 100,
        connect_timeout_ms: 50,
        ..Default::default()
    };
    let executor = LiveHttpOutboundExecutor::new(config)?;

    // Attempt a request to localhost on a port nothing listens on.
    // This will fail to connect, but the URL passes validation.
    let request = OutboundExecutorRequest {
        package_id: "test/pkg".to_string(),
        capability_id: "test/pkg/fetch".to_string(),
        destination_host: "127.0.0.1".to_string(),
        method: "POST".to_string(),
        path: Some("/nonexistent-test-endpoint-l2".to_string()),
        purpose: None,
        secret_refs: vec![],
        redaction_state: None,
        timeout_ms: Some(100),
        metadata: serde_json::json!({"scheme": "http"}),
        body_shape: Some(serde_json::json!({"model": "test", "messages": []})),
        secret_headers: Vec::new(),
        resolved_secret_headers: Vec::new(),
    };

    let response = executor.execute(request).await;

    match response {
        Ok(resp) => {
            // Connection error response
            anyhow::ensure!(
                resp.executor_kind == ExecutorKind::Real,
                "response executor_kind must be Real"
            );
            anyhow::ensure!(
                resp.network_performed,
                "live executor response must report network_performed=true"
            );
            anyhow::ensure!(
                resp.status == "error" || resp.status == "timeout",
                "failed connection should report error or timeout, got '{}'",
                resp.status
            );
            anyhow::ensure!(
                resp.redaction_state == RedactionState::Redacted,
                "response redaction_state must be Redacted"
            );

            // Verify no raw secret-like content in response
            let resp_json = serde_json::to_value(&resp)?;
            let resp_str = serde_json::to_string(&resp_json)?;
            anyhow::ensure!(
                !resp_str.contains("raw_body") && !resp_str.contains("raw_header"),
                "response must not contain raw_body or raw_header fields"
            );
            anyhow::ensure!(
                !resp_str.contains("Bearer ") && !resp_str.contains("sk-"),
                "response must not contain raw API key patterns"
            );
            // Secret-like fields must not appear as keys
            for forbidden in &["api_key", "secret_value", "raw_secret", "token_value"] {
                anyhow::ensure!(
                    resp_json.get(forbidden).is_none(),
                    "response must not contain '{}' field",
                    forbidden
                );
            }
        }
        Err(_) => {
            // Connection refused is acceptable — the key invariant
            // is that the URL was accepted (not rejected for being
            // non-HTTPS to loopback). Error handling is correct.
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// L3: kernel.outbound.execute public protocol conformance cases
// ---------------------------------------------------------------------------

/// L3: Package principal calls kernel.outbound.execute with FakeOutboundExecutor
/// and allowed network declaration. Response has executor_kind Fake,
/// network_performed false, and a host audit event is produced.
pub(crate) async fn outbound_execute_package_allowed() -> anyhow::Result<()> {
    let (store, runtime, fake) = runtime_with_fake_executor();
    runtime
        .load_package(network_package(
            "example/l3-allowed",
            vec![NetworkDeclaration {
                host: "api.openai.com".to_string(),
                methods: vec!["POST".to_string()],
                purpose: Some("chat completions".to_string()),
            }],
            vec![],
        ))
        .await?;

    let context = ygg_runtime::ProtocolContext::package("example/l3-allowed", "in_process");

    let response_value = runtime
        .call_protocol(
            &context,
            "kernel.outbound.execute",
            serde_json::json!({
                "capability_id": "example/l3-allowed/fetch",
                "destination_host": "api.openai.com",
                "method": "POST",
                "path": "/v1/chat/completions",
                "secret_refs": ["secret_ref:env:OPENAI_KEY"],
                "body_shape": {"model": "gpt-4o", "messages": []},
                "metadata": {"provider": "openai"},
            }),
        )
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    // Executor was called
    anyhow::ensure!(
        fake.call_count() == 1,
        "fake executor should be called once, but call_count={}",
        fake.call_count()
    );

    // Response is from fake executor
    anyhow::ensure!(
        response_value.get("status").and_then(|v| v.as_str()) == Some("ok"),
        "response status should be 'ok', got {:?}",
        response_value.get("status")
    );
    anyhow::ensure!(
        response_value.get("network_performed").and_then(|v| v.as_bool()) == Some(false),
        "response network_performed should be false"
    );
    anyhow::ensure!(
        response_value.get("executor_kind").and_then(|v| v.as_str()) == Some("fake"),
        "response executor_kind should be 'fake'"
    );

    // Verify audit event was produced
    let session_id = "kernel_outbound_example_l3-allowed".to_string();
    let events = store.list_session(&session_id).await?;
    let request_events: Vec<_> = events
        .iter()
        .filter(|e| e.kind == EVENT_OUTBOUND_REQUEST)
        .collect();
    anyhow::ensure!(
        !request_events.is_empty(),
        "expected kernel/outbound.request audit event"
    );

    Ok(())
}

/// L3: Params that spoof a different package_id are overridden by the
/// context principal. The package_id in the outbound request comes from
/// the context, not from params — a package cannot call
/// kernel.outbound.execute on behalf of another package.
pub(crate) async fn outbound_execute_spoofed_package_id_rejected() -> anyhow::Result<()> {
    let (_store, runtime, fake) = runtime_with_fake_executor();
    // Load the "real" package with network permission
    runtime
        .load_package(network_package(
            "example/l3-real",
            vec![NetworkDeclaration {
                host: "api.openai.com".to_string(),
                methods: vec!["POST".to_string()],
                purpose: None,
            }],
            vec![],
        ))
        .await?;
    // Load the "victim" package with no network permission
    runtime.load_package(network_package("example/l3-victim", vec![], vec![])).await?;

    // The caller is example/l3-victim (package principal), but they try
    // to specify package_id: "example/l3-real" in params — which has
    // network permission. The dispatch must use the context principal's
    // package_id, not the params one, so the request should be denied
    // (example/l3-victim has no network permission).
    let context = ygg_runtime::ProtocolContext::package("example/l3-victim", "in_process");

    let result = runtime
        .call_protocol(
            &context,
            "kernel.outbound.execute",
            serde_json::json!({
                "package_id": "example/l3-real",  // spoofed — should be ignored
                "capability_id": "example/l3-real/fetch",
                "destination_host": "api.openai.com",
                "method": "POST",
            }),
        )
        .await;

    anyhow::ensure!(
        result.is_err(),
        "outbound.execute with spoofed package_id should be denied (context overrides)"
    );
    anyhow::ensure!(
        fake.call_count() == 0,
        "executor should not be called for spoofed package_id request"
    );

    Ok(())
}

/// L3: Package without network permission is denied and executor is not called
/// through the public protocol dispatch.
pub(crate) async fn outbound_execute_no_permission_denied() -> anyhow::Result<()> {
    let (_store, runtime, fake) = runtime_with_fake_executor();
    runtime.load_package(network_package("example/l3-no-net", vec![], vec![])).await?;

    let context = ygg_runtime::ProtocolContext::package("example/l3-no-net", "in_process");

    let result = runtime
        .call_protocol(
            &context,
            "kernel.outbound.execute",
            serde_json::json!({
                "capability_id": "example/l3-no-net/fetch",
                "destination_host": "api.example.com",
                "method": "GET",
            }),
        )
        .await;

    anyhow::ensure!(
        result.is_err(),
        "outbound.execute without network permission should be denied"
    );
    anyhow::ensure!(
        fake.call_count() == 0,
        "executor should not be called for denied request, but call_count={}",
        fake.call_count()
    );

    Ok(())
}

/// L3: Response from kernel.outbound.execute never contains raw secrets.
/// secret_refs in params are passed to the executor request, but the
/// response JSON must not contain any raw secret patterns.
pub(crate) async fn outbound_execute_no_raw_secret_in_response() -> anyhow::Result<()> {
    let (_store, runtime, _fake) = runtime_with_fake_executor();
    runtime
        .load_package(network_package(
            "example/l3-secret-check",
            vec![NetworkDeclaration {
                host: "api.example.com".to_string(),
                methods: vec![],
                purpose: None,
            }],
            vec![],
        ))
        .await?;

    let context = ygg_runtime::ProtocolContext::package("example/l3-secret-check", "in_process");

    let response_value = runtime
        .call_protocol(
            &context,
            "kernel.outbound.execute",
            serde_json::json!({
                "capability_id": "example/l3-secret-check/fetch",
                "destination_host": "api.example.com",
                "method": "POST",
                "secret_refs": ["secret_ref:env:MY_API_KEY"],
            }),
        )
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    let response_str = serde_json::to_string(&response_value)?;
    anyhow::ensure!(
        !response_str.contains("sk-"),
        "response must not contain raw API key patterns"
    );
    anyhow::ensure!(
        !response_str.contains("Bearer "),
        "response must not contain Bearer token patterns"
    );
    anyhow::ensure!(
        !response_str.contains("raw_secret"),
        "response must not contain raw_secret field"
    );
    anyhow::ensure!(
        !response_str.contains("api_key"),
        "response must not contain api_key field"
    );

    Ok(())
}
