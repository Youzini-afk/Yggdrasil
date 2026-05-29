//! Handler for `official/docker-runtime-lab` capabilities.
//!
//! Docker Runtime Lab is an ordinary package. It performs Docker container
//! lifecycle actions only after host-provided approval and metadata are present.
//! The host/kernel owns port leases and proxy routes; this package never invokes
//! HostAdmin-only kernel port/proxy capabilities.

use std::collections::HashMap;

use bollard::container::LogOutput;
use bollard::models::{ContainerCreateBody, HostConfig, PortBinding, PortMap};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, CreateImageOptionsBuilder, InspectContainerOptionsBuilder,
    LogsOptionsBuilder, RemoveContainerOptionsBuilder, StopContainerOptionsBuilder,
};
use bollard::Docker;
use futures::StreamExt;
use serde_json::Value;

use super::safety;
use super::InprocInvocation;

const PACKAGE_ID: &str = "official/docker-runtime-lab";
const BIND_HOST: &str = "127.0.0.1";
const DEFAULT_TAIL: u32 = 200;
const DEFAULT_MAX_BYTES: usize = 65_536;
const MAX_TAIL: u32 = 5_000;

pub fn try_handle(request: &InprocInvocation) -> Option<anyhow::Result<Value>> {
    if request.provider_package_id != PACKAGE_ID {
        return None;
    }

    let id = request.capability_id.as_str();
    if id.ends_with("/describe_contract") {
        Some(describe_contract(request))
    } else if id.ends_with("/validate_spec") {
        Some(validate_spec(request))
    } else if id.ends_with("/plan_container") {
        Some(plan_container(request))
    } else if id.ends_with("/start_container") {
        Some(start_container(request))
    } else if id.ends_with("/status") {
        Some(status(request))
    } else if id.ends_with("/logs") {
        Some(logs(request))
    } else if id.ends_with("/stop_container") {
        Some(stop_container(request))
    } else {
        None
    }
}

fn describe_contract(request: &InprocInvocation) -> anyhow::Result<Value> {
    Ok(serde_json::json!({
        "kind": "docker_runtime_lab_contract",
        "package_id": PACKAGE_ID,
        "package_kind": "ordinary",
        "capabilities": [
            {"id": "official/docker-runtime-lab/describe_contract", "docker_performed": false},
            {"id": "official/docker-runtime-lab/validate_spec", "docker_performed": false},
            {"id": "official/docker-runtime-lab/plan_container", "docker_performed": false},
            {"id": "official/docker-runtime-lab/start_container", "docker_performed": true, "requires_approved_true": true},
            {"id": "official/docker-runtime-lab/status", "docker_performed": true},
            {"id": "official/docker-runtime-lab/logs", "docker_performed": true, "bounded": true, "redacted": true},
            {"id": "official/docker-runtime-lab/stop_container", "docker_performed": true}
        ],
        "host_owned_resources": {
            "port_lease": true,
            "proxy_route": true,
            "accepted_metadata_fields": ["host_port", "port_lease_id", "route_id"]
        },
        "enforced_policy": {
            "privileged": false,
            "network_mode": "bridge",
            "bind_host": BIND_HOST,
            "bind_mounts": false,
            "mounts": false,
            "arbitrary_env": false,
            "secrets_injection": false,
            "publish_all_ports": false
        },
        "inference_performed": false,
        "network_performed": false,
        "execution_performed": false,
        "docker_performed": false,
        "provenance": provenance(request)
    }))
}

fn validate_spec(request: &InprocInvocation) -> anyhow::Result<Value> {
    let mut diagnostics = validate_input(&request.input);
    if safety::contains_raw_secret(&request.input) {
        diagnostics.push(error(
            "raw_secret_blocked",
            "input contains raw-secret-like content; use secret_ref references instead",
        ));
    }
    let valid = diagnostics.iter().all(|d| d.severity != "error");

    Ok(serde_json::json!({
        "kind": "docker_runtime_lab_validation",
        "valid": valid,
        "diagnostics": diagnostics_to_json(&mut diagnostics),
        "docker_performed": false,
        "provenance": provenance(request)
    }))
}

