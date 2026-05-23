use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde_json::{json, Value};
use tempfile::TempDir;
use ygg_runtime::CapabilityInvocationRequest;

use super::fixtures::*;
use crate::commands::manifest;

const INSTALL_MANIFEST: &str = "packages/official/install-lab/manifest.yaml";
const GIT_MANIFEST: &str = "packages/official/git-tools-lab/manifest.yaml";
const INTEGRITY_MANIFEST: &str = "packages/official/integrity-lab/manifest.yaml";
const PACKAGE_ID: &str = "official/install-lab";

async fn load_install_lab() -> anyhow::Result<ygg_runtime::Runtime<ygg_runtime::InMemoryEventStore>> {
    let (_store, runtime) = runtime();
    for path in [GIT_MANIFEST, INTEGRITY_MANIFEST, INSTALL_MANIFEST] {
        runtime
            .load_package(manifest::read_manifest(PathBuf::from(path)).await?)
            .await?;
    }
    Ok(runtime)
}

async fn invoke(
    runtime: &ygg_runtime::Runtime<ygg_runtime::InMemoryEventStore>,
    cap: &str,
    input: Value,
) -> anyhow::Result<ygg_runtime::CapabilityInvocationResult> {
    runtime
        .invoke_capability(CapabilityInvocationRequest {
            handle: None,
            capability_id: Some(cap.to_string()),
            caller_package_id: None,
            provider_package_id: Some(PACKAGE_ID.to_string()),
            version: None,
            input,
        })
        .await
        .map_err(Into::into)
}

pub(crate) async fn resolve_plan_local_source() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let pkg = fixture_path("pkg-local");
    let out = invoke(&rt, "official/install-lab/resolve_plan", json!({ "root_url": pkg })).await?;
    let plan = &out.output["plan"];
    anyhow::ensure!(plan["root_id"] == json!("fixture/pkg-local"));
    anyhow::ensure!(plan["packages"].as_array().context("packages")?.len() == 1);
    anyhow::ensure!(plan["packages"][0]["source"] == json!("local"));
    anyhow::ensure!(plan["packages"][0]["manifest_hash"].as_str().unwrap_or("").starts_with("sha256:"));
    anyhow::ensure!(plan["packages"][0]["tree_hash"].as_str().unwrap_or("").starts_with("sha256:"));
    Ok(())
}

pub(crate) async fn resolve_plan_with_transitive() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let out = invoke(&rt, "official/install-lab/resolve_plan", json!({ "root_url": fixture_path("pkg-a") })).await?;
    let packages = out.output["plan"]["packages"].as_array().context("packages")?;
    let ids: Vec<_> = packages.iter().filter_map(|pkg| pkg["id"].as_str()).collect();
    anyhow::ensure!(ids.contains(&"fixture/pkg-a"));
    anyhow::ensure!(ids.contains(&"fixture/pkg-b"));
    Ok(())
}

pub(crate) async fn resolve_plan_cycle_detection() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let err = invoke(&rt, "official/install-lab/resolve_plan", json!({ "root_url": fixture_path("pkg-cycle-a") }))
        .await
        .expect_err("cycle should fail");
    anyhow::ensure!(err.to_string().contains("cycle"), "unexpected error: {err}");
    Ok(())
}

pub(crate) async fn execute_plan_local() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let tmp = TempDir::new()?;
    let plan = plan_for(&rt, "pkg-local").await?;
    let out = execute_with_full_consent(&rt, plan, tmp.path()).await?;
    anyhow::ensure!(Path::new(out.output["installed"][0]["store_path"].as_str().context("store_path")?).is_dir());
    anyhow::ensure!(Path::new(out.output["profile_path"].as_str().context("profile_path")?).is_file());
    anyhow::ensure!(Path::new(out.output["lockfile_path"].as_str().context("lockfile_path")?).is_file());
    anyhow::ensure!(out.output["lockfile"].as_str().unwrap_or("").contains("fixture/pkg-local"));
    Ok(())
}

