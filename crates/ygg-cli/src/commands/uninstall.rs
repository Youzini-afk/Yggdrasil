use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use serde_json::json;

use crate::commands::install::{invoke_install_lab, load_install_runtime};
use crate::install::default_data_dir;

#[derive(Args, Debug)]
pub struct UninstallArgs {
    pub package_id: String,

    #[arg(short, long, default_value = "default")]
    pub profile: String,

    #[arg(long)]
    pub data_dir: Option<PathBuf>,
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
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
