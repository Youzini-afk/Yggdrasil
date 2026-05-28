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

async fn load_install_lab() -> anyhow::Result<ygg_runtime::Runtime<ygg_runtime::InMemoryEventStore>>
{
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
            session_id: None,
            input,
        })
        .await
        .map_err(Into::into)
}

pub(crate) async fn resolve_plan_local_source() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let pkg = fixture_path("pkg-local");
    let out = invoke(
        &rt,
        "official/install-lab/resolve_plan",
        json!({ "root_url": pkg }),
    )
    .await?;
    let plan = &out.output["plan"];
    anyhow::ensure!(plan["root_id"] == json!("fixture/pkg-local"));
    anyhow::ensure!(plan["packages"].as_array().context("packages")?.len() == 1);
    anyhow::ensure!(plan["packages"][0]["source"] == json!("local"));
    anyhow::ensure!(plan["packages"][0]["manifest_hash"]
        .as_str()
        .unwrap_or("")
        .starts_with("sha256:"));
    anyhow::ensure!(plan["packages"][0]["tree_hash"]
        .as_str()
        .unwrap_or("")
        .starts_with("sha256:"));
    Ok(())
}

pub(crate) async fn resolve_plan_runs_conformance() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let pkg = fixture_path("pkg-local");
    let out = invoke(
        &rt,
        "official/install-lab/resolve_plan",
        json!({ "root_url": pkg }),
    )
    .await?;
    let report = &out.output["plan"]["packages"][0]["conformance"];
    anyhow::ensure!(report["summary"]["passed"].as_u64().unwrap_or(0) >= 2);
    anyhow::ensure!(report["summary"]["skipped"].as_u64().unwrap_or(0) >= 5);
    anyhow::ensure!(report["checks"]
        .as_array()
        .context("checks")?
        .iter()
        .any(|check| {
            check["id"] == json!("manifest.schema_valid") && check["status"] == json!("Pass")
        }));
    Ok(())
}

pub(crate) async fn resolve_plan_blocks_when_strict() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let err = invoke(
        &rt,
        "official/install-lab/resolve_plan",
        json!({
            "root_url": fixture_path("pkg-broken-manifest"),
            "strict_conformance": true,
        }),
    )
    .await
    .expect_err("broken manifest should fail conformance in strict mode");
    anyhow::ensure!(
        err.to_string().contains("fails v1 conformance"),
        "unexpected error: {err}"
    );
    Ok(())
}

pub(crate) async fn strict_conformance_blocks() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let err = invoke(
        &rt,
        "official/install-lab/resolve_plan",
        json!({
            "root_url": fixture_path("pkg-broken-manifest"),
            "strict_conformance": true,
        }),
    )
    .await
    .expect_err("strict mode should block broken manifest");
    anyhow::ensure!(
        err.to_string().contains("fails v1 conformance"),
        "unexpected error: {err}"
    );

    let out = invoke(
        &rt,
        "official/install-lab/resolve_plan",
        json!({
            "root_url": fixture_path("pkg-broken-manifest"),
            "strict_conformance": false,
        }),
    )
    .await?;
    let report = &out.output["plan"]["packages"][0]["conformance"];
    anyhow::ensure!(report["summary"]["failed"].as_u64().unwrap_or(0) > 0);
    Ok(())
}

pub(crate) async fn lenient_conformance_warns_not_blocks() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let out = invoke(
        &rt,
        "official/install-lab/resolve_plan",
        json!({
            "root_url": fixture_path("pkg-broken-manifest"),
            "strict_conformance": false,
        }),
    )
    .await?;
    let report = &out.output["plan"]["packages"][0]["conformance"];
    anyhow::ensure!(report["summary"]["failed"].as_u64().unwrap_or(0) > 0);
    Ok(())
}

pub(crate) async fn transitive_conformance_propagates() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let out = invoke(
        &rt,
        "official/install-lab/resolve_plan",
        json!({
            "root_url": fixture_path("pkg-a-broken-dep"),
            "strict_conformance": false,
        }),
    )
    .await?;
    let packages = out.output["plan"]["packages"]
        .as_array()
        .context("packages")?;
    let dep = packages
        .iter()
        .find(|pkg| pkg["id"] == json!("fixture/pkg-broken-manifest"))
        .context("broken dependency package")?;
    anyhow::ensure!(
        dep["conformance"]["summary"]["failed"]
            .as_u64()
            .unwrap_or(0)
            > 0
    );
    Ok(())
}

