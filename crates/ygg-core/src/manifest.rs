use serde::{Deserialize, Deserializer, Serialize};
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
    RustInproc {
        crate_ref: String,
        symbol: String,
        abi_version: u16,
    },
    Subprocess {
        command: Vec<String>,
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
    pub git_fetch: GitFetchPermissions,
    #[serde(default)]
    pub filesystem: FilesystemPermissions,
    /// Declared secret references this package may use in
    /// `kernel.outbound.execute` calls. Each entry must be a valid
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

/// A single network access declaration in a package manifest.
///
/// Each entry describes an allowed outbound destination with host,
/// permitted HTTP methods, and a human-readable purpose. The runtime
/// / host policy checker matches outbound requests against declared
/// entries. Packages with no `network` declarations must not make
/// any outbound network requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkPermissions {
    /// Flat host list for backward compatibility. Packages using the
    /// structured form should populate `declarations` instead.
    #[serde(default)]
    pub hosts: Vec<String>,
    /// Structured network declarations (host, methods, purpose).
    #[serde(default)]
    pub declarations: Vec<NetworkDeclaration>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitFetchPermissions {
    /// HTTPS git hosts this package may fetch through `kernel.outbound.git_fetch`.
    /// Host policy must still independently allow the same destination.
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
        // Y2: Validate permissions.secret_refs entries.
        // Each entry must be an env-backed secret reference (the only
        // currently supported vault type). Malformed or non-env refs
        // produce a clear manifest parse error.
        for secret_ref in &self.permissions.secret_refs {
            if !crate::is_env_backed_ref(secret_ref) {
                return Err(ManifestError::InvalidSecretRef(secret_ref.clone()));
            }
        }
        for declaration in &self.permissions.network.declarations {
            for method in &declaration.methods {
                validate_network_method(method)?;
            }
        }
        Ok(())
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
            entry: PackageEntry::RustInproc {
                crate_ref: "org-test".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            },
            provides: Vec::new(),
            consumes: Vec::new(),
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
            entry: PackageEntry::RustInproc {
                crate_ref: "org-test".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            },
            provides: Vec::new(),
            consumes: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet::default(),
            sandbox_policy: SandboxPolicy::default(),
        };
        assert!(manifest.permissions.secret_refs.is_empty(), "default secret_refs should be empty");
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
            entry: PackageEntry::RustInproc {
                crate_ref: "org-test".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            },
            provides: Vec::new(),
            consumes: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet {
                secret_refs: vec!["secret_ref:env:OPENAI_API_KEY".to_string()],
                ..PermissionSet::default()
            },
            sandbox_policy: SandboxPolicy::default(),
        };
        assert_eq!(manifest.permissions.secret_refs, vec!["secret_ref:env:OPENAI_API_KEY"]);
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
            entry: PackageEntry::RustInproc {
                crate_ref: "org-test".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            },
            provides: Vec::new(),
            consumes: Vec::new(),
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
            entry: PackageEntry::RustInproc {
                crate_ref: "org-test".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            },
            provides: Vec::new(),
            consumes: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet {
                secret_refs: vec!["not-a-secret-ref".to_string()],
                ..PermissionSet::default()
            },
            sandbox_policy: SandboxPolicy::default(),
        };
        let err = manifest.validate_basic().expect_err("malformed secret_ref should be rejected");
        assert_eq!(err, ManifestError::InvalidSecretRef("not-a-secret-ref".to_string()));
    }

    #[test]
    fn permissions_secret_refs_rejects_non_env_scheme() {
        let manifest = PackageManifest {
            schema_version: 1,
            id: "org/test".to_string(),
            version: "0.1.0".to_string(),
            display_name: None,
            description: None,
            author: None,
            license: None,
            entry: PackageEntry::RustInproc {
                crate_ref: "org-test".to_string(),
                symbol: "register".to_string(),
                abi_version: 1,
            },
            provides: Vec::new(),
            consumes: Vec::new(),
            contributes: PackageContributions::default(),
            permissions: PermissionSet {
                secret_refs: vec!["secret_ref:vault:NAME".to_string()],
                ..PermissionSet::default()
            },
            sandbox_policy: SandboxPolicy::default(),
        };
        let err = manifest.validate_basic().expect_err("non-env secret_ref should be rejected");
        assert_eq!(err, ManifestError::InvalidSecretRef("secret_ref:vault:NAME".to_string()));
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
}
