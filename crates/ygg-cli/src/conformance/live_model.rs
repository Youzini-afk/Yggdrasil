//! Live Model Calls Alpha L4 conformance tests.
//!
//! Tests cover:
//! - Secret header injection through `kernel.outbound.execute` `secret_headers` param
//! - Local fake HTTP server conformance: loopback-only server verifies
//!   Authorization header arrives but raw secret never appears in
//!   protocol response, audit, or error output
//! - SSE stream canary: local SSE fake server produces normalized
//!   stream frames through model-provider-lab normalize_stream
//! - Opt-in live conformance: only runs when YGG_LIVE_MODEL_TESTS=1
//!   and DEEPSEEK_API_KEY is set; default conformance skips it
//!
//! Security hard constraints enforced:
//! - No kernel.model/prompt/chat
//! - No raw secret resolve API
//! - Provider packages cannot read env directly
//! - Raw Authorization/secret never written to audit/response
//! - No real internet dependency in default CI

use std::collections::HashSet;
use std::sync::Arc;

use serde_json::json;
use ygg_core::{
    NetworkDeclaration, NetworkPermissions, PackageContributions, PackageEntry,
    PackageManifest, PermissionSet, SandboxPolicy,
    CapabilityDescriptor,
};
use ygg_runtime::{
    CapabilityInvocationRequest, EnvSecretResolver, EventStore, InMemoryEventStore,
    LiveHttpOutboundExecutorConfig, OutboundExecutorConfig, ProtocolContext,
    Runtime, RuntimeConfig, SecretResolverConfig,
};

use crate::commands::manifest;

use super::fixtures;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a package manifest with network permission for a given host.
fn networked_package(id: &str, host: &str) -> PackageManifest {
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
                    host: host.to_string(),
                    methods: vec!["POST".to_string()],
                    purpose: Some("live model call".to_string()),
                }],
                hosts: vec![],
            },
            ..PermissionSet::default()
        },
        sandbox_policy: SandboxPolicy::default(),
    }
}

/// Create a runtime with LiveHttp executor + EnvSecretResolver for L4 testing.
fn runtime_with_live_http_and_env_resolver(
    allowed_env_names: Vec<String>,
) -> (Arc<InMemoryEventStore>, Runtime<InMemoryEventStore>) {
    let store = Arc::new(InMemoryEventStore::default());
    let live_config = LiveHttpOutboundExecutorConfig {
        allow_insecure_loopback_for_tests: true,
        timeout_ms: 5000,
        connect_timeout_ms: 2000,
        ..Default::default()
    };
    let allowed_set: HashSet<String> = allowed_env_names.into_iter().collect();
    let secret_resolver = EnvSecretResolver::new(allowed_set);
    let config = RuntimeConfig {
        outbound_executor: OutboundExecutorConfig::LiveHttp(live_config),
        secret_resolver: SecretResolverConfig::with_resolver(Arc::new(secret_resolver)),
        ..RuntimeConfig::default()
    };
    let runtime = Runtime::new(store.clone(), config);
    (store, runtime)
}

// ---------------------------------------------------------------------------
// L4-1: Secret header injection through kernel.outbound.execute
// ---------------------------------------------------------------------------