pub(crate) async fn resolve_plan_with_transitive() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let out = invoke(
        &rt,
        "official/install-lab/resolve_plan",
        json!({ "root_url": fixture_path("pkg-a") }),
    )
    .await?;
    let packages = out.output["plan"]["packages"]
        .as_array()
        .context("packages")?;
    let ids: Vec<_> = packages
        .iter()
        .filter_map(|pkg| pkg["id"].as_str())
        .collect();
    anyhow::ensure!(ids.contains(&"fixture/pkg-a"));
    anyhow::ensure!(ids.contains(&"fixture/pkg-b"));
    Ok(())
}

pub(crate) async fn resolve_plan_cycle_detection() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let err = invoke(
        &rt,
        "official/install-lab/resolve_plan",
        json!({ "root_url": fixture_path("pkg-cycle-a") }),
    )
    .await
    .expect_err("cycle should fail");
    anyhow::ensure!(err.to_string().contains("cycle"), "unexpected error: {err}");
    Ok(())
}

pub(crate) async fn project_root_install_registers_surface_dist() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let tmp = TempDir::new()?;
    let out = invoke(
        &rt,
        "official/install-lab/resolve_plan",
        json!({ "root_url": fixture_path("project-root") }),
    )
    .await?;
    let plan = out.output["plan"].clone();
    let packages = plan["packages"].as_array().context("packages")?;
    anyhow::ensure!(packages.len() == 2);
    anyhow::ensure!(packages.iter().any(|pkg| {
        pkg["id"] == json!("fixture/project-engine")
            && pkg["manifest_relative_path"] == json!("packages/engine/manifest.yaml")
    }));
    anyhow::ensure!(packages.iter().any(|pkg| {
        pkg["id"] == json!("fixture/project-surface")
            && pkg["manifest_relative_path"] == json!("packages/surface/manifest.yaml")
    }));
    anyhow::ensure!(
        plan["project_descriptor"]["project"]["id"] == json!("fixture-project__abc12345")
    );

    fs::create_dir_all(tmp.path().join("profiles"))?;
    fs::write(
        tmp.path().join("profiles/default.yaml"),
        "title: Existing profile\nevent_store:\n  kind: sqlite\n  path: /tmp/events.sqlite\nsecret_resolver:\n  store_enabled: true\nautoload:\n  - /existing/package/manifest.yaml\n",
    )?;
    let executed = execute_with_full_consent(&rt, plan, tmp.path()).await?;
    anyhow::ensure!(executed.output["project"]["project_id"] == json!("fixture-project__abc12345"));
    anyhow::ensure!(tmp
        .path()
        .join("projects/fixture-project__abc12345/project.yaml")
        .is_file());
    anyhow::ensure!(tmp
        .path()
        .join("projects/fixture-project__abc12345/dist/bundle.mjs")
        .is_file());
    let profile = fs::read_to_string(tmp.path().join("profiles/default.yaml"))?;
    anyhow::ensure!(profile.contains("event_store:"));
    anyhow::ensure!(profile.contains("secret_resolver:"));
    anyhow::ensure!(profile.contains("/existing/package/manifest.yaml"));
    anyhow::ensure!(profile.contains("packages/engine/manifest.yaml"));
    anyhow::ensure!(profile.contains("packages/surface/manifest.yaml"));
    let checked = invoke(
        &rt,
        "official/install-lab/check_lockfile",
        json!({ "data_dir": tmp.path() }),
    )
    .await?;
    anyhow::ensure!(checked.output["ok"] == json!(true));

    let lockfile = fs::read_to_string(tmp.path().join("profiles/default.lock.toml"))?;
    let surface_store = lockfile
        .lines()
        .skip_while(|line| !line.trim().starts_with("id = \"fixture/project-surface\""))
        .find_map(|line| {
            line.trim()
                .strip_prefix("installed_at_store = ")
                .and_then(|value| value.trim().trim_matches('"').parse::<String>().ok())
        })
        .context("surface installed_at_store in lockfile")?;
    fs::write(
        Path::new(&surface_store).join("packages/surface/dist/bundle.mjs"),
        "tampered",
    )?;
    let checked = invoke(
        &rt,
        "official/install-lab/check_lockfile",
        json!({ "data_dir": tmp.path() }),
    )
    .await?;
    anyhow::ensure!(checked.output["ok"] == json!(false));
    let drift = checked.output["drift"].as_array().context("drift array")?;
    anyhow::ensure!(drift
        .iter()
        .any(|entry| entry["kind"] == json!("surface_bundle_hash")));
    Ok(())
}

