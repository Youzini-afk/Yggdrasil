use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{json, Value};
use uuid::Uuid;
use ygg_core::{LockEntry, LockRequirement, LockSource, Lockfile, PackageEntry, PackageManifest};

use crate::inproc::invoke_capability_from_inproc;
use crate::CapabilityInvocationRequest;

use super::fs_copy::{
    copy_dir_atomic, installed_manifest_path, safe_relative_path, verify_installed_hashes,
};
use super::layout::{
    atomic_write, ensure_layout, lockfile_path, profile_path, profiles_dir, store_dir,
    store_path_for_hash,
};
use super::planner::verify_consent;
use super::project_kind::{read_project_descriptor, write_and_register_project};
use super::source::{manifest_path_in, parse_manifest_at, value_str};
use super::types::{
    Consent, ExecutePlanInput, InstallPlan, ProfileInput, RegisterProjectInput, UninstallInput,
};
use super::PACKAGE_ID;

pub(super) async fn execute_plan(input: Value) -> Result<Value> {
    if input.as_object().is_some_and(|object| object.is_empty()) {
        anyhow::bail!("unsupported smoke input: missing plan");
    }
    let input: ExecutePlanInput = serde_json::from_value(input)?;
    verify_consent(&input.plan, &input.consent)?;
    ensure_layout(input.data_dir.as_deref())?;
    let store_dir = store_dir(input.data_dir.as_deref())?;
    let profiles_dir = profiles_dir(input.data_dir.as_deref())?;
    let staging_dir = store_dir.join(".staging");
    fs::create_dir_all(&staging_dir)?;
    fs::create_dir_all(&profiles_dir)?;

    let mut installed = Vec::new();
    for pkg in input.plan.packages.iter().rev() {
        let store_path = store_path_for_hash(&pkg.tree_hash, input.data_dir.as_deref())?;
        if !store_path.exists() {
            let staging = staging_dir.join(Uuid::new_v4().to_string());
            match pkg.source.as_str() {
                "local" => copy_dir_atomic(
                    Path::new(pkg.path.as_deref().context("local package missing path")?),
                    &staging,
                    &store_path,
                )?,
                "git" => {
                    let url = pkg.url.as_deref().context("git package missing url")?;
                    let commit = pkg
                        .commit_sha
                        .as_deref()
                        .context("git package missing commit_sha")?;
                    let ref_name = pkg.ref_name.as_deref().unwrap_or(commit);
                    invoke_package_capability(
                        "official/git-tools-lab",
                        "official/git-tools-lab/fetch_tree",
                        json!({ "remote_url": url, "commit_sha": commit, "ref_name": ref_name, "dest_dir": staging.to_string_lossy() }),
                    )
                    .await?;
                    fs::rename(&staging, &store_path).with_context(|| {
                        format!(
                            "failed to atomically move {} to {}",
                            staging.display(),
                            store_path.display()
                        )
                    })?;
                }
                other => anyhow::bail!("unsupported install source: {other}"),
            }
        }
        verify_installed_hashes(pkg, &store_path).await?;
        installed.push(json!({
            "id": pkg.id,
            "store_path": store_path.to_string_lossy(),
            "manifest_path": installed_manifest_path(pkg, &store_path)?.to_string_lossy(),
        }));
    }

    let profile_path = profile_path(&input.profile, input.data_dir.as_deref())?;
    let lockfile_path = lockfile_path(&input.profile, input.data_dir.as_deref())?;
    write_profile(&profile_path, &input.plan, input.data_dir.as_deref())?;
    let lock = build_lockfile(
        &input.profile,
        &input.plan,
        &input.consent,
        input.data_dir.as_deref(),
    )?;
    let lockfile = toml::to_string_pretty(&lock)?;
    atomic_write(&lockfile_path, lockfile.as_bytes())?;

    let project_descriptor = input
        .project_descriptor
        .or_else(|| input.plan.project_descriptor.clone());
    let registered_project = if let Some(descriptor) = project_descriptor {
        let registered = write_and_register_project(descriptor.clone(), input.data_dir.as_deref())?;
        copy_project_surface_dist(&descriptor, &input.plan, input.data_dir.as_deref())?;
        Some(registered)
    } else {
        let root = input.plan.packages.first();
        if let Some(root) = root {
            let store_path = store_path_for_hash(&root.tree_hash, input.data_dir.as_deref())?;
            let project_yaml = store_path.join("project.yaml");
            if project_yaml.is_file() {
                let descriptor = read_project_descriptor(&project_yaml)?;
                Some(write_and_register_project(
                    descriptor,
                    input.data_dir.as_deref(),
                )?)
            } else {
                None
            }
        } else {
            None
        }
    };

    Ok(json!({
        "installed": installed,
        "lockfile_path": lockfile_path.to_string_lossy(),
        "profile_path": profile_path.to_string_lossy(),
        "lockfile": lockfile,
        "project": registered_project,
    }))
}

