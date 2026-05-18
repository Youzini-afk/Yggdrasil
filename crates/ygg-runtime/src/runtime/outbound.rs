//! Content-free outbound executor abstraction (M3).
//!
//! This module defines the outbound executor boundary: the trait,
//! request/response types, and built-in deny-all and fake executors.
//! No real network I/O happens here. The executor is called *after*
//! the policy check (`check_and_audit_outbound`) passes; if the
//! policy denies the request, the executor is never invoked.
//!
//! Design principles:
//! - Fail-closed: default executor is `DenyAllOutboundExecutor`.
//! - Content-free: `body_shape` carries the JSON shape of the request
//!   body (e.g. `{"model": "...", "messages": [...]}`), but the raw
//!   body bytes are never persisted in audit records.
//! - `secret_refs` are references only; resolved secrets are never
//!   echoed back or stored.
//! - No provider-specific fields in core; opaque `metadata` for
//!   executor-specific data.
//! - This boundary secures the Ygg-provided outbound path. It does not
//!   claim to intercept arbitrary subprocess OS network calls.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ygg_core::RedactionState;

use crate::EventStore;
use super::Runtime;

// ---------------------------------------------------------------------------
// Executor request / response types
// ---------------------------------------------------------------------------

/// The kind of outbound executor that produced a response.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutorKind {
    /// All outbound is denied; no network performed.
    DenyAll,
    /// Fake/deterministic fixture executor; no real network.
    Fake,
    /// Real network executor (future M4+).
    Real,
}

/// Request sent to an outbound executor.
///
/// This is the content-free shape of an outbound request. It carries
/// identifiers, routing metadata, and the JSON *shape* of the body
/// (not the raw bytes). Secret references are just that — references,
/// never resolved raw values.
#[derive(Debug, Clone)]
pub struct OutboundExecutorRequest {
    /// Package that initiated the outbound request.
    pub package_id: String,
    /// Capability through which the request was made.
    pub capability_id: String,
    /// Destination host (e.g. `api.openai.com`).
    pub destination_host: String,
    /// HTTP method (GET, POST, etc).
    pub method: String,
    /// Optional URL path (e.g. `/v1/chat/completions`).
    pub path: Option<String>,
    /// Declared purpose (from manifest or request context).
    pub purpose: Option<String>,
    /// Secret references used (e.g. `secret_ref:env:MY_KEY`).
    /// These are identifiers only; resolved values are never stored.
    pub secret_refs: Vec<String>,
    /// Redaction state carried forward from the policy check.
    pub redaction_state: Option<RedactionState>,
    /// Optional timeout in milliseconds.
    pub timeout_ms: Option<u64>,
    /// Opaque executor-specific metadata (provider name, headers shape, etc).
    pub metadata: Value,
    /// JSON *shape* of the request body (e.g.
    /// `{"model": "gpt-4o", "messages": [...]}`).
    /// This is a structural description, not raw bytes.
    /// Raw body content is never persisted in audit.
    pub body_shape: Option<Value>,
}

/// Response returned by an outbound executor.
///
/// Like the request, this is content-free. It carries status, the
/// *shape* of headers/body, usage/cost placeholders, and metadata
/// identifying what kind of executor produced the response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundExecutorResponse {
    /// High-level status: "ok", "error", "denied", "timeout".
    pub status: String,
    /// HTTP-style status code if applicable (e.g. 200, 429, 503).
    #[serde(default)]
    pub status_code: Option<u16>,
    /// JSON *shape* of the response headers (e.g.
    /// `{"content-type": "application/json", "x-ratelimit-remaining": "..."}`).
    #[serde(default)]
    pub headers_shape: Option<Value>,
    /// JSON *shape* of the response body.
    #[serde(default)]
    pub body_shape: Option<Value>,
    /// Provider-assigned request id (e.g. `req-abc123`).
    #[serde(default)]
    pub provider_request_id: Option<String>,
    /// Usage placeholder (e.g. `{"prompt_tokens": 10, "completion_tokens": 20}`).
    #[serde(default)]
    pub usage: Value,
    /// Cost placeholder (e.g. `{"total_cost_usd": 0.002}`).
    #[serde(default)]
    pub cost: Value,
    /// Redaction state applied to the response.
    #[serde(default)]
    pub redaction_state: RedactionState,
    /// Whether real network I/O was performed.
    pub network_performed: bool,
    /// What kind of executor produced this response.
    pub executor_kind: ExecutorKind,
}

// ---------------------------------------------------------------------------
// OutboundExecutor trait
// ---------------------------------------------------------------------------

