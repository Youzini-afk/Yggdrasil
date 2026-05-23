use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use serde_json::json;
use ygg_core::project::{ProjectDescriptor, ProjectId};

use crate::commands::install::{invoke_install_lab, load_install_runtime};
use crate::install::default_data_dir;

#[derive(Args, Debug)]
pub struct UninstallArgs {
    pub package_id: String,

    #[arg(short, long, default_value = "default")]
    pub profile: String,

    #[arg(long)]
    pub data_dir: Option<PathBuf>,

    /// Keep project data by archiving it when uninstalling a project
    #[arg(long, conflicts_with = "delete_data")]
    pub keep_data: bool,

    /// Delete project data immediately when uninstalling a project
    #[arg(long, conflicts_with = "keep_data")]
    pub delete_data: bool,
}

pub async fn run(args: UninstallArgs) -> Result<()> {
    let data_dir = args.data_dir.unwrap_or_else(default_data_dir);
    let runtime = load_install_runtime().await?;
    let result = invoke_install_lab(
        &runtime,
        "official/install-lab/uninstall",
        json!({
            "package_id": args.package_id,
            "profile": args.profile,
            "data_dir": data_dir.display().to_string(),
        }),
    )
    .await?
    .output;

    println!(
        "Removed from profile: {}",
        result["removed_from_profile"].as_bool().unwrap_or(false)
    );
    if let Some(store) = result["store_path_orphaned"].as_str() {
        println!("Store path orphaned: {store}");
    }
    maybe_archive_project(
        &args.package_id,
        &data_dir,
        args.keep_data,
        args.delete_data,
    )?;
    Ok(())
}

enum ArchiveAction {
    Keep,
    Delete,
}

fn maybe_archive_project(
    package_id: &str,
    data_dir: &Path,
    keep_data: bool,
    delete_data: bool,
) -> Result<()> {
    let Some(descriptor) = get_project_descriptor(package_id, data_dir)? else {
        return Ok(());
    };
    let action = if keep_data {
        ArchiveAction::Keep
    } else if delete_data {
        ArchiveAction::Delete
    } else if !std::io::stdin().is_terminal() {
        eprintln!(
            "Note: project '{}' has data; keeping (use --delete-data to remove).",
            package_id
        );
        ArchiveAction::Keep
    } else {
        let session_count = count_project_sessions(&descriptor.project.id, data_dir)?;
        let secret_count = count_project_secrets(&descriptor.project.id, data_dir)?;
        println!();
        println!("Project '{}' is installed:", descriptor.project.title);
        println!("  - {} session(s)", session_count);
        println!("  - {} project-scoped secret(s)", secret_count);
        println!();
        println!("What about the project data?");
        let choice = dialoguer::Select::new()
            .items(&[
                "Keep data (archive to ~/.yggdrasil/projects/.archived/, 30-day cleanup)",
                "Delete data immediately",
                "Cancel uninstall",
            ])
            .default(0)
            .interact()?;
        match choice {
            0 => ArchiveAction::Keep,
            1 => ArchiveAction::Delete,
            _ => anyhow::bail!("uninstall cancelled"),
        }
    };

    match action {
        ArchiveAction::Keep => archive_project(&descriptor.project.id, data_dir)?,
        ArchiveAction::Delete => {
            let dir = project_dir(data_dir, &descriptor.project.id);
            if dir.exists() {
                std::fs::remove_dir_all(dir)?;
            }
        }
    }
    Ok(())
}

fn get_project_descriptor(package_id: &str, data_dir: &Path) -> Result<Option<ProjectDescriptor>> {
    let projects = data_dir.join("projects");
    if !projects.is_dir() {
        return Ok(None);
    }
    for entry in std::fs::read_dir(projects)? {
        let path = entry?.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        let descriptor_path = path.join("project.yaml");
        if !descriptor_path.is_file() {
            continue;
        }
        let descriptor: ProjectDescriptor = serde_yaml::from_str(&std::fs::read_to_string(
            &descriptor_path,
        )?)
        .with_context(|| format!("invalid project descriptor {}", descriptor_path.display()))?;
        if descriptor.project.id.as_str() == package_id
            || descriptor
                .project
                .packages
                .iter()
                .any(|pkg| pkg == package_id)
        {
            return Ok(Some(descriptor));
        }
    }
    Ok(None)
}

fn count_project_sessions(id: &ProjectId, data_dir: &Path) -> Result<usize> {
    count_entries(project_dir(data_dir, id).join("sessions"))
}

fn count_project_secrets(id: &ProjectId, data_dir: &Path) -> Result<usize> {
    Ok(usize::from(
        project_dir(data_dir, id).join("secrets.dat").is_file(),
    ))
}

fn count_entries(path: PathBuf) -> Result<usize> {
    if !path.is_dir() {
        return Ok(0);
    }
    Ok(std::fs::read_dir(path)?.count())
}

pub(crate) fn archive_project(id: &ProjectId, data_dir: &Path) -> Result<()> {
    let from = project_dir(data_dir, id);
    if !from.exists() {
        return Ok(());
    }
    let archived = data_dir.join("projects/.archived");
    std::fs::create_dir_all(&archived)?;
    let to = archived.join(id.as_str());
    if to.exists() {
        std::fs::remove_dir_all(&to)?;
    }
    std::fs::rename(&from, &to).or_else(|_| -> std::io::Result<()> {
        copy_dir_recursive(&from, &to)?;
        std::fs::remove_dir_all(&from)?;
        Ok(())
    })?;
    Ok(())
}

fn project_dir(data_dir: &Path, id: &ProjectId) -> PathBuf {
    data_dir.join("projects").join(id.as_str())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        let meta = std::fs::symlink_metadata(&from)?;
        if meta.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if meta.is_file() {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;
    use crate::cli::{Cli, Command};

    #[test]
    fn parses_uninstall_args() {
        let cli = Cli::try_parse_from([
            "ygg",
            "uninstall",
            "fixture/pkg-local",
            "--profile",
            "dev",
            "--data-dir",
            "/tmp/ygg",
        ])
        .unwrap();
        match cli.command {
            Command::Uninstall(args) => {
                assert_eq!(args.package_id, "fixture/pkg-local");
                assert_eq!(args.profile, "dev");
                assert_eq!(args.data_dir, Some(PathBuf::from("/tmp/ygg")));
                assert!(!args.keep_data);
                assert!(!args.delete_data);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
