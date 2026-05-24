use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::ids::{CapabilityId, ExtensionPointId, PackageId};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    pub entry: EntryDescriptor,
    #[serde(default)]
    pub provides: Vec<CapabilityDescriptor>,
    #[serde(default)]
    pub consumes: Vec<CapabilityRequirement>,
    /// First-class package dependency declarations.
    /// Distinct from `consumes` (which declares capability requirements).
    /// Empty by default for backward compat.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<PackageDependency>,
    #[serde(default)]
    pub contributes: PackageContributions,
    #[serde(default)]
    pub permissions: PermissionSet,
    #[serde(default)]
    pub sandbox_policy: SandboxPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntryDescriptor {
    #[serde(flatten)]
    pub kind: PackageEntry,
    /// Path A (`v1`) enforces the Yggdrasil package contract. Path B (`none`)
    /// is a self-contained app hosted by the kernel without manifest contract
    /// enforcement.
    #[serde(default)]
    pub contract: ContractMode,
}

impl std::ops::Deref for EntryDescriptor {
    type Target = PackageEntry;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl std::ops::DerefMut for EntryDescriptor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kind
    }
}

impl EntryDescriptor {
    pub fn v1(kind: PackageEntry) -> Self {
        Self {
            kind,
            contract: ContractMode::V1,
        }
    }

    pub fn contract_none(kind: PackageEntry) -> Self {
        Self {
            kind,
            contract: ContractMode::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContractMode {
    /// Path A: full v1 contract enforcement (default).
    V1,
    /// Path B: self-contained app — kernel hosts process, no contract enforcement.
    None,
}

impl std::fmt::Display for ContractMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1 => f.write_str("v1"),
            Self::None => f.write_str("none"),
        }
    }
}