/// Trait for outbound request execution.
///
/// Implementations range from deny-all (no network), fake
/// (deterministic fixtures, no network), to real HTTP executors
/// (M4+). The executor is called only after the policy check passes.
#[async_trait]
pub trait OutboundExecutor: Send + Sync + 'static {
    /// Execute an outbound request.
    ///
    /// Called only after `check_and_audit_outbound` has approved the
    /// request. Implementations must not persist raw body/header/
    /// secret content — only shapes, refs, and metadata.
    async fn execute(&self, request: OutboundExecutorRequest) -> anyhow::Result<OutboundExecutorResponse>;
}

// ---------------------------------------------------------------------------
// DenyAllOutboundExecutor
// ---------------------------------------------------------------------------

/// An outbound executor that denies all requests without network.
///
/// This is the default, fail-closed executor. Any request that
/// reaches it returns a denied response. (In practice, the policy
/// check should already have denied the request, but this provides
/// a defense-in-depth fallback.)
pub struct DenyAllOutboundExecutor;

#[async_trait]
impl OutboundExecutor for DenyAllOutboundExecutor {
    async fn execute(&self, _request: OutboundExecutorRequest) -> anyhow::Result<OutboundExecutorResponse> {
        Ok(OutboundExecutorResponse {
            status: "denied".to_string(),
            status_code: None,
            headers_shape: None,
            body_shape: None,
            provider_request_id: None,
            usage: Value::Null,
            cost: Value::Null,
            redaction_state: RedactionState::NotCaptured,
            network_performed: false,
            executor_kind: ExecutorKind::DenyAll,
        })
    }
}

// ---------------------------------------------------------------------------
// FakeOutboundExecutor
// ---------------------------------------------------------------------------

/// A deterministic fake outbound executor for testing and conformance.
///
/// Returns fixture responses based on `(host, method, path)` keys.
/// No real network I/O is performed. If no fixture matches, returns
/// a generic 200 OK response with empty body shape.
pub struct FakeOutboundExecutor {
    /// Optional call counter for conformance assertions.
    call_count: std::sync::atomic::AtomicU64,
    /// Fixture map: `(host, method, path) → OutboundExecutorResponse`.
    fixtures: HashMap<(String, String, Option<String>), OutboundExecutorResponse>,
}

impl FakeOutboundExecutor {
    /// Create a fake executor with no fixtures (returns generic 200 OK).
    pub fn new() -> Self {
        Self {
            call_count: std::sync::atomic::AtomicU64::new(0),
            fixtures: HashMap::new(),
        }
    }

    /// Add a fixture for a given host/method/path combination.
    pub fn add_fixture(
        &mut self,
        host: &str,
        method: &str,
        path: Option<&str>,
        response: OutboundExecutorResponse,
    ) {
        self.fixtures.insert(
            (host.to_string(), method.to_string(), path.map(|s| s.to_string())),
            response,
        );
    }

