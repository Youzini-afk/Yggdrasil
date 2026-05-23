use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Args;
use serde_json::{json, Value};
use ygg_runtime::{CapabilityInvocationRequest, InMemoryEventStore, Runtime, RuntimeConfig};

use crate::commands::manifest::read_manifest;
use crate::install::consent::{approve_all, prompt_for_consent};
use crate::install::default_data_dir;
use crate::install::url_parser::parse_install_url;

const INSTALL_PACKAGE_ID: &str = "official/install-lab";
const OFFICIAL_MANIFESTS: [&str; 3] = [
    "packages/official/git-tools-lab/manifest.yaml",
    "packages/official/integrity-lab/manifest.yaml",
    "packages/official/install-lab/manifest.yaml",
];

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// GitHub URL, full HTTPS URL, or local path
    pub source: String,

    /// Profile to install into (default: "default")
    #[arg(short, long, default_value = "default")]
    pub profile: String,

    /// Data directory (default: ~/.yggdrasil or $YGG_DATA_DIR)
    #[arg(long)]
    pub data_dir: Option<PathBuf>,

    /// Require GPG-signed git tags (off by default — matches cargo/npm baseline)
    #[arg(long)]
    pub require_signed: bool,

    /// Treat conformance failures as errors instead of warnings
    #[arg(long)]
    pub strict: bool,

    /// Skip interactive consent prompt (CI mode)
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Output format
    #[arg(long, default_value = "human")]
    pub format: OutputFormat,
}

#[derive(Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
}

pub async fn run(args: InstallArgs) -> Result<()> {
    let install_url = parse_install_url(&args.source)?;
    let data_dir = args.data_dir.unwrap_or_else(default_data_dir);
    let lockfile_path = lockfile_path(&data_dir, &args.profile);
    let existing_lockfile = if lockfile_path.exists() {
        Some(std::fs::read_to_string(&lockfile_path)?)
    } else {
        None
    };

    let runtime = load_install_runtime().await?;
    let resolved = invoke_install_lab(
        &runtime,
        "official/install-lab/resolve_plan",
        json!({
            "root_url": install_url.url_for_resolver(),
            "root_ref": install_url.ref_or_default(),
            "lockfile": existing_lockfile,
            "require_signed": args.require_signed,
            "strict_conformance": args.strict,
        }),
    )
    .await?;
    let plan = resolved
        .output
        .get("plan")
        .cloned()
        .context("install-lab resolve_plan response missing plan")?;
    print_conformance_warnings(&plan, args.strict);

    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&plan)?),
        OutputFormat::Human => {
            print_plan_human(&plan);
        }
    }

    let consent = if args.yes {
        approve_all(&plan)
    } else {
        prompt_for_consent(&plan, existing_lockfile.as_deref())?
    };

    let result = invoke_install_lab(
        &runtime,
        "official/install-lab/execute_plan",
        json!({
            "plan": plan,
            "consent": consent,
            "profile": args.profile,
            "data_dir": data_dir.display().to_string(),
        }),
    )
    .await?
    .output;

    println!(
        "Installed {} packages.",
        result["installed"].as_array().map(Vec::len).unwrap_or(0)
    );
    println!(
        "Lockfile: {}",
        result["lockfile_path"].as_str().unwrap_or("(unknown)")
    );
    println!(
        "Profile: {}",
        result["profile_path"].as_str().unwrap_or("(unknown)")
    );

    Ok(())
}

pub(crate) async fn load_install_runtime() -> Result<Runtime<InMemoryEventStore>> {
    let store = Arc::new(InMemoryEventStore::default());
    let runtime = Runtime::new(store, RuntimeConfig::default());
    for manifest in OFFICIAL_MANIFESTS {
        runtime
            .load_package(read_manifest(workspace_path(manifest)).await?)
            .await?;
    }
    Ok(runtime)
}

fn workspace_path(relative: &str) -> PathBuf {
    let from_crate = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    if from_crate.exists() {
        from_crate
    } else {
        PathBuf::from(relative)
    }
}

pub(crate) async fn invoke_install_lab(
    runtime: &Runtime<InMemoryEventStore>,
    capability_id: &str,
    input: Value,
) -> Result<ygg_runtime::CapabilityInvocationResult> {
    runtime
        .invoke_capability(CapabilityInvocationRequest {
            handle: None,
            capability_id: Some(capability_id.to_string()),
            caller_package_id: None,
            provider_package_id: Some(INSTALL_PACKAGE_ID.to_string()),
            version: None,
            session_id: None,
            input,
        })
        .await
        .map_err(Into::into)
}

pub(crate) fn lockfile_path(data_dir: &Path, profile: &str) -> PathBuf {
    data_dir
        .join("profiles")
        .join(format!("{profile}.lock.toml"))
}

pub(crate) fn print_plan_human(plan: &Value) {
    println!(
        "Resolved {} package(s):",
        plan.pointer("/packages")
            .and_then(Value::as_array)
            .map(|a| a.len())
            .unwrap_or(0)
    );
    if let Some(packages) = plan.pointer("/packages").and_then(Value::as_array) {
        for pkg in packages {
            let id = pkg.pointer("/id").and_then(Value::as_str).unwrap_or("?");
            let version = pkg
                .pointer("/version")
                .and_then(Value::as_str)
                .unwrap_or("");
            let signed = pkg
                .pointer("/signed")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let signed_marker = if signed { " (signed)" } else { "" };
            println!("  {id} @ {version}{signed_marker}");
        }
    }
}

fn print_conformance_warnings(plan: &Value, strict: bool) {
    if strict {
        return;
    }
    if let Some(packages) = plan.pointer("/packages").and_then(Value::as_array) {
        for pkg in packages {
            let Some(report) = pkg.pointer("/conformance") else {
                continue;
            };
            let summary = report.pointer("/summary");
            let failed = summary
                .and_then(|s| s.pointer("/failed_blocking"))
                .and_then(Value::as_u64)
                .or_else(|| {
                    summary
                        .and_then(|s| s.pointer("/failed"))
                        .and_then(Value::as_u64)
                })
                .unwrap_or(0);
            if failed > 0 {
                eprintln!(
                    "⚠ {} has {} conformance warning(s) (use --strict to block)",
                    pkg.pointer("/id").and_then(Value::as_str).unwrap_or("?"),
                    failed
                );
            }
        }
    }
}

pub(crate) fn join_or_none(values: Option<&Vec<Value>>) -> String {
    let rendered = values
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    if rendered.is_empty() {
        "(none)".to_string()
    } else {
        rendered.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;
    use crate::cli::{Cli, Command};

    #[test]
    fn parses_install_args() {
        let cli = Cli::try_parse_from([
            "ygg",
            "install",
            "github.com/user/repo#v1.0",
            "--profile",
            "dev",
            "--data-dir",
            "/tmp/ygg",
            "--require-signed",
            "--strict",
            "--yes",
            "--format",
            "json",
        ])
        .unwrap();
        match cli.command {
            Command::Install(args) => {
                assert_eq!(args.source, "github.com/user/repo#v1.0");
                assert_eq!(args.profile, "dev");
                assert_eq!(args.data_dir, Some(PathBuf::from("/tmp/ygg")));
                assert!(args.require_signed);
                assert!(args.strict);
                assert!(args.yes);
                assert_eq!(args.format, OutputFormat::Json);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