fn plan_container(request: &InprocInvocation) -> anyhow::Result<Value> {
    let mut diagnostics = validate_input(&request.input);
    if safety::contains_raw_secret(&request.input) {
        diagnostics.push(error(
            "raw_secret_blocked",
            "input contains raw-secret-like content; use secret_ref references instead",
        ));
    }
    let valid = diagnostics.iter().all(|d| d.severity != "error");
    let image = string_field(&request.input, "image");
    let container_port = u16_field(&request.input, "container_port");
    let host_port = u16_field(&request.input, "host_port");
    let route_id = string_field(&request.input, "route_id");
    let port_lease_id = string_field(&request.input, "port_lease_id");

    Ok(serde_json::json!({
        "kind": "docker_runtime_lab_container_plan",
        "valid": valid,
        "diagnostics": diagnostics_to_json(&mut diagnostics),
        "image": value_or_null(image),
        "container_port": container_port,
        "host_port": host_port,
        "bind_host": BIND_HOST,
        "route_id": value_or_null(route_id),
        "port_lease_id": value_or_null(port_lease_id),
        "requires_host_port_lease": true,
        "requires_proxy_route": true,
        "docker_performed": false,
        "host_owned_resources": ["host_port", "port_lease_id", "route_id"],
        "enforced_policy": {
            "privileged": false,
            "network_mode": "bridge",
            "bind_mounts": false,
            "mounts": false,
            "env": [],
            "publish_all_ports": false
        },
        "provenance": provenance(request)
    }))
}

fn start_container(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request));
    }
    if !request
        .input
        .get("approved")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Ok(serde_json::json!({
            "kind": "docker_runtime_lab_rejected",
            "reason": "start_container requires approved: true; fail-closed",
            "docker_performed": false,
            "container_started": false,
            "provenance": provenance(request)
        }));
    }

    let diagnostics = validate_input(&request.input);
    if diagnostics.iter().any(|d| d.severity == "error") {
        let mut diagnostics = diagnostics;
        return Ok(serde_json::json!({
            "kind": "docker_runtime_lab_rejected",
            "reason": "container spec failed validation; fail-closed",
            "diagnostics": diagnostics_to_json(&mut diagnostics),
            "docker_performed": false,
            "container_started": false,
            "provenance": provenance(request)
        }));
    }

    let image = string_field(&request.input, "image").unwrap_or_default();
    let container_port = u16_field(&request.input, "container_port").unwrap_or_default();
    let host_port = u16_field(&request.input, "host_port").unwrap_or_default();
    let route_id = string_field(&request.input, "route_id").unwrap_or_default();
    let port_lease_id = string_field(&request.input, "port_lease_id").unwrap_or_default();
    let pull_if_missing = request
        .input
        .get("pull_if_missing")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let container_name = string_field(&request.input, "container_name")
        .or_else(|| string_field(&request.input, "name"))
        .unwrap_or_else(|| {
            format!(
                "ygg-docker-runtime-lab-{}-{}",
                sanitize_name(&route_id),
                host_port
            )
        });

    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async move {
            start_container_async(
                image,
                container_port,
                host_port,
                route_id,
                port_lease_id,
                container_name,
                pull_if_missing,
            )
            .await
        })
    });

    match result {
        Ok(value) => Ok(value),
        Err(error) => Ok(with_provenance(
            docker_error_output("start_container", error),
            request,
        )),
    }
}

fn status(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request));
    }
    let Some(container) = container_ref(&request.input) else {
        return Ok(missing_container_ref_output(request));
    };
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async move { status_async(container).await })
    });
    match result {
        Ok(value) => Ok(with_provenance(value, request)),
        Err(error) => Ok(with_provenance(
            docker_error_output("status", error),
            request,
        )),
    }
}

fn logs(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request));
    }
    let Some(container) = container_ref(&request.input) else {
        return Ok(missing_container_ref_output(request));
    };
    let tail = request
        .input
        .get("tail")
        .and_then(Value::as_u64)
        .map(|n| n.min(MAX_TAIL as u64) as u32)
        .unwrap_or(DEFAULT_TAIL);
    let max_bytes = request
        .input
        .get("max_bytes")
        .and_then(Value::as_u64)
        .map(|n| n.min(DEFAULT_MAX_BYTES as u64) as usize)
        .unwrap_or(DEFAULT_MAX_BYTES);
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async move { logs_async(container, tail, max_bytes).await })
    });
    match result {
        Ok(value) => Ok(with_provenance(value, request)),
        Err(error) => Ok(with_provenance(docker_error_output("logs", error), request)),
    }
}

