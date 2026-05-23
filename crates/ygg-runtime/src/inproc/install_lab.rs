//! Handler for `official/install-lab` capabilities.
//!
//! Orchestrates package installation by composing git-tools-lab and
//! integrity-lab through normal capability dispatch.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use ygg_core::{
    conformance::PackageConformanceReport, paths, DependencySource, LockEntry, LockRequirement,
    LockSource, Lockfile, PackageDependency, PackageManifest, PermissionSet, ProjectDescriptor,
    ProjectType,
};

use crate::CapabilityInvocationRequest;

use super::{invoke_capability_from_inproc, project_registry_from_inproc, InprocInvocation};

const PACKAGE_ID: &str = "official/install-lab";
const MAX_DEPTH: usize = 32;

pub async fn try_handle(request: &InprocInvocation) -> Option<Result<Value>> {
    if request.provider_package_id != PACKAGE_ID {
        return None;
    }
    match request.capability_id.as_str() {
        "install.resolve_plan" | "official/install-lab/resolve_plan" => {
            Some(resolve_plan(request.input.clone()).await)
        }
        "install.execute_plan" | "official/install-lab/execute_plan" => {
            Some(execute_plan(request.input.clone()).await)
        }
        "install.detect_kind" | "official/install-lab/detect_kind" => {
            Some(detect_kind(request.input.clone()).await)
        }
        "install.register_project" | "official/install-lab/register_project" => {
            Some(register_project_capability(request.input.clone()).await)
        }
        "install.uninstall" | "official/install-lab/uninstall" => {
            Some(uninstall(request.input.clone()).await)
        }
        "install.list_installed" | "official/install-lab/list_installed" => {
            Some(list_installed(request.input.clone()).await)
        }
        "install.check_lockfile" | "official/install-lab/check_lockfile" => {
            Some(check_lockfile(request.input.clone()).await)
        }
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
struct ResolvePlanInput {
    root_url: String,
    #[serde(default)]
    root_ref: String,
    #[serde(default)]
    lockfile: Option<String>,
    /// Require GPG-signed git tags. When false (default), unsigned packages
    /// install without signature verification (matches cargo/npm/pip baseline).
    /// When true, missing or invalid signatures abort install.
    #[serde(default)]
    require_signed: bool,
    /// Block install on conformance failure. When false (default), failures
    /// are reported as warnings but install proceeds. When true, any conformance
    /// failure aborts install.
    #[serde(default)]
    strict_conformance: bool,
}

#[derive(Debug, Deserialize)]
struct ExecutePlanInput {
    plan: InstallPlan,
    consent: Consent,
    #[serde(default)]
    project_descriptor: Option<ProjectDescriptor>,
    #[serde(default = "default_profile")]
    profile: String,
    #[serde(default)]
    data_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProfileInput {
    #[serde(default = "default_profile")]
    profile: String,
    #[serde(default)]
    data_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DetectKindInput {
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default = "default_head_ref")]
    root_ref: String,
}

#[derive(Debug, Deserialize)]
struct RegisterProjectInput {
    descriptor: ProjectDescriptor,
    #[serde(default)]
    data_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UninstallInput {
    package_id: String,
    #[serde(default = "default_profile")]
    profile: String,
    #[serde(default)]
    data_dir: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Consent {
    #[serde(default)]
    approved_capabilities: Vec<String>,
    #[serde(default)]
    approved_network_hosts: Vec<String>,
    #[serde(default)]
    approved_secret_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstallPlan {
    root_id: String,
    packages: Vec<PlannedPackage>,
    permissions_summary: PermissionsSummary,
    signature_summary: SignatureSummary,
    integrity_summary: IntegritySummary,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DetectedProjectKind {
    /// Has project.yaml that parses as YggdrasilNative.
    Native { descriptor: ProjectDescriptor },
    /// Has project.yaml that parses as ExternalWrapped or ExternalWorkspace.
    DeclaredExternal { descriptor: ProjectDescriptor },
    /// No project.yaml; this is an external project that needs wrapping or workspace mode.
    External { has_manifest_yaml: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlannedPackage {
    id: String,
    version: String,
    source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(default, rename = "ref", skip_serializing_if = "Option::is_none")]
    ref_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    commit_sha: Option<String>,
    manifest_hash: String,
    tree_hash: String,
    signed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    signed_by: Option<String>,
    permissions: PlannedPermissions,
    #[serde(default)]
    requires: Vec<PlannedRequirement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    conformance: Option<PackageConformanceReport>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PlannedPermissions {
    #[serde(default)]
    capabilities_invoke: Vec<String>,
    #[serde(default)]
    network_hosts: Vec<String>,
    #[serde(default)]
    secret_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlannedRequirement {
    id: String,
    source: Value,
    #[serde(default)]
    version: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PermissionsSummary {
    new_capabilities: Vec<String>,
    new_network_hosts: Vec<String>,
    new_secret_refs: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SignatureSummary {
    all_signed: bool,
    unsigned_packages: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct IntegritySummary {
    manifest_hashes_match_lockfile: bool,
    drift_detected: Vec<Value>,
}

#[derive(Debug, Clone)]
struct PackageDescriptor {
    source: SourceDescriptor,
}

#[derive(Debug, Clone)]
enum SourceDescriptor {
    Git { url: String, ref_name: String },
    Local { path: PathBuf },
    Internal,
}

pub async fn resolve_plan(input: Value) -> Result<Value> {
    if input.as_object().is_some_and(|object| object.is_empty()) {
        anyhow::bail!("unsupported smoke input: missing root_url");
    }
    let input: ResolvePlanInput = serde_json::from_value(input)?;
    let root = parse_root_descriptor(&input.root_url, &input.root_ref)?;
    let mut packages = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = Vec::new();
    let mut resolving = HashSet::new();
    resolve_transitive(
        root,
        input.require_signed,
        input.strict_conformance,
        &mut visited,
        &mut resolving,
        &mut stack,
        &mut packages,
        MAX_DEPTH,
    )?;

    let mut lock_map = HashMap::new();
    if let Some(lockfile) = input.lockfile.as_deref() {
        let lock: Lockfile = toml::from_str(lockfile).context("failed to parse lockfile TOML")?;
        for entry in lock.package {
            lock_map.insert(entry.id, entry.manifest_hash);
        }
    }
    let mut drift = Vec::new();
    for pkg in &packages {
        if let Some(expected) = lock_map.get(&pkg.id) {
            if expected != &pkg.manifest_hash {
                drift.push(json!({
                    "id": pkg.id,
                    "kind": "manifest_hash",
                    "expected": expected,
                    "actual": pkg.manifest_hash,
                }));
            }
        }
    }

    let permissions_summary = aggregate_permissions(&packages);
    let unsigned_packages = packages
        .iter()
        .filter(|pkg| !pkg.signed)
        .map(|pkg| pkg.id.clone())
        .collect::<Vec<_>>();
    let root_id = packages
        .first()
        .map(|pkg| pkg.id.clone())
        .unwrap_or_default();
    let plan = InstallPlan {
        root_id,
        packages,
        permissions_summary,
        signature_summary: SignatureSummary {
            all_signed: unsigned_packages.is_empty(),
            unsigned_packages,
        },
        integrity_summary: IntegritySummary {
            manifest_hashes_match_lockfile: drift.is_empty(),
            drift_detected: drift,
        },
    };
    Ok(json!({ "plan": plan }))
}

fn resolve_transitive(
    root: PackageDescriptor,
    require_signed: bool,
    strict_conformance: bool,
    visited: &mut HashSet<String>,
    resolving: &mut HashSet<String>,
    stack: &mut Vec<String>,
    plan: &mut Vec<PlannedPackage>,
    max_depth: usize,
) -> Result<()> {
    if max_depth == 0 {
        anyhow::bail!("dependency depth exceeded {MAX_DEPTH} (possible cycle)");
    }

    match &root.source {
        SourceDescriptor::Internal => return Ok(()),
        SourceDescriptor::Local { path } => {
            if !path.exists() {
                anyhow::bail!("local dependency path does not exist: {}", path.display());
            }
        }
        SourceDescriptor::Git { .. } => {}
    }

    let resolved = resolve_one(root, require_signed, strict_conformance)?;
    if let Some(pos) = stack.iter().position(|id| id == &resolved.id) {
        let mut cycle = stack[pos..].to_vec();
        cycle.push(resolved.id.clone());
        anyhow::bail!("dependency cycle detected: {}", cycle.join(" -> "));
    }
    if visited.contains(&resolved.id) {
        return Ok(());
    }
    if !resolving.insert(resolved.id.clone()) {
        anyhow::bail!("dependency cycle detected at {}", resolved.id);
    }
    visited.insert(resolved.id.clone());
    stack.push(resolved.id.clone());

    let requires = resolved.requires.clone();
    plan.push(resolved);
    for req in requires {
        let next = resolve_dep(&req)?;
        resolve_transitive(
            next,
            require_signed,
            strict_conformance,
            visited,
            resolving,
            stack,
            plan,
            max_depth - 1,
        )?;
    }
    let done = stack.pop();
    if let Some(done) = done {
        resolving.remove(&done);
    }
    Ok(())
}

fn resolve_one(
    desc: PackageDescriptor,
    require_signed: bool,
    strict_conformance: bool,
) -> Result<PlannedPackage> {
    match desc.source {
        SourceDescriptor::Local { path } => resolve_local_package(path, strict_conformance),
        SourceDescriptor::Git { url, ref_name } => block_on_current(resolve_git_package(
            url,
            ref_name,
            require_signed,
            strict_conformance,
        )),
        SourceDescriptor::Internal => {
            anyhow::bail!("internal packages do not require installation")
        }
    }
}

fn resolve_local_package(path: PathBuf, strict_conformance: bool) -> Result<PlannedPackage> {
    let path = fs::canonicalize(&path)
        .with_context(|| format!("failed to canonicalize local package {}", path.display()))?;
    let manifest_path = manifest_path_in(&path)?;
    let manifest = parse_manifest_at(&manifest_path)?;
    let manifest_hash = block_on_current(compute_manifest_hash(&manifest_path))?;
    let tree_hash = block_on_current(compute_tree_hash(&path))?;
    let mut planned = planned_from_manifest(
        &manifest,
        manifest_path.parent().unwrap_or(&path),
        "local".to_string(),
        None,
        None,
        Some(path.to_string_lossy().to_string()),
        None,
        manifest_hash,
        tree_hash,
        false,
        None,
    );
    attach_conformance(&mut planned, &path, strict_conformance)?;
    Ok(planned)
}

async fn resolve_git_package(
    url: String,
    ref_name: String,
    require_signed: bool,
    strict_conformance: bool,
) -> Result<PlannedPackage> {
    let resolved = invoke_package_capability(
        "official/git-tools-lab",
        "official/git-tools-lab/resolve_ref",
        json!({ "remote_url": url, "ref": ref_name }),
    )
    .await?;
    let commit_sha = value_str(&resolved, "commit_sha")?.to_string();
    let tmp = std::env::temp_dir().join(format!("yggdrasil-git-install-{}", Uuid::new_v4()));
    let fetch = invoke_package_capability(
        "official/git-tools-lab",
        "official/git-tools-lab/fetch_tree",
        json!({ "remote_url": url, "commit_sha": commit_sha, "dest_dir": tmp.to_string_lossy() }),
    )
    .await?;
    let _git_tree_hash = value_str(&fetch, "tree_hash")?.to_string();
    let result = async {
        let manifest_path = manifest_path_in(&tmp)?;
        let manifest = parse_manifest_at(&manifest_path)?;
        let manifest_hash = compute_manifest_hash(&manifest_path).await?;
        let tree_hash = compute_tree_hash(&tmp).await?;
        let mut signed = false;
        if require_signed {
            let tag = invoke_package_capability(
                "official/git-tools-lab",
                "official/git-tools-lab/read_signed_tag",
                json!({ "remote_url": url, "tag": ref_name }),
            )
            .await?;
            if tag.get("pgp_signature").and_then(Value::as_str).is_none() {
                anyhow::bail!("git source {}@{} is unsigned", url, ref_name);
            }
            signed = true;
        }
        let mut planned = planned_from_manifest(
            &manifest,
            manifest_path.parent().unwrap_or(&tmp),
            "git".to_string(),
            Some(url),
            Some(ref_name),
            None,
            Some(commit_sha),
            manifest_hash,
            tree_hash,
            signed,
            None,
        );
        attach_conformance(&mut planned, &tmp, strict_conformance)?;
        Ok(planned)
    }
    .await;
    fs::remove_dir_all(&tmp).ok();
    result
}

pub async fn execute_plan(input: Value) -> Result<Value> {
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
                    invoke_package_capability(
                        "official/git-tools-lab",
                        "official/git-tools-lab/fetch_tree",
                        json!({ "remote_url": url, "commit_sha": commit, "dest_dir": staging.to_string_lossy() }),
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
            "manifest_path": manifest_path_in(&store_path)?.to_string_lossy(),
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

    let registered_project = if let Some(descriptor) = input.project_descriptor {
        Some(write_and_register_project(
            descriptor,
            input.data_dir.as_deref(),
        )?)
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

pub async fn detect_kind(input: Value) -> Result<Value> {
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

pub async fn register_project_capability(input: Value) -> Result<Value> {
    let input: RegisterProjectInput = serde_json::from_value(input)?;
    let info = write_and_register_project(input.descriptor, input.data_dir.as_deref())?;
    Ok(info)
}

pub fn detect_project_kind(staging_dir: &Path) -> Result<DetectedProjectKind> {
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

pub async fn uninstall(input: Value) -> Result<Value> {
    if input.as_object().is_some_and(|object| object.is_empty()) {
        return Ok(json!({ "removed_from_profile": false, "store_path_orphaned": null }));
    }
    let input: UninstallInput = serde_json::from_value(input)?;
    let profile_path = profile_path(&input.profile, input.data_dir.as_deref())?;
    let lockfile_path = lockfile_path(&input.profile, input.data_dir.as_deref())?;
    let mut orphaned = None;
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
                    manifest_yaml.as_deref() == Some(s) || manifest_json.as_deref() == Some(s)
                })
            });
            removed |= before != autoload.len();
        }
        atomic_write(&profile_path, serde_yaml::to_string(&value)?.as_bytes())?;
    }
    Ok(json!({ "removed_from_profile": removed, "store_path_orphaned": orphaned }))
}

pub async fn list_installed(input: Value) -> Result<Value> {
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
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({ "packages": packages }))
}

pub async fn check_lockfile(input: Value) -> Result<Value> {
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
        let manifest_path = match manifest_path_in(&store) {
            Ok(path) => path,
            Err(_) => {
                drift.push(json!({ "id": entry.id, "kind": "missing_manifest", "expected": "manifest.yaml or manifest.json", "actual": null }));
                continue;
            }
        };
        let manifest_hash = compute_manifest_hash(&manifest_path).await?;
        if manifest_hash != entry.manifest_hash {
            drift.push(json!({ "id": entry.id, "kind": "manifest_hash", "expected": entry.manifest_hash, "actual": manifest_hash }));
        }
        let tree_hash = compute_tree_hash(&store).await?;
        if tree_hash != entry.tree_hash {
            drift.push(json!({ "id": entry.id, "kind": "tree_hash", "expected": entry.tree_hash, "actual": tree_hash }));
        }
    }
    Ok(json!({ "ok": drift.is_empty(), "drift": drift }))
}

async fn invoke_package_capability(
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

async fn compute_manifest_hash(path: &Path) -> Result<String> {
    let output = invoke_package_capability(
        "official/integrity-lab",
        "official/integrity-lab/compute_manifest_hash",
        json!({ "manifest_path": path.to_string_lossy() }),
    )
    .await?;
    Ok(value_str(&output, "sha256")?.to_string())
}

async fn compute_tree_hash(path: &Path) -> Result<String> {
    let output = invoke_package_capability(
        "official/integrity-lab",
        "official/integrity-lab/compute_tree_hash",
        json!({ "dir": path.to_string_lossy() }),
    )
    .await?;
    Ok(value_str(&output, "sha256")?.to_string())
}

fn planned_from_manifest(
    manifest: &PackageManifest,
    base_dir: &Path,
    source: String,
    url: Option<String>,
    ref_name: Option<String>,
    path: Option<String>,
    commit_sha: Option<String>,
    manifest_hash: String,
    tree_hash: String,
    signed: bool,
    signed_by: Option<String>,
) -> PlannedPackage {
    PlannedPackage {
        id: manifest.id.clone(),
        version: manifest.version.clone(),
        source,
        url,
        ref_name,
        path,
        commit_sha,
        manifest_hash,
        tree_hash,
        signed,
        signed_by,
        permissions: permissions_from_manifest(&manifest.permissions),
        requires: manifest
            .requires
            .iter()
            .map(|req| planned_requirement(req, base_dir))
            .collect(),
        conformance: None,
    }
}

fn attach_conformance(
    planned: &mut PlannedPackage,
    package_path: &Path,
    strict_conformance: bool,
) -> Result<()> {
    let report = block_on_current(ygg_core::conformance::run_checks(package_path, "v1", true))?;
    if !report.summary.passed_all_blocking() && strict_conformance {
        let errors = report.failed_checks().join("; ");
        anyhow::bail!("package {} fails v1 conformance: {errors}", planned.id);
    }
    planned.conformance = Some(report);
    Ok(())
}

fn planned_requirement(req: &PackageDependency, base_dir: &Path) -> PlannedRequirement {
    let source = match &req.source {
        DependencySource::Local { path } => {
            let dep_path = PathBuf::from(path);
            let absolute = if dep_path.is_absolute() {
                dep_path
            } else {
                base_dir.join(dep_path)
            };
            serde_json::to_value(DependencySource::Local {
                path: absolute.to_string_lossy().to_string(),
            })
            .unwrap_or(Value::Null)
        }
        other => serde_json::to_value(other).unwrap_or(Value::Null),
    };
    PlannedRequirement {
        id: req.id.clone(),
        source,
        version: req.version.clone(),
    }
}

fn permissions_from_manifest(permissions: &PermissionSet) -> PlannedPermissions {
    let mut network_hosts = permissions.network.hosts.clone();
    network_hosts.extend(
        permissions
            .network
            .declarations
            .iter()
            .map(|d| d.host.clone()),
    );
    PlannedPermissions {
        capabilities_invoke: sorted_vec(permissions.capabilities.invoke.iter().cloned()),
        network_hosts: sorted_vec(network_hosts),
        secret_refs: sorted_vec(permissions.secret_refs.iter().cloned()),
    }
}

fn aggregate_permissions(packages: &[PlannedPackage]) -> PermissionsSummary {
    PermissionsSummary {
        new_capabilities: sorted_vec(
            packages
                .iter()
                .flat_map(|pkg| pkg.permissions.capabilities_invoke.clone()),
        ),
        new_network_hosts: sorted_vec(
            packages
                .iter()
                .flat_map(|pkg| pkg.permissions.network_hosts.clone()),
        ),
        new_secret_refs: sorted_vec(
            packages
                .iter()
                .flat_map(|pkg| pkg.permissions.secret_refs.clone()),
        ),
    }
}

fn verify_consent(plan: &InstallPlan, consent: &Consent) -> Result<()> {
    ensure_subset(
        &plan.permissions_summary.new_capabilities,
        &consent.approved_capabilities,
        "capability",
    )?;
    ensure_subset(
        &plan.permissions_summary.new_network_hosts,
        &consent.approved_network_hosts,
        "network host",
    )?;
    ensure_subset(
        &plan.permissions_summary.new_secret_refs,
        &consent.approved_secret_refs,
        "secret ref",
    )?;
    Ok(())
}

fn ensure_subset(required: &[String], approved: &[String], kind: &str) -> Result<()> {
    for item in required {
        if !approved.iter().any(|approved| approved == item) {
            anyhow::bail!("consent missing required {kind}: {item}");
        }
    }
    Ok(())
}

fn parse_root_descriptor(root_url: &str, root_ref: &str) -> Result<PackageDescriptor> {
    if let Some(path) = root_url.strip_prefix("file://") {
        return Ok(PackageDescriptor {
            source: SourceDescriptor::Local {
                path: PathBuf::from(path),
            },
        });
    }
    if let Some(path) = root_url.strip_prefix("local:") {
        return Ok(PackageDescriptor {
            source: SourceDescriptor::Local {
                path: PathBuf::from(path),
            },
        });
    }
    let path = PathBuf::from(root_url);
    if path.exists() || root_url.starts_with('/') || root_url.starts_with('.') {
        return Ok(PackageDescriptor {
            source: SourceDescriptor::Local { path },
        });
    }
    let parsed = url::Url::parse(root_url)?;
    let mut url = parsed.clone();
    url.set_fragment(None);
    let ref_name = if root_ref.trim().is_empty() {
        parsed.fragment().unwrap_or("HEAD").to_string()
    } else {
        root_ref.to_string()
    };
    Ok(PackageDescriptor {
        source: SourceDescriptor::Git {
            url: url.to_string(),
            ref_name,
        },
    })
}

fn resolve_dep(req: &PlannedRequirement) -> Result<PackageDescriptor> {
    let source: DependencySource = serde_json::from_value(req.source.clone())?;
    let source = match source {
        DependencySource::Internal => SourceDescriptor::Internal,
        DependencySource::Git { url, r#ref } => SourceDescriptor::Git {
            url,
            ref_name: r#ref,
        },
        DependencySource::Local { path } => SourceDescriptor::Local {
            path: PathBuf::from(path),
        },
    };
    Ok(PackageDescriptor { source })
}

fn parse_manifest_at(path: &Path) -> Result<PackageManifest> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let manifest = match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => serde_json::from_str(&raw)?,
        _ => serde_yaml::from_str(&raw)?,
    };
    Ok(manifest)
}

fn manifest_path_in(dir: &Path) -> Result<PathBuf> {
    for name in ["manifest.yaml", "manifest.yml", "manifest.json"] {
        let path = dir.join(name);
        if path.is_file() {
            return Ok(path);
        }
    }
    anyhow::bail!("no manifest.yaml or manifest.json in {}", dir.display())
}

fn copy_dir_atomic(src: &Path, staging: &Path, dest: &Path) -> Result<()> {
    if staging.exists() {
        fs::remove_dir_all(staging).ok();
    }
    copy_dir_recursive(src, staging)?;
    fs::rename(staging, dest).with_context(|| {
        format!(
            "failed to atomically move {} to {}",
            staging.display(),
            dest.display()
        )
    })?;
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        let meta = fs::symlink_metadata(&from)?;
        if meta.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if meta.is_file() {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

async fn verify_installed_hashes(pkg: &PlannedPackage, store_path: &Path) -> Result<()> {
    let manifest_hash = compute_manifest_hash(&manifest_path_in(store_path)?).await?;
    if manifest_hash != pkg.manifest_hash {
        anyhow::bail!("manifest hash mismatch for {}", pkg.id);
    }
    let tree_hash = compute_tree_hash(store_path).await?;
    if tree_hash != pkg.tree_hash {
        anyhow::bail!("tree hash mismatch for {}", pkg.id);
    }
    Ok(())
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
        .collect::<HashSet<_>>();
    for pkg in &plan.packages {
        let package_store = store_path_for_hash(&pkg.tree_hash, data_dir_override)?;
        let manifest = manifest_path_in(&package_store)?;
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
            signed: pkg.signed,
            signed_by: pkg.signed_by.clone(),
            installed_at_store: installed_at_store.to_string_lossy().to_string(),
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

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!("tmp-{}", Uuid::new_v4()));
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn read_project_descriptor(path: &Path) -> Result<ProjectDescriptor> {
    let yaml = fs::read_to_string(path)
        .with_context(|| format!("failed to read project descriptor {}", path.display()))?;
    let descriptor: ProjectDescriptor =
        serde_yaml::from_str(&yaml).map_err(|e| anyhow::anyhow!("invalid project.yaml: {e}"))?;
    descriptor.validate()?;
    Ok(descriptor)
}

fn write_and_register_project(
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

fn ensure_project_initialized_for(
    project_id: &ygg_core::ProjectId,
    data_dir_override: Option<&str>,
) -> Result<PathBuf> {
    if let Some(dir) = data_dir_override {
        let project_dir = PathBuf::from(dir)
            .join("projects")
            .join(project_id.as_str());
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

fn default_head_ref() -> String {
    "HEAD".to_string()
}

fn ensure_layout(data_dir_override: Option<&str>) -> Result<()> {
    if let Some(dir) = data_dir_override {
        let data = PathBuf::from(dir);
        fs::create_dir_all(&data)?;
        fs::create_dir_all(data.join("store"))?;
        fs::create_dir_all(data.join("profiles"))?;
        fs::create_dir_all(data.join("keys"))?;
        fs::create_dir_all(data.join("cache"))?;
        fs::create_dir_all(data.join("projects"))?;
        fs::create_dir_all(data.join("projects/.archived"))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&data)?.permissions();
            perms.set_mode(0o700);
            fs::set_permissions(&data, perms)?;
        }

        return Ok(());
    }
    paths::ensure_initialized()
}

fn store_dir(data_dir_override: Option<&str>) -> Result<PathBuf> {
    if let Some(dir) = data_dir_override {
        return Ok(PathBuf::from(dir).join("store"));
    }
    paths::store_dir()
}

fn profiles_dir(data_dir_override: Option<&str>) -> Result<PathBuf> {
    if let Some(dir) = data_dir_override {
        return Ok(PathBuf::from(dir).join("profiles"));
    }
    paths::profiles_dir()
}

fn lockfile_path(profile: &str, data_dir_override: Option<&str>) -> Result<PathBuf> {
    if let Some(dir) = data_dir_override {
        return Ok(PathBuf::from(dir)
            .join("profiles")
            .join(format!("{profile}.lock.toml")));
    }
    paths::lockfile_path(profile)
}

fn profile_path(profile: &str, data_dir_override: Option<&str>) -> Result<PathBuf> {
    if let Some(dir) = data_dir_override {
        return Ok(PathBuf::from(dir)
            .join("profiles")
            .join(format!("{profile}.yaml")));
    }
    paths::profile_path(profile)
}

fn store_path_for_hash(tree_hash: &str, data_dir_override: Option<&str>) -> Result<PathBuf> {
    if let Some(dir) = data_dir_override {
        return Ok(PathBuf::from(dir)
            .join("store")
            .join(tree_hash.replace(':', "-")));
    }
    paths::store_path_for_hash(tree_hash)
}

fn value_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("missing string field '{key}'"))
}

fn sorted_vec(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn default_profile() -> String {
    "default".to_string()
}

fn block_on_current<F>(future: F) -> F::Output
where
    F: Future,
{
    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(future))
}
