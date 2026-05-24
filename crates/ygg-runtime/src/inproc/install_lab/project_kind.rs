use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{json, Value};
use uuid::Uuid;
use ygg_core::{paths, ProjectDescriptor, ProjectType};

use crate::inproc::project_registry_from_inproc;

use super::executor::invoke_package_capability;
use super::layout::atomic_write;
use super::source::{parse_root_descriptor, value_str};
use super::types::{DetectedProjectKind, DetectKindInput, SourceDescriptor};

pub(super) async fn detect_kind(input: Value) -> Result<Value> {
    let input: DetectKindInput = serde_json::from_value(input)?;
    let source = input
        .path
        .or(input.url)
        .ok_or_else(|| anyhow::anyhow!("detect_kind requires path or url"))?;
    let root = parse_root_descriptor(&source, &input.root_ref)?;
    let detected = match root.source {
        SourceDescriptor::Local { path } => detect_project_kind(&path)?,
        SourceDescriptor::Git { url, ref_name } => {
            let resolved = invoke_package_capability(
                "official/git-tools-lab",
                "official/git-tools-lab/resolve_ref",
                json!({ "remote_url": url, "ref": ref_name }),
            )
            .await?;
            let commit_sha = value_str(&resolved, "commit_sha")?.to_string();
            let tmp =
                std::env::temp_dir().join(format!("yggdrasil-detect-kind-{}", Uuid::new_v4()));
            let result = async {
                invoke_package_capability(
                    "official/git-tools-lab",
                    "official/git-tools-lab/fetch_tree",
                    json!({ "remote_url": url, "commit_sha": commit_sha, "dest_dir": tmp.to_string_lossy() }),
                )
                .await?;
                detect_project_kind(&tmp)
            }
            .await;
            fs::remove_dir_all(&tmp).ok();
            result?
        }
        SourceDescriptor::Internal => anyhow::bail!("internal packages cannot be detected"),
    };
    Ok(serde_json::to_value(detected)?)
}

pub(super) fn detect_project_kind(staging_dir: &Path) -> Result<DetectedProjectKind> {
    let project_yaml = staging_dir.join("project.yaml");
    if project_yaml.exists() {
        let descriptor = read_project_descriptor(&project_yaml)?;
        return Ok(match descriptor.project.project_type {
            ProjectType::YggdrasilNative => DetectedProjectKind::Native { descriptor },
            ProjectType::ExternalWrapped | ProjectType::ExternalWorkspace => {
                DetectedProjectKind::DeclaredExternal { descriptor }
            }
        });
    }

    let manifest_yaml = staging_dir.join("manifest.yaml");
    Ok(DetectedProjectKind::External {
        has_manifest_yaml: manifest_yaml.exists(),
    })
}

pub(super) fn read_project_descriptor(path: &Path) -> Result<ProjectDescriptor> {
    let yaml = fs::read_to_string(path)
        .with_context(|| format!("failed to read project descriptor {}", path.display()))?;
    let descriptor: ProjectDescriptor =
        serde_yaml::from_str(&yaml).map_err(|e| anyhow::anyhow!("invalid project.yaml: {e}"))?;
    descriptor.validate()?;
    Ok(descriptor)
}

pub(super) fn write_and_register_project(
    descriptor: ProjectDescriptor,
    data_dir_override: Option<&str>,
) -> Result<Value> {
    descriptor.validate()?;
    let project_id = descriptor.project.id.clone();
    let project_dir = ensure_project_initialized_for(&project_id, data_dir_override)?;
    let descriptor_path = project_dir.join("project.yaml");
    let yaml = serde_yaml::to_string(&descriptor)?;
    atomic_write(&descriptor_path, yaml.as_bytes())?;
    project_registry_from_inproc()?.register(descriptor)?;
    Ok(json!({
        "project_id": project_id.as_str(),
        "project_dir": project_dir.to_string_lossy(),
    }))
}

pub(super) fn ensure_project_initialized_for(
    project_id: &ygg_core::ProjectId,
    data_dir_override: Option<&str>,
) -> Result<PathBuf> {
    if let Some(dir) = data_dir_override {
        let project_dir = PathBuf::from(dir).join("projects").join(project_id.as_str());
        fs::create_dir_all(project_dir.join("sessions"))?;
        fs::create_dir_all(project_dir.join("state"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&project_dir)?.permissions();
            perms.set_mode(0o700);
            fs::set_permissions(&project_dir, perms)?;
        }
        Ok(project_dir)
    } else {
        paths::ensure_project_initialized(project_id)?;
        paths::project_dir(project_id)
    }
}