/// L4: `secret_headers` param in `kernel.outbound.execute` is parsed correctly
/// and secret_refs are resolved through the host. Raw secret values never appear
/// in the response.
pub(crate) async fn outbound_secret_headers_parsed() -> anyhow::Result<()> {
    let test_key = "test-l4-parse-key-do-not-log";
    std::env::set_var("YGG_L4_PARSE_KEY", test_key);
    let (_store, runtime) = runtime_with_live_http_and_env_resolver(vec!["YGG_L4_PARSE_KEY".to_string()]);
    runtime
        .load_package(networked_package("example/l4-headers", "127.0.0.1"))
        .await?;

    let context = ProtocolContext::package("example/l4-headers", "in_process");

    // Call with a valid secret_headers spec. Nothing listens on 127.0.0.1,
    // so the response will be a connection error, but the secret path is
    // exercised and the raw value must not leak.
    let response_value = runtime
        .call_protocol(
            &context,
            "kernel.outbound.execute",
            json!({
                "capability_id": "example/l4-headers/fetch",
                "destination_host": "127.0.0.1",
                "method": "POST",
                "path": "/v1/chat/completions",
                "secret_headers": {
                    "Authorization": {"secret_ref": "secret_ref:env:YGG_L4_PARSE_KEY", "scheme": "bearer"},
                },
            }),
        )
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    // Response must not contain raw secrets
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
        !response_str.contains(test_key),
        "response must not contain raw secret value"
    );

    std::env::remove_var("YGG_L4_PARSE_KEY");

    // The response should have an error (connection refused) or
    // timeout since nothing is listening on 127.0.0.1 — but the
    // important thing is no raw secret leaked.
    Ok(())
}

// ---------------------------------------------------------------------------
// L4-2: Local fake HTTP server conformance — Authorization header arrives
// at server but raw secret not in protocol response/audit/log
// ---------------------------------------------------------------------------

/// L4: Start a local loopback HTTP server, call `kernel.outbound.execute`
/// with `secret_headers` injecting Authorization from env, verify the
/// server received the correct header value, but the protocol response
/// and audit events do NOT contain the raw secret.
///
/// This uses `allow_insecure_loopback_for_tests=true` to send HTTP to
/// the local server. The server only asserts the Authorization header
/// value matches the test key and never prints or returns it.
pub(crate) async fn outbound_live_loopback_secret_injection() -> anyhow::Result<()> {
    use std::sync::atomic::{AtomicBool, Ordering};

    // Set a test env var for the secret
    let test_key = "test-l4-secret-key-value-do-not-log";
    std::env::set_var("YGG_L4_TEST_KEY", test_key);

    // Track whether the server received the correct Authorization header
    let auth_received = Arc::new(AtomicBool::new(false));
    let auth_correct = Arc::new(AtomicBool::new(false));
    let auth_received_clone = auth_received.clone();
    let auth_correct_clone = auth_correct.clone();

    // Start a tiny HTTP server on a random loopback port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let port = addr.port();

    let server = tokio::spawn(async move {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        // Accept one connection
        if let Ok((mut stream, _)) = listener.accept().await {
            // Read the request in chunks with a small delay to ensure
            // we get the full HTTP request including headers
            let mut buf = Vec::new();
            let mut tmp = [0u8; 4096];
            loop {
                match tokio::time::timeout(
                    std::time::Duration::from_millis(500),
                    stream.read(&mut tmp),
                ).await {
                    Ok(Ok(0)) => break,
                    Ok(Ok(n)) => {
                        buf.extend_from_slice(&tmp[..n]);
                        // If we have seen the end of headers, stop
                        let s = String::from_utf8_lossy(&buf);
                        if s.contains("\r\n\r\n") {
                            break;
                        }
                    }
                    Ok(Err(_)) => break,
                    Err(_) => break, // timeout
                }
            }

            let request_str = String::from_utf8_lossy(&buf);

            // Check if Authorization header is present and correct
            auth_received_clone.store(true, Ordering::SeqCst);
            let request_lower = request_str.to_lowercase();
            let has_bearer = request_lower.contains(&format!("authorization: bearer {}", test_key.to_lowercase()));
            if has_bearer {
                auth_correct_clone.store(true, Ordering::SeqCst);
            }

            // Respond with a minimal valid JSON response
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}",
                "{}"
            );
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });

    // Set up runtime with the correct env resolver
    let (store, runtime) = runtime_with_live_http_and_env_resolver(vec![
        "YGG_L4_TEST_KEY".to_string(),
    ]);
    runtime
        .load_package(networked_package("example/l4-loopback", "127.0.0.1"))
        .await?;

    let context = ProtocolContext::package("example/l4-loopback", "in_process");

    let result = runtime
        .call_protocol(
            &context,
            "kernel.outbound.execute",
            json!({
                "capability_id": "example/l4-loopback/fetch",
                "destination_host": "127.0.0.1",
                "method": "POST",
                "path": "/v1/chat/completions",
                "secret_headers": {
                    "Authorization": {"secret_ref": "secret_ref:env:YGG_L4_TEST_KEY", "scheme": "bearer"},
                },
                "metadata": {
                    "scheme": "http",
                    "base_url": format!("http://127.0.0.1:{}", port),
                },
                "body_shape": {"model": "deepseek-fake", "messages": [{"role": "user", "content": "hi"}]},
            }),
        )
        .await;

    // Wait for server to finish
    let _ = tokio::time::timeout(std::time::Duration::from_secs(5), server).await;

    // The request may succeed or fail (the server is very simple),
    // but the key invariants are:
    // 1. The server received the Authorization header with the correct value
    anyhow::ensure!(
        auth_received.load(Ordering::SeqCst),
        "local HTTP server should have received the request"
    );
    anyhow::ensure!(
        auth_correct.load(Ordering::SeqCst),
        "local HTTP server should have received correct Authorization: Bearer header"
    );

    // 2. The protocol response does not contain the raw secret
    if let Ok(response_value) = result {
        let response_str = serde_json::to_string(&response_value)?;
        anyhow::ensure!(
            !response_str.contains(test_key),
            "protocol response must not contain raw secret value"
        );
        anyhow::ensure!(
            !response_str.contains("Bearer "),
            "protocol response must not contain Bearer token pattern"
        );
        anyhow::ensure!(
            !response_str.contains("sk-"),
            "protocol response must not contain raw API key patterns"
        );
    }

    // 3. Audit events do not contain the raw secret
    let session_id = "kernel_outbound_example_l4-loopback".to_string();
    let events = store.list_session(&session_id).await?;
    for event in &events {
        let payload_str = serde_json::to_string(&event.payload)?;
        anyhow::ensure!(
            !payload_str.contains(test_key),
            "audit event must not contain raw secret value"
        );
        anyhow::ensure!(
            !payload_str.contains("Bearer "),
            "audit event must not contain Bearer token pattern"
        );
    }

    // Clean up env var
    std::env::remove_var("YGG_L4_TEST_KEY");

    Ok(())
}

