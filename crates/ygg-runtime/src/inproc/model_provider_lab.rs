//! Handler for `official/model-provider-lab` capabilities.
//!
//! No-network, no-inference provider family metadata, profile validation,
//! request normalization, and error explanation across eight families:
//! openai, anthropic, gemini, openai_compatible, openrouter, deepseek, xai, fireworks.

use serde_json::Value;

use super::InprocInvocation;

const PACKAGE_ID: &str = "official/model-provider-lab";

const SUPPORTED_FAMILIES: &[&str] = &[
    "openai",
    "anthropic",
    "gemini",
    "openai_compatible",
    "openrouter",
    "deepseek",
    "xai",
    "fireworks",
];

pub fn try_handle(request: &InprocInvocation) -> Option<anyhow::Result<Value>> {
    if request.provider_package_id != PACKAGE_ID {
        return None;
    }
    let id = request.capability_id.as_str();
    if id.ends_with("/list_supported_families") {
        Some(list_supported_families(request))
    } else if id.ends_with("/validate_profile") {
        Some(validate_profile(request))
    } else if id.ends_with("/normalize_request") {
        Some(normalize_request(request))
    } else if id.ends_with("/explain_error") {
        Some(explain_error(request))
    } else if id.ends_with("/echo") {
        Some(Ok(request.input.clone()))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// list_supported_families
// ---------------------------------------------------------------------------

fn list_supported_families(request: &InprocInvocation) -> anyhow::Result<Value> {
    let families = serde_json::json!([
        {
            "id": "openai",
            "request_dialect": "openai_responses",
            "stream_family": "semantic_sse",
            "default_base_url": "https://api.openai.com",
            "auth_scheme": "bearer_secret_ref",
            "tool_modes": ["functions", "built_in_tools"],
            "usage_modes": ["top_level", "final_chunk"],
            "network_performed": false
        },
        {
            "id": "anthropic",
            "request_dialect": "anthropic_messages",
            "stream_family": "semantic_sse",
            "default_base_url": "https://api.anthropic.com",
            "auth_scheme": "x-api-key_secret_ref",
            "tool_modes": ["tool_use"],
            "usage_modes": ["cumulative_delta"],
            "network_performed": false
        },
        {
            "id": "gemini",
            "request_dialect": "gemini_generate_content",
            "stream_family": "typed_chunk_stream",
            "default_base_url": "https://generativelanguage.googleapis.com",
            "auth_scheme": "x-goog-api-key_secret_ref",
            "tool_modes": ["functions", "code_execution"],
            "usage_modes": ["usage_metadata"],
            "network_performed": false
        },
        {
            "id": "openai_compatible",
            "request_dialect": "openai_chat",
            "stream_family": "delta_sse",
            "default_base_url": null,
            "auth_scheme": "bearer_secret_ref",
            "tool_modes": ["functions"],
            "usage_modes": ["final_chunk", "top_level"],
            "network_performed": false
        },
        {
            "id": "openrouter",
            "request_dialect": "stateless_responses",
            "stream_family": "semantic_sse",
            "default_base_url": "https://openrouter.ai/api/v1",
            "auth_scheme": "bearer_secret_ref",
            "tool_modes": ["functions", "web_search"],
            "usage_modes": ["top_level"],
            "network_performed": false
        },
        {
            "id": "deepseek",
            "request_dialect": "openai_chat",
            "stream_family": "delta_sse",
            "default_base_url": "https://api.deepseek.com",
            "auth_scheme": "bearer_secret_ref",
            "tool_modes": ["functions"],
            "usage_modes": ["top_level", "final_chunk"],
            "network_performed": false
        },
        {
            "id": "xai",
            "request_dialect": "openai_responses",
            "stream_family": "semantic_sse",
            "default_base_url": "https://api.x.ai",
            "auth_scheme": "bearer_secret_ref",
            "tool_modes": ["functions", "web_search"],
            "usage_modes": ["top_level"],
            "network_performed": false
        },
        {
            "id": "fireworks",
            "request_dialect": "openai_chat",
            "stream_family": "delta_sse",
            "default_base_url": "https://api.fireworks.ai/inference/v1",
            "auth_scheme": "bearer_secret_ref",
            "tool_modes": ["functions", "mcp"],
            "usage_modes": ["top_level", "final_chunk"],
            "network_performed": false
        }
    ]);
    Ok(serde_json::json!({
        "kind": "model_provider_families",
        "families": families,
        "network_performed": false,
        "inference_performed": false,
        "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
    }))
}

// ---------------------------------------------------------------------------
// validate_profile
// ---------------------------------------------------------------------------

fn validate_profile(request: &InprocInvocation) -> anyhow::Result<Value> {
    // Accept profile as object or flat fields
    let profile = request.input.get("profile");
    let family = profile
        .and_then(|p| p.get("family"))
        .or_else(|| request.input.get("family"))
        .and_then(Value::as_str)
        .unwrap_or_default();

    let credential = profile
        .and_then(|p| p.get("credential"))
        .or_else(|| request.input.get("credential"))
        .and_then(Value::as_str)
        .unwrap_or_default();

    let base_url = profile
        .and_then(|p| p.get("baseUrl"))
        .or_else(|| request.input.get("base_url"))
        .and_then(Value::as_str)
        .unwrap_or_default();

    let headers = profile
        .and_then(|p| p.get("headers"))
        .or_else(|| request.input.get("headers"));

    let mut diagnostics = Vec::new();
    let mut network_required_hosts: Vec<String> = Vec::new();
    let mut secret_refs: Vec<String> = Vec::new();

    // Check family support
    if !SUPPORTED_FAMILIES.contains(&family) {
        diagnostics.push(serde_json::json!({
            "severity": "error",
            "field": "family",
            "message": format!("unsupported provider family: '{}'", family)
        }));
    }

    // Reject raw-looking API keys / headers
    let has_raw_secret = request.input.get("api_key").is_some()
        || request.input.get("secret").is_some()
        || looks_like_raw_secret(credential);

    if has_raw_secret {
        diagnostics.push(serde_json::json!({
            "severity": "error",
            "field": "credential",
            "message": "raw secrets are not accepted; use secret_ref: or host: reference"
        }));
    } else if !is_valid_secret_ref(credential) {
        diagnostics.push(serde_json::json!({
            "severity": "error",
            "field": "credential",
            "message": "credential must be a secret_ref: or host: reference"
        }));
    } else if !credential.is_empty() {
        secret_refs.push(credential.to_string());
    }

    // openai_compatible requires HTTPS base_url
    if family == "openai_compatible" {
        if base_url.is_empty() {
            diagnostics.push(serde_json::json!({
                "severity": "error",
                "field": "base_url",
                "message": "openai_compatible requires a base_url"
            }));
        } else if !base_url.starts_with("https://") {
            diagnostics.push(serde_json::json!({
                "severity": "error",
                "field": "base_url",
                "message": "openai_compatible requires HTTPS base_url"
            }));
        }
    }

    // General HTTPS check for base_url when provided
    if !base_url.is_empty() && !base_url.starts_with("https://") && !base_url.starts_with("http://") {
        diagnostics.push(serde_json::json!({
            "severity": "error",
            "field": "base_url",
            "message": "base_url must use HTTPS"
        }));
    }

    // OpenRouter optional headers warning
    if family == "openrouter" {
        let has_referer = headers_value(headers, "HTTP-Referer").or_else(|| headers_value(headers, "http-referer")).is_some();
        if !has_referer {
            diagnostics.push(serde_json::json!({
                "severity": "warning",
                "field": "headers",
                "message": "OpenRouter recommends setting HTTP-Referer header for attribution"
            }));
        }
        let has_title = headers_value(headers, "X-OpenRouter-Title").or_else(|| headers_value(headers, "x-openrouter-title")).is_some();
        if !has_title {
            diagnostics.push(serde_json::json!({
                "severity": "info",
                "field": "headers",
                "message": "OpenRouter supports X-OpenRouter-Title header for request labeling"
            }));
        }
    }

    // Anthropic required headers hint
    if family == "anthropic" {
        let has_version = headers_value(headers, "anthropic-version").is_some();
        if !has_version {
            diagnostics.push(serde_json::json!({
                "severity": "warning",
                "field": "headers",
                "message": "Anthropic requires the anthropic-version header (e.g. '2023-06-01')"
            }));
        }
    }

    // Gemini auth hint
    if family == "gemini" {
        let has_api_key_header = headers_value(headers, "x-goog-api-key").or_else(|| headers_value(headers, "X-Goog-Api-Key")).is_some();
        if !has_api_key_header {
            diagnostics.push(serde_json::json!({
                "severity": "info",
                "field": "headers",
                "message": "Gemini uses x-goog-api-key header for authentication; the adapter will include it from the credential ref"
            }));
        }
    }

    // Header values should not contain raw secrets
    if let Some(headers_obj) = headers.and_then(Value::as_object) {
        for (key, value) in headers_obj {
            if let Some(s) = value.as_str() {
                if looks_like_raw_secret(s) {
                    diagnostics.push(serde_json::json!({
                        "severity": "error",
                        "field": format!("headers.{}", key),
                        "message": format!("header '{}' contains a raw secret; use secret_ref: or host: reference", key)
                    }));
                }
            }
        }
    }

    // Determine network_required hosts from base_url or default
    if !base_url.is_empty() {
        if let Some(host) = extract_host(base_url) {
            network_required_hosts.push(host);
        }
    } else if let Some(default_url) = default_base_url_for_family(family) {
        if let Some(host) = extract_host(default_url) {
            network_required_hosts.push(host);
        }
    }

    let valid = !diagnostics
        .iter()
        .any(|d| d.get("severity").and_then(Value::as_str) == Some("error"));

    Ok(serde_json::json!({
        "kind": "model_provider_profile_validation",
        "valid": valid,
        "diagnostics": diagnostics,
        "network_required": network_required_hosts,
        "secret_refs": secret_refs,
        "network_performed": false,
        "inference_performed": false,
        "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
    }))
}

// ---------------------------------------------------------------------------
// normalize_request
// ---------------------------------------------------------------------------

fn normalize_request(request: &InprocInvocation) -> anyhow::Result<Value> {
    let profile = request.input.get("profile");
    let family = profile
        .and_then(|p| p.get("family"))
        .or_else(|| request.input.get("family"))
        .and_then(Value::as_str)
        .unwrap_or_default();

    let model = profile
        .and_then(|p| p.get("model"))
        .or_else(|| request.input.get("model"))
        .and_then(Value::as_str)
        .unwrap_or_default();

    let credential = profile
        .and_then(|p| p.get("credential"))
        .or_else(|| request.input.get("credential"))
        .and_then(Value::as_str)
        .unwrap_or_default();

    let base_url = profile
        .and_then(|p| p.get("baseUrl"))
        .or_else(|| request.input.get("base_url"))
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .or_else(|| default_base_url_for_family(family).map(|s| s.to_string()))
        .unwrap_or_default();

    let extra = request.input.get("extra").or_else(|| profile.and_then(|p| p.get("extra")));
    let prefer_responses = extra
        .and_then(|e| e.get("preferResponses"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let stream = request.input.get("stream").and_then(Value::as_bool).unwrap_or(false);

    let (endpoint, request_dialect, stream_family, headers, body_shape) = match family {
        "openai" => {
            let is_responses = prefer_responses;
            let dialect = if is_responses { "openai_responses" } else { "openai_chat" };
            let sf = if is_responses { "semantic_sse" } else { "delta_sse" };
            let ep = if is_responses {
                format!("{}/v1/responses", base_url)
            } else {
                format!("{}/v1/chat/completions", base_url)
            };
            let hdrs = credential_header("Authorization", "Bearer", credential);
            let body = if is_responses {
                serde_json::json!({
                    "model": model,
                    "input": request.input.get("messages"),
                    "stream": stream,
                    "max_output_tokens": request.input.get("max_tokens"),
                    "temperature": request.input.get("temperature"),
                    "tools": request.input.get("tools"),
                })
            } else {
                serde_json::json!({
                    "model": model,
                    "messages": request.input.get("messages"),
                    "stream": stream,
                    "max_tokens": request.input.get("max_tokens"),
                    "temperature": request.input.get("temperature"),
                    "tools": request.input.get("tools"),
                })
            };
            (ep, dialect, sf, hdrs, body)
        }
        "anthropic" => {
            let ep = format!("{}/v1/messages", base_url);
            let mut hdrs = serde_json::json!({
                "x-api-key": credential_ref_placeholder(credential),
                "anthropic-version": "2023-06-01",
                "Content-Type": "application/json",
            });
            // Merge profile headers
            if let Some(ph) = profile.and_then(|p| p.get("headers")).and_then(Value::as_object) {
                for (k, v) in ph {
                    if k != "anthropic-version" && v.is_string() {
                        hdrs[k] = v.clone();
                    }
                }
            }
            let body = serde_json::json!({
                "model": model,
                "messages": request.input.get("messages"),
                "system": request.input.get("system"),
                "stream": stream,
                "max_tokens": request.input.get("max_tokens"),
                "temperature": request.input.get("temperature"),
                "tools": request.input.get("tools"),
            });
            (ep, "anthropic_messages", "semantic_sse", hdrs, body)
        }
        "gemini" => {
            let stream_suffix = if stream { "?alt=sse" } else { "" };
            let ep = format!(
                "{}/v1beta/models/{}:generateContent{}",
                base_url, model, stream_suffix
            );
            let hdrs = serde_json::json!({
                "x-goog-api-key": credential_ref_placeholder(credential),
                "Content-Type": "application/json",
            });
            let body = serde_json::json!({
                "contents": request.input.get("messages"),
                "systemInstruction": request.input.get("systemInstruction"),
                "generationConfig": {
                    "maxOutputTokens": request.input.get("max_tokens"),
                    "temperature": request.input.get("temperature"),
                },
            });
            (ep, "gemini_generate_content", "typed_chunk_stream", hdrs, body)
        }
        "openai_compatible" => {
            let ep = format!("{}/chat/completions", base_url);
            let hdrs = credential_header("Authorization", "Bearer", credential);
            let body = serde_json::json!({
                "model": model,
                "messages": request.input.get("messages"),
                "stream": stream,
                "max_tokens": request.input.get("max_tokens"),
                "temperature": request.input.get("temperature"),
                "tools": request.input.get("tools"),
            });
            (ep, "openai_chat", "delta_sse", hdrs, body)
        }
        "openrouter" => {
            let is_responses = prefer_responses;
            let dialect = if is_responses { "stateless_responses" } else { "openai_chat" };
            let sf = if is_responses { "semantic_sse" } else { "delta_sse" };
            let ep = if is_responses {
                format!("{}/responses", base_url)
            } else {
                format!("{}/chat/completions", base_url)
            };
            let hdrs = credential_header("Authorization", "Bearer", credential);
            let body = if is_responses {
                serde_json::json!({
                    "model": model,
                    "input": request.input.get("messages"),
                    "stream": stream,
                })
            } else {
                serde_json::json!({
                    "model": model,
                    "messages": request.input.get("messages"),
                    "stream": stream,
                    "max_tokens": request.input.get("max_tokens"),
                    "temperature": request.input.get("temperature"),
                    "tools": request.input.get("tools"),
                })
            };
            (ep, dialect, sf, hdrs, body)
        }
        "deepseek" => {
            let ep = format!("{}/chat/completions", base_url);
            let hdrs = credential_header("Authorization", "Bearer", credential);
            let mut body = serde_json::json!({
                "model": model,
                "messages": request.input.get("messages"),
                "stream": stream,
                "max_tokens": request.input.get("max_tokens"),
                "temperature": request.input.get("temperature"),
                "tools": request.input.get("tools"),
            });
            // DeepSeek supports reasoning_effort via extra
            if let Some(effort) = extra.and_then(|e| e.get("reasoning_effort")) {
                body["reasoning_effort"] = effort.clone();
            }
            (ep, "openai_chat", "delta_sse", hdrs, body)
        }
        "xai" => {
            let is_responses = prefer_responses;
            let dialect = if is_responses { "openai_responses" } else { "openai_chat" };
            let sf = if is_responses { "semantic_sse" } else { "delta_sse" };
            let ep = if is_responses {
                format!("{}/v1/responses", base_url)
            } else {
                format!("{}/v1/chat/completions", base_url)
            };
            let hdrs = credential_header("Authorization", "Bearer", credential);
            let body = if is_responses {
                serde_json::json!({
                    "model": model,
                    "input": request.input.get("messages"),
                    "stream": stream,
                    "max_output_tokens": request.input.get("max_tokens"),
                })
            } else {
                serde_json::json!({
                    "model": model,
                    "messages": request.input.get("messages"),
                    "stream": stream,
                    "max_completion_tokens": request.input.get("max_tokens"),
                    "temperature": request.input.get("temperature"),
                    "tools": request.input.get("tools"),
                })
            };
            (ep, dialect, sf, hdrs, body)
        }
        "fireworks" => {
            let is_responses = prefer_responses;
            let dialect = if is_responses { "fireworks_responses" } else { "openai_chat" };
            let sf = if is_responses { "semantic_sse" } else { "delta_sse" };
            let ep = if is_responses {
                format!("{}/responses", base_url)
            } else {
                format!("{}/chat/completions", base_url)
            };
            let hdrs = credential_header("Authorization", "Bearer", credential);
            let body = if is_responses {
                serde_json::json!({
                    "model": model,
                    "input": request.input.get("messages"),
                    "stream": stream,
                })
            } else {
                serde_json::json!({
                    "model": model,
                    "messages": request.input.get("messages"),
                    "stream": stream,
                    "max_tokens": request.input.get("max_tokens"),
                    "temperature": request.input.get("temperature"),
                    "tools": request.input.get("tools"),
                })
            };
            (ep, dialect, sf, hdrs, body)
        }
        _ => {
            return Ok(serde_json::json!({
                "kind": "model_provider_normalized_request",
                "error": format!("unsupported provider family: '{}'", family),
                "network_performed": false,
                "inference_performed": false,
            }));
        }
    };

    Ok(serde_json::json!({
        "kind": "model_provider_normalized_request",
        "family": family,
        "method": "POST",
        "endpoint": endpoint,
        "request_dialect": request_dialect,
        "stream_family": stream_family,
        "headers": headers,
        "body_shape": body_shape,
        "credential_ref": credential_ref_placeholder(credential),
        "provider_options_namespaced": true,
        "network_performed": false,
        "inference_performed": false,
        "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
    }))
}

// ---------------------------------------------------------------------------
// explain_error
// ---------------------------------------------------------------------------

fn explain_error(request: &InprocInvocation) -> anyhow::Result<Value> {
    let status = request.input.get("status").and_then(Value::as_i64);
    let code = request.input.get("code").and_then(Value::as_str).unwrap_or_default();
    let family = request.input.get("family").and_then(Value::as_str).unwrap_or_default();
    let stage = request.input.get("stage").and_then(Value::as_str).unwrap_or("request");

    let code_lower = code.to_lowercase();

    // Try provider code mapping first
    let (kind, retryable) = if !code.is_empty() {
        map_provider_code(&code_lower)
    } else if let Some(s) = status {
        map_http_status(s)
    } else {
        ("unknown".to_string(), false)
    };

    Ok(serde_json::json!({
        "kind": "model_provider_error_explanation",
        "error_kind": kind,
        "retryable": retryable,
        "stage": stage,
        "provider_family": family,
        "provider_code": if code.is_empty() { Value::Null } else { serde_json::json!(code) },
        "http_status": status,
        "network_performed": false,
        "inference_performed": false,
        "provenance": {"package_id": request.provider_package_id, "capability_id": request.capability_id}
    }))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn is_valid_secret_ref(s: &str) -> bool {
    for prefix in &["secret_ref:", "secretRef:", "secret-ref:"] {
        if s.starts_with(prefix) {
            let after = &s[prefix.len()..];
            return after.contains(':') && after.len() > 2;
        }
    }
    if s.starts_with("host:") {
        return s.len() > 5;
    }
    false
}

fn looks_like_raw_secret(value: &str) -> bool {
    if is_valid_secret_ref(value) {
        return false;
    }
    if value.starts_with("Bearer ") || value.starts_with("bearer ") {
        return true;
    }
    if value.starts_with("sk-") || value.starts_with("sk_") {
        return true;
    }
    if value.starts_with("key-") || value.starts_with("key_") {
        return true;
    }
    if value.starts_with("AIza") {
        return true; // Gemini keys
    }
    if value.len() >= 32 {
        let alphanum: bool = value.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_');
        if alphanum {
            let has_upper = value.chars().any(|c| c.is_uppercase());
            let has_lower = value.chars().any(|c| c.is_lowercase());
            let has_digit = value.chars().any(|c| c.is_ascii_digit());
            if has_upper && has_lower && has_digit {
                return true;
            }
            // Hex string of length >= 32
            if value.chars().all(|c| c.is_ascii_hexdigit()) && value.len() >= 32 {
                return true;
            }
        }
    }
    false
}

fn credential_header(auth_header: &str, scheme: &str, credential: &str) -> Value {
    let placeholder = credential_ref_placeholder(credential);
    serde_json::json!({
        auth_header: format!("{} {}", scheme, placeholder),
        "Content-Type": "application/json",
    })
}

fn credential_ref_placeholder(credential: &str) -> String {
    if is_valid_secret_ref(credential) {
        format!("<{}>", credential)
    } else {
        "<credential_ref>".to_string()
    }
}

fn headers_value<'a>(headers: Option<&'a Value>, key: &str) -> Option<&'a str> {
    headers
        .and_then(Value::as_object)
        .and_then(|obj| obj.get(key))
        .and_then(Value::as_str)
}

fn default_base_url_for_family(family: &str) -> Option<&'static str> {
    match family {
        "openai" => Some("https://api.openai.com"),
        "anthropic" => Some("https://api.anthropic.com"),
        "gemini" => Some("https://generativelanguage.googleapis.com"),
        "openrouter" => Some("https://openrouter.ai/api/v1"),
        "deepseek" => Some("https://api.deepseek.com"),
        "xai" => Some("https://api.x.ai"),
        "fireworks" => Some("https://api.fireworks.ai/inference/v1"),
        _ => None,
    }
}

fn extract_host(url: &str) -> Option<String> {
    // Simple host extraction without regex
    let stripped = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"))?;
    let host_part = stripped.split('/').next()?;
    let host = host_part.split(':').next()?;
    Some(host.to_string())
}

fn map_provider_code(code_lower: &str) -> (String, bool) {
    // Anthropic codes
    if code_lower.contains("invalid_request") {
        return ("bad_request".to_string(), false);
    }
    if code_lower.contains("authentication_error") {
        return ("authentication".to_string(), false);
    }
    if code_lower.contains("permission_error") {
        return ("permission".to_string(), false);
    }
    if code_lower.contains("not_found_error") {
        return ("not_found".to_string(), false);
    }
    if code_lower.contains("rate_limit_error") {
        return ("rate_limit".to_string(), true);
    }
    if code_lower.contains("overloaded_error") || code_lower == "529" {
        return ("overloaded".to_string(), true);
    }
    if code_lower.contains("timeout_error") {
        return ("timeout".to_string(), true);
    }
    if code_lower.contains("api_error") {
        return ("upstream_malformed".to_string(), true);
    }
    // Gemini codes
    if code_lower == "invalid_argument" {
        return ("bad_request".to_string(), false);
    }
    if code_lower == "permission_denied" {
        return ("permission".to_string(), false);
    }
    if code_lower == "resource_exhausted" {
        return ("rate_limit".to_string(), true);
    }
    if code_lower == "not_found" {
        return ("not_found".to_string(), false);
    }
    if code_lower == "unavailable" {
        return ("overloaded".to_string(), true);
    }
    if code_lower == "deadline_exceeded" {
        return ("timeout".to_string(), true);
    }
    if code_lower == "unauthenticated" {
        return ("authentication".to_string(), false);
    }
    // OpenAI codes
    if code_lower == "invalid_api_key" {
        return ("authentication".to_string(), false);
    }
    if code_lower == "model_not_found" {
        return ("not_found".to_string(), false);
    }
    if code_lower == "insufficient_quota" {
        return ("billing".to_string(), false);
    }
    // Tool schema
    if code_lower.contains("tool") && code_lower.contains("schema") {
        return ("tool_schema".to_string(), false);
    }
    // Stream error
    if code_lower.contains("stream") {
        return ("stream_error".to_string(), true);
    }
    ("unknown".to_string(), false)
}

fn map_http_status(status: i64) -> (String, bool) {
    match status {
        400 => ("bad_request".to_string(), false),
        401 => ("authentication".to_string(), false),
        402 => ("billing".to_string(), false),
        403 => ("permission".to_string(), false),
        404 => ("not_found".to_string(), false),
        408 => ("timeout".to_string(), true),
        422 => ("tool_schema".to_string(), false),
        429 => ("rate_limit".to_string(), true),
        500 => ("upstream_malformed".to_string(), true),
        502 => ("overloaded".to_string(), true),
        503 => ("overloaded".to_string(), true),
        504 => ("timeout".to_string(), true),
        529 => ("overloaded".to_string(), true),
        _ => ("unknown".to_string(), false),
    }
}