fn stop_container(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request));
    }
    let Some(container) = container_ref(&request.input) else {
        return Ok(missing_container_ref_output(request));
    };
    let timeout_secs = request
        .input
        .get("timeout_secs")
        .and_then(Value::as_i64)
        .map(|n| n.clamp(0, 60) as i32)
        .unwrap_or(10);
    let force = request
        .input
        .get("force")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async move { stop_container_async(container, timeout_secs, force).await })
    });
    match result {
        Ok(value) => Ok(with_provenance(value, request)),
        Err(error) => Ok(with_provenance(
            docker_error_output("stop_container", error),
            request,
        )),
    }
}

async fn docker() -> Result<Docker, String> {
    let docker = Docker::connect_with_local_defaults()
        .or_else(|_| Docker::connect_with_defaults())
        .map_err(|e| format!("docker connection unavailable: {e}"))?;
    docker
        .ping()
        .await
        .map_err(|e| format!("docker daemon unavailable: {e}"))?;
    Ok(docker)
}

async fn start_container_async(
    image: String,
    container_port: u16,
    host_port: u16,
    route_id: String,
    port_lease_id: String,
    container_name: String,
    pull_if_missing: bool,
) -> Result<Value, String> {
    let docker = docker().await?;
    if pull_if_missing {
        let options = CreateImageOptionsBuilder::default()
            .from_image(&image)
            .build();
        let mut stream = docker.create_image(Some(options), None, None);
        while let Some(item) = stream.next().await {
            item.map_err(|e| format!("docker image pull failed: {e}"))?;
        }
    }

    let container_port_key = format!("{container_port}/tcp");
    let mut port_bindings: PortMap = HashMap::new();
    port_bindings.insert(
        container_port_key.clone(),
        Some(vec![PortBinding {
            host_ip: Some(BIND_HOST.to_string()),
            host_port: Some(host_port.to_string()),
        }]),
    );

    let labels = HashMap::from([
        ("managed-by".to_string(), "yggdrasil".to_string()),
        ("yggdrasil.package_id".to_string(), PACKAGE_ID.to_string()),
        ("yggdrasil.route_id".to_string(), route_id.clone()),
        ("yggdrasil.port_lease_id".to_string(), port_lease_id.clone()),
    ]);
    let config = ContainerCreateBody {
        image: Some(image.clone()),
        labels: Some(labels),
        exposed_ports: Some(vec![container_port_key]),
        env: None,
        host_config: Some(HostConfig {
            binds: None,
            mounts: None,
            network_mode: Some("bridge".to_string()),
            port_bindings: Some(port_bindings),
            privileged: Some(false),
            publish_all_ports: Some(false),
            ..Default::default()
        }),
        network_disabled: Some(false),
        ..Default::default()
    };
    let options = CreateContainerOptionsBuilder::default()
        .name(&container_name)
        .build();
    let created = docker
        .create_container(Some(options), config)
        .await
        .map_err(|e| format!("docker create failed: {e}"))?;
    docker
        .start_container(&created.id, None)
        .await
        .map_err(|e| format!("docker start failed: {e}"))?;

    Ok(serde_json::json!({
        "kind": "docker_runtime_lab_container_started",
        "container_id": created.id,
        "container_name": container_name,
        "status": "started",
        "image": image,
        "container_port": container_port,
        "host_port": host_port,
        "bind_host": BIND_HOST,
        "route_id": route_id,
        "port_lease_id": port_lease_id,
        "docker_performed": true,
        "container_started": true,
        "warnings": created.warnings
    }))
}