pub(super) async fn register_project_capability(input: Value) -> Result<Value> {
    let input: RegisterProjectInput = serde_json::from_value(input)?;
    let info = write_and_register_project(input.descriptor, input.data_dir.as_deref())?;
    Ok(info)
}

pub(super) async fn uninstall(input: Value) -> Result<Value> {
    if input.as_object().is_some_and(|object| object.is_empty()) {
        return Ok(json!({ "removed_from_profile": false, "store_path_orphaned": null }));
    }
    let input: UninstallInput = serde_json::from_value(input)?;
    let profile_path = profile_path(&input.profile, input.data_dir.as_deref())?;
    let lockfile_path = lockfile_path(&input.profile, input.data_dir.as_deref())?;
    let mut orphaned = None;
    let mut manifest_paths_for_removed = Vec::new();
    let mut removed = false;

    if lockfile_path.exists() {
        let raw = fs::read_to_string(&lockfile_path)?;
        let mut lock: Lockfile = toml::from_str(&raw)?;
        if let Some(entry) = lock
            .package
            .iter()
            .find(|entry| entry.id == input.package_id)
        {
            orphaned = Some(entry.installed_at_store.clone());
            manifest_paths_for_removed = manifest_candidates_for_lock_entry(entry);
        }
        let before = lock.package.len();
        lock.package.retain(|entry| entry.id != input.package_id);
        removed = before != lock.package.len();
        atomic_write(&lockfile_path, toml::to_string_pretty(&lock)?.as_bytes())?;
    }

    if profile_path.exists() {
        let raw = fs::read_to_string(&profile_path)?;
        let mut value: Value = serde_yaml::from_str(&raw)?;
        if let Some(autoload) = value.get_mut("autoload").and_then(Value::as_array_mut) {
            let before = autoload.len();
            let manifest_yaml = orphaned
                .as_ref()
                .map(|store| format!("{store}/manifest.yaml"));
            let manifest_json = orphaned
                .as_ref()
                .map(|store| format!("{store}/manifest.json"));
            autoload.retain(|entry| {
                !entry.as_str().is_some_and(|s| {
                    manifest_yaml.as_deref() == Some(s)
                        || manifest_json.as_deref() == Some(s)
                        || manifest_paths_for_removed
                            .iter()
                            .any(|candidate| candidate == s)
                })
            });
            removed |= before != autoload.len();
        }
        atomic_write(&profile_path, serde_yaml::to_string(&value)?.as_bytes())?;
    }
    Ok(json!({ "removed_from_profile": removed, "store_path_orphaned": orphaned }))
}

