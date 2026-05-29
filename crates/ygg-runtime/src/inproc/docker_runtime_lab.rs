//! Handler for `official/docker-runtime-lab` capabilities.
//!
//! Docker Runtime Lab is an ordinary package. It performs Docker container
//! lifecycle actions only after host-provided approval and metadata are present.
//! The host/kernel owns port leases and proxy routes; this package never invokes
//! HostAdmin-only kernel port/proxy capabilities.

use std::collections::HashMap;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use bollard::body_full;
use bollard::container::LogOutput;
use bollard::models::{
    ContainerCreateBody, ContainerSummary, ContainerSummaryStateEnum, HostConfig, PortBinding,
    PortMap,
};
use bollard::query_parameters::{
    BuildImageOptionsBuilder, CreateContainerOptionsBuilder, CreateImageOptionsBuilder,
    InspectContainerOptionsBuilder, ListContainersOptionsBuilder, LogsOptionsBuilder,
    RemoveContainerOptionsBuilder, StopContainerOptionsBuilder,
};
use bollard::Docker;
use bytes::Bytes;
use futures::StreamExt;
use serde_json::Value;

use super::safety;
use super::InprocInvocation;

const PACKAGE_ID: &str = "official/docker-runtime-lab";
const BIND_HOST: &str = "127.0.0.1";
const DEFAULT_TAIL: u32 = 200;
const DEFAULT_MAX_BYTES: usize = 65_536;
const MAX_TAIL: u32 = 5_000;
const DEFAULT_MAX_CONTEXT_BYTES: u64 = 256 * 1024 * 1024;
const DEFAULT_MAX_CONTEXT_FILES: u64 = 25_000;
const MAX_BUILD_LOG_BYTES: usize = 64 * 1024;
const DEFAULT_BUILD_TIMEOUT_SECS: u64 = 15 * 60;
const MAX_BUILD_TIMEOUT_SECS: u64 = 60 * 60;
const DEFAULT_BUILD_MEMORY_BYTES: i32 = 1024 * 1024 * 1024;
const DEFAULT_BUILD_CPU_QUOTA: i32 = 100_000;

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
    } else if id.ends_with("/build_image") {
        Some(build_image(request))
    } else if id.ends_with("/start_container") {
        Some(start_container(request))
    } else if id.ends_with("/status") {
        Some(status(request))
    } else if id.ends_with("/logs") {
        Some(logs(request))
    } else if id.ends_with("/stop_container") {
        Some(stop_container(request))
    } else if id.ends_with("/list_managed") {
        Some(list_managed(request))
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
            {"id": "official/docker-runtime-lab/build_image", "docker_performed": true, "requires_approved_true": true, "strategies": ["dockerfile"]},
            {"id": "official/docker-runtime-lab/start_container", "docker_performed": true, "requires_approved_true": true},
            {"id": "official/docker-runtime-lab/status", "docker_performed": true},
            {"id": "official/docker-runtime-lab/logs", "docker_performed": true, "bounded": true, "redacted": true},
            {"id": "official/docker-runtime-lab/stop_container", "docker_performed": true},
            {"id": "official/docker-runtime-lab/list_managed", "docker_performed": true, "label_filter": "managed-by=yggdrasil"}
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

fn build_image(request: &InprocInvocation) -> anyhow::Result<Value> {
    match parse_build_image_request(&request.input) {
        Ok(spec) => {
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async move { build_image_async(spec).await })
            });
            match result {
                Ok(value) => Ok(with_provenance(value, request)),
                Err(error) => Ok(with_provenance(
                    docker_error_output("build_image", error),
                    request,
                )),
            }
        }
        Err(reason) => Ok(serde_json::json!({
            "kind": "docker_runtime_lab_rejected",
            "reason": reason,
            "docker_performed": false,
            "image_built": false,
            "provenance": provenance(request)
        })),
    }
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