async fn status_async(container: String) -> Result<Value, String> {
    let docker = docker().await?;
    let options = InspectContainerOptionsBuilder::default()
        .size(false)
        .build();
    let inspected = docker
        .inspect_container(&container, Some(options))
        .await
        .map_err(|e| format!("docker inspect failed: {e}"))?;
    let state = inspected.state;
    let running = state.as_ref().and_then(|s| s.running).unwrap_or(false);
    let status = state
        .as_ref()
        .and_then(|s| s.status.map(|status| status.to_string()))
        .unwrap_or_else(|| "unknown".to_string());
    let exit_code = state.as_ref().and_then(|s| s.exit_code);
    Ok(serde_json::json!({
        "kind": "docker_runtime_lab_status",
        "container_ref": container,
        "container_id": inspected.id,
        "container_name": inspected.name.map(|name| name.trim_start_matches('/').to_string()),
        "running": running,
        "state": status,
        "exit_code": exit_code,
        "docker_performed": true
    }))
}

async fn logs_async(container: String, tail: u32, max_bytes: usize) -> Result<Value, String> {
    let docker = docker().await?;
    let options = LogsOptionsBuilder::default()
        .stdout(true)
        .stderr(true)
        .follow(false)
        .tail(&tail.to_string())
        .build();
    let mut stream = docker.logs(&container, Some(options));
    let mut bytes = Vec::new();
    let mut truncated = false;
    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("docker logs failed: {e}"))?;
        let message = match chunk {
            LogOutput::StdErr { message }
            | LogOutput::StdOut { message }
            | LogOutput::StdIn { message }
            | LogOutput::Console { message } => message,
        };
        let remaining = max_bytes.saturating_sub(bytes.len());
        if message.len() > remaining {
            bytes.extend_from_slice(&message[..remaining]);
            truncated = true;
            break;
        }
        bytes.extend_from_slice(&message);
        if bytes.len() >= max_bytes {
            truncated = true;
            break;
        }
    }
    let text = String::from_utf8_lossy(&bytes).into_owned();
    Ok(serde_json::json!({
        "kind": "docker_runtime_lab_logs",
        "container_ref": container,
        "tail": tail,
        "max_bytes": max_bytes,
        "bytes_returned": bytes.len(),
        "truncated": truncated,
        "redaction_applied": true,
        "logs": redact_log_text(&text),
        "docker_performed": true
    }))
}

async fn stop_container_async(
    container: String,
    timeout_secs: i32,
    force: bool,
) -> Result<Value, String> {
    let docker = docker().await?;
    let stop_options = StopContainerOptionsBuilder::default()
        .t(timeout_secs)
        .build();
    let stop_result = docker.stop_container(&container, Some(stop_options)).await;
    let mut stop_error = None;
    if let Err(error) = stop_result {
        let msg = error.to_string();
        if !msg.contains("304") && !msg.to_lowercase().contains("not modified") {
            stop_error = Some(msg);
        }
    }
    let remove_options = RemoveContainerOptionsBuilder::default()
        .force(force)
        .v(false)
        .build();
    docker
        .remove_container(&container, Some(remove_options))
        .await
        .map_err(|e| format!("docker remove failed: {e}"))?;

    Ok(serde_json::json!({
        "kind": "docker_runtime_lab_container_stopped",
        "container_ref": container,
        "stopped": stop_error.is_none(),
        "removed": true,
        "force": force,
        "timeout_secs": timeout_secs,
        "stop_error": stop_error,
        "docker_performed": true
    }))
}

#[derive(Debug)]
struct Diagnostic {
    severity: &'static str,
    code: &'static str,
    message: String,
}