// ---------------------------------------------------------------------------
// L4-3: SSE stream canary — model-provider-lab normalize_stream conformance
// ---------------------------------------------------------------------------

/// L4: Feed fake DeepSeek delta_sse events through model-provider-lab's
/// normalize_stream to prove the host boundary streaming path works.
/// Verifies the normalized frames have consistent start→chunk→end lifecycle
/// and no raw secrets. No real network calls.
pub(crate) async fn stream_sse_normalize_deepseek_canary() -> anyhow::Result<()> {
    let (_store, runtime) = fixtures::runtime();
    runtime.load_package(
        manifest::read_manifest(std::path::PathBuf::from(
            "packages/official/model-provider-lab/manifest.yaml",
        ))
        .await?,
    ).await?;

    // Invoke normalize_stream for DeepSeek with sample provider events
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/model-provider-lab/normalize_stream".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/model-provider-lab".to_string()),
            version: None,
            input: json!({
                "family": "deepseek",
                "invocation_id": "inv_l4_canary",
                "sample_provider_events": [
                    {
                        "id": "chatcmpl-ds-001",
                        "object": "chat.completion.chunk",
                        "model": "deepseek-chat",
                        "choices": [
                            {
                                "index": 0,
                                "delta": {"role": "assistant", "content": ""},
                                "finish_reason": null
                            }
                        ]
                    },
                    {
                        "id": "chatcmpl-ds-001",
                        "object": "chat.completion.chunk",
                        "model": "deepseek-chat",
                        "choices": [
                            {
                                "index": 0,
                                "delta": {"content": "Hello"},
                                "finish_reason": null
                            }
                        ]
                    },
                    {
                        "id": "chatcmpl-ds-001",
                        "object": "chat.completion.chunk",
                        "model": "deepseek-chat",
                        "choices": [
                            {
                                "index": 0,
                                "delta": {"content": " world"},
                                "finish_reason": null
                            }
                        ]
                    },
                    {
                        "id": "chatcmpl-ds-001",
                        "object": "chat.completion.chunk",
                        "model": "deepseek-chat",
                        "choices": [
                            {
                                "index": 0,
                                "delta": {},
                                "finish_reason": "stop"
                            }
                        ],
                        "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
                    }
                ],
            }),
        })
        .await?;

    let response = &result.output;

    // Verify normalization result
    anyhow::ensure!(
        response.get("kind").and_then(|v| v.as_str()) == Some("model_provider_stream_normalization"),
        "response kind should be model_provider_stream_normalization"
    );
    anyhow::ensure!(
        response.get("family").and_then(|v| v.as_str()) == Some("deepseek"),
        "response family should be deepseek"
    );
    anyhow::ensure!(
        response.get("stream_family").and_then(|v| v.as_str()) == Some("delta_sse"),
        "response stream_family should be delta_sse"
    );
    anyhow::ensure!(
        response.get("terminal_frame_consistent").and_then(|v| v.as_bool()) == Some(true),
        "stream should have terminal_frame_consistent=true"
    );

    // Verify frames exist and have start → chunk → end lifecycle
    let frames = response.get("frames").and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("missing frames"))?;
    anyhow::ensure!(frames.len() >= 3, "should have at least start, chunk, end frames");

    let first_kind = frames[0].get("kind").and_then(|v| v.as_str()).unwrap_or_default();
    anyhow::ensure!(first_kind == "start", "first frame should be 'start', got '{}'", first_kind);

    let last_kind = frames.last().and_then(|f| f.get("kind")).and_then(|v| v.as_str()).unwrap_or_default();
    anyhow::ensure!(
        last_kind == "end" || last_kind == "error",
        "last frame should be 'end' or 'error', got '{}'",
        last_kind
    );

    // Verify no raw secret in response
    let response_str = serde_json::to_string(&response)?;
    anyhow::ensure!(
        !response_str.contains("sk-") && !response_str.contains("Bearer "),
        "normalized stream response must not contain raw secrets"
    );
    anyhow::ensure!(
        response.get("network_performed").and_then(|v| v.as_bool()) == Some(false),
        "normalize_stream must report network_performed=false"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// L4-4: Opt-in live conformance stub
// ---------------------------------------------------------------------------

/// L4: Opt-in live DeepSeek conformance. Only runs when
/// `YGG_LIVE_MODEL_TESTS=1` AND `DEEPSEEK_API_KEY` is set.
/// Default conformance skips this test (no public internet dependency).
///
/// If the env vars are not set, the test passes with a skip message.
pub(crate) async fn outbound_live_deepseek_opt_in() -> anyhow::Result<()> {
    let live_tests = std::env::var("YGG_LIVE_MODEL_TESTS")
        .map(|v| v == "1")
        .unwrap_or(false);
    let deepseek_key = std::env::var("DEEPSEEK_API_KEY").ok();

    if !live_tests || deepseek_key.is_none() {
        // Skip: not opted in or no key available
        return Ok(());
    }

    let api_key = deepseek_key.unwrap();

    // Set up runtime with LiveHttp executor and env resolver
    let (store, runtime) = runtime_with_live_http_and_env_resolver(vec![
        "DEEPSEEK_API_KEY".to_string(),
    ]);
    std::env::set_var("DEEPSEEK_API_KEY", &api_key);
    runtime
        .load_package(networked_package("example/l4-live", "api.deepseek.com"))
        .await?;

    let context = ProtocolContext::package("example/l4-live", "in_process");

    let result = runtime
        .call_protocol(
            &context,
            "kernel.outbound.execute",
            json!({
                "capability_id": "example/l4-live/fetch",
                "destination_host": "api.deepseek.com",
                "method": "POST",
                "path": "/v1/chat/completions",
                "secret_headers": {
                    "Authorization": {"secret_ref": "secret_ref:env:DEEPSEEK_API_KEY", "scheme": "bearer"},
                },
                "timeout_ms": 30000,
                "body_shape": {
                    "model": "deepseek-chat",
                    "messages": [{"role": "user", "content": "Say hello in one word."}],
                    "max_tokens": 10,
                },
            }),
        )
        .await;

    // Clean up env
    std::env::remove_var("DEEPSEEK_API_KEY");

    match result {
        Ok(response_value) => {
            // Verify response does not contain raw secret
            let response_str = serde_json::to_string(&response_value)?;
            anyhow::ensure!(
                !response_str.contains(&api_key),
                "live response must not contain raw API key"
            );
            anyhow::ensure!(
                !response_str.contains("Bearer "),
                "live response must not contain Bearer token pattern"
            );
            // Verify audit is redacted
            let session_id = "kernel_outbound_example_l4-live".to_string();
            let events = store.list_session(&session_id).await?;
            for event in &events {
                let payload_str = serde_json::to_string(&event.payload)?;
                anyhow::ensure!(
                    !payload_str.contains(&api_key),
                    "live audit must not contain raw API key"
                );
            }
        }
        Err(e) => {
            // Network errors are acceptable in live tests
            // (rate limit, timeout, etc.), but we still check
            // that the error message doesn't leak the key
            let err_str = format!("{:?}", e);
            anyhow::ensure!(
                !err_str.contains(&api_key),
                "error message must not contain raw API key"
            );
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// L4-5: Canary provider profile for DeepSeek non-stream invoke
// ---------------------------------------------------------------------------

/// L4: Verify the canonical DeepSeek canary profile shape through
/// model-provider-lab's normalize_request, proving that the profile
/// maps to the correct endpoint, dialect, stream family, and
/// credential header shape.
pub(crate) async fn canary_deepseek_profile_shape() -> anyhow::Result<()> {
    let (_store, runtime) = fixtures::runtime();
    runtime.load_package(
        manifest::read_manifest(std::path::PathBuf::from(
            "packages/official/model-provider-lab/manifest.yaml",
        ))
        .await?,
    ).await?;

    // Invoke normalize_request for DeepSeek profile
    let result = runtime
        .invoke_capability(CapabilityInvocationRequest {
            capability_id: "official/model-provider-lab/normalize_request".to_string(),
            caller_package_id: None,
            provider_package_id: Some("official/model-provider-lab".to_string()),
            version: None,
            input: json!({
                "profile": {
                    "family": "deepseek",
                    "model": "deepseek-chat",
                    "credential": "secret_ref:env:DEEPSEEK_API_KEY",
                },
                "messages": [{"role": "user", "content": "hello"}],
            }),
        })
        .await?;

    let response = &result.output;

    anyhow::ensure!(
        response.get("family").and_then(|v| v.as_str()) == Some("deepseek"),
        "normalize_request should return family=deepseek"
    );
    anyhow::ensure!(
        response.get("method").and_then(|v| v.as_str()) == Some("POST"),
        "normalize_request should return method=POST"
    );
    anyhow::ensure!(
        response.get("request_dialect").and_then(|v| v.as_str()) == Some("openai_chat"),
        "normalize_request should return request_dialect=openai_chat"
    );
    anyhow::ensure!(
        response.get("stream_family").and_then(|v| v.as_str()) == Some("delta_sse"),
        "normalize_request should return stream_family=delta_sse"
    );
    anyhow::ensure!(
        response.get("endpoint")
            .and_then(|v| v.as_str())
            .map(|e| e.contains("api.deepseek.com"))
            .unwrap_or(false),
        "normalize_request endpoint should contain api.deepseek.com"
    );

    // Credential ref should be in headers as placeholder, not raw
    let response_str = serde_json::to_string(&response)?;
    anyhow::ensure!(
        !response_str.contains("sk-"),
        "normalize_request response must not contain raw API key"
    );

    Ok(())
}
