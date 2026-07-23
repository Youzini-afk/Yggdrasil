use std::collections::HashMap;
use std::time::Duration;

use anyhow::Context;
use bollard::models::{
    ContainerCreateBody, ContainerSummary, ContainerSummaryStateEnum, HostConfig, PortBinding,
    PortMap,
};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, CreateImageOptionsBuilder, ListContainersOptionsBuilder,
    RemoveContainerOptionsBuilder, StopContainerOptionsBuilder,
};
use bollard::Docker;
use futures::StreamExt;
use serde::{Serialize, Serializer};
use sha2::{Digest, Sha256};
use thiserror::Error;

const BIND_HOST: &str = "127.0.0.1";
const DRIVER_ID: &str = "yggdrasil-target-agent-v1";
const DOCKER_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const DOCKER_EFFECT_TIMEOUT: Duration = Duration::from_secs(60);
const DOCKER_PULL_TIMEOUT: Duration = Duration::from_secs(10 * 60);

#[derive(Debug, Error)]
#[error("managed target deployment outcome is unknown after {stage}")]
pub struct ManagedTargetDeploymentOutcomeUnknown {
    stage: &'static str,
}

pub fn is_managed_target_deployment_outcome_unknown(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<ManagedTargetDeploymentOutcomeUnknown>()
            .is_some()
    })
}

fn outcome_unknown(stage: &'static str) -> anyhow::Error {
    ManagedTargetDeploymentOutcomeUnknown { stage }.into()
}

mod docker_container_id {
    use super::*;