fn validate_input(input: &Value) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let Some(image) = string_field(input, "image") else {
        diagnostics.push(error("image_required", "image is required"));
        return diagnostics;
    };
    if !valid_image(&image) {
        diagnostics.push(error(
            "image_invalid",
            "image must be a safe docker image reference without whitespace or shell metacharacters",
        ));
    }

    for (field, label) in [
        ("container_port", "container_port"),
        ("host_port", "host_port"),
    ] {
        match u16_field(input, field) {
            Some(port) if port > 0 => {}
            _ => diagnostics.push(error(
                "port_invalid",
                format!("{label} must be an integer in 1..=65535"),
            )),
        }
    }

    for field in ["route_id", "port_lease_id"] {
        match string_field(input, field) {
            Some(value) if valid_label_value(&value) => {}
            _ => diagnostics.push(error(
                "label_invalid",
                format!("{field} is required and must be label-safe"),
            )),
        }
    }

    if input
        .get("privileged")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        diagnostics.push(error(
            "privileged_blocked",
            "privileged containers are blocked",
        ));
    }
    if input
        .get("network_mode")
        .and_then(Value::as_str)
        .is_some_and(|mode| mode != "bridge")
    {
        diagnostics.push(error(
            "network_mode_blocked",
            "only bridge network_mode is allowed; host network is blocked",
        ));
    }
    for field in ["binds", "mounts", "volumes"] {
        if input.get(field).is_some_and(|v| !v.is_null()) {
            diagnostics.push(error(
                "mounts_blocked",
                format!("{field} are blocked; host bind mounts are not allowed"),
            ));
        }
    }
    for field in ["env", "environment", "secrets"] {
        if input.get(field).is_some_and(|v| !empty_collection(v)) {
            diagnostics.push(error(
                "env_or_secret_blocked",
                format!("{field} is blocked; arbitrary env/secrets are not accepted"),
            ));
        }
    }
    if input
        .get("publish_all_ports")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        diagnostics.push(error(
            "publish_all_ports_blocked",
            "publish_all_ports is blocked; host_port must come from host lease metadata",
        ));
    }
    diagnostics
}

fn error(code: &'static str, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        severity: "error",
        code,
        message: message.into(),
    }
}

fn diagnostics_to_json(diagnostics: &mut [Diagnostic]) -> Vec<Value> {
    diagnostics
        .iter()
        .map(|d| serde_json::json!({"severity": d.severity, "code": d.code, "message": d.message}))
        .collect()
}

fn string_field(input: &Value, field: &str) -> Option<String> {
    input
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

fn u16_field(input: &Value, field: &str) -> Option<u16> {
    input
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|n| u16::try_from(n).ok())
}

fn value_or_null(value: Option<String>) -> Value {
    value.map(Value::String).unwrap_or(Value::Null)
}

fn empty_collection(value: &Value) -> bool {
    value.is_null()
        || value.as_array().is_some_and(Vec::is_empty)
        || value.as_object().is_some_and(serde_json::Map::is_empty)
}

fn valid_image(image: &str) -> bool {
    image.len() <= 255
        && image.bytes().all(|b| {
            b.is_ascii_alphanumeric() || matches!(b, b'.' | b'/' | b':' | b'_' | b'-' | b'@')
        })
        && !image.starts_with('-')
        && !image.contains("..")
}

fn valid_label_value(value: &str) -> bool {
    value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b'/' | b':'))
}

fn sanitize_name(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '-'
            }
        })
        .collect();
    sanitized.trim_matches('-').chars().take(48).collect()
}

fn container_ref(input: &Value) -> Option<String> {
    string_field(input, "container_id")
        .or_else(|| string_field(input, "container_name"))
        .or_else(|| string_field(input, "container"))
}

fn provenance(request: &InprocInvocation) -> Value {
    serde_json::json!({
        "package_id": request.provider_package_id,
        "capability_id": request.capability_id
    })
}

fn with_provenance(mut value: Value, request: &InprocInvocation) -> Value {
    if let Some(obj) = value.as_object_mut() {
        obj.insert("provenance".to_string(), provenance(request));
    }
    value
}

fn rejected_output(request: &InprocInvocation) -> Value {
    serde_json::json!({
        "kind": "docker_runtime_lab_rejected",
        "reason": "input contains raw-secret-like content; pass only host-owned metadata and secret_ref references",
        "docker_performed": false,
        "provenance": provenance(request)
    })
}

fn missing_container_ref_output(request: &InprocInvocation) -> Value {
    serde_json::json!({
        "kind": "docker_runtime_lab_rejected",
        "reason": "container_id or container_name is required",
        "docker_performed": false,
        "provenance": provenance(request)
    })
}

fn docker_error_output(operation: &str, error: String) -> Value {
    serde_json::json!({
        "kind": "docker_runtime_lab_error",
        "operation": operation,
        "docker_performed": false,
        "error": {
            "code": "docker_unavailable_or_failed",
            "message": error
        }
    })
}