fn list_managed(request: &InprocInvocation) -> anyhow::Result<Value> {
    if safety::contains_raw_secret(&request.input) {
        return Ok(rejected_output(request));
    }
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async move { list_managed_async().await })
    });
    match result {
        Ok(value) => Ok(with_provenance(value, request)),
        Err(error) => Ok(with_provenance(
            docker_error_output("list_managed", error),
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

#[derive(Debug, Clone)]
struct BuildImageSpec {
    project_id: String,
    build_id: String,
    context_dir: PathBuf,
    dockerfile: String,
    source_commit: Option<String>,
    build_descriptor_hash: Option<String>,
    build_args: HashMap<String, String>,
    max_context_bytes: u64,
    max_context_files: u64,
    build_timeout_secs: u64,
}

#[derive(Debug, Clone)]
struct ContextTar {
    bytes: Vec<u8>,
    files: u64,
    total_bytes: u64,
}

async fn build_image_async(spec: BuildImageSpec) -> Result<Value, String> {
    let docker = docker().await?;
    let tag = image_tag(&spec.project_id, &spec.build_id);
    let context = tokio::task::spawn_blocking({
        let spec = spec.clone();
        move || create_context_tar(&spec)
    })
    .await
    .map_err(|e| format!("context tar task failed: {e}"))??;

    let mut labels = build_labels(&spec);
    labels.insert("managed-by".to_string(), "yggdrasil".to_string());
    labels.insert("yggdrasil.package_id".to_string(), PACKAGE_ID.to_string());

    let options = BuildImageOptionsBuilder::default()
        .dockerfile(&spec.dockerfile)
        .t(&tag)
        .q(false)
        .rm(true)
        .forcerm(true)
        .memory(DEFAULT_BUILD_MEMORY_BYTES)
        .cpuquota(DEFAULT_BUILD_CPU_QUOTA)
        .networkmode("bridge")
        .labels(&labels)
        .buildargs(&spec.build_args)
        .build();
    let body = body_full(Bytes::from(context.bytes));

    let build = async {
        let mut stream = docker.build_image(options, None, Some(body));
        let mut log_tail = String::new();
        while let Some(item) = stream.next().await {
            let info = item.map_err(|e| format!("docker build failed: {e}"))?;
            if let Some(error) = info.error_detail.and_then(|detail| detail.message) {
                append_log_tail(&mut log_tail, &redact_log_text(&error));
                return Err(format!("docker build failed: {}", redact_log_line(&error)));
            }
            if let Some(stream) = info.stream {
                append_log_tail(&mut log_tail, &redact_log_text(&stream));
            } else if let Some(status) = info.status {
                append_log_tail(&mut log_tail, &redact_log_text(&status));
            }
        }
        Ok(log_tail)
    };

    let log_tail = tokio::time::timeout(Duration::from_secs(spec.build_timeout_secs), build)
        .await
        .map_err(|_| "docker build timed out".to_string())??;

    Ok(serde_json::json!({
        "kind": "docker_runtime_lab_image_built",
        "image": tag,
        "build_id": spec.build_id,
        "strategy": "dockerfile",
        "source_commit": spec.source_commit,
        "build_descriptor_hash": spec.build_descriptor_hash,
        "context": {
            "files": context.files,
            "total_bytes": context.total_bytes,
            "max_files": spec.max_context_files,
            "max_bytes": spec.max_context_bytes,
        },
        "log_tail": log_tail,
        "docker_performed": true,
        "image_built": true,
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

async fn list_managed_async() -> Result<Value, String> {
    let docker = docker().await?;
    let filters = HashMap::from([(
        "label".to_string(),
        vec!["managed-by=yggdrasil".to_string()],
    )]);
    let options = ListContainersOptionsBuilder::default()
        .all(true)
        .filters(&filters)
        .build();
    let containers = docker
        .list_containers(Some(options))
        .await
        .map_err(|e| format!("docker list managed failed: {e}"))?;
    let managed: Vec<Value> = containers
        .iter()
        .filter_map(managed_container_json)
        .collect();
    let count = managed.len();
    Ok(serde_json::json!({
        "kind": "docker_runtime_lab_managed_containers",
        "managed": managed,
        "count": count,
        "docker_performed": true
    }))
}

fn managed_container_json(container: &ContainerSummary) -> Option<Value> {
    let labels = container.labels.as_ref()?;
    let route_id = labels.get("yggdrasil.route_id")?.clone();
    let port_lease_id = labels.get("yggdrasil.port_lease_id")?.clone();
    let running = matches!(container.state, Some(ContainerSummaryStateEnum::RUNNING));
    let host_port = container
        .ports
        .as_ref()
        .and_then(|ports| ports.iter().find_map(|port| port.public_port));
    let container_id = container.id.clone();
    let container_name = container
        .names
        .as_ref()
        .and_then(|names| names.first())
        .map(|name| name.trim_start_matches('/').to_string());
    Some(serde_json::json!({
        "container_id": container_id,
        "container_name": container_name,
        "route_id": route_id,
        "port_lease_id": port_lease_id,
        "running": running,
        "host_port": host_port
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

fn parse_build_image_request(input: &Value) -> Result<BuildImageSpec, String> {
    if has_build_secret_request(input) || safety::contains_raw_secret(input) {
        return Err("build-time secrets are not supported; fail-closed".to_string());
    }
    if !input
        .get("approved")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err("build_image requires approved: true; fail-closed".to_string());
    }
    let strategy = string_field(input, "strategy").unwrap_or_else(|| "dockerfile".to_string());
    if strategy != "dockerfile" {
        return Err(format!(
            "unsupported build strategy '{strategy}'; only dockerfile is supported"
        ));
    }
    let project_id = string_field(input, "project_id").ok_or("project_id is required")?;
    if !valid_label_value(&project_id) {
        return Err("project_id must be label-safe".to_string());
    }
    let project = ygg_core::ProjectId::new(&project_id)
        .map_err(|_| "project_id must be a valid project id".to_string())?;
    let build_id = string_field(input, "build_id").ok_or("build_id is required")?;
    if !valid_build_id(&build_id) {
        return Err("build_id must be label-safe".to_string());
    }
    let context_dir = string_field(input, "context_dir").ok_or("context_dir is required")?;
    let context_dir = PathBuf::from(context_dir);
    if !context_dir.is_absolute() {
        return Err("context_dir must be absolute".to_string());
    }
    if context_dir
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("context_dir must not contain parent components".to_string());
    }
    let expected_workspace = ygg_core::paths::project_workspace_dir(&project)
        .map_err(|_| "failed to resolve project workspace".to_string())?;
    if context_dir != expected_workspace {
        return Err("context_dir must be the project's managed workspace".to_string());
    }
    let dockerfile = string_field(input, "dockerfile").unwrap_or_else(|| "Dockerfile".to_string());
    validate_dockerfile_path(&dockerfile)?;
    let source_commit = optional_labelish(input, "source_commit")?;
    let build_descriptor_hash = optional_labelish(input, "build_descriptor_hash")?;
    let build_args = parse_build_args(input.get("build_args"))?;
    let max_context_bytes = input
        .get("max_context_bytes")
        .and_then(Value::as_u64)
        .map(|n| n.min(DEFAULT_MAX_CONTEXT_BYTES))
        .unwrap_or(DEFAULT_MAX_CONTEXT_BYTES);
    let max_context_files = input
        .get("max_context_files")
        .and_then(Value::as_u64)
        .map(|n| n.min(DEFAULT_MAX_CONTEXT_FILES))
        .unwrap_or(DEFAULT_MAX_CONTEXT_FILES);
    let build_timeout_secs = input
        .get("build_timeout_secs")
        .and_then(Value::as_u64)
        .map(|n| n.clamp(1, MAX_BUILD_TIMEOUT_SECS))
        .unwrap_or(DEFAULT_BUILD_TIMEOUT_SECS);

    Ok(BuildImageSpec {
        project_id,
        build_id,
        context_dir,
        dockerfile,
        source_commit,
        build_descriptor_hash,
        build_args,
        max_context_bytes,
        max_context_files,
        build_timeout_secs,
    })
}

fn has_build_secret_request(input: &Value) -> bool {
    const BLOCKED: &[&str] = &[
        "secret",
        "secrets",
        "secret_refs",
        "build_secrets",
        "build_secret",
    ];
    match input {
        Value::Object(map) => map.iter().any(|(key, value)| {
            BLOCKED
                .iter()
                .any(|blocked| key.eq_ignore_ascii_case(blocked))
                || has_build_secret_request(value)
        }),
        Value::Array(items) => items.iter().any(has_build_secret_request),
        Value::String(value) => safety::is_secret_ref_value(value),
        _ => false,
    }
}

fn validate_dockerfile_path(path: &str) -> Result<(), String> {
    if path.trim().is_empty() || path.len() > 255 {
        return Err("dockerfile must be non-empty and at most 255 bytes".to_string());
    }
    let path = Path::new(path);
    if path.is_absolute() {
        return Err("dockerfile must be relative".to_string());
    }
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            _ => return Err("dockerfile must not contain parent or special components".to_string()),
        }
    }
    Ok(())
}

fn parse_build_args(value: Option<&Value>) -> Result<HashMap<String, String>, String> {
    let Some(value) = value else {
        return Ok(HashMap::new());
    };
    let object = value
        .as_object()
        .ok_or("build_args must be an object of string values")?;
    if object.len() > 64 {
        return Err("build_args may contain at most 64 entries".to_string());
    }
    let mut args = HashMap::new();
    for (key, value) in object {
        if !valid_build_arg_key(key) {
            return Err(format!("build arg key '{key}' is invalid"));
        }
        let Some(value) = value.as_str() else {
            return Err("build arg values must be strings".to_string());
        };
        if value.len() > 4096
            || safety::is_secret_ref_value(value)
            || safety::looks_like_raw_secret_value(value)
        {
            return Err(format!(
                "build arg '{key}' contains unsupported secret-like value"
            ));
        }
        args.insert(key.clone(), value.to_string());
    }
    Ok(args)
}

fn valid_build_arg_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 128
        && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        && !key.to_lowercase().contains("secret")
        && !key.to_lowercase().contains("token")
        && !key.to_lowercase().contains("key")
        && !key.to_lowercase().contains("password")
        && !key.to_lowercase().contains("auth")
}

fn optional_labelish(input: &Value, field: &str) -> Result<Option<String>, String> {
    let Some(value) = string_field(input, field) else {
        return Ok(None);
    };
    if value.len() <= 128 && valid_label_value(&value) {
        Ok(Some(value))
    } else {
        Err(format!("{field} must be label-safe"))
    }
}

fn valid_build_id(value: &str) -> bool {
    value.len() <= 128
        && value.len() >= 3
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        && !value.starts_with('-')
        && !value.contains("..")
}

fn image_tag(project_id: &str, build_id: &str) -> String {
    format!(
        "yggdrasil/{}:{}",
        sanitize_image_component(project_id, 80),
        sanitize_image_component(build_id, 120)
    )
}

fn sanitize_image_component(value: &str, max_len: usize) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for c in value.chars() {
        let mapped = if c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-' {
            c.to_ascii_lowercase()
        } else {
            '-'
        };
        if mapped == '-' && prev_dash {
            continue;
        }
        prev_dash = mapped == '-';
        out.push(mapped);
        if out.len() >= max_len {
            break;
        }
    }
    let trimmed = out.trim_matches(&['-', '.'][..]).to_string();
    if trimmed.is_empty() {
        "build".to_string()
    } else {
        trimmed
    }
}

fn build_labels(spec: &BuildImageSpec) -> HashMap<String, String> {
    let mut labels = HashMap::from([
        ("yggdrasil.project_id".to_string(), spec.project_id.clone()),
        ("yggdrasil.build_id".to_string(), spec.build_id.clone()),
        ("yggdrasil.strategy".to_string(), "dockerfile".to_string()),
        (
            "yggdrasil.dockerfile_path".to_string(),
            spec.dockerfile.clone(),
        ),
    ]);
    if let Some(value) = &spec.source_commit {
        labels.insert("yggdrasil.source_commit".to_string(), value.clone());
    }
    if let Some(value) = &spec.build_descriptor_hash {
        labels.insert("yggdrasil.build_descriptor_hash".to_string(), value.clone());
    }
    labels
}

fn create_context_tar(spec: &BuildImageSpec) -> Result<ContextTar, String> {
    let root = std::fs::canonicalize(&spec.context_dir)
        .map_err(|e| format!("failed to canonicalize context_dir: {e}"))?;
    if !root.is_dir() {
        return Err("context_dir must be a directory".to_string());
    }
    let dockerfile = root.join(&spec.dockerfile);
    let dockerfile = std::fs::canonicalize(&dockerfile)
        .map_err(|e| format!("dockerfile not found or inaccessible: {e}"))?;
    if !dockerfile.starts_with(&root) || !dockerfile.is_file() {
        return Err("dockerfile must resolve inside context_dir".to_string());
    }

    let ignore =
        DockerIgnore::load(&root).map_err(|e| format!("failed to read .dockerignore: {e}"))?;
    let mut tar = tar::Builder::new(Vec::new());
    let mut stats = ContextStats::default();
    add_context_dir(&root, &root, &ignore, spec, &mut stats, &mut tar)?;
    let bytes = tar
        .into_inner()
        .map_err(|e| format!("failed to finish context tar: {e}"))?;
    Ok(ContextTar {
        bytes,
        files: stats.files,
        total_bytes: stats.total_bytes,
    })
}

#[derive(Debug, Default)]
struct ContextStats {
    files: u64,
    total_bytes: u64,
}

#[derive(Debug, Default)]
struct DockerIgnore {
    patterns: Vec<String>,
}

impl DockerIgnore {
    fn load(root: &Path) -> std::io::Result<Self> {
        let path = root.join(".dockerignore");
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)?;
        let patterns = raw
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with('!'))
            .map(|line| {
                line.trim_start_matches('/')
                    .trim_end_matches('/')
                    .to_string()
            })
            .filter(|line| !line.is_empty() && !line.contains(".."))
            .collect();
        Ok(Self { patterns })
    }

    fn is_ignored(&self, relative: &str, file_name: &str) -> bool {
        if matches!(file_name, ".git" | "node_modules" | "target" | ".yggdrasil") {
            return true;
        }
        self.patterns.iter().any(|pattern| {
            relative == pattern
                || relative.starts_with(&format!("{pattern}/"))
                || file_name == pattern
        })
    }
}

fn add_context_dir(
    root: &Path,
    dir: &Path,
    ignore: &DockerIgnore,
    spec: &BuildImageSpec,
    stats: &mut ContextStats,
    tar: &mut tar::Builder<Vec<u8>>,
) -> Result<(), String> {
    let mut entries = std::fs::read_dir(dir)
        .map_err(|e| format!("failed to read context dir {}: {e}", dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("failed to read context entry: {e}"))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let rel = relative_tar_path(root, &path)?;
        if ignore.is_ignored(&rel, &file_name) {
            continue;
        }
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|e| format!("failed to stat {}: {e}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "symlinks are not supported in build context: {rel}"
            ));
        }
        if metadata.is_dir() {
            let canonical = std::fs::canonicalize(&path)
                .map_err(|e| format!("failed to canonicalize context dir {rel}: {e}"))?;
            if !canonical.starts_with(root) {
                return Err(format!("context directory escaped root: {rel}"));
            }
            add_context_dir(root, &path, ignore, spec, stats, tar)?;
        } else if metadata.is_file() {
            stats.files += 1;
            stats.total_bytes = stats.total_bytes.saturating_add(metadata.len());
            if stats.files > spec.max_context_files {
                return Err("build context file count limit exceeded".to_string());
            }
            if stats.total_bytes > spec.max_context_bytes {
                return Err("build context byte limit exceeded".to_string());
            }
            let mut file = std::fs::File::open(&path)
                .map_err(|e| format!("failed to open context file {rel}: {e}"))?;
            let mut data = Vec::with_capacity(metadata.len().min(1024 * 1024) as usize);
            file.read_to_end(&mut data)
                .map_err(|e| format!("failed to read context file {rel}: {e}"))?;
            tar_append_file(tar, &rel, &data, metadata.len())?;
        } else {
            return Err(format!(
                "special files are not supported in build context: {rel}"
            ));
        }
    }
    Ok(())
}