    pub fn serialize<S>(container_id: &String, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("docker:{container_id}"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedTargetDeploymentApply {
    pub target_id: String,
    pub project_id: String,
    pub deployment_id: String,
    pub route_id: String,
    pub port_lease_id: String,
    pub port_name: String,
    pub image: String,
    pub container_port: u16,
    pub requested_host_port: Option<u16>,
    pub pull_if_missing: bool,
    pub operation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedTargetDeploymentRef {
    pub target_id: String,
    pub project_id: String,
    pub deployment_id: String,
    pub route_id: String,
    pub port_lease_id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ManagedTargetDeploymentObservation {
    #[serde(skip_serializing)]
    pub target_id: String,
    #[serde(skip_serializing)]
    pub project_id: String,
    #[serde(skip_serializing)]
    pub deployment_id: String,
    #[serde(skip_serializing)]
    pub route_id: String,
    #[serde(skip_serializing)]
    pub port_lease_id: String,
    #[serde(skip_serializing)]
    pub port_name: String,
    #[serde(serialize_with = "docker_container_id::serialize")]
    pub container_id: String,
    pub container_name: String,
    #[serde(skip_serializing)]
    pub image: String,
    pub image_id: Option<String>,
    pub container_port: u16,
    pub host_port: u16,
    pub bind_host: String,
    pub running: bool,
    pub state: String,
    #[serde(skip_serializing)]
    pub owner_operation_id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ManagedTargetDeploymentDrainReceipt {
    pub deployment: Option<ManagedTargetDeploymentObservation>,
    pub stopped: bool,
    pub grace_seconds: u16,
    pub container_retained: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ManagedTargetDeploymentStopReceipt {
    pub stopped: bool,
    pub removed: bool,
    pub force_remove: bool,
    pub grace_seconds: u16,
}

pub async fn validate_managed_target_deployment_runtime() -> anyhow::Result<()> {
    docker().await.map(|_| ())
}

pub async fn apply_managed_target_deployment(
    request: &ManagedTargetDeploymentApply,
) -> anyhow::Result<ManagedTargetDeploymentObservation> {
    validate_apply_request(request)?;
    let docker = docker().await?;
    if let Some(container) = find_target_container(&docker, &request.reference()).await? {
        let labels = validated_ownership_labels(&request.reference(), &container)?;
        anyhow::ensure!(
            labels.get("yggdrasil.deployment_operation_id") == Some(&request.operation_id)
                && labels.get("yggdrasil.port_name") == Some(&request.port_name)
                && labels.get("yggdrasil.image_ref") == Some(&request.image)
                && labels.get("yggdrasil.container_port")
                    == Some(&request.container_port.to_string()),
            "existing managed deployment conflicts with the requested operation"
        );
        if !matches!(container.state, Some(ContainerSummaryStateEnum::RUNNING)) {
            let container_id = container
                .id
                .as_deref()
                .context("managed deployment has no container id")?;
            let started = tokio::time::timeout(
                DOCKER_EFFECT_TIMEOUT,
                docker.start_container(container_id, None),
            )
            .await;
            if !matches!(started, Ok(Ok(()))) {
                if let Ok(observation) =
                    find_required_target_deployment(&docker, &request.reference()).await
                {
                    if observation.running {
                        validate_requested_host_port(request, &observation)?;
                        return Ok(observation);
                    }
                }
                return Err(outcome_unknown("docker container start"));
            }
            let observation = find_required_target_deployment(&docker, &request.reference())
                .await
                .map_err(|_| outcome_unknown("docker container start verification"))?;
            validate_requested_host_port(request, &observation)?;
            return Ok(observation);
        }
        let observation = find_required_target_deployment(&docker, &request.reference()).await?;
        validate_requested_host_port(request, &observation)?;
        return Ok(observation);
    }

    if request.pull_if_missing {
        let options = CreateImageOptionsBuilder::default()
            .from_image(&request.image)
            .build();
        tokio::time::timeout(DOCKER_PULL_TIMEOUT, async {
            let mut stream = docker.create_image(Some(options), None, None);
            while let Some(item) = stream.next().await {
                item.context("docker image pull failed")?;
            }
            Ok::<_, anyhow::Error>(())
        })
        .await
        .context("docker image pull timed out")??;
    }
    let inspected_image =
        tokio::time::timeout(DOCKER_EFFECT_TIMEOUT, docker.inspect_image(&request.image))
            .await
            .context("docker image inspect timed out")??;
    let image_id = inspected_image
        .id
        .context("docker image has no content-addressable id")?;

    let container_port_key = format!("{}/tcp", request.container_port);
    let mut port_bindings: PortMap = HashMap::new();
    port_bindings.insert(
        container_port_key.clone(),
        Some(vec![PortBinding {
            host_ip: Some(BIND_HOST.to_string()),
            host_port: Some(
                request
                    .requested_host_port
                    .map(|port| port.to_string())
                    .unwrap_or_default(),
            ),
        }]),
    );
    let labels = deployment_labels(request);
    let config = ContainerCreateBody {
        image: Some(image_id),
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
    let container_name = deployment_container_name(
        &request.target_id,
        &request.project_id,
        &request.deployment_id,
    );
    let options = CreateContainerOptionsBuilder::default()
        .name(&container_name)
        .build();
    let created = match tokio::time::timeout(
        DOCKER_EFFECT_TIMEOUT,
        docker.create_container(Some(options), config),
    )
    .await
    {
        Ok(Ok(created)) => created,
        Ok(Err(_)) | Err(_) => {
            if let Ok(observation) =
                find_required_target_deployment(&docker, &request.reference()).await
            {
                if observation.running && observation.owner_operation_id == request.operation_id {
                    validate_requested_host_port(request, &observation)?;
                    return Ok(observation);
                }
            }
            return Err(outcome_unknown("docker container create"));
        }
    };
    let started = tokio::time::timeout(
        DOCKER_EFFECT_TIMEOUT,
        docker.start_container(&created.id, None),
    )
    .await;
    if !matches!(started, Ok(Ok(()))) {
        if let Ok(observation) =
            find_required_target_deployment(&docker, &request.reference()).await
        {
            if observation.running {
                validate_requested_host_port(request, &observation)?;
                return Ok(observation);
            }
        }
        return Err(outcome_unknown("docker container start"));
    }
    let observation = find_required_target_deployment(&docker, &request.reference())
        .await
        .map_err(|_| outcome_unknown("docker container start verification"))?;
    validate_requested_host_port(request, &observation)?;
    Ok(observation)
}

pub async fn observe_managed_target_deployment(
    reference: &ManagedTargetDeploymentRef,
) -> anyhow::Result<Option<ManagedTargetDeploymentObservation>> {
    validate_reference(reference)?;
    let docker = docker().await?;
    Ok(find_target_deployment(&docker, reference)
        .await?
        .map(|(_, observation)| observation))
}

pub async fn drain_managed_target_deployment(
    reference: &ManagedTargetDeploymentRef,
    grace_seconds: u16,
) -> anyhow::Result<ManagedTargetDeploymentDrainReceipt> {
    validate_reference(reference)?;
    anyhow::ensure!(grace_seconds <= 300, "deployment grace period is too large");
    let docker = docker().await?;
    let Some((_, observation)) = find_target_deployment(&docker, reference).await? else {
        return Ok(ManagedTargetDeploymentDrainReceipt {
            deployment: None,
            stopped: true,
            grace_seconds,
            container_retained: false,
        });
    };
    if observation.running {
        let options = StopContainerOptionsBuilder::default()
            .t(i32::from(grace_seconds))
            .build();
        let stopped = tokio::time::timeout(
            Duration::from_secs(u64::from(grace_seconds).saturating_add(30)),
            docker.stop_container(&observation.container_id, Some(options)),
        )
        .await;
        if !matches!(stopped, Ok(Ok(()))) {
            match find_target_deployment(&docker, reference).await {
                Ok(Some((_, current))) if !current.running => {}
                _ => return Err(outcome_unknown("docker container drain")),
            }
        }
    }
    let after = match find_target_deployment(&docker, reference).await {
        Ok(Some((_, current))) if !current.running => current,
        _ => return Err(outcome_unknown("docker container drain verification")),
    };
    Ok(ManagedTargetDeploymentDrainReceipt {
        deployment: Some(after),
        stopped: true,
        grace_seconds,
        container_retained: true,
    })
}

pub async fn stop_managed_target_deployment(
    reference: &ManagedTargetDeploymentRef,
    grace_seconds: u16,
    force_remove: bool,
) -> anyhow::Result<ManagedTargetDeploymentStopReceipt> {
    validate_reference(reference)?;
    anyhow::ensure!(grace_seconds <= 300, "deployment grace period is too large");
    let docker = docker().await?;
    let Some((_, observation)) = find_target_deployment(&docker, reference).await? else {
        return Ok(ManagedTargetDeploymentStopReceipt {
            stopped: true,
            removed: true,
            force_remove,
            grace_seconds,
        });
    };
    if observation.running {
        let options = StopContainerOptionsBuilder::default()
            .t(i32::from(grace_seconds))
            .build();
        let stop = tokio::time::timeout(
            Duration::from_secs(u64::from(grace_seconds).saturating_add(30)),
            docker.stop_container(&observation.container_id, Some(options)),
        )
        .await;
        match stop {
            Ok(Ok(())) => {}
            Ok(Err(_)) | Err(_) if force_remove => {}
            Ok(Err(_)) | Err(_) => match find_target_deployment(&docker, reference).await {
                Ok(Some((_, current))) if !current.running => {}
                _ => return Err(outcome_unknown("docker container stop")),
            },
        }
    }
    let options = RemoveContainerOptionsBuilder::default()
        .force(force_remove)
        .v(false)
        .build();
    let removed = tokio::time::timeout(
        DOCKER_EFFECT_TIMEOUT,
        docker.remove_container(&observation.container_id, Some(options)),
    )
    .await;
    if !matches!(removed, Ok(Ok(()))) {
        match find_target_container(&docker, reference).await {
            Ok(None) => {}
            _ => return Err(outcome_unknown("docker container removal")),
        }
    }
    Ok(ManagedTargetDeploymentStopReceipt {
        stopped: true,
        removed: true,
        force_remove,
        grace_seconds,
    })
}

pub async fn count_managed_target_deployments(target_id: &str) -> anyhow::Result<u64> {
    validate_label_value("target_id", target_id)?;
    let docker = docker().await?;
    let filters = HashMap::from([(
        "label".to_string(),
        vec![
            format!("yggdrasil.target_driver={DRIVER_ID}"),
            format!("yggdrasil.target_id={target_id}"),
        ],
    )]);
    let options = ListContainersOptionsBuilder::default()
        .all(true)
        .filters(&filters)
        .build();
    let containers =
        tokio::time::timeout(DOCKER_EFFECT_TIMEOUT, docker.list_containers(Some(options)))
            .await
            .context("docker managed deployment list timed out")??;
    Ok(u64::try_from(containers.len()).unwrap_or(u64::MAX))
}

impl ManagedTargetDeploymentApply {
    fn reference(&self) -> ManagedTargetDeploymentRef {
        ManagedTargetDeploymentRef {
            target_id: self.target_id.clone(),
            project_id: self.project_id.clone(),
            deployment_id: self.deployment_id.clone(),
            route_id: self.route_id.clone(),
            port_lease_id: self.port_lease_id.clone(),
        }
    }
}

async fn docker() -> anyhow::Result<Docker> {
    let docker = Docker::connect_with_local_defaults()
        .or_else(|_| Docker::connect_with_defaults())
        .context("docker connection unavailable")?;
    tokio::time::timeout(DOCKER_CONNECT_TIMEOUT, docker.ping())
        .await
        .context("docker ping timed out")??;
    Ok(docker)
}

fn validate_apply_request(request: &ManagedTargetDeploymentApply) -> anyhow::Result<()> {
    validate_reference(&request.reference())?;
    validate_label_value("port_name", &request.port_name)?;
    validate_label_value("operation_id", &request.operation_id)?;
    anyhow::ensure!(
        request.container_port > 0,
        "container port must be non-zero"
    );
    anyhow::ensure!(
        request.requested_host_port.is_none_or(|port| port > 0),
        "requested host port must be non-zero"
    );
    anyhow::ensure!(
        valid_image_reference(&request.image),
        "deployment image reference is invalid"
    );
    Ok(())
}

fn validate_requested_host_port(
    request: &ManagedTargetDeploymentApply,
    observation: &ManagedTargetDeploymentObservation,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        request
            .requested_host_port
            .is_none_or(|port| observation.host_port == port),
        "managed deployment actual port conflicts with the request"
    );
    Ok(())
}

fn valid_image_reference(image: &str) -> bool {
    if image.is_empty()
        || image.len() > 512
        || image.contains("://")
        || crate::scan_effect_value_for_raw_secrets(
            &serde_json::json!({ "image": image }),
            "deployment",
        )
        .has_findings()
        || !image
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:/@+-".contains(&byte))
    {
        return false;
    }
    let Some((name, digest)) = image.rsplit_once('@') else {
        return true;
    };
    !name.is_empty()
        && !name.contains('@')
        && digest.len() == 71
        && digest.starts_with("sha256:")
        && digest[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_reference(reference: &ManagedTargetDeploymentRef) -> anyhow::Result<()> {
    for (name, value) in [
        ("target_id", reference.target_id.as_str()),
        ("project_id", reference.project_id.as_str()),
        ("deployment_id", reference.deployment_id.as_str()),
        ("route_id", reference.route_id.as_str()),
        ("port_lease_id", reference.port_lease_id.as_str()),
    ] {
        validate_label_value(name, value)?;
    }
    Ok(())
}

fn validate_label_value(name: &str, value: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        !value.is_empty()
            && value.len() <= 256
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-._:/".contains(&byte)),
        "deployment {name} is invalid"
    );
    Ok(())
}

fn deployment_labels(request: &ManagedTargetDeploymentApply) -> HashMap<String, String> {
    HashMap::from([
        ("managed-by".to_string(), "yggdrasil".to_string()),
        ("yggdrasil.target_driver".to_string(), DRIVER_ID.to_string()),
        ("yggdrasil.target_id".to_string(), request.target_id.clone()),
        (
            "yggdrasil.project_id".to_string(),
            request.project_id.clone(),
        ),
        (
            "yggdrasil.deployment_id".to_string(),
            request.deployment_id.clone(),
        ),
        ("yggdrasil.route_id".to_string(), request.route_id.clone()),
        (
            "yggdrasil.port_lease_id".to_string(),
            request.port_lease_id.clone(),
        ),
        ("yggdrasil.port_name".to_string(), request.port_name.clone()),
        ("yggdrasil.image_ref".to_string(), request.image.clone()),
        (
            "yggdrasil.container_port".to_string(),
            request.container_port.to_string(),
        ),
        (
            "yggdrasil.deployment_operation_id".to_string(),
            request.operation_id.clone(),
        ),
    ])
}

fn deployment_container_name(target_id: &str, project_id: &str, deployment_id: &str) -> String {
    let digest = Sha256::digest(format!("{target_id}\0{project_id}\0{deployment_id}").as_bytes());
    format!("ygg-target-{}", &format!("{digest:x}")[..24])
}

async fn find_required_target_deployment(
    docker: &Docker,
    reference: &ManagedTargetDeploymentRef,
) -> anyhow::Result<ManagedTargetDeploymentObservation> {
    find_target_deployment(docker, reference)
        .await?
        .map(|(_, observation)| observation)
        .context("managed deployment disappeared after Docker effect")
}

async fn find_target_deployment(
    docker: &Docker,
    reference: &ManagedTargetDeploymentRef,
) -> anyhow::Result<Option<(ContainerSummary, ManagedTargetDeploymentObservation)>> {
    find_target_container(docker, reference)
        .await?
        .map(|container| {
            let observation = observation_from_summary(reference, &container)?;
            Ok((container, observation))
        })
        .transpose()
}

async fn find_target_container(
    docker: &Docker,
    reference: &ManagedTargetDeploymentRef,
) -> anyhow::Result<Option<ContainerSummary>> {
    validate_reference(reference)?;
    let filters = HashMap::from([(
        "label".to_string(),
        vec![
            format!("yggdrasil.target_driver={DRIVER_ID}"),
            format!("yggdrasil.target_id={}", reference.target_id),
            format!("yggdrasil.project_id={}", reference.project_id),
            format!("yggdrasil.deployment_id={}", reference.deployment_id),
        ],
    )]);
    let options = ListContainersOptionsBuilder::default()
        .all(true)
        .filters(&filters)
        .build();
    let containers =
        tokio::time::timeout(DOCKER_EFFECT_TIMEOUT, docker.list_containers(Some(options)))
            .await
            .context("docker deployment lookup timed out")??;
    anyhow::ensure!(
        containers.len() <= 1,
        "multiple containers claim one target deployment identity"
    );
    let container = containers.into_iter().next();
    if let Some(container) = &container {
        validated_ownership_labels(reference, container)?;
    }
    Ok(container)
}

fn observation_from_summary(
    reference: &ManagedTargetDeploymentRef,
    container: &ContainerSummary,
) -> anyhow::Result<ManagedTargetDeploymentObservation> {
    let labels = validated_ownership_labels(reference, container)?;
    let labels = &labels;
    let container_port = labels
        .get("yggdrasil.container_port")
        .context("managed deployment has no container port label")?
        .parse::<u16>()?;
    let port = container
        .ports
        .as_ref()
        .into_iter()
        .flatten()
        .find(|port| port.private_port == container_port)
        .context("managed deployment has no published port")?;
    let host_port = port
        .public_port
        .context("managed deployment has no actual host port")?;
    anyhow::ensure!(host_port > 0, "managed deployment actual host port is zero");
    let bind_host = port.ip.clone().unwrap_or_default();
    anyhow::ensure!(
        bind_host == BIND_HOST,
        "managed deployment port is not loopback-only"
    );
    Ok(ManagedTargetDeploymentObservation {
        target_id: reference.target_id.clone(),
        project_id: reference.project_id.clone(),
        deployment_id: reference.deployment_id.clone(),
        route_id: reference.route_id.clone(),
        port_lease_id: reference.port_lease_id.clone(),
        port_name: labels
            .get("yggdrasil.port_name")
            .context("managed deployment has no port name label")?
            .clone(),
        container_id: container
            .id
            .clone()
            .context("managed deployment has no container id")?,
        container_name: container
            .names
            .as_ref()
            .and_then(|names| names.first())
            .map(|name| name.trim_start_matches('/').to_string())
            .context("managed deployment has no container name")?,
        image: labels
            .get("yggdrasil.image_ref")
            .context("managed deployment has no image reference label")?
            .clone(),
        image_id: container.image_id.clone(),
        container_port,
        host_port,
        bind_host,
        running: matches!(container.state, Some(ContainerSummaryStateEnum::RUNNING)),
        state: container
            .state
            .map(|state| state.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        owner_operation_id: labels
            .get("yggdrasil.deployment_operation_id")
            .context("managed deployment has no operation label")?
            .clone(),
    })
}

fn validated_ownership_labels<'a>(
    reference: &ManagedTargetDeploymentRef,
    container: &'a ContainerSummary,
) -> anyhow::Result<&'a HashMap<String, String>> {
    let labels = container
        .labels
        .as_ref()
        .context("managed deployment has no ownership labels")?;
    for (key, expected) in [
        ("managed-by", "yggdrasil"),
        ("yggdrasil.target_driver", DRIVER_ID),
        ("yggdrasil.target_id", reference.target_id.as_str()),
        ("yggdrasil.project_id", reference.project_id.as_str()),
        ("yggdrasil.deployment_id", reference.deployment_id.as_str()),
        ("yggdrasil.route_id", reference.route_id.as_str()),
        ("yggdrasil.port_lease_id", reference.port_lease_id.as_str()),
    ] {
        anyhow::ensure!(
            labels.get(key).map(String::as_str) == Some(expected),
            "managed deployment ownership label mismatch"
        );
    }
    Ok(labels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_identity_is_deterministic_and_not_caller_controlled() {
        let first = deployment_container_name("target-1", "project-1", "deployment-1");
        assert_eq!(
            first,
            deployment_container_name("target-1", "project-1", "deployment-1")
        );
        assert_ne!(
            first,
            deployment_container_name("target-2", "project-1", "deployment-1")
        );
        assert_ne!(
            first,
            deployment_container_name("target-1", "project-2", "deployment-1")
        );
        assert!(first.starts_with("ygg-target-"));
        assert_eq!(first.len(), "ygg-target-".len() + 24);
    }

    #[test]
    fn apply_validation_rejects_address_and_command_shaped_images() {
        let mut request = ManagedTargetDeploymentApply {
            target_id: "target-1".to_string(),
            project_id: "project-1".to_string(),
            deployment_id: "deployment-1".to_string(),
            route_id: "route-1".to_string(),
            port_lease_id: "lease-1".to_string(),
            port_name: "http".to_string(),
            image: format!("registry.example/app@sha256:{}", "a".repeat(64)),
            container_port: 8080,
            requested_host_port: None,
            pull_if_missing: false,
            operation_id: "operation-1".to_string(),
        };
        validate_apply_request(&request).unwrap();
        request.requested_host_port = Some(0);
        assert!(validate_apply_request(&request).is_err());
        request.requested_host_port = None;
        request.image = "https://registry.example/app".to_string();
        assert!(validate_apply_request(&request).is_err());
        request.image = "app; whoami".to_string();
        assert!(validate_apply_request(&request).is_err());
        request.image = "user@registry.example/app".to_string();
        assert!(validate_apply_request(&request).is_err());
    }

    #[test]
    fn ambiguous_effect_errors_are_distinguishable() {
        let error = outcome_unknown("test effect");
        assert!(is_managed_target_deployment_outcome_unknown(&error));
        assert!(!is_managed_target_deployment_outcome_unknown(
            &anyhow::anyhow!("known failure")
        ));
    }

    #[test]
    fn receipt_encodes_container_identity_as_a_typed_non_secret_reference() {
        let observation = ManagedTargetDeploymentObservation {
            target_id: "target-1".to_string(),
            project_id: "project-1".to_string(),
            deployment_id: "deployment-1".to_string(),
            route_id: "route-1".to_string(),
            port_lease_id: "lease-1".to_string(),
            port_name: "http".to_string(),
            container_id: "a".repeat(64),
            container_name: "ygg-target-test".to_string(),
            image: "registry.example/app:latest".to_string(),
            image_id: Some(format!("sha256:{}", "b".repeat(64))),
            container_port: 8080,
            host_port: 49152,
            bind_host: BIND_HOST.to_string(),
            running: true,
            state: "running".to_string(),
            owner_operation_id: "operation-1".to_string(),
        };
        let value = serde_json::to_value(&observation).unwrap();
        assert_eq!(value["container_id"], format!("docker:{}", "a".repeat(64)));
        assert!(value.get("target_id").is_none());
        assert!(value.get("project_id").is_none());
        assert!(!crate::scan_effect_value_for_raw_secrets(&value, "receipt.output").has_findings());
    }
}
