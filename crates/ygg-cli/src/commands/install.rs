use std::collections::BTreeMap;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Args;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use ygg_core::project::{
    ExternalProjectData, ProjectDescriptor, ProjectId, ProjectInner, ProjectType, SecretPolicy,
};
use ygg_runtime::{CapabilityInvocationRequest, InMemoryEventStore, Runtime, RuntimeConfig};

use crate::commands::manifest::read_manifest;
use crate::install::consent::{approve_all, prompt_for_consent};
use crate::install::default_data_dir;
use crate::install::url_parser::{parse_install_url, InstallSource};

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

    /// For external projects: generate an adapter package wrapping the project
    #[arg(long, conflicts_with = "workspace_only")]
    pub wrap_as_adapter: bool,

    /// For external projects: open as workspace without wrapping
    #[arg(long, conflicts_with = "wrap_as_adapter")]
    pub workspace_only: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
}

pub async fn run(args: InstallArgs) -> Result<()> {
    let install_url = parse_install_url(&args.source)?;
    let data_dir = args.data_dir.clone().unwrap_or_else(default_data_dir);
    let lockfile_path = lockfile_path(&data_dir, &args.profile);
    let existing_lockfile = if lockfile_path.exists() {
        Some(std::fs::read_to_string(&lockfile_path)?)
    } else {
        None
    };

    let runtime = load_install_runtime().await?;
    let mut resolved = invoke_install_lab(
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
        .get_mut("plan")
        .map(Value::take)
        .context("install-lab resolve_plan response missing plan")?;
    print_conformance_warnings(&plan, args.strict);

    let detected = invoke_install_lab(
        &runtime,
        "official/install-lab/detect_kind",
        json!({
            "path": install_url.url_for_resolver(),
            "root_ref": install_url.ref_or_default(),
        }),
    )
    .await?
    .output;

    let project_descriptor = match detected.get("kind").and_then(Value::as_str) {
        Some("native") | Some("declared_external") => None,
        Some("external") => {
            let staging_dir = staging_dir_for_external(&install_url.source, &data_dir);
            Some(serde_json::to_value(external_project_wizard(
                &args,
                &install_url.url_for_resolver(),
                &staging_dir,
            )?)?)
        }
        _ => None,
    };

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

    let result = invoke_install_lab(&runtime, "official/install-lab/execute_plan", {
        let mut input = serde_json::Map::with_capacity(5);
        input.insert("plan".to_string(), plan);
        input.insert("consent".to_string(), consent);
        input.insert("profile".to_string(), Value::String(args.profile));
        input.insert(
            "data_dir".to_string(),
            Value::String(data_dir.display().to_string()),
        );
        if let Some(descriptor) = project_descriptor {
            input.insert("project_descriptor".to_string(), descriptor);
        }
        Value::Object(input)
    })
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

    if let Some(project_id) = result
        .pointer("/project/project_id")
        .and_then(Value::as_str)
    {
        println!("Project registered: {project_id}");
    }

    Ok(())
}

fn external_project_wizard(
    args: &InstallArgs,
    source_url: &str,
    staging_dir: &Path,
) -> Result<ProjectDescriptor> {
    if args.wrap_as_adapter {
        return create_external_wrapped_descriptor(source_url, staging_dir);
    }
    if args.workspace_only {
        return create_external_workspace_descriptor(source_url, staging_dir);
    }

    if !std::io::stdin().is_terminal() {
        eprintln!("Note: external project, no TTY available. Using workspace mode.");
        eprintln!("      (Use --wrap-as-adapter to wrap with adapter package.)");
        return create_external_workspace_descriptor(source_url, staging_dir);
    }

    println!();
    println!(
        "The repository at {} does not declare itself as a Yggdrasil project.",
        source_url
    );
    println!("How do you want to use it?");
    println!();
    println!("  [1] Wrap with adapter (creates a Yggdrasil package that wraps the tool)");
    println!("  [2] Open as workspace (run in agent-driven workspace; no wrapping)");
    println!("  [3] Cancel");
    println!();

    let choice = dialoguer::Select::new()
        .items(&["Wrap with adapter", "Open as workspace", "Cancel"])
        .default(1)
        .interact()?;

    match choice {
        0 => create_external_wrapped_descriptor(source_url, staging_dir),
        1 => create_external_workspace_descriptor(source_url, staging_dir),
        _ => anyhow::bail!("install cancelled by user"),
    }
}

fn create_external_workspace_descriptor(
    source_url: &str,
    staging_dir: &Path,
) -> Result<ProjectDescriptor> {
    let id = derive_project_id_from_url(source_url)?;
    let title = derive_title_from_url(source_url);
    Ok(ProjectDescriptor {
        schema_version: 1,
        project: ProjectInner {
            id,
            title,
            description: format!("External project from {source_url}"),
            project_type: ProjectType::ExternalWorkspace,
            icon: None,
            entry_surface_id: Some("official/workspace-lab/workspace_view".to_string()),
            packages: vec!["packages/official/workspace-lab/manifest.yaml".to_string()],
            optional_packages: vec![],
            required_surfaces: vec![],
            required_capabilities: vec![],
            secret_policy: SecretPolicy::default(),
            external: Some(ExternalProjectData {
                source: source_url.to_string(),
                source_ref: None,
                adapter_manifest: None,
                workspace_root: Some(staging_dir.display().to_string()),
            }),
            metadata: BTreeMap::new(),
        },
    })
}

fn create_external_wrapped_descriptor(
    source_url: &str,
    staging_dir: &Path,
) -> Result<ProjectDescriptor> {
    let id = derive_project_id_from_url(source_url)?;
    let title = derive_title_from_url(source_url);
    let adapter_manifest_path = format!("{}/adapter/manifest.yaml", staging_dir.display());
    Ok(ProjectDescriptor {
        schema_version: 1,
        project: ProjectInner {
            id,
            title,
            description: format!("Adapter-wrapped external project from {source_url}"),
            project_type: ProjectType::ExternalWrapped,
            icon: None,
            entry_surface_id: None,
            packages: vec![adapter_manifest_path.clone()],
            optional_packages: vec![],
            required_surfaces: vec![],
            required_capabilities: vec![],
            secret_policy: SecretPolicy::default(),
            external: Some(ExternalProjectData {
                source: source_url.to_string(),
                source_ref: None,
                adapter_manifest: Some(adapter_manifest_path),
                workspace_root: Some(staging_dir.display().to_string()),
            }),
            metadata: BTreeMap::new(),
        },
    })
}

fn derive_project_id_from_url(url: &str) -> Result<ProjectId> {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = hasher.finalize();
    let short_hash = hash[..4]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    let slug = if let Ok(parsed) = url::Url::parse(url) {
        parsed
            .path()
            .trim_start_matches('/')
            .trim_end_matches(".git")
            .replace('/', "__")
    } else {
        Path::new(url)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string()
    };
    let safe_slug: String = slug
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let safe_slug = safe_slug.trim_matches('_');
    let safe_slug = if safe_slug.is_empty() {
        "project"
    } else {
        safe_slug
    };
    let safe_slug = if safe_slug.len() > 64 {
        &safe_slug[..64]
    } else {
        safe_slug
    };

    ProjectId::new(format!("{safe_slug}__{short_hash}"))
}

fn derive_title_from_url(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        let path = parsed
            .path()
            .trim_start_matches('/')
            .trim_end_matches(".git");
        if let Some(repo) = path.rsplit('/').next() {
            return repo.to_string();
        }
    }
    Path::new(url)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Project")
        .to_string()
}

fn staging_dir_for_external(source: &InstallSource, data_dir: &Path) -> PathBuf {
    match source {
        InstallSource::Local { path } => path.clone(),
        InstallSource::Git { .. } => data_dir.join("workspaces").join("external"),
    }
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