pub(super) async fn list_installed(input: Value) -> Result<Value> {
    let input: ProfileInput = serde_json::from_value(input)?;
    let lockfile_path = lockfile_path(&input.profile, input.data_dir.as_deref())?;
    if !lockfile_path.exists() {
        return Ok(json!({ "packages": [] }));
    }
    let lock: Lockfile = toml::from_str(&fs::read_to_string(lockfile_path)?)?;
    let packages = lock
        .package
        .into_iter()
        .map(|entry| {
            json!({
                "id": entry.id,
                "version": entry.version,
                "store_path": entry.installed_at_store,
                "granted_capabilities": entry.granted_capabilities,
                "granted_network": entry.granted_network,
                "granted_secrets": entry.granted_secrets,
                "tree_hash": entry.tree_hash,
                "manifest_hash": entry.manifest_hash,
                "manifest_relative_path": entry.manifest_relative_path,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({ "packages": packages }))
}

pub(super) async fn check_lockfile(input: Value) -> Result<Value> {
    let input: ProfileInput = serde_json::from_value(input)?;
    let lockfile_path = lockfile_path(&input.profile, input.data_dir.as_deref())?;
    if !lockfile_path.exists() {
        return Ok(
            json!({ "ok": false, "drift": [{ "id": "lockfile", "kind": "missing", "expected": lockfile_path.to_string_lossy(), "actual": null }] }),
        );
    }
    let lock: Lockfile = toml::from_str(&fs::read_to_string(lockfile_path)?)?;
    let mut drift = Vec::new();
    for entry in &lock.package {
        let store = PathBuf::from(&entry.installed_at_store);
        if !store.is_dir() {
            drift.push(json!({ "id": entry.id, "kind": "missing_store", "expected": entry.installed_at_store, "actual": null }));
            continue;
        }
        let manifest_path = if let Some(relative) = &entry.manifest_relative_path {
            store.join(safe_relative_path(relative)?)
        } else {
            match manifest_path_in(&store) {
                Ok(path) => path,
                Err(_) => {
                    drift.push(json!({ "id": entry.id, "kind": "missing_manifest", "expected": "manifest.yaml or manifest.json", "actual": null }));
                    continue;
                }
            }
        };
        if !manifest_path.is_file() {
            drift.push(json!({ "id": entry.id, "kind": "missing_manifest", "expected": manifest_path.to_string_lossy(), "actual": null }));
            continue;
        }
        let manifest_hash = compute_manifest_hash(&manifest_path).await?;
        if manifest_hash != entry.manifest_hash {
            drift.push(json!({ "id": entry.id, "kind": "manifest_hash", "expected": entry.manifest_hash, "actual": manifest_hash }));
        }
        if let Some(expected) = &entry.surface_bundle_hash {
            match surface_bundle_path_for_manifest(&manifest_path) {
                Ok(bundle_path) => {
                    let actual = compute_file_hash(&bundle_path)?;
                    if &actual != expected {
                        drift.push(json!({ "id": entry.id, "kind": "surface_bundle_hash", "expected": expected, "actual": actual }));
                    }
                }
                Err(error) => {
                    drift.push(json!({ "id": entry.id, "kind": "surface_bundle_hash", "expected": expected, "actual": error.to_string() }));
                }
            }
        }
        let tree_hash = compute_tree_hash(&store).await?;
        if tree_hash != entry.tree_hash {
            drift.push(json!({ "id": entry.id, "kind": "tree_hash", "expected": entry.tree_hash, "actual": tree_hash }));
        }
    }
    Ok(json!({ "ok": drift.is_empty(), "drift": drift }))
}

fn copy_project_surface_dist(
    descriptor: &ygg_core::ProjectDescriptor,
    plan: &InstallPlan,
    data_dir_override: Option<&str>,
) -> Result<()> {
    let Some((pkg, manifest)) = find_surface_bundle_package(plan, data_dir_override)? else {
        return Ok(());
    };
    let PackageEntry::SurfaceBundle { bundle } = &manifest.entry.kind else {
        return Ok(());
    };
    let store_path = store_path_for_hash(&pkg.tree_hash, data_dir_override)?;
    let manifest_path = installed_manifest_path(pkg, &store_path)?;
    if let Some(expected) = &pkg.surface_bundle_hash {
        let actual =
            compute_file_hash(&surface_bundle_path_from_manifest(&manifest_path, bundle)?)?;
        if &actual != expected {
            anyhow::bail!("surface bundle hash mismatch for {}", manifest.id);
        }
    }
    let package_root = manifest_path.parent().unwrap_or(&store_path);
    let bundle_path = safe_relative_path(bundle.as_str())?;
    let source_dist = bundle_path
        .parent()
        .map(|parent| package_root.join(parent))
        .unwrap_or_else(|| package_root.to_path_buf());
    if !source_dist.is_dir() {
        anyhow::bail!(
            "surface bundle dist directory does not exist for {}: {}",
            manifest.id,
            source_dist.display()
        );
    }
    let project_dir = super::project_kind::ensure_project_initialized_for(
        &descriptor.project.id,
        data_dir_override,
    )?;
    let dest_dist = project_dir.join("dist");
    if dest_dist.exists() {
        fs::remove_dir_all(&dest_dist)?;
    }
    super::fs_copy::copy_dir_recursive(&source_dist, &dest_dist)
}

fn find_surface_bundle_package<'a>(
    plan: &'a InstallPlan,
    data_dir_override: Option<&str>,
) -> Result<Option<(&'a super::types::PlannedPackage, PackageManifest)>> {
    for pkg in &plan.packages {
        let store_path = store_path_for_hash(&pkg.tree_hash, data_dir_override)?;
        let manifest_path = installed_manifest_path(pkg, &store_path)?;
        let manifest = parse_manifest_at(&manifest_path)?;
        if matches!(manifest.entry.kind, PackageEntry::SurfaceBundle { .. }) {
            return Ok(Some((pkg, manifest)));
        }
    }
    Ok(None)
}

fn manifest_candidates_for_lock_entry(entry: &LockEntry) -> Vec<String> {
    if let Some(relative) = &entry.manifest_relative_path {
        vec![format!("{}/{}", entry.installed_at_store, relative)]
    } else {
        vec![
            format!("{}/manifest.yaml", entry.installed_at_store),
            format!("{}/manifest.json", entry.installed_at_store),
        ]
    }
}

pub(super) async fn invoke_package_capability(
    provider: &str,
    capability_id: &str,
    input: Value,
) -> Result<Value> {
    Ok(invoke_capability_from_inproc(CapabilityInvocationRequest {
        handle: None,
        capability_id: Some(capability_id.to_string()),
        caller_package_id: Some(PACKAGE_ID.to_string()),
        provider_package_id: Some(provider.to_string()),
        version: None,
        session_id: None,
        input,
    })
    .await?
    .output)
}

pub(super) async fn compute_manifest_hash(path: &Path) -> Result<String> {
    let output = invoke_package_capability(
        "official/integrity-lab",
        "official/integrity-lab/compute_manifest_hash",
        json!({ "manifest_path": path.to_string_lossy() }),
    )
    .await?;
    Ok(value_str(&output, "sha256")?.to_string())
}

pub(super) fn compute_file_hash(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(&bytes);
    let mut out = String::from("sha256:");
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    Ok(out)
}

fn surface_bundle_path_for_manifest(manifest_path: &Path) -> Result<PathBuf> {
    let manifest = parse_manifest_at(manifest_path)?;
    let PackageEntry::SurfaceBundle { bundle } = &manifest.entry.kind else {
        anyhow::bail!("manifest is not surface_bundle");
    };
    surface_bundle_path_from_manifest(manifest_path, bundle)
}

fn surface_bundle_path_from_manifest(manifest_path: &Path, bundle: &str) -> Result<PathBuf> {
    Ok(manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(safe_relative_path(bundle)?))
}

pub(super) async fn compute_tree_hash(path: &Path) -> Result<String> {
    let output = invoke_package_capability(
        "official/integrity-lab",
        "official/integrity-lab/compute_tree_hash",
        json!({ "dir": path.to_string_lossy() }),
    )
    .await?;
    Ok(value_str(&output, "sha256")?.to_string())
}

fn write_profile(
    profile_path: &Path,
    plan: &InstallPlan,
    data_dir_override: Option<&str>,
) -> Result<()> {
    let mut autoload = if profile_path.exists() {
        let raw = fs::read_to_string(profile_path)?;
        serde_yaml::from_str::<Value>(&raw)
            .ok()
            .and_then(|v| v.get("autoload").and_then(Value::as_array).cloned())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let existing = autoload
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<std::collections::HashSet<_>>();
    for pkg in &plan.packages {
        let package_store = store_path_for_hash(&pkg.tree_hash, data_dir_override)?;
        let manifest = installed_manifest_path(pkg, &package_store)?;
        let entry = manifest.to_string_lossy().to_string();
        if !existing.contains(&entry) {
            autoload.push(Value::String(entry));
        }
    }
    atomic_write(
        profile_path,
        serde_yaml::to_string(&json!({ "autoload": autoload }))?.as_bytes(),
    )
}

fn build_lockfile(
    profile: &str,
    plan: &InstallPlan,
    consent: &Consent,
    data_dir_override: Option<&str>,
) -> Result<Lockfile> {
    let mut lock = Lockfile::new(profile, "sha256:profile");
    for pkg in &plan.packages {
        let installed_at_store = store_path_for_hash(&pkg.tree_hash, data_dir_override)?;
        lock.package.push(LockEntry {
            id: pkg.id.clone(),
            version: pkg.version.clone(),
            source: match pkg.source.as_str() {
                "git" => LockSource::Git,
                "local" => LockSource::Local,
                _ => LockSource::Internal,
            },
            url: pkg.url.clone(),
            r#ref: pkg.ref_name.clone(),
            commit: pkg.commit_sha.clone(),
            tree_hash: pkg.tree_hash.clone(),
            manifest_hash: pkg.manifest_hash.clone(),
            surface_bundle_hash: pkg.surface_bundle_hash.clone(),
            signed: pkg.signed,
            signed_by: pkg.signed_by.clone(),
            installed_at_store: installed_at_store.to_string_lossy().to_string(),
            manifest_relative_path: pkg.manifest_relative_path.clone(),
            granted_capabilities: consent.approved_capabilities.clone(),
            granted_network: consent.approved_network_hosts.clone(),
            granted_secrets: consent.approved_secret_refs.clone(),
            requires: pkg
                .requires
                .iter()
                .map(|req| LockRequirement {
                    id: req.id.clone(),
                    constraint: req.version.clone(),
                    resolved_to: req.id.clone(),
                })
                .collect(),
        });
    }
    Ok(lock)
}
