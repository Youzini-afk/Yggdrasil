use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::executor::{compute_manifest_hash, compute_tree_hash};
use super::source::manifest_path_in;
use super::types::PlannedPackage;

pub(super) fn copy_dir_atomic(src: &Path, staging: &Path, dest: &Path) -> Result<()> {
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

pub(super) fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
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

pub(super) async fn verify_installed_hashes(
    pkg: &PlannedPackage,
    store_path: &Path,
) -> Result<()> {
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