pub(crate) async fn execute_plan_local() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let tmp = TempDir::new()?;
    let plan = plan_for(&rt, "pkg-local").await?;
    let out = execute_with_full_consent(&rt, plan, tmp.path()).await?;
    anyhow::ensure!(Path::new(
        out.output["installed"][0]["store_path"]
            .as_str()
            .context("store_path")?
    )
    .is_dir());
    anyhow::ensure!(Path::new(
        out.output["profile_path"]
            .as_str()
            .context("profile_path")?
    )
    .is_file());
    anyhow::ensure!(Path::new(
        out.output["lockfile_path"]
            .as_str()
            .context("lockfile_path")?
    )
    .is_file());
    anyhow::ensure!(out.output["lockfile"]
        .as_str()
        .unwrap_or("")
        .contains("fixture/pkg-local"));
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

    let plan = plan_for(&rt, "project-root").await?;
    execute_with_full_consent(&rt, plan, tmp.path()).await?;
    let out = invoke(
        &rt,
        "official/install-lab/uninstall",
        json!({ "project_id": "fixture-project__abc12345", "data_dir": tmp.path() }),
    )
    .await?;
    anyhow::ensure!(out.output["removed_from_profile"] == json!(true));
    anyhow::ensure!(out.output["project"]["data_action"] == json!("archived"));
    let profile = fs::read_to_string(tmp.path().join("profiles/default.yaml"))?;
    anyhow::ensure!(!profile.contains("packages/engine/manifest.yaml"));
    anyhow::ensure!(!profile.contains("packages/surface/manifest.yaml"));
    anyhow::ensure!(!tmp
        .path()
        .join("projects/fixture-project__abc12345")
        .exists());
    anyhow::ensure!(tmp
        .path()
        .join("projects/.archived/fixture-project__abc12345/project.yaml")
        .is_file());
    Ok(())
}

pub(crate) async fn list_installed_reflects_lockfile() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let tmp = TempDir::new()?;
    let plan = plan_for(&rt, "pkg-local").await?;
    execute_with_full_consent(&rt, plan, tmp.path()).await?;
    let out = invoke(
        &rt,
        "official/install-lab/list_installed",
        json!({ "data_dir": tmp.path() }),
    )
    .await?;
    let packages = out.output["packages"].as_array().context("packages")?;
    anyhow::ensure!(packages
        .iter()
        .any(|pkg| pkg["id"] == json!("fixture/pkg-local")));
    Ok(())
}

pub(crate) async fn check_lockfile_drift_detection() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let tmp = TempDir::new()?;
    let plan = plan_for(&rt, "pkg-local").await?;
    let out = execute_with_full_consent(&rt, plan, tmp.path()).await?;
    let store = PathBuf::from(
        out.output["installed"][0]["store_path"]
            .as_str()
            .context("store_path")?,
    );
    fs::write(store.join("tamper.txt"), "changed")?;
    let checked = invoke(
        &rt,
        "official/install-lab/check_lockfile",
        json!({ "data_dir": tmp.path() }),
    )
    .await?;
    anyhow::ensure!(checked.output["ok"] == json!(false));
    anyhow::ensure!(!checked.output["drift"]
        .as_array()
        .context("drift")?
        .is_empty());
    Ok(())
}