    /// Get the number of times `execute` was called.
    pub fn call_count(&self) -> u64 {
        self.call_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Default for FakeOutboundExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OutboundExecutor for FakeOutboundExecutor {
    async fn execute(&self, request: OutboundExecutorRequest) -> anyhow::Result<OutboundExecutorResponse> {
        self.call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let key = (
            request.destination_host.clone(),
            request.method.clone(),
            request.path.clone(),
        );

        if let Some(fixture) = self.fixtures.get(&key) {
            return Ok(fixture.clone());
        }

        // Default: generic 200 OK with fake body shape, no network
        Ok(OutboundExecutorResponse {
            status: "ok".to_string(),
            status_code: Some(200),
            headers_shape: Some(serde_json::json!({
                "content-type": "application/json"
            })),
            body_shape: Some(serde_json::json!({
                "id": "fake_response",
                "object": "fake",
                "choices": []
            })),
            provider_request_id: Some("fake_req_001".to_string()),
            usage: serde_json::json!({"prompt_tokens": 0, "completion_tokens": 0}),
            cost: serde_json::json!({}),
            redaction_state: RedactionState::Redacted,
            network_performed: false,
            executor_kind: ExecutorKind::Fake,
        })
    }
}

// ---------------------------------------------------------------------------
// OutboundExecutorConfig — configuration for RuntimeConfig
// ---------------------------------------------------------------------------

/// Configuration for which outbound executor the runtime uses.
///
/// Defaults to `DenyAll` (fail-closed).
#[derive(Clone)]
pub enum OutboundExecutorConfig {
    /// Deny all outbound (default, fail-closed).
    DenyAll,
    /// Use a custom executor (e.g. FakeOutboundExecutor for testing).
    Custom(Arc<dyn OutboundExecutor>),
}

impl Default for OutboundExecutorConfig {
    fn default() -> Self {
        Self::DenyAll
    }
}

// ---------------------------------------------------------------------------
// Runtime method: execute_outbound_with_policy
// ---------------------------------------------------------------------------

impl<S> Runtime<S>
where
    S: EventStore,
{
    /// Execute an outbound request through policy + executor.
    ///
    /// This is the primary entry point for the M3 outbound executor
    /// boundary. It:
    ///
    /// 1. Fail-closed if the policy/audit request and executor request
    ///    disagree on package, capability, host, method, or secret refs.
    /// 2. Calls `check_and_audit_outbound` to verify network policy.
    /// 3. If denied, returns an error and **does not** call the executor.
    /// 4. If allowed, calls the configured executor.
    /// 5. Returns the executor response.
    ///
    /// Raw body/header content is never persisted in audit records.
    /// Secret references are stored as refs only.
    pub async fn execute_outbound_with_policy(
        &self,
        policy_request: super::OutboundRequest,
        executor_request: OutboundExecutorRequest,
    ) -> anyhow::Result<OutboundExecutorResponse> {
        validate_policy_executor_consistency(&policy_request, &executor_request)?;

        // Step 1: Policy check + audit. If denied, this returns an
        // error and the executor is never called.
        let _audit_record = self.check_and_audit_outbound(policy_request).await?;

        // Step 2: Policy passed — call the configured executor.
        let executor = self.outbound_executor();
        let response = executor.execute(executor_request).await?;

        Ok(response)
    }

    /// Get a reference to the configured outbound executor.
    fn outbound_executor(&self) -> Arc<dyn OutboundExecutor> {
        match &self.config.outbound_executor {
            OutboundExecutorConfig::DenyAll => Arc::new(DenyAllOutboundExecutor),
            OutboundExecutorConfig::Custom(executor) => executor.clone(),
        }
    }
}

fn validate_policy_executor_consistency(
    policy_request: &super::OutboundRequest,
    executor_request: &OutboundExecutorRequest,
) -> anyhow::Result<()> {
    if policy_request.package_id != executor_request.package_id {
        anyhow::bail!("outbound package_id mismatch between policy and executor request");
    }
    if policy_request.capability_id != executor_request.capability_id {
        anyhow::bail!("outbound capability_id mismatch between policy and executor request");
    }
    if !policy_request.destination_host.eq_ignore_ascii_case(&executor_request.destination_host) {
        anyhow::bail!("outbound destination_host mismatch between policy and executor request");
    }
    if !policy_request.method.eq_ignore_ascii_case(&executor_request.method) {
        anyhow::bail!("outbound method mismatch between policy and executor request");
    }
    if policy_request.secret_refs_used != executor_request.secret_refs {
        anyhow::bail!("outbound secret_refs mismatch between policy and executor request");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executor_kind_serialization() {
        assert_eq!(
            serde_json::to_string(&ExecutorKind::DenyAll).unwrap(),
            "\"deny_all\""
        );
        assert_eq!(
            serde_json::to_string(&ExecutorKind::Fake).unwrap(),
            "\"fake\""
        );
        assert_eq!(
            serde_json::to_string(&ExecutorKind::Real).unwrap(),
            "\"real\""
        );
    }

    #[tokio::test]
    async fn deny_all_executor_returns_denied() {
        let executor = DenyAllOutboundExecutor;
        let request = OutboundExecutorRequest {
            package_id: "test/pkg".to_string(),
            capability_id: "test/pkg/fetch".to_string(),
            destination_host: "api.example.com".to_string(),
            method: "POST".to_string(),
            path: Some("/v1/chat/completions".to_string()),
            purpose: None,
            secret_refs: vec![],
            redaction_state: None,
            timeout_ms: None,
            metadata: Value::Null,
            body_shape: None,
        };
        let response = executor.execute(request).await.unwrap();
        assert_eq!(response.status, "denied");
        assert!(!response.network_performed);
        assert_eq!(response.executor_kind, ExecutorKind::DenyAll);
    }

    #[tokio::test]
    async fn fake_executor_returns_default_ok() {
        let executor = FakeOutboundExecutor::new();
        let request = OutboundExecutorRequest {
            package_id: "test/pkg".to_string(),
            capability_id: "test/pkg/fetch".to_string(),
            destination_host: "api.openai.com".to_string(),
            method: "POST".to_string(),
            path: Some("/v1/chat/completions".to_string()),
            purpose: Some("chat completions".to_string()),
            secret_refs: vec!["secret_ref:env:OPENAI_KEY".to_string()],
            redaction_state: Some(RedactionState::Redacted),
            timeout_ms: Some(30000),
            metadata: serde_json::json!({"provider": "openai"}),
            body_shape: Some(serde_json::json!({"model": "gpt-4o", "messages": []})),
        };
        let response = executor.execute(request).await.unwrap();
        assert_eq!(response.status, "ok");
        assert_eq!(response.status_code, Some(200));
        assert!(!response.network_performed);
        assert_eq!(response.executor_kind, ExecutorKind::Fake);
        assert_eq!(executor.call_count(), 1);
    }

    #[tokio::test]
    async fn fake_executor_returns_fixture() {
        let mut executor = FakeOutboundExecutor::new();
        executor.add_fixture(
            "api.anthropic.com",
            "POST",
            Some("/v1/messages"),
            OutboundExecutorResponse {
                status: "ok".to_string(),
                status_code: Some(200),
                headers_shape: Some(serde_json::json!({"content-type": "application/json"})),
                body_shape: Some(serde_json::json!({
                    "id": "msg_fake",
                    "type": "message",
                    "content": [{"type": "text", "text": "fixture response"}]
                })),
                provider_request_id: Some("msg_fake_001".to_string()),
                usage: serde_json::json!({"input_tokens": 10, "output_tokens": 5}),
                cost: serde_json::json!({}),
                redaction_state: RedactionState::Redacted,
                network_performed: false,
                executor_kind: ExecutorKind::Fake,
            },
        );

        let request = OutboundExecutorRequest {
            package_id: "test/pkg".to_string(),
            capability_id: "test/pkg/fetch".to_string(),
            destination_host: "api.anthropic.com".to_string(),
            method: "POST".to_string(),
            path: Some("/v1/messages".to_string()),
            purpose: None,
            secret_refs: vec![],
            redaction_state: None,
            timeout_ms: None,
            metadata: Value::Null,
            body_shape: None,
        };
        let response = executor.execute(request).await.unwrap();
        assert_eq!(response.status, "ok");
        assert_eq!(response.status_code, Some(200));
        assert!(response.body_shape.is_some());
        assert_eq!(response.executor_kind, ExecutorKind::Fake);
        assert_eq!(executor.call_count(), 1);
    }

    #[tokio::test]
    async fn fake_executor_call_count_increments() {
        let executor = FakeOutboundExecutor::new();
        assert_eq!(executor.call_count(), 0);

        let request = OutboundExecutorRequest {
            package_id: "test/pkg".to_string(),
            capability_id: "test/pkg/fetch".to_string(),
            destination_host: "api.example.com".to_string(),
            method: "GET".to_string(),
            path: None,
            purpose: None,
            secret_refs: vec![],
            redaction_state: None,
            timeout_ms: None,
            metadata: Value::Null,
            body_shape: None,
        };

        let _ = executor.execute(request.clone()).await.unwrap();
        assert_eq!(executor.call_count(), 1);

        let _ = executor.execute(request.clone()).await.unwrap();
        assert_eq!(executor.call_count(), 2);
    }

    #[test]
    fn outbound_executor_config_default_is_deny_all() {
        let config = OutboundExecutorConfig::default();
        matches!(config, OutboundExecutorConfig::DenyAll);
    }

    #[test]
    fn consistency_rejects_host_mismatch() {
        let policy = super::super::OutboundRequest {
            principal: crate::ProtocolPrincipal::Package { package_id: "test/pkg".to_string() },
            package_id: "test/pkg".to_string(),
            capability_id: "test/pkg/fetch".to_string(),
            destination_host: "api.allowed.example".to_string(),
            method: "POST".to_string(),
            purpose: None,
            secret_refs_used: vec!["secret_ref:env:KEY".to_string()],
        };
        let executor = OutboundExecutorRequest {
            package_id: "test/pkg".to_string(),
            capability_id: "test/pkg/fetch".to_string(),
            destination_host: "api.evil.example".to_string(),
            method: "POST".to_string(),
            path: None,
            purpose: None,
            secret_refs: vec!["secret_ref:env:KEY".to_string()],
            redaction_state: None,
            timeout_ms: None,
            metadata: Value::Null,
            body_shape: None,
        };
        assert!(validate_policy_executor_consistency(&policy, &executor).is_err());
    }
}
