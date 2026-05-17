use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use ygg_core::PackageManifest;

pub(crate) async fn read_manifest(path: PathBuf) -> Result<PackageManifest> {
    let raw = fs::read_to_string(&path)?;
    let manifest = match path.extension().and_then(|ext| ext.to_str()) {
        Some("yaml") | Some("yml") => serde_yaml::from_str(&raw)?,
        _ => serde_json::from_str(&raw)?,
    };
    Ok(manifest)
}

pub(crate) async fn validate_manifest(path: PathBuf) -> Result<()> {
    let manifest = read_manifest(path).await?;
    manifest.validate_basic()?;
    println!("valid manifest: {}@{}", manifest.id, manifest.version);
    Ok(())
}