fn relative_tar_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| "context path escaped root".to_string())?;
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("context path must be relative without parent components".to_string());
    }
    Ok(relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

fn tar_append_file(
    tar: &mut tar::Builder<Vec<u8>>,
    relative: &str,
    data: &[u8],
    size: u64,
) -> Result<(), String> {
    let mut header = tar::Header::new_gnu();
    header.set_size(size);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append_data(&mut header, relative, data)
        .map_err(|e| format!("failed to append {relative} to context tar: {e}"))
}

fn append_log_tail(tail: &mut String, chunk: &str) {
    tail.push_str(chunk);
    if tail.len() > MAX_BUILD_LOG_BYTES {
        let drop_bytes = tail.len() - MAX_BUILD_LOG_BYTES;
        let drop_at = tail
            .char_indices()
            .find_map(|(idx, _)| (idx >= drop_bytes).then_some(idx))
            .unwrap_or(drop_bytes);
        tail.drain(..drop_at);
    }
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
    fn docker_runtime_lab_list_managed_shape_from_summary() {
        let container = ContainerSummary {
            id: Some("container-000001".to_string()),
            names: Some(vec!["/ygg-container-000001".to_string()]),
            labels: Some(HashMap::from([
                ("managed-by".to_string(), "yggdrasil".to_string()),
                (
                    "yggdrasil.route_id".to_string(),
                    "proxy-route-000001".to_string(),
                ),
                (
                    "yggdrasil.port_lease_id".to_string(),
                    "port-lease-000001".to_string(),
                ),
            ])),
            state: Some(ContainerSummaryStateEnum::RUNNING),
            ports: Some(vec![bollard::models::PortSummary {
                ip: Some(BIND_HOST.to_string()),
                private_port: 3000,
                public_port: Some(39000),
                typ: None,
            }]),
            ..ContainerSummary::default()
        };
        let output = managed_container_json(&container).expect("managed container report");
        assert_eq!(output["container_id"], "container-000001");
        assert_eq!(output["container_name"], "ygg-container-000001");
        assert_eq!(output["route_id"], "proxy-route-000001");
        assert_eq!(output["port_lease_id"], "port-lease-000001");
        assert_eq!(output["running"], true);
        assert_eq!(output["host_port"], 39000);
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
    fn docker_runtime_lab_build_image_rejects_unsupported_strategy() {
        let output = build_image(&request(
            "build_image",
            serde_json::json!({
                "approved": true,
                "strategy": "nixpacks",
                "project_id": "project-1",
                "build_id": "build-1",
                "context_dir": "/tmp/project"
            }),
        ))
        .unwrap();
        assert_eq!(output["kind"], "docker_runtime_lab_rejected");
        assert_eq!(output["docker_performed"], false);
        assert!(output["reason"].as_str().unwrap().contains("unsupported"));
    }

    #[test]
    fn docker_runtime_lab_build_image_rejects_build_secrets() {
        for input in [
            serde_json::json!({
                "approved": true,
                "project_id": "project-1",
                "build_id": "build-1",
                "context_dir": "/tmp/project",
                "secrets": ["secret_ref:env:TOKEN"]
            }),
            serde_json::json!({
                "approved": true,
                "project_id": "project-1",
                "build_id": "build-1",
                "context_dir": "/tmp/project",
                "build_args": {"TOKEN": "secret_ref:env:TOKEN"}
            }),
        ] {
            let output = build_image(&request("build_image", input)).unwrap();
            assert_eq!(output["kind"], "docker_runtime_lab_rejected");
            assert_eq!(output["docker_performed"], false);
        }
    }

    #[test]
    fn docker_runtime_lab_image_tag_sanitizes_project_and_build() {
        assert_eq!(
            image_tag("My Project/Alpha", "Build_001"),
            "yggdrasil/my-project-alpha:build_001"
        );
        assert_eq!(image_tag("***", "---"), "yggdrasil/build:build");
    }

    #[test]
    fn docker_runtime_lab_dockerfile_path_policy() {
        assert!(validate_dockerfile_path("Dockerfile").is_ok());
        assert!(validate_dockerfile_path("docker/Dockerfile").is_ok());
        assert!(validate_dockerfile_path("/tmp/Dockerfile").is_err());
        assert!(validate_dockerfile_path("../Dockerfile").is_err());
        assert!(validate_dockerfile_path("docker/../Dockerfile").is_err());
    }

    #[test]
    fn docker_runtime_lab_context_tar_rejects_symlink_and_oversize() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Dockerfile"), "FROM scratch\n").unwrap();
        std::fs::write(tmp.path().join("app.txt"), "hello").unwrap();
        let spec = BuildImageSpec {
            project_id: "project-1".to_string(),
            build_id: "build-1".to_string(),
            context_dir: tmp.path().to_path_buf(),
            dockerfile: "Dockerfile".to_string(),
            source_commit: None,
            build_descriptor_hash: None,
            build_args: HashMap::new(),
            max_context_bytes: 1024,
            max_context_files: 10,
            build_timeout_secs: 1,
        };
        let tar = create_context_tar(&spec).expect("context tar builds");
        assert_eq!(tar.files, 2);
        assert!(tar.total_bytes > 0);

        let mut tiny = spec.clone();
        tiny.max_context_bytes = 1;
        assert!(create_context_tar(&tiny).is_err());

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink("app.txt", tmp.path().join("link.txt")).unwrap();
            assert!(create_context_tar(&spec).is_err());
        }
    }

    #[test]
    fn docker_runtime_lab_context_tar_skips_default_heavy_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Dockerfile"), "FROM scratch\n").unwrap();
        std::fs::create_dir(tmp.path().join("node_modules")).unwrap();
        std::fs::write(tmp.path().join("node_modules/huge.js"), "ignored").unwrap();
        let spec = BuildImageSpec {
            project_id: "project-1".to_string(),
            build_id: "build-1".to_string(),
            context_dir: tmp.path().to_path_buf(),
            dockerfile: "Dockerfile".to_string(),
            source_commit: None,
            build_descriptor_hash: None,
            build_args: HashMap::new(),
            max_context_bytes: 1024,
            max_context_files: 10,
            build_timeout_secs: 1,
        };
        let tar = create_context_tar(&spec).expect("context tar builds");
        assert_eq!(tar.files, 1);
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
