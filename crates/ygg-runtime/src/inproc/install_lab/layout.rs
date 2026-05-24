use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use uuid::Uuid;
use ygg_core::paths;

pub(super) fn ensure_layout(data_dir_override: Option<&str>) -> Result<()> {
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

pub(super) fn store_dir(data_dir_override: Option<&str>) -> Result<PathBuf> {
    if let Some(dir) = data_dir_override {
        return Ok(PathBuf::from(dir).join("store"));
    }
    paths::store_dir()
}

pub(super) fn profiles_dir(data_dir_override: Option<&str>) -> Result<PathBuf> {
    if let Some(dir) = data_dir_override {
        return Ok(PathBuf::from(dir).join("profiles"));
    }
    paths::profiles_dir()
}

pub(super) fn lockfile_path(profile: &str, data_dir_override: Option<&str>) -> Result<PathBuf> {
    if let Some(dir) = data_dir_override {
        return Ok(PathBuf::from(dir)
            .join("profiles")
            .join(format!("{profile}.lock.toml")));
    }
    paths::lockfile_path(profile)
}

pub(super) fn profile_path(profile: &str, data_dir_override: Option<&str>) -> Result<PathBuf> {
    if let Some(dir) = data_dir_override {
        return Ok(PathBuf::from(dir)
            .join("profiles")
            .join(format!("{profile}.yaml")));
    }
    paths::profile_path(profile)
}

pub(super) fn store_path_for_hash(
    tree_hash: &str,
    data_dir_override: Option<&str>,
) -> Result<PathBuf> {
    if let Some(dir) = data_dir_override {
        return Ok(PathBuf::from(dir)
            .join("store")
            .join(tree_hash.replace(':', "-")));
    }
    paths::store_path_for_hash(tree_hash)
}

pub(super) fn default_profile() -> String {
    "default".to_string()
}

pub(super) fn default_head_ref() -> String {
    "HEAD".to_string()
}

pub(super) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!("tmp-{}", Uuid::new_v4()));
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