pub(crate) async fn check_for_updates_local_dangling_unsupported() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let tmp = TempDir::new()?;
    let source = tmp.path().join("local-source");
    fs::create_dir_all(&source)?;
    fs::write(
        source.join("manifest.yaml"),
        "schema_version: 1\nid: fixture/update-local\nversion: 0.1.0\nentry:\n  kind: rust_inproc\n  crate_ref: fixture\n  contract: v1\n  symbol: register\n  abi_version: 1\nprovides: []\npermissions: {}\n",
    )?;
    fs::write(source.join("content.txt"), "one")?;
    let plan = invoke(
        &rt,
        "official/install-lab/resolve_plan",
        json!({ "root_url": source }),
    )
    .await?
    .output["plan"]
        .clone();
    let executed = execute_with_full_consent(&rt, plan, tmp.path()).await?;
    let store = PathBuf::from(
        executed.output["installed"][0]["store_path"]
            .as_str()
            .context("store path")?,
    );

    let current = invoke(
        &rt,
        "official/install-lab/check_for_updates",
        json!({ "data_dir": tmp.path(), "package_id": "fixture/update-local" }),
    )
    .await?;
    anyhow::ensure!(current.output["results"][0]["status"] == json!("current"));

    fs::write(source.join("content.txt"), "two")?;
    let changed = invoke(
        &rt,
        "official/install-lab/check_for_updates",
        json!({ "data_dir": tmp.path(), "package_id": "fixture/update-local" }),
    )
    .await?;
    anyhow::ensure!(changed.output["results"][0]["status"] == json!("update_available"));
    anyhow::ensure!(changed.output["results"][0]["available"] == json!(true));

    let lock_path = tmp.path().join("profiles/default.lock.toml");
    let mut lock: ygg_core::Lockfile = toml::from_str(&fs::read_to_string(&lock_path)?)?;
    let saved_source_path = lock.package[0].source_path.take();
    fs::write(&lock_path, toml::to_string_pretty(&lock)?)?;
    let missing_source_path = invoke(
        &rt,
        "official/install-lab/check_for_updates",
        json!({ "data_dir": tmp.path(), "package_id": "fixture/update-local" }),
    )
    .await?;
    anyhow::ensure!(missing_source_path.output["results"][0]["status"] == json!("not_applicable"));
    anyhow::ensure!(missing_source_path.output["results"][0]["available"] == json!(false));
    anyhow::ensure!(missing_source_path.output["results"][0]["source_kind"] == json!("local"));
    anyhow::ensure!(missing_source_path.output["results"][0]["reason"]
        .as_str()
        .unwrap_or_default()
        .contains("source_path"));

    lock.package[0].source_path = saved_source_path;
    fs::write(&lock_path, toml::to_string_pretty(&lock)?)?;

    fs::remove_dir_all(&store)?;
    let dangling = invoke(
        &rt,
        "official/install-lab/check_for_updates",
        json!({ "data_dir": tmp.path(), "package_id": "fixture/update-local" }),
    )
    .await?;
    anyhow::ensure!(dangling.output["results"][0]["status"] == json!("repair_required"));
    anyhow::ensure!(dangling.output["results"][0]["dangling"] == json!(true));

    let mut lock: ygg_core::Lockfile = toml::from_str(&fs::read_to_string(&lock_path)?)?;
    lock.package[0].id = "official/internal-fixture".to_string();
    lock.package[0].source = ygg_core::LockSource::Internal;
    lock.package[0].manifest_relative_path = None;
    fs::write(&lock_path, toml::to_string_pretty(&lock)?)?;
    let unsupported = invoke(
        &rt,
        "official/install-lab/check_for_updates",
        json!({ "data_dir": tmp.path(), "package_id": "official/internal-fixture" }),
    )
    .await?;
    anyhow::ensure!(unsupported.output["results"][0]["status"] == json!("not_applicable"));
    anyhow::ensure!(unsupported.output["results"][0]["applicable"] == json!(false));
    Ok(())
}

pub(crate) async fn check_for_updates_external_project_not_applicable() -> anyhow::Result<()> {
    let rt = load_install_lab().await?;
    let tmp = TempDir::new()?;
    let source = tmp.path().join("external-source");
    fs::create_dir_all(&source)?;
    let project_id = "external_updates__abc123";
    let project_dir = tmp.path().join("projects").join(project_id);
    fs::create_dir_all(&project_dir)?;
    fs::write(
        project_dir.join("project.yaml"),
        format!(
            "schema_version: 1\nproject:\n  id: {project_id}\n  title: External Updates\n  type: external_workspace\n  packages: []\n  external:\n    source: {}\n    workspace_root: {}\n",
            source.display(),
            source.display()
        ),
    )?;

    let checked = invoke(
        &rt,
        "official/install-lab/check_for_updates",
        json!({ "data_dir": tmp.path(), "project_id": project_id }),
    )
    .await?;
    let results = checked.output["results"].as_array().context("results")?;
    anyhow::ensure!(results.len() == 1);
    anyhow::ensure!(results[0]["status"] == json!("not_applicable"));
    anyhow::ensure!(results[0]["available"] == json!(false));
    anyhow::ensure!(results[0]["project_id"] == json!(project_id));
    anyhow::ensure!(results[0]["source_kind"] == json!("external_workspace"));
    Ok(())
}

async fn plan_for(
    runtime: &ygg_runtime::Runtime<ygg_runtime::InMemoryEventStore>,
    fixture: &str,
) -> anyhow::Result<Value> {
    Ok(invoke(
        runtime,
        "official/install-lab/resolve_plan",
        json!({ "root_url": fixture_path(fixture) }),
    )
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