impl Default for ContractMode {
    fn default() -> Self {
        Self::V1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PackageEntry {
    RustInproc {
        crate_ref: String,
        symbol: String,
        abi_version: u16,
    },
    Subprocess {
        command: Vec<String>,
        #[serde(default)]
        transport: SubprocessTransport,
    },
    Wasm {
        module: String,
        abi_version: u16,
        memory_limit_mb: u64,
    },
    Remote {
        endpoint: String,
        auth: RemoteAuth,
    },
    /// Static surface bundle package. This entry is not executed by the host;
    /// it contributes surfaces and optional static assets only.
    SurfaceBundle {
        bundle: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubprocessTransport {
    JsonRpcStdio,
    JsonRpcTcp,
}

impl Default for SubprocessTransport {
    fn default() -> Self {
        Self::JsonRpcStdio
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RemoteAuth {
    pub scheme: String,
    #[serde(default)]
    pub config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CapabilityRequirement {
    pub id: CapabilityId,
    pub version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PackageDependency {
    /// Logical package id (must match the dependency's manifest.id once resolved)
    pub id: PackageId,

    /// Where the package comes from
    pub source: DependencySource,

    /// Semver constraint (e.g., ">=1.0.0", "^2.1", "=1.2.3")
    /// Empty string means "any version"
    #[serde(default)]
    pub version: String,

    /// Optional: GPG public key fingerprints required to sign this dep
    /// If empty, no signature verification required
    /// If set, the dep's GPG-signed tag must be signed by one of these
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub minimum_signed_by: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DependencySource {
    /// Built-in to Yggdrasil (no fetch needed; resolves at host runtime)
    Internal,

    /// Fetched from a git remote
    Git {
        url: String,
        /// Tag, branch, or commit ref
        #[serde(default)]
        r#ref: String,
    },

    /// Local filesystem path (for development)
    Local { path: String },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    /// Additional exact capability ids this surface bundle may invoke through
    /// the web surface bridge. This is a typed declaration; bridge policy must
    /// not infer invoke authority from opaque metadata.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_capability_ids: Vec<CapabilityId>,
    #[serde(default)]
    pub activation: SurfaceActivation,
    #[serde(default)]
    pub required_permissions: Vec<SurfacePermissionRequirement>,
    #[serde(default)]
    pub approval_policy: Option<SurfaceApprovalPolicy>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct SurfaceActivation {
    #[serde(default)]
    pub launch_capability_id: Option<CapabilityId>,
    #[serde(default)]
    pub session_template: Value,
    #[serde(default)]
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SurfacePermissionRequirement {
    pub permission: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub risk: SurfaceRisk,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceRisk {
    #[default]
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceApprovalPolicy {
    None,
    UserApproval,
    ForkThenApprove,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceSlot {
    ExperienceEntry,
    HomeCard,
    PlayRenderer,
    ForgePanel,
    AssetEditor,
    AssistantAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SchemaContribution {
    pub id: String,
    pub schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtensionPointDescriptor {
    pub id: ExtensionPointId,
    pub version: String,
    #[serde(default)]
    pub payload_schema: Value,
    pub timing: HookTiming,
    pub modifiable: bool,
    pub short_circuit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HookSubscription {
    pub extension_point: ExtensionPointId,
    pub handler: String,
    pub timing: HookTiming,
    #[serde(default)]
    pub precedence: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HookTiming {
    Sync,
    Async,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
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
    /// Declared secret references this package may use in
    /// `kernel.v1.outbound.execute` calls. Each entry must be a valid
    /// env-backed secret reference (e.g. `secret_ref:env:OPENAI_API_KEY`,
    /// `secretRef:env:MY_KEY`, `secret-ref:env:NAME`, `host:env:NAME`).
    ///
    /// The runtime enforces fail-closed: any `secret_ref` used in
    /// `secret_headers` or top-level `secret_refs` at dispatch time
    /// **must** appear in this list, or the request is denied.
    ///
    /// Default: empty vec (no secret refs allowed; backward compatible).
    #[serde(default)]
    pub secret_refs: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct EventPermissions {
    #[serde(default)]
    pub read: bool,
    #[serde(default)]
    pub append: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct CapabilityPermissions {
    #[serde(default)]
    pub invoke: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct PackagePermissions {
    #[serde(default)]
    pub call: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct AssetPermissions {
    #[serde(default)]
    pub read: bool,
    #[serde(default)]
    pub write: bool,
}

/// A single network access declaration in a package manifest.
///
/// Each entry describes an allowed outbound destination with host,
/// permitted HTTP methods, and a human-readable purpose. The runtime
/// / host policy checker matches outbound requests against declared
/// entries. Packages with no `network` declarations must not make
/// any outbound network requests.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NetworkDeclaration {
    /// Destination host (e.g. `"api.openai.com"` or `"*.example.org"`).
    pub host: String,
    /// Permitted HTTP/WebSocket methods (e.g. `["GET", "POST", "WEBSOCKET"]`). Empty means all.
    #[serde(default, deserialize_with = "deserialize_network_methods")]
    pub methods: Vec<String>,
    /// Human-readable purpose for this network access.
    #[serde(default)]
    pub purpose: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct NetworkPermissions {
    /// Flat host list for backward compatibility. Packages using the
    /// structured form should populate `declarations` instead.
    #[serde(default)]
    pub hosts: Vec<String>,
    /// Structured network declarations (host, methods, purpose).
    #[serde(default)]
    pub declarations: Vec<NetworkDeclaration>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct FilesystemPermissions {
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
        for dependency in &self.requires {
            validate_package_id(&dependency.id)?;
            validate_dependency_source(&dependency.source)?;
            if !dependency.version.trim().is_empty() {
                validate_semver_constraint(&dependency.version)?;
            }
            for fingerprint in &dependency.minimum_signed_by {
                validate_gpg_fingerprint(fingerprint)?;
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
            for capability_id in &surface.allowed_capability_ids {
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
        // Y2/B: Validate permissions.secret_refs entries.
        // Each entry must be a supported host-backed secret reference.
        // Malformed or unsupported refs produce a clear manifest parse error.
        for secret_ref in &self.permissions.secret_refs {
            if !crate::is_env_backed_ref(secret_ref)
                && !crate::is_store_backed_ref(secret_ref)
                && !crate::is_project_backed_ref(secret_ref)
            {
                return Err(ManifestError::InvalidSecretRef(secret_ref.clone()));
            }
        }
        for declaration in &self.permissions.network.declarations {
            for method in &declaration.methods {
                validate_network_method(method)?;
            }
        }
        if self.entry.contract == ContractMode::None {
            match self.entry.kind {
                PackageEntry::Wasm { .. }
                | PackageEntry::Remote { .. }
                | PackageEntry::SurfaceBundle { .. } => {
                    return Err(ManifestError::InvalidContractMode {
                        kind: self.entry_kind().to_string(),
                        contract: self.entry.contract.clone(),
                    });
                }
                PackageEntry::RustInproc { .. } | PackageEntry::Subprocess { .. } => {}
            }
        }
        if matches!(self.entry.kind, PackageEntry::SurfaceBundle { .. }) {
            self.validate_static_surface_bundle()?;
        }
        Ok(())
    }

    fn validate_static_surface_bundle(&self) -> Result<(), ManifestError> {
        let invalid = !self.provides.is_empty()
            || !self.contributes.hooks.is_empty()
            || !self.contributes.extension_points.is_empty()
            || self.permissions.events.read
            || self.permissions.events.append
            || !self.permissions.capabilities.invoke.is_empty()
            || !self.permissions.packages.call.is_empty()
            || self.permissions.assets.read
            || self.permissions.assets.write
            || !self.permissions.network.hosts.is_empty()
            || !self.permissions.network.declarations.is_empty()
            || !self.permissions.filesystem.read.is_empty()
            || !self.permissions.filesystem.write.is_empty()
            || !self.permissions.secret_refs.is_empty();
        if invalid {
            return Err(ManifestError::InvalidSurfaceBundleAuthority(
                self.id.clone(),
            ));
        }
        Ok(())
    }

    pub fn entry_kind(&self) -> &'static str {
        match &self.entry.kind {
            PackageEntry::RustInproc { .. } => "rust_inproc",
            PackageEntry::Subprocess { .. } => "subprocess",
            PackageEntry::Wasm { .. } => "wasm",
            PackageEntry::Remote { .. } => "remote",
            PackageEntry::SurfaceBundle { .. } => "surface_bundle",
        }
    }
}

const ALLOWED_NETWORK_METHODS: &[&str] = &[
    "GET",
    "POST",
    "PUT",
    "DELETE",
    "PATCH",
    "HEAD",
    "OPTIONS",
    "WEBSOCKET",
];

fn deserialize_network_methods<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let methods = Vec::<String>::deserialize(deserializer)?;
    Ok(methods
        .into_iter()
        .map(|method| method.trim().to_ascii_uppercase())
        .collect())
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
    #[error("invalid secret_ref in permissions: {0}")]
    InvalidSecretRef(String),
    #[error("invalid network method in permissions.network.declarations: {0}")]
    InvalidNetworkMethod(String),
    #[error("invalid dependency source for {id}: {reason}")]
    InvalidDependencySource { id: String, reason: String },
    #[error("invalid GPG fingerprint in package dependency: {0}")]
    InvalidGpgFingerprint(String),
    #[error("contract mode '{contract:?}' is not valid for entry kind '{kind}'")]
    InvalidContractMode {
        kind: String,
        contract: ContractMode,
    },
    #[error("surface_bundle package must be static and cannot declare executable authority: {0}")]
    InvalidSurfaceBundleAuthority(String),
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

fn validate_semver_constraint(version: &str) -> Result<(), ManifestError> {
    semver::VersionReq::parse(version)
        .map(|_| ())
        .map_err(|_| ManifestError::InvalidVersion(version.to_string()))
}

fn validate_dependency_source(source: &DependencySource) -> Result<(), ManifestError> {
    if let DependencySource::Git { url, .. } = source {
        let parsed =
            url::Url::parse(url).map_err(|error| ManifestError::InvalidDependencySource {
                id: url.clone(),
                reason: error.to_string(),
            })?;
        if parsed.scheme() != "https" {
            return Err(ManifestError::InvalidDependencySource {
                id: url.clone(),
                reason: "git dependency URL must use https".to_string(),
            });
        }
        if !parsed.username().is_empty() || parsed.password().is_some() {
            return Err(ManifestError::InvalidDependencySource {
                id: url.clone(),
                reason: "git dependency URL must not contain userinfo".to_string(),
            });
        }
        if parsed.host_str().is_none() {
            return Err(ManifestError::InvalidDependencySource {
                id: url.clone(),
                reason: "git dependency URL must include a host".to_string(),
            });
        }
    }
    Ok(())
}

fn validate_gpg_fingerprint(fingerprint: &str) -> Result<(), ManifestError> {
    let trimmed = fingerprint.trim();
    if trimmed.len() == 40 && trimmed.chars().all(|ch| ch.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(ManifestError::InvalidGpgFingerprint(
            fingerprint.to_string(),
        ))
    }
}

fn validate_schema_shape(schema: &Value) -> Result<(), ManifestError> {
    if schema.is_null() || schema.is_object() {
        Ok(())
    } else {
        Err(ManifestError::InvalidSchema(schema.to_string()))
    }
}

fn validate_network_method(method: &str) -> Result<(), ManifestError> {
    if ALLOWED_NETWORK_METHODS
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(method))
    {
        Ok(())
    } else {
        Err(ManifestError::InvalidNetworkMethod(method.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_manifest_with_network_methods(methods: Vec<String>) -> PackageManifest {
        PackageManifest {
            schema_version: 1,
            id: "org/test".to_string(),
            version: "0.1.0".to_string(),
            display_name: None,
            description: None,
            author: None,
            license: None,
            entry: EntryDescriptor::v1(PackageEntry::RustInproc {
                crate_ref: "org-test".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            }),
            provides: Vec::new(),
            consumes: Vec::new(),
            requires: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet {
                network: NetworkPermissions {
                    hosts: Vec::new(),
                    declarations: vec![NetworkDeclaration {
                        host: "api.example.com".to_string(),
                        methods,
                        purpose: Some("test".to_string()),
                    }],
                },
                ..PermissionSet::default()
            },
            sandbox_policy: SandboxPolicy::default(),
        }
    }

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
            entry: EntryDescriptor::v1(PackageEntry::RustInproc {
                crate_ref: "org-example".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            }),
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
            requires: Vec::new(),
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

    // --- Y2: permissions.secret_refs tests ---

    #[test]
    fn permissions_secret_refs_default_empty() {
        let manifest = PackageManifest {
            schema_version: 1,
            id: "org/test".to_string(),
            version: "0.1.0".to_string(),
            display_name: None,
            description: None,
            author: None,
            license: None,
            entry: EntryDescriptor::v1(PackageEntry::RustInproc {
                crate_ref: "org-test".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            }),
            provides: Vec::new(),
            consumes: Vec::new(),
            requires: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet::default(),
            sandbox_policy: SandboxPolicy::default(),
        };
        assert!(
            manifest.permissions.secret_refs.is_empty(),
            "default secret_refs should be empty"
        );
        assert_eq!(manifest.validate_basic(), Ok(()));
    }

    #[test]
    fn permissions_secret_refs_parses_canonical_form() {
        let manifest = PackageManifest {
            schema_version: 1,
            id: "org/test".to_string(),
            version: "0.1.0".to_string(),
            display_name: None,
            description: None,
            author: None,
            license: None,
            entry: EntryDescriptor::v1(PackageEntry::RustInproc {
                crate_ref: "org-test".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            }),
            provides: Vec::new(),
            consumes: Vec::new(),
            requires: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet {
                secret_refs: vec!["secret_ref:env:OPENAI_API_KEY".to_string()],
                ..PermissionSet::default()
            },
            sandbox_policy: SandboxPolicy::default(),
        };
        assert_eq!(
            manifest.permissions.secret_refs,
            vec!["secret_ref:env:OPENAI_API_KEY"]
        );
        assert_eq!(manifest.validate_basic(), Ok(()));
    }

    #[test]
    fn permissions_secret_refs_parses_store_form() {
        let manifest = PackageManifest {
            schema_version: 1,
            id: "org/test".to_string(),
            version: "0.1.0".to_string(),
            display_name: None,
            description: None,
            author: None,
            license: None,
            entry: EntryDescriptor::v1(PackageEntry::RustInproc {
                crate_ref: "org-test".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            }),
            provides: Vec::new(),
            consumes: Vec::new(),
            requires: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet {
                secret_refs: vec!["secret_ref:store:OPENAI_API_KEY".to_string()],
                ..PermissionSet::default()
            },
            sandbox_policy: SandboxPolicy::default(),
        };
        assert_eq!(manifest.validate_basic(), Ok(()));
    }

    #[test]
    fn permissions_secret_refs_parses_project_form() {
        let manifest = PackageManifest {
            schema_version: 1,
            id: "org/test".to_string(),
            version: "0.1.0".to_string(),
            display_name: None,
            description: None,
            author: None,
            license: None,
            entry: EntryDescriptor::v1(PackageEntry::RustInproc {
                crate_ref: "org-test".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            }),
            provides: Vec::new(),
            consumes: Vec::new(),
            requires: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet {
                secret_refs: vec!["secret_ref:project:OPENAI_API_KEY".to_string()],
                ..PermissionSet::default()
            },
            sandbox_policy: SandboxPolicy::default(),
        };
        assert_eq!(manifest.validate_basic(), Ok(()));
    }

    #[test]
    fn permissions_secret_refs_parses_compat_prefixes() {
        let manifest = PackageManifest {
            schema_version: 1,
            id: "org/test".to_string(),
            version: "0.1.0".to_string(),
            display_name: None,
            description: None,
            author: None,
            license: None,
            entry: EntryDescriptor::v1(PackageEntry::RustInproc {
                crate_ref: "org-test".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            }),
            provides: Vec::new(),
            consumes: Vec::new(),
            requires: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet {
                secret_refs: vec![
                    "secretRef:env:MY_KEY".to_string(),
                    "secret-ref:env:ANOTHER_KEY".to_string(),
                    "host:env:DEEPSEEK_KEY".to_string(),
                ],
                ..PermissionSet::default()
            },
            sandbox_policy: SandboxPolicy::default(),
        };
        assert_eq!(
            manifest.permissions.secret_refs,
            vec![
                "secretRef:env:MY_KEY",
                "secret-ref:env:ANOTHER_KEY",
                "host:env:DEEPSEEK_KEY",
            ]
        );
        assert_eq!(manifest.validate_basic(), Ok(()));
    }

    #[test]
    fn permissions_secret_refs_rejects_malformed() {
        let manifest = PackageManifest {
            schema_version: 1,
            id: "org/test".to_string(),
            version: "0.1.0".to_string(),
            display_name: None,
            description: None,
            author: None,
            license: None,
            entry: EntryDescriptor::v1(PackageEntry::RustInproc {
                crate_ref: "org-test".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            }),
            provides: Vec::new(),
            consumes: Vec::new(),
            requires: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet {
                secret_refs: vec!["not-a-secret-ref".to_string()],
                ..PermissionSet::default()
            },
            sandbox_policy: SandboxPolicy::default(),
        };
        let err = manifest
            .validate_basic()
            .expect_err("malformed secret_ref should be rejected");
        assert_eq!(
            err,
            ManifestError::InvalidSecretRef("not-a-secret-ref".to_string())
        );
    }

    #[test]
    fn permissions_secret_refs_rejects_unsupported_scheme() {
        let manifest = PackageManifest {
            schema_version: 1,
            id: "org/test".to_string(),
            version: "0.1.0".to_string(),
            display_name: None,
            description: None,
            author: None,
            license: None,
            entry: EntryDescriptor::v1(PackageEntry::RustInproc {
                crate_ref: "org-test".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            }),
            provides: Vec::new(),
            consumes: Vec::new(),
            requires: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet {
                secret_refs: vec!["secret_ref:vault:NAME".to_string()],
                ..PermissionSet::default()
            },
            sandbox_policy: SandboxPolicy::default(),
        };
        let err = manifest
            .validate_basic()
            .expect_err("non-env secret_ref should be rejected");
        assert_eq!(
            err,
            ManifestError::InvalidSecretRef("secret_ref:vault:NAME".to_string())
        );
    }

    #[test]
    fn permissions_secret_refs_round_trips() {
        let mut perms = PermissionSet::default();
        perms.secret_refs = vec![
            "secret_ref:env:OPENAI_API_KEY".to_string(),
            "secretRef:env:MY_KEY".to_string(),
        ];
        let json = serde_json::to_string(&perms).expect("serialize");
        let perms2: PermissionSet = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(perms.secret_refs, perms2.secret_refs);
    }

    #[test]
    fn manifest_network_method_websocket_parses() {
        let manifest = base_manifest_with_network_methods(vec!["WEBSOCKET".to_string()]);
        assert_eq!(manifest.validate_basic(), Ok(()));
    }

    #[test]
    fn contract_mode_defaults_to_v1_and_serializes_lowercase() {
        let raw = serde_yaml::from_str::<PackageManifest>(
            r#"
schema_version: 1
id: org/test
version: 0.1.0
entry:
  kind: subprocess
  command: ["node", "index.mjs"]
"#,
        )
        .expect("manifest parses");
        assert_eq!(raw.entry.contract, ContractMode::V1);

        let none = EntryDescriptor::contract_none(PackageEntry::Subprocess {
            command: vec!["node".to_string(), "index.mjs".to_string()],
            transport: SubprocessTransport::JsonRpcStdio,
        });
        let yaml = serde_yaml::to_string(&none).expect("entry serializes");
        assert!(
            yaml.contains("contract: none"),
            "contract none should serialize as lowercase string: {yaml}"
        );
    }

    #[test]
    fn contract_none_rejected_for_wasm_and_remote() {
        let mut wasm = base_manifest_with_network_methods(vec![]);
        wasm.entry = EntryDescriptor::contract_none(PackageEntry::Wasm {
            module: "pkg.wasm".to_string(),
            abi_version: 1,
            memory_limit_mb: 64,
        });
        assert!(matches!(
            wasm.validate_basic(),
            Err(ManifestError::InvalidContractMode { .. })
        ));

        let mut remote = base_manifest_with_network_methods(vec![]);
        remote.entry = EntryDescriptor::contract_none(PackageEntry::Remote {
            endpoint: "https://example.com/rpc".to_string(),
            auth: RemoteAuth {
                scheme: "none".to_string(),
                config: Value::Null,
            },
        });
        assert!(matches!(
            remote.validate_basic(),
            Err(ManifestError::InvalidContractMode { .. })
        ));
    }

    #[test]
    fn surface_bundle_entry_is_valid_and_non_executing() {
        let mut manifest = base_manifest_with_network_methods(vec![]);
        manifest.entry = EntryDescriptor::v1(PackageEntry::SurfaceBundle {
            bundle: "dist/bundle.mjs".to_string(),
        });
        manifest.permissions = PermissionSet::default();
        assert_eq!(manifest.entry_kind(), "surface_bundle");
        assert_eq!(manifest.validate_basic(), Ok(()));
    }

    #[test]
    fn surface_bundle_rejects_executable_authority_and_contract_none() {
        let mut manifest = base_manifest_with_network_methods(vec![]);
        manifest.entry = EntryDescriptor::v1(PackageEntry::SurfaceBundle {
            bundle: "dist/bundle.mjs".to_string(),
        });
        manifest.permissions = PermissionSet::default();
        manifest.provides = vec![CapabilityDescriptor {
            id: "org/test/run".to_string(),
            version: "0.1.0".to_string(),
            input_schema: Value::Null,
            output_schema: Value::Null,
            streaming: false,
            side_effects: Vec::new(),
            description: None,
        }];
        assert!(matches!(
            manifest.validate_basic(),
            Err(ManifestError::InvalidSurfaceBundleAuthority(_))
        ));

        let mut manifest = base_manifest_with_network_methods(vec![]);
        manifest.entry = EntryDescriptor::v1(PackageEntry::SurfaceBundle {
            bundle: "dist/bundle.mjs".to_string(),
        });
        manifest.permissions = PermissionSet::default();
        manifest.permissions.network.hosts = vec!["example.com".to_string()];
        assert!(matches!(
            manifest.validate_basic(),
            Err(ManifestError::InvalidSurfaceBundleAuthority(_))
        ));

        let mut manifest = base_manifest_with_network_methods(vec![]);
        manifest.entry = EntryDescriptor::contract_none(PackageEntry::SurfaceBundle {
            bundle: "dist/bundle.mjs".to_string(),
        });
        manifest.permissions = PermissionSet::default();
        assert!(matches!(
            manifest.validate_basic(),
            Err(ManifestError::InvalidContractMode { .. })
        ));
    }

    #[test]
    fn manifest_network_method_websocket_lowercase_normalized() {
        let raw = serde_json::json!({
            "schema_version": 1,
            "id": "org/test",
            "version": "0.1.0",
            "entry": {
                "kind": "rust_inproc",
                "crate_ref": "org-test",
                "symbol": "register",
                "abi_version": 1
            },
            "permissions": {
                "network": {
                    "declarations": [{
                        "host": "api.example.com",
                        "methods": ["websocket"],
                        "purpose": "test"
                    }]
                }
            }
        });
        let manifest: PackageManifest = serde_json::from_value(raw).expect("parse manifest");
        assert_eq!(
            manifest.permissions.network.declarations[0].methods,
            vec!["WEBSOCKET".to_string()]
        );
        assert_eq!(manifest.validate_basic(), Ok(()));
    }

    #[test]
    fn manifest_network_method_invalid_method_rejected() {
        let manifest = base_manifest_with_network_methods(vec!["CONNECT_TO_ANYTHING".to_string()]);
        let err = manifest
            .validate_basic()
            .expect_err("invalid network method should be rejected");
        assert_eq!(
            err,
            ManifestError::InvalidNetworkMethod("CONNECT_TO_ANYTHING".to_string())
        );
    }

    fn base_manifest() -> PackageManifest {
        PackageManifest {
            schema_version: 1,
            id: "org/test".to_string(),
            version: "0.1.0".to_string(),
            display_name: None,
            description: None,
            author: None,
            license: None,
            entry: EntryDescriptor::v1(PackageEntry::RustInproc {
                crate_ref: "org-test".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            }),
            provides: Vec::new(),
            consumes: Vec::new(),
            requires: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet::default(),
            sandbox_policy: SandboxPolicy::default(),
        }
    }

    #[test]
    fn requires_default_empty_and_skips_serialization() {
        let yaml = r#"
schema_version: 1
id: org/test
version: 0.1.0
entry:
  kind: rust_inproc
  crate_ref: org-test
  symbol: register
  abi_version: 1
"#;
        let manifest: PackageManifest = serde_yaml::from_str(yaml).expect("parse manifest");
        assert!(manifest.requires.is_empty());
        let json = serde_json::to_string(&manifest).expect("serialize manifest");
        assert!(!json.contains("requires"));
        let decoded: PackageManifest = serde_json::from_str(&json).expect("round trip json");
        assert!(decoded.requires.is_empty());
    }

    #[test]
    fn requires_internal_source_serializes() {
        let mut manifest = base_manifest();
        manifest.requires = vec![PackageDependency {
            id: "official/core".to_string(),
            source: DependencySource::Internal,
            version: String::new(),
            minimum_signed_by: Vec::new(),
        }];
        assert_eq!(manifest.validate_basic(), Ok(()));
        let yaml = serde_yaml::to_string(&manifest).expect("serialize yaml");
        assert!(yaml.contains("kind: internal"), "{yaml}");
        let decoded: PackageManifest = serde_yaml::from_str(&yaml).expect("round trip yaml");
        assert!(matches!(
            decoded.requires[0].source,
            DependencySource::Internal
        ));
    }

    #[test]
    fn requires_git_source_https_tag_serializes() {
        let mut manifest = base_manifest();
        manifest.requires = vec![PackageDependency {
            id: "vendor/tool".to_string(),
            source: DependencySource::Git {
                url: "https://example.com/vendor/tool.git".to_string(),
                r#ref: "v1.2.3".to_string(),
            },
            version: "^1.2".to_string(),
            minimum_signed_by: vec!["0123456789abcdef0123456789ABCDEF01234567".to_string()],
        }];
        assert_eq!(manifest.validate_basic(), Ok(()));
        let json = serde_json::to_string(&manifest).expect("serialize json");
        assert!(json.contains("\"kind\":\"git\""), "{json}");
        assert!(json.contains("\"ref\":\"v1.2.3\""), "{json}");
        let decoded: PackageManifest = serde_json::from_str(&json).expect("round trip json");
        assert!(matches!(
            decoded.requires[0].source,
            DependencySource::Git { .. }
        ));
    }

    #[test]
    fn requires_local_source_relative_path_serializes() {
        let mut manifest = base_manifest();
        manifest.requires = vec![PackageDependency {
            id: "local/dev-tool".to_string(),
            source: DependencySource::Local {
                path: "../dev-tool".to_string(),
            },
            version: String::new(),
            minimum_signed_by: Vec::new(),
        }];
        assert_eq!(manifest.validate_basic(), Ok(()));
        let yaml = serde_yaml::to_string(&manifest).expect("serialize yaml");
        assert!(yaml.contains("kind: local"), "{yaml}");
        assert!(yaml.contains("path: ../dev-tool"), "{yaml}");
        let decoded: PackageManifest = serde_yaml::from_str(&yaml).expect("round trip yaml");
        assert!(matches!(
            decoded.requires[0].source,
            DependencySource::Local { .. }
        ));
    }

    #[test]
    fn requires_invalid_git_urls_rejected() {
        for url in [
            "ssh://git@example.com/vendor/tool.git",
            "git://example.com/vendor/tool.git",
            "https://user@example.com/vendor/tool.git",
        ] {
            let mut manifest = base_manifest();
            manifest.requires = vec![PackageDependency {
                id: "vendor/tool".to_string(),
                source: DependencySource::Git {
                    url: url.to_string(),
                    r#ref: "v1.0.0".to_string(),
                },
                version: ">=1.0.0".to_string(),
                minimum_signed_by: Vec::new(),
            }];
            assert!(matches!(
                manifest.validate_basic(),
                Err(ManifestError::InvalidDependencySource { .. })
            ));
        }
    }

    #[test]
    fn requires_invalid_semver_constraint_rejected() {
        let mut manifest = base_manifest();
        manifest.requires = vec![PackageDependency {
            id: "vendor/tool".to_string(),
            source: DependencySource::Internal,
            version: "not a constraint".to_string(),
            minimum_signed_by: Vec::new(),
        }];
        assert_eq!(
            manifest.validate_basic(),
            Err(ManifestError::InvalidVersion(
                "not a constraint".to_string()
            ))
        );
    }

    #[test]
    fn requires_invalid_gpg_fingerprint_rejected() {
        let mut manifest = base_manifest();
        manifest.requires = vec![PackageDependency {
            id: "vendor/tool".to_string(),
            source: DependencySource::Internal,
            version: String::new(),
            minimum_signed_by: vec!["not-a-fingerprint".to_string()],
        }];
        assert_eq!(
            manifest.validate_basic(),
            Err(ManifestError::InvalidGpgFingerprint(
                "not-a-fingerprint".to_string()
            ))
        );
    }

    #[test]
    fn requires_yaml_and_json_round_trip() {
        let mut manifest = base_manifest();
        manifest.requires = vec![
            PackageDependency {
                id: "official/core".to_string(),
                source: DependencySource::Internal,
                version: String::new(),
                minimum_signed_by: Vec::new(),
            },
            PackageDependency {
                id: "vendor/tool".to_string(),
                source: DependencySource::Git {
                    url: "https://example.com/vendor/tool.git".to_string(),
                    r#ref: "v1.2.3".to_string(),
                },
                version: "=1.2.3".to_string(),
                minimum_signed_by: Vec::new(),
            },
            PackageDependency {
                id: "local/dev-tool".to_string(),
                source: DependencySource::Local {
                    path: "../dev-tool".to_string(),
                },
                version: String::new(),
                minimum_signed_by: Vec::new(),
            },
        ];

        let yaml = serde_yaml::to_string(&manifest).expect("serialize yaml");
        let from_yaml: PackageManifest = serde_yaml::from_str(&yaml).expect("deserialize yaml");
        assert_eq!(from_yaml.requires.len(), 3);
        assert_eq!(from_yaml.validate_basic(), Ok(()));

        let json = serde_json::to_string(&from_yaml).expect("serialize json");
        let from_json: PackageManifest = serde_json::from_str(&json).expect("deserialize json");
        assert_eq!(from_json.requires.len(), 3);
        assert_eq!(from_json.validate_basic(), Ok(()));
    }
}
