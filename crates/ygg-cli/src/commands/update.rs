use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use serde_json::json;

use crate::commands::install::{
    invoke_install_lab, load_install_runtime, lockfile_path, print_plan_human,
};
use crate::install::consent::approve_all;
use crate::install::default_data_dir;

#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// Specific package to update (default: check all)
    pub package_id: Option<String>,

    #[arg(short, long, default_value = "default")]
    pub profile: String,

    #[arg(long)]
    pub data_dir: Option<PathBuf>,

    /// Only check for updates, don't install
    #[arg(long)]
    pub check_only: bool,
}

pub async fn run(args: UpdateArgs) -> Result<()> {
    let data_dir = args.data_dir.unwrap_or_else(default_data_dir);
    let runtime = load_install_runtime().await?;
    let listed = invoke_install_lab(
        &runtime,
        "official/install-lab/list_installed",
        json!({
            "profile": args.profile,
            "data_dir": data_dir.display().to_string(),
        }),
    )
    .await?
    .output;

    let packages = listed["packages"].as_array().cloned().unwrap_or_default();
    let lock_entries = read_lock_entries(&data_dir, &args.profile)?;
    let mut checked = 0usize;
    let mut updated = 0usize;

    for pkg in packages {
        let id = pkg["id"].as_str().unwrap_or_default();
        if args
            .package_id
            .as_deref()
            .is_some_and(|wanted| wanted != id)
        {
            continue;
        }
        checked += 1;
        let Some(entry) = lock_entries.get(id) else {
            println!("{id}: missing lockfile entry");
            continue;
        };
        if entry.source != "git" {
            println!("{id}: skipped ({} source)", entry.source);
            continue;
        }
        let Some(url) = entry.url.as_deref() else {
            println!("{id}: skipped (missing git URL)");
            continue;
        };
        let ref_name = entry.ref_name.as_deref().unwrap_or("HEAD");
        let plan = invoke_install_lab(
            &runtime,
            "official/install-lab/resolve_plan",
            json!({
                "root_url": url,
                "root_ref": ref_name,
                "lockfile": std::fs::read_to_string(lockfile_path(&data_dir, &args.profile)).ok(),
                "require_signed": false,
                "strict_conformance": false,
            }),
        )
        .await?
        .output["plan"]
            .clone();
        let new_commit = plan["packages"]
            .as_array()
            .and_then(|packages| {
                packages
                    .iter()
                    .find(|candidate| candidate["id"] == json!(id))
            })
            .and_then(|candidate| candidate["commit_sha"].as_str())
            .unwrap_or("");
        if entry.commit.as_deref() == Some(new_commit) {
            println!("{id}: up to date");
            continue;
        }
        println!(
            "{id}: update available {} -> {}",
            entry.commit.as_deref().unwrap_or("(unknown)"),
            if new_commit.is_empty() {
                "(unknown)"
            } else {
                new_commit
            }
        );
        if args.check_only {
            continue;
        }
        print_plan_human(&plan);
        let consent = approve_all(&plan);
        invoke_install_lab(
            &runtime,
            "official/install-lab/execute_plan",
            json!({
                "plan": plan,
                "consent": consent,
                "profile": args.profile,
                "data_dir": data_dir.display().to_string(),
            }),
        )
        .await?;
        updated += 1;
    }

    if checked == 0 {
        if let Some(package_id) = args.package_id {
            anyhow::bail!("package '{package_id}' is not installed");
        }
        println!("No installed packages.");
    } else if !args.check_only {
        println!("Updated {updated} packages.");
    }
    Ok(())
}

#[derive(Debug)]
struct LockEntrySummary {
    source: String,
    url: Option<String>,
    ref_name: Option<String>,
    commit: Option<String>,
}

fn read_lock_entries(
    data_dir: &std::path::Path,
    profile: &str,
) -> Result<HashMap<String, LockEntrySummary>> {
    let path = lockfile_path(data_dir, profile);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let raw = std::fs::read_to_string(path)?;
    let value: toml::Value = toml::from_str(&raw)?;
    let mut entries = HashMap::new();
    for package in value
        .get("package")
        .and_then(toml::Value::as_array)
        .context("lockfile package array")?
    {
        let id = package
            .get("id")
            .and_then(toml::Value::as_str)
            .context("lockfile package id")?
            .to_string();
        entries.insert(
            id,
            LockEntrySummary {
                source: package
                    .get("source")
                    .and_then(toml::Value::as_str)
                    .unwrap_or("unknown")
                    .to_string(),
                url: package
                    .get("url")
                    .and_then(toml::Value::as_str)
                    .map(ToOwned::to_owned),
                ref_name: package
                    .get("ref")
                    .and_then(toml::Value::as_str)
                    .map(ToOwned::to_owned),
                commit: package
                    .get("commit")
                    .and_then(toml::Value::as_str)
                    .map(ToOwned::to_owned),
            },
        );
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;
    use crate::cli::{Cli, Command};

    #[test]
    fn parses_update_args() {
        let cli = Cli::try_parse_from([
            "ygg",
            "update",
            "fixture/pkg-local",
            "--profile",
            "dev",
            "--data-dir",
            "/tmp/ygg",
            "--check-only",
        ])
        .unwrap();
        match cli.command {
            Command::Update(args) => {
                assert_eq!(args.package_id.as_deref(), Some("fixture/pkg-local"));
                assert_eq!(args.profile, "dev");
                assert_eq!(args.data_dir, Some(PathBuf::from("/tmp/ygg")));
                assert!(args.check_only);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
