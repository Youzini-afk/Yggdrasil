//! Profile lockfile for reproducible package installations.
//!
//! Lockfile format documented in docs/spec/v1/LOCKFILE_FORMAT.md.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lockfile {
    /// Schema identifier; must be "yggdrasil.lock.v1"
    pub schema: String,

    /// Profile name this lockfile pins
    pub profile: String,

    /// When this lockfile was generated
    pub generated_at: DateTime<Utc>,

    /// Hash of the profile manifest at generation time (detect drift)
    pub manifest_hash: String,

    /// Locked package entries
    #[serde(default)]
    pub package: Vec<LockEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockEntry {
    /// Package id (matches the manifest.id)
    pub id: String,

    /// Resolved version
    pub version: String,

    /// Where this came from
    pub source: LockSource,

    /// Origin URL (for git sources)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Original ref (tag/branch) at lock time
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,

    /// Resolved commit SHA (for git sources)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,

    /// SHA-256 of the package tree at install time
    pub tree_hash: String,

    /// SHA-256 of the canonicalized manifest
    pub manifest_hash: String,

    /// SHA-256 of the referenced static surface bundle artifact when this is a
    /// surface_bundle package. Covers browser JS that is intentionally excluded
    /// from the package tree hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_bundle_hash: Option<String>,

    /// Whether the source was GPG-signed and verified
    pub signed: bool,

    /// GPG fingerprint of signing key (if signed)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_by: Option<String>,

    /// Path in the immutable store
    pub installed_at_store: String,

    /// Manifest path relative to installed_at_store when the package was
    /// installed as part of a larger project tree.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_relative_path: Option<String>,

    /// Capabilities the user granted at install time
    #[serde(default)]
    pub granted_capabilities: Vec<String>,

    /// Network hosts the user granted at install time
    #[serde(default)]
    pub granted_network: Vec<String>,

    /// Secret refs the user granted at install time
    #[serde(default)]
    pub granted_secrets: Vec<String>,

    /// Transitive dep graph (resolved)
    #[serde(default)]
    pub requires: Vec<LockRequirement>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockRequirement {
    pub id: String,
    pub constraint: String,
    pub resolved_to: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockSource {
    Internal,
    Git,
    Local,
}

impl Lockfile {
    pub const SCHEMA: &'static str = "yggdrasil.lock.v1";

    pub fn new(profile: impl Into<String>, manifest_hash: impl Into<String>) -> Self {
        Self {
            schema: Self::SCHEMA.to_string(),
            profile: profile.into(),
            generated_at: Utc::now(),
            manifest_hash: manifest_hash.into(),
            package: Vec::new(),
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.schema != Self::SCHEMA {
            anyhow::bail!("unsupported lockfile schema: {}", self.schema);
        }
        // each package entry must have valid hash format
        for pkg in &self.package {
            if !pkg.tree_hash.starts_with("sha256:") {
                anyhow::bail!(
                    "invalid tree_hash format for {}: must start with sha256:",
                    pkg.id
                );
            }
            if !pkg.manifest_hash.starts_with("sha256:") {
                anyhow::bail!(
                    "invalid manifest_hash format for {}: must start with sha256:",
                    pkg.id
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git_entry() -> LockEntry {
        LockEntry {
            id: "vendor/tool".to_string(),
            version: "1.2.3".to_string(),
            source: LockSource::Git,
            url: Some("https://example.com/vendor/tool.git".to_string()),
            r#ref: Some("v1.2.3".to_string()),
            commit: Some("0123456789abcdef0123456789abcdef01234567".to_string()),
            tree_hash: "sha256:tree".to_string(),
            manifest_hash: "sha256:manifest".to_string(),
            surface_bundle_hash: None,
            signed: true,
            signed_by: Some("0123456789ABCDEF0123456789ABCDEF01234567".to_string()),
            installed_at_store: "/nix/store/ygg/vendor-tool".to_string(),
            manifest_relative_path: None,
            granted_capabilities: vec!["model/live_call".to_string()],
            granted_network: vec!["api.example.com".to_string()],
            granted_secrets: vec!["secret_ref:env:API_KEY".to_string()],
            requires: Vec::new(),
        }
    }

    #[test]
    fn empty_lockfile_toml_round_trip() {
        let lockfile = Lockfile::new("default", "sha256:profile");
        lockfile.validate().expect("valid lockfile");
        let toml = toml::to_string(&lockfile).expect("serialize toml");
        let decoded: Lockfile = toml::from_str(&toml).expect("deserialize toml");
        assert_eq!(decoded.schema, Lockfile::SCHEMA);
        assert_eq!(decoded.profile, "default");
        assert!(decoded.package.is_empty());
        decoded.validate().expect("round-trip lockfile validates");
    }

    #[test]
    fn git_package_toml_round_trip() {
        let mut lockfile = Lockfile::new("default", "sha256:profile");
        lockfile.package.push(git_entry());
        let toml = toml::to_string(&lockfile).expect("serialize toml");
        let decoded: Lockfile = toml::from_str(&toml).expect("deserialize toml");
        assert_eq!(decoded.package.len(), 1);
        assert!(matches!(decoded.package[0].source, LockSource::Git));
        assert_eq!(
            decoded.package[0].url.as_deref(),
            Some("https://example.com/vendor/tool.git")
        );
        decoded.validate().expect("valid git lockfile");
    }

    #[test]
    fn transitive_requires_round_trip() {
        let mut lockfile = Lockfile::new("default", "sha256:profile");
        let mut entry = git_entry();
        entry.requires = vec![LockRequirement {
            id: "official/core".to_string(),
            constraint: ">=1.0.0".to_string(),
            resolved_to: "official/core@1.0.0".to_string(),
        }];
        lockfile.package.push(entry);
        let toml = toml::to_string(&lockfile).expect("serialize toml");
        let decoded: Lockfile = toml::from_str(&toml).expect("deserialize toml");
        assert_eq!(decoded.package[0].requires.len(), 1);
        assert_eq!(decoded.package[0].requires[0].id, "official/core");
        decoded.validate().expect("valid transitive lockfile");
    }

    #[test]
    fn invalid_schema_rejected() {
        let mut lockfile = Lockfile::new("default", "sha256:profile");
        lockfile.schema = "yggdrasil.lock.v2".to_string();
        let err = lockfile.validate().expect_err("schema should be rejected");
        assert!(err.to_string().contains("unsupported lockfile schema"));
    }

    #[test]
    fn invalid_hash_format_rejected() {
        let mut lockfile = Lockfile::new("default", "sha256:profile");
        let mut entry = git_entry();
        entry.tree_hash = "tree".to_string();
        lockfile.package.push(entry);
        let err = lockfile.validate().expect_err("hash should be rejected");
        assert!(err.to_string().contains("invalid tree_hash format"));

        let mut lockfile = Lockfile::new("default", "sha256:profile");
        let mut entry = git_entry();
        entry.manifest_hash = "manifest".to_string();
        lockfile.package.push(entry);
        let err = lockfile
            .validate()
            .expect_err("manifest hash should be rejected");
        assert!(err.to_string().contains("invalid manifest_hash format"));
    }
}