pub(crate) async fn execute_plan_consent_mismatch() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let tmp = TempDir::new()?;
    let plan = plan_for(&rt, "pkg-local").await?;
    let err = invoke(
        &rt,
        "official/install-lab/execute_plan",
        json!({
            "plan": plan,
            "consent": { "approved_capabilities": [], "approved_network_hosts": [], "approved_secret_refs": [] },
            "data_dir": tmp.path(),
        }),
    )
    .await
    .expect_err("consent mismatch should fail");
    anyhow::ensure!(err.to_string().contains("consent missing"));
    Ok(())
}

pub(crate) async fn uninstall_removes_from_profile() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let tmp = TempDir::new()?;
    let plan = plan_for(&rt, "pkg-local").await?;
    execute_with_full_consent(&rt, plan, tmp.path()).await?;
    let out = invoke(
        &rt,
        "official/install-lab/uninstall",
        json!({ "package_id": "fixture/pkg-local", "data_dir": tmp.path() }),
    )
    .await?;
    anyhow::ensure!(out.output["removed_from_profile"] == json!(true));
    let profile = fs::read_to_string(tmp.path().join("profiles/default.yaml"))?;
    anyhow::ensure!(!profile.contains("pkg-local"));
    Ok(())
}

pub(crate) async fn list_installed_reflects_lockfile() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let tmp = TempDir::new()?;
    let plan = plan_for(&rt, "pkg-local").await?;
    execute_with_full_consent(&rt, plan, tmp.path()).await?;
    let out = invoke(&rt, "official/install-lab/list_installed", json!({ "data_dir": tmp.path() })).await?;
    let packages = out.output["packages"].as_array().context("packages")?;
    anyhow::ensure!(packages.iter().any(|pkg| pkg["id"] == json!("fixture/pkg-local")));
    Ok(())
}

pub(crate) async fn check_lockfile_drift_detection() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let tmp = TempDir::new()?;
    let plan = plan_for(&rt, "pkg-local").await?;
    let out = execute_with_full_consent(&rt, plan, tmp.path()).await?;
    let store = PathBuf::from(out.output["installed"][0]["store_path"].as_str().context("store_path")?);
    fs::write(store.join("tamper.txt"), "changed")?;
    let checked = invoke(&rt, "official/install-lab/check_lockfile", json!({ "data_dir": tmp.path() })).await?;
    anyhow::ensure!(checked.output["ok"] == json!(false));
    anyhow::ensure!(!checked.output["drift"].as_array().context("drift")?.is_empty());
    Ok(())
}

async fn plan_for(
    runtime: &ygg_runtime::Runtime<ygg_runtime::InMemoryEventStore>,
    fixture: &str,
) -> anyhow::Result<Value> {
    Ok(invoke(runtime, "official/install-lab/resolve_plan", json!({ "root_url": fixture_path(fixture) }))
        .await?
        .output["plan"]
        .clone())
}

async fn execute_with_full_consent(
    runtime: &ygg_runtime::Runtime<ygg_runtime::InMemoryEventStore>,
    plan: Value,
    data_dir: &Path,
) -> anyhow::Result<ygg_runtime::CapabilityInvocationResult> {
    let summary = &plan["permissions_summary"];
    invoke(
        runtime,
        "official/install-lab/execute_plan",
        json!({
            "plan": plan,
            "consent": {
                "approved_capabilities": summary["new_capabilities"].clone(),
                "approved_network_hosts": summary["new_network_hosts"].clone(),
                "approved_secret_refs": summary["new_secret_refs"].clone(),
            },
            "data_dir": data_dir,
        }),
    )
    .await
}

fn fixture_path(name: &str) -> String {
    format!(
        "{}",
        PathBuf::from("crates/ygg-cli/src/conformance/fixtures/install")
            .join(name)
            .display()
    )
}