fn redact_log_text(input: &str) -> String {
    input
        .split('\n')
        .map(redact_log_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn redact_log_line(line: &str) -> String {
    let mut out = Vec::new();
    for token in line.split_whitespace() {
        let lower = token.to_lowercase();
        if lower.starts_with("token=")
            || lower.starts_with("secret=")
            || lower.starts_with("password=")
            || lower.starts_with("api_key=")
            || token.starts_with("sk-")
            || token.starts_with("Bearer")
            || safety::looks_like_raw_secret_value(token)
        {
            out.push("[REDACTED]".to_string());
        } else {
            out.push(token.to_string());
        }
    }
    out.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(capability: &str, input: Value) -> InprocInvocation {
        InprocInvocation {
            capability_id: format!("{PACKAGE_ID}/{capability}"),
            provider_package_id: PACKAGE_ID.to_string(),
            session_id: None,
            input,
        }
    }

    #[test]
    fn docker_runtime_lab_validate_accepts_minimal_safe_spec() {
        let output = validate_spec(&request(
            "validate_spec",
            serde_json::json!({
                "image": "nginx:1.25-alpine",
                "container_port": 80,
                "host_port": 18080,
                "route_id": "route-test",
                "port_lease_id": "lease-test"
            }),
        ))
        .unwrap();
        assert_eq!(output["valid"], true);
        assert_eq!(output["docker_performed"], false);
    }

    #[test]
    fn docker_runtime_lab_validate_blocks_privileged_host_network_mounts_env() {
        let output = validate_spec(&request(
            "validate_spec",
            serde_json::json!({
                "image": "nginx:latest",
                "container_port": 80,
                "host_port": 18080,
                "route_id": "route-test",
                "port_lease_id": "lease-test",
                "privileged": true,
                "network_mode": "host",
                "binds": ["/tmp:/tmp"],
                "env": {"DEBUG": "1"}
            }),
        ))
        .unwrap();
        assert_eq!(output["valid"], false);
        let diagnostics = output["diagnostics"].as_array().unwrap();
        assert!(diagnostics
            .iter()
            .any(|d| d["code"] == "privileged_blocked"));
        assert!(diagnostics
            .iter()
            .any(|d| d["code"] == "network_mode_blocked"));
        assert!(diagnostics.iter().any(|d| d["code"] == "mounts_blocked"));
        assert!(diagnostics
            .iter()
            .any(|d| d["code"] == "env_or_secret_blocked"));
    }

    #[test]
    fn docker_runtime_lab_plan_is_deterministic_and_no_docker() {
        let output = plan_container(&request(
            "plan_container",
            serde_json::json!({
                "image": "ghcr.io/example/app:sha-abc",
                "container_port": 3000,
                "host_port": 13000,
                "route_id": "route-1",
                "port_lease_id": "lease-1"
            }),
        ))
        .unwrap();
        assert_eq!(output["kind"], "docker_runtime_lab_container_plan");
        assert_eq!(output["bind_host"], BIND_HOST);
        assert_eq!(output["requires_host_port_lease"], true);
        assert_eq!(output["requires_proxy_route"], true);
        assert_eq!(output["docker_performed"], false);
    }

    #[test]
    fn docker_runtime_lab_start_fail_closed_without_approval() {
        let output = start_container(&request(
            "start_container",
            serde_json::json!({
                "image": "nginx:latest",
                "container_port": 80,
                "host_port": 18080,
                "route_id": "route-test",
                "port_lease_id": "lease-test"
            }),
        ))
        .unwrap();
        assert_eq!(output["kind"], "docker_runtime_lab_rejected");
        assert_eq!(output["docker_performed"], false);
    }

    #[test]
    fn docker_runtime_lab_redacts_logs() {
        let redacted = redact_log_text("token=abc password=hunter2 sk-test Bearer secret safe");
        assert!(redacted.contains("[REDACTED]"));
        assert!(!redacted.contains("hunter2"));
        assert!(redacted.contains("safe"));
    }

    #[tokio::test]
    #[ignore]
    async fn docker_runtime_lab_real_docker_smoke_env_gated() {
        if std::env::var("YGG_DOCKER_RUNTIME_LAB_SMOKE")
            .ok()
            .as_deref()
            != Some("1")
        {
            return;
        }
        let _ = docker().await.expect("docker daemon available");
    }
}
