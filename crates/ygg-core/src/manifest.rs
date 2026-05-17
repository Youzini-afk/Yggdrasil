use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ids::{CapabilityId, ExtensionPointId, PackageId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub schema_version: u16,
    pub id: PackageId,
    pub version: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    pub entry: PackageEntry,
    #[serde(default)]
    pub provides: Vec<CapabilityDescriptor>,
    #[serde(default)]
    pub consumes: Vec<CapabilityRequirement>,
    #[serde(default)]
    pub contributes: PackageContributions,
    #[serde(default)]
    pub permissions: PermissionSet,
    #[serde(default)]
    pub sandbox_policy: SandboxPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PackageEntry {
    RustInproc { crate_ref: String, symbol: String, abi_version: u16 },
    Subprocess { command: Vec<String>, transport: SubprocessTransport },
    Wasm { module: String, abi_version: u16, memory_limit_mb: u64 },
    Remote { endpoint: String, auth: RemoteAuth },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubprocessTransport {
    JsonRpcStdio,
    JsonRpcTcp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteAuth {
    pub scheme: String,
    #[serde(default)]
    pub config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    pub id: CapabilityId,
    pub version: String,
    #[serde(default)]
    pub input_schema: Value,
    #[serde(default)]
    pub output_schema: Value,
    #[serde(default)]
    pub streaming: bool,
    #[serde(default)]
    pub side_effects: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequirement {
    pub id: CapabilityId,
    pub version: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageContributions {
    #[serde(default)]
    pub schemas: Vec<SchemaContribution>,
    #[serde(default)]
    pub hooks: Vec<HookSubscription>,
    #[serde(default)]
    pub extension_points: Vec<ExtensionPointDescriptor>,
    #[serde(default)]
    pub surfaces: Vec<SurfaceContribution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceContribution {
    pub id: String,
    #[serde(default = "default_surface_version")]
    pub version: String,
    pub slot: SurfaceSlot,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub capability_id: Option<CapabilityId>,
    #[serde(default)]
    pub activation: SurfaceActivation,
    #[serde(default)]
    pub required_permissions: Vec<SurfacePermissionRequirement>,
    #[serde(default)]
    pub approval_policy: Option<SurfaceApprovalPolicy>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SurfaceActivation {
    #[serde(default)]
    pub launch_capability_id: Option<CapabilityId>,
    #[serde(default)]
    pub session_template: Value,
    #[serde(default)]
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfacePermissionRequirement {
    pub permission: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub risk: SurfaceRisk,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceRisk {
    #[default]
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceApprovalPolicy {
    None,
    UserApproval,
    ForkThenApprove,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceSlot {
    ExperienceEntry,
    HomeCard,
    PlayRenderer,
    ForgePanel,
    AssetEditor,
    AssistantAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaContribution {
    pub id: String,
    pub schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionPointDescriptor {
    pub id: ExtensionPointId,
    pub version: String,
    #[serde(default)]
    pub payload_schema: Value,
    pub timing: HookTiming,
    pub modifiable: bool,
    pub short_circuit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookSubscription {
    pub extension_point: ExtensionPointId,
    pub handler: String,
    pub timing: HookTiming,
    #[serde(default)]
    pub precedence: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HookTiming {
    Sync,
    Async,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionSet {
    #[serde(default)]
    pub events: EventPermissions,
    #[serde(default)]
    pub capabilities: CapabilityPermissions,
    #[serde(default)]
    pub packages: PackagePermissions,
    #[serde(default)]
    pub assets: AssetPermissions,
    #[serde(default)]
    pub network: NetworkPermissions,
    #[serde(default)]
    pub filesystem: FilesystemPermissions,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventPermissions {
    #[serde(default)]
    pub read: bool,
    #[serde(default)]
    pub append: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityPermissions {
    #[serde(default)]
    pub invoke: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackagePermissions {
    #[serde(default)]
    pub call: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetPermissions {
    #[serde(default)]
    pub read: bool,
    #[serde(default)]
    pub write: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkPermissions {
    #[serde(default)]
    pub hosts: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilesystemPermissions {
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPolicy {
    #[serde(default = "default_cpu_quota_ms")]
    pub cpu_quota_ms_per_invoke: u64,
    #[serde(default = "default_memory_mb")]
    pub memory_mb: u64,
    #[serde(default = "default_wall_clock_ms")]
    pub wall_clock_ms: u64,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self {
            cpu_quota_ms_per_invoke: default_cpu_quota_ms(),
            memory_mb: default_memory_mb(),
            wall_clock_ms: default_wall_clock_ms(),
        }
    }
}

fn default_cpu_quota_ms() -> u64 {
    5_000
}

fn default_memory_mb() -> u64 {
    128
}

fn default_wall_clock_ms() -> u64 {
    30_000
}

impl PackageManifest {
    pub fn validate_basic(&self) -> Result<(), ManifestError> {
        validate_package_id(&self.id)?;
        validate_semver_like(&self.version)?;
        for capability in &self.provides {
            validate_namespaced_id(&capability.id)?;
            validate_semver_like(&capability.version)?;
            validate_schema_shape(&capability.input_schema)?;
            validate_schema_shape(&capability.output_schema)?;
        }
        for requirement in &self.consumes {
            validate_namespaced_id(&requirement.id)?;
            if requirement.version.trim().is_empty() {
                return Err(ManifestError::InvalidVersion(requirement.version.clone()));
            }
        }
        for schema in &self.contributes.schemas {
            validate_namespaced_id(&schema.id)?;
            validate_schema_shape(&schema.schema)?;
        }
        for surface in &self.contributes.surfaces {
            validate_namespaced_id(&surface.id)?;
            if surface.title.trim().is_empty() {
                return Err(ManifestError::InvalidSurface(surface.id.clone()));
            }
            validate_semver_like(&surface.version)?;
            if let Some(capability_id) = &surface.capability_id {
                validate_namespaced_id(capability_id)?;
            }
            if let Some(capability_id) = &surface.activation.launch_capability_id {
                validate_namespaced_id(capability_id)?;
            }
            validate_schema_shape(&surface.activation.input_schema)?;
            for requirement in &surface.required_permissions {
                if requirement.permission.trim().is_empty() {
                    return Err(ManifestError::InvalidSurface(surface.id.clone()));
                }
            }
        }
        Ok(())
    }
}

fn default_surface_version() -> String {
    "0.1.0".to_string()
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ManifestError {
    #[error("invalid package id: {0}")]
    InvalidPackageId(String),
    #[error("invalid namespaced id: {0}")]
    InvalidNamespacedId(String),
    #[error("invalid semver-like version: {0}")]
    InvalidVersion(String),
    #[error("invalid schema shape: {0}")]
    InvalidSchema(String),
    #[error("invalid surface contribution: {0}")]
    InvalidSurface(String),
}

fn validate_package_id(id: &str) -> Result<(), ManifestError> {
    validate_namespaced_id(id).map_err(|_| ManifestError::InvalidPackageId(id.to_string()))
}

fn validate_namespaced_id(id: &str) -> Result<(), ManifestError> {
    let parts: Vec<&str> = id.split('/').collect();
    if parts.len() < 2 || parts.iter().any(|part| part.is_empty()) {
        return Err(ManifestError::InvalidNamespacedId(id.to_string()));
    }
    Ok(())
}

fn validate_semver_like(version: &str) -> Result<(), ManifestError> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 || parts.iter().any(|part| part.parse::<u64>().is_err()) {
        return Err(ManifestError::InvalidVersion(version.to_string()));
    }
    Ok(())
}

fn validate_schema_shape(schema: &Value) -> Result<(), ManifestError> {
    if schema.is_null() || schema.is_object() {
        Ok(())
    } else {
        Err(ManifestError::InvalidSchema(schema.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_manifest_identity_and_capabilities() {
        let manifest = PackageManifest {
            schema_version: 1,
            id: "org/example".to_string(),
            version: "0.1.0".to_string(),
            display_name: None,
            description: None,
            author: None,
            license: None,
            entry: PackageEntry::RustInproc {
                crate_ref: "org-example".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            },
            provides: vec![CapabilityDescriptor {
                id: "org/example/echo".to_string(),
                version: "0.1.0".to_string(),
                input_schema: Value::Null,
                output_schema: Value::Null,
                streaming: false,
                side_effects: Vec::new(),
                description: None,
            }],
            consumes: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet::default(),
            sandbox_policy: SandboxPolicy::default(),
        };

        assert_eq!(manifest.validate_basic(), Ok(()));
    }

    #[test]
    fn rejects_bad_package_id() {
        let err = validate_package_id("bad").expect_err("bad id rejected");
        assert_eq!(err, ManifestError::InvalidPackageId("bad".to_string()));
    }
}
