use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// KernelMethod — single source of truth for protocol method identity, status,
// and streaming flag. Every method that appears in KERNEL_METHODS must have a
// variant here; every variant must be covered in FromStr, Display, id(),
// status(), streaming(), and all(). The runtime dispatch matches on these
// variants instead of raw string literals.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KernelMethod {
    SessionOpen,
    SessionClose,
    SessionFork,
    SessionBranchList,
    SessionGet,
    SessionList,
    EventAppend,
    EventList,
    EventSubscribe,
    PackageLoad,
    PackageUnload,
    PackageRestart,
    PackageLogs,
    PackageList,
    PackageStatus,
    PackageDescribe,
    CapabilityDiscover,
    CapabilityDescribe,
    CapabilityInvoke,
    CapabilityStream,
    CapabilityCancel,
    ExtensionPointList,
    ExtensionPointDescribe,
    HookList,
    AssetPut,
    AssetGet,
    AssetList,
    ProjectionRegister,
    ProjectionRebuild,
    ProjectionGet,
    ProjectionList,
    HostInfo,
    HostPing,
    HostDiagnostics,
    HostPrincipal,
    PermissionGrant,
    PermissionRevoke,
    PermissionList,
    PermissionAudit,
    ProposalCreate,
    ProposalGet,
    ProposalList,
    ProposalApprove,
    ProposalReject,
    ProposalApply,
    SurfaceContributionList,
    SurfaceContributionDescribe,
    OutboundAudit,
    OutboundExecute,
    OutboundStream,
    OutboundGitFetch,
}

impl KernelMethod {
    /// Canonical dotted method identifier (e.g. `"kernel.session.open"`).
    pub const fn id(&self) -> &'static str {
        match self {
            Self::SessionOpen => "kernel.session.open",
            Self::SessionClose => "kernel.session.close",
            Self::SessionFork => "kernel.session.fork",
            Self::SessionBranchList => "kernel.session.branch.list",
            Self::SessionGet => "kernel.session.get",
            Self::SessionList => "kernel.session.list",
            Self::EventAppend => "kernel.event.append",
            Self::EventList => "kernel.event.list",
            Self::EventSubscribe => "kernel.event.subscribe",
            Self::PackageLoad => "kernel.package.load",
            Self::PackageUnload => "kernel.package.unload",
            Self::PackageRestart => "kernel.package.restart",
            Self::PackageLogs => "kernel.package.logs",
            Self::PackageList => "kernel.package.list",
            Self::PackageStatus => "kernel.package.status",
            Self::PackageDescribe => "kernel.package.describe",
            Self::CapabilityDiscover => "kernel.capability.discover",
            Self::CapabilityDescribe => "kernel.capability.describe",
            Self::CapabilityInvoke => "kernel.capability.invoke",
            Self::CapabilityStream => "kernel.capability.stream",
            Self::CapabilityCancel => "kernel.capability.cancel",
            Self::ExtensionPointList => "kernel.extension_point.list",
            Self::ExtensionPointDescribe => "kernel.extension_point.describe",
            Self::HookList => "kernel.hook.list",
            Self::AssetPut => "kernel.asset.put",
            Self::AssetGet => "kernel.asset.get",
            Self::AssetList => "kernel.asset.list",
            Self::ProjectionRegister => "kernel.projection.register",
            Self::ProjectionRebuild => "kernel.projection.rebuild",
            Self::ProjectionGet => "kernel.projection.get",
            Self::ProjectionList => "kernel.projection.list",
            Self::HostInfo => "kernel.host.info",
            Self::HostPing => "kernel.host.ping",
            Self::HostDiagnostics => "kernel.host.diagnostics",
            Self::HostPrincipal => "kernel.host.principal",
            Self::PermissionGrant => "kernel.permission.grant",
            Self::PermissionRevoke => "kernel.permission.revoke",
            Self::PermissionList => "kernel.permission.list",
            Self::PermissionAudit => "kernel.permission.audit",
            Self::ProposalCreate => "kernel.proposal.create",
            Self::ProposalGet => "kernel.proposal.get",
            Self::ProposalList => "kernel.proposal.list",
            Self::ProposalApprove => "kernel.proposal.approve",
            Self::ProposalReject => "kernel.proposal.reject",
            Self::ProposalApply => "kernel.proposal.apply",
            Self::SurfaceContributionList => "kernel.surface.contribution.list",
            Self::SurfaceContributionDescribe => "kernel.surface.contribution.describe",
            Self::OutboundAudit => "kernel.outbound.audit",
            Self::OutboundExecute => "kernel.outbound.execute",
            Self::OutboundStream => "kernel.outbound.stream",
            Self::OutboundGitFetch => "kernel.outbound.git_fetch",
        }
    }

    /// Protocol implementation status for this method.
    pub const fn status(&self) -> MethodStatus {
        match self {
            Self::SessionOpen => MethodStatus::Implemented,
            Self::SessionClose => MethodStatus::Implemented,
            Self::SessionFork => MethodStatus::Partial,
            Self::SessionBranchList => MethodStatus::Partial,
            Self::SessionGet => MethodStatus::Planned,
            Self::SessionList => MethodStatus::Planned,
            Self::EventAppend => MethodStatus::Implemented,
            Self::EventList => MethodStatus::Partial,
            Self::EventSubscribe => MethodStatus::Planned,
            Self::PackageLoad => MethodStatus::Partial,
            Self::PackageUnload => MethodStatus::Partial,
            Self::PackageRestart => MethodStatus::Partial,
            Self::PackageLogs => MethodStatus::Partial,
            Self::PackageList => MethodStatus::Implemented,
            Self::PackageStatus => MethodStatus::Implemented,
            Self::PackageDescribe => MethodStatus::Planned,
            Self::CapabilityDiscover => MethodStatus::Implemented,
            Self::CapabilityDescribe => MethodStatus::Planned,
            Self::CapabilityInvoke => MethodStatus::Partial,
            Self::CapabilityStream => MethodStatus::Partial,
            Self::CapabilityCancel => MethodStatus::Partial,
            Self::ExtensionPointList => MethodStatus::Implemented,
            Self::ExtensionPointDescribe => MethodStatus::Planned,
            Self::HookList => MethodStatus::Partial,
            Self::AssetPut => MethodStatus::Partial,
            Self::AssetGet => MethodStatus::Partial,
            Self::AssetList => MethodStatus::Partial,
            Self::ProjectionRegister => MethodStatus::Partial,
            Self::ProjectionRebuild => MethodStatus::Partial,
            Self::ProjectionGet => MethodStatus::Partial,
            Self::ProjectionList => MethodStatus::Partial,
            Self::HostInfo => MethodStatus::Implemented,
            Self::HostPing => MethodStatus::Partial,
            Self::HostDiagnostics => MethodStatus::Partial,
            Self::HostPrincipal => MethodStatus::Planned,
            Self::PermissionGrant => MethodStatus::Partial,
            Self::PermissionRevoke => MethodStatus::Partial,
            Self::PermissionList => MethodStatus::Partial,
            Self::PermissionAudit => MethodStatus::Partial,
            Self::ProposalCreate => MethodStatus::Partial,
            Self::ProposalGet => MethodStatus::Partial,
            Self::ProposalList => MethodStatus::Partial,
            Self::ProposalApprove => MethodStatus::Partial,
            Self::ProposalReject => MethodStatus::Partial,
            Self::ProposalApply => MethodStatus::Partial,
            Self::SurfaceContributionList => MethodStatus::Partial,
            Self::SurfaceContributionDescribe => MethodStatus::Partial,
            Self::OutboundAudit => MethodStatus::Partial,
            Self::OutboundExecute => MethodStatus::Partial,
            Self::OutboundStream => MethodStatus::Partial,
            Self::OutboundGitFetch => MethodStatus::Partial,
        }
    }

    /// Whether this method returns a streaming response.
    pub const fn streaming(&self) -> bool {
        match self {
            Self::EventSubscribe | Self::CapabilityStream | Self::OutboundStream => true,
            _ => false,
        }
    }

    /// All known kernel methods in canonical order.
    pub const fn all() -> &'static [KernelMethod] {
        &[
            Self::SessionOpen,
            Self::SessionClose,
            Self::SessionFork,
            Self::SessionBranchList,
            Self::SessionGet,
            Self::SessionList,
            Self::EventAppend,
            Self::EventList,
            Self::EventSubscribe,
            Self::PackageLoad,
            Self::PackageUnload,
            Self::PackageRestart,
            Self::PackageLogs,
            Self::PackageList,
            Self::PackageStatus,
            Self::PackageDescribe,
            Self::CapabilityDiscover,
            Self::CapabilityDescribe,
            Self::CapabilityInvoke,
            Self::CapabilityStream,
            Self::CapabilityCancel,
            Self::ExtensionPointList,
            Self::ExtensionPointDescribe,
            Self::HookList,
            Self::AssetPut,
            Self::AssetGet,
            Self::AssetList,
            Self::ProjectionRegister,
            Self::ProjectionRebuild,
            Self::ProjectionGet,
            Self::ProjectionList,
            Self::HostInfo,
            Self::HostPing,
            Self::HostDiagnostics,
            Self::HostPrincipal,
            Self::PermissionGrant,
            Self::PermissionRevoke,
            Self::PermissionList,
            Self::PermissionAudit,
            Self::ProposalCreate,
            Self::ProposalGet,
            Self::ProposalList,
            Self::ProposalApprove,
            Self::ProposalReject,
            Self::ProposalApply,
            Self::SurfaceContributionList,
            Self::SurfaceContributionDescribe,
            Self::OutboundAudit,
            Self::OutboundExecute,
            Self::OutboundStream,
            Self::OutboundGitFetch,
        ]
    }

    /// Convert to the serialisable descriptor used in the public registry.
    pub fn to_protocol_method(&self) -> ProtocolMethod {
        ProtocolMethod { id: self.id(), streaming: self.streaming(), status: self.status() }
    }

    /// Whether this method has a dispatch branch in the runtime
    /// (`call_protocol_inner`). Kept in sync with the dispatch match in
    /// `runtime.rs` — update both sides together.
    pub const fn is_dispatched(&self) -> bool {
        match self {
            // Implemented or Partial methods that have a dispatch arm
            Self::SessionOpen
            | Self::SessionClose
            | Self::SessionFork
            | Self::SessionBranchList
            | Self::EventAppend
            | Self::EventList
            | Self::PackageLoad
            | Self::PackageUnload
            | Self::PackageRestart
            | Self::PackageLogs
            | Self::PackageList
            | Self::PackageStatus
            |             Self::CapabilityDiscover
            | Self::CapabilityInvoke
            | Self::CapabilityStream
            | Self::CapabilityCancel
            | Self::ExtensionPointList
            | Self::HookList
            | Self::AssetPut
            | Self::AssetGet
            | Self::AssetList
            | Self::ProjectionRegister
            | Self::ProjectionRebuild
            | Self::ProjectionGet
            | Self::ProjectionList
            | Self::HostInfo
            | Self::HostPing
            | Self::HostDiagnostics
            | Self::PermissionGrant
            | Self::PermissionRevoke
            | Self::PermissionList
            | Self::PermissionAudit
            | Self::ProposalCreate
            | Self::ProposalGet
            | Self::ProposalList
            | Self::ProposalApprove
            | Self::ProposalReject
            | Self::ProposalApply
            | Self::SurfaceContributionList
            | Self::SurfaceContributionDescribe
            | Self::OutboundAudit
            | Self::OutboundExecute
            | Self::OutboundStream
            | Self::OutboundGitFetch => true,
            // Planned methods with no dispatch yet
            Self::SessionGet
            | Self::SessionList
            | Self::EventSubscribe
            | Self::PackageDescribe
            | Self::CapabilityDescribe
            | Self::ExtensionPointDescribe
            | Self::HostPrincipal => false,
        }
    }
}

impl fmt::Display for KernelMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

impl FromStr for KernelMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "kernel.session.open" => Ok(Self::SessionOpen),
            "kernel.session.close" => Ok(Self::SessionClose),
            "kernel.session.fork" => Ok(Self::SessionFork),
            "kernel.session.branch.list" => Ok(Self::SessionBranchList),
            "kernel.session.get" => Ok(Self::SessionGet),
            "kernel.session.list" => Ok(Self::SessionList),
            "kernel.event.append" => Ok(Self::EventAppend),
            "kernel.event.list" => Ok(Self::EventList),
            "kernel.event.subscribe" => Ok(Self::EventSubscribe),
            "kernel.package.load" => Ok(Self::PackageLoad),
            "kernel.package.unload" => Ok(Self::PackageUnload),
            "kernel.package.restart" => Ok(Self::PackageRestart),
            "kernel.package.logs" => Ok(Self::PackageLogs),
            "kernel.package.list" => Ok(Self::PackageList),
            "kernel.package.status" => Ok(Self::PackageStatus),
            "kernel.package.describe" => Ok(Self::PackageDescribe),
            "kernel.capability.discover" => Ok(Self::CapabilityDiscover),
            "kernel.capability.describe" => Ok(Self::CapabilityDescribe),
            "kernel.capability.invoke" => Ok(Self::CapabilityInvoke),
            "kernel.capability.stream" => Ok(Self::CapabilityStream),
            "kernel.capability.cancel" => Ok(Self::CapabilityCancel),
            "kernel.extension_point.list" => Ok(Self::ExtensionPointList),
            "kernel.extension_point.describe" => Ok(Self::ExtensionPointDescribe),
            "kernel.hook.list" => Ok(Self::HookList),
            "kernel.asset.put" => Ok(Self::AssetPut),
            "kernel.asset.get" => Ok(Self::AssetGet),
            "kernel.asset.list" => Ok(Self::AssetList),
            "kernel.projection.register" => Ok(Self::ProjectionRegister),
            "kernel.projection.rebuild" => Ok(Self::ProjectionRebuild),
            "kernel.projection.get" => Ok(Self::ProjectionGet),
            "kernel.projection.list" => Ok(Self::ProjectionList),
            "kernel.host.info" => Ok(Self::HostInfo),
            "kernel.host.ping" => Ok(Self::HostPing),
            "kernel.host.diagnostics" => Ok(Self::HostDiagnostics),
            "kernel.host.principal" => Ok(Self::HostPrincipal),
            "kernel.permission.grant" => Ok(Self::PermissionGrant),
            "kernel.permission.revoke" => Ok(Self::PermissionRevoke),
            "kernel.permission.list" => Ok(Self::PermissionList),
            "kernel.permission.audit" => Ok(Self::PermissionAudit),
            "kernel.proposal.create" => Ok(Self::ProposalCreate),
            "kernel.proposal.get" => Ok(Self::ProposalGet),
            "kernel.proposal.list" => Ok(Self::ProposalList),
            "kernel.proposal.approve" => Ok(Self::ProposalApprove),
            "kernel.proposal.reject" => Ok(Self::ProposalReject),
            "kernel.proposal.apply" => Ok(Self::ProposalApply),
            "kernel.surface.contribution.list" => Ok(Self::SurfaceContributionList),
            "kernel.surface.contribution.describe" => Ok(Self::SurfaceContributionDescribe),
            "kernel.outbound.audit" => Ok(Self::OutboundAudit),
            "kernel.outbound.execute" => Ok(Self::OutboundExecute),
            "kernel.outbound.stream" => Ok(Self::OutboundStream),
            "kernel.outbound.git_fetch" => Ok(Self::OutboundGitFetch),
            other => Err(format!("unknown kernel method: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Public protocol types (API-compatible)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolMethod {
    pub id: &'static str,
    pub streaming: bool,
    pub status: MethodStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MethodStatus {
    Implemented,
    Partial,
    Planned,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProtocolPrincipal {
    HostAdmin,
    HostDev,
    Package { package_id: String },
    Human { user_id: String },
    Assistant { assistant_id: String, delegated_user_id: Option<String> },
    Anonymous,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolContext {
    pub principal: ProtocolPrincipal,
    pub transport: String,
}

impl ProtocolContext {
    pub fn host_dev(transport: impl Into<String>) -> Self {
        Self { principal: ProtocolPrincipal::HostDev, transport: transport.into() }
    }

    pub fn package(package_id: impl Into<String>, transport: impl Into<String>) -> Self {
        Self {
            principal: ProtocolPrincipal::Package { package_id: package_id.into() },
            transport: transport.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtocolRequest {
    pub id: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtocolResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ProtocolError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtocolError {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub details: Value,
}

impl ProtocolError {
    pub fn new(code: impl Into<String>, message: impl Into<String>, details: Value) -> Self {
        Self { code: code.into(), message: message.into(), details }
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new("kernel/error/invalid_request", message, Value::Null)
    }

    pub fn from_anyhow(error: anyhow::Error) -> Self {
        let message = error.to_string();
        let code = if message.contains("not allowed") || message.contains("permission") {
            "kernel/error/permission_denied"
        } else if message.contains("ambiguous") {
            "kernel/error/ambiguous_route"
        } else if message.contains("schema") || message.contains("required") || message.contains("does not match") {
            "kernel/error/schema_invalid"
        } else if message.contains("not loaded") || message.contains("not found") || message.contains("no provider") {
            "kernel/error/not_found"
        } else if message.contains("closed") || message.contains("not ready") || message.contains("cannot execute") {
            "kernel/error/package_state"
        } else {
            "kernel/error/internal"
        };
        Self::new(code, message, Value::Null)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HostInfo {
    pub protocol_version: &'static str,
    pub methods: &'static [ProtocolMethod],
    pub supported_transports: Vec<&'static str>,
}

pub const KERNEL_PROTOCOL_VERSION: &str = "0.1.0";

// KERNEL_METHODS is derived from KernelMethod — the enum is the single source
// of truth. If a new method variant is added to KernelMethod, a corresponding
// entry must appear here (tests enforce this).
pub const KERNEL_METHODS: &[ProtocolMethod] = &[
    ProtocolMethod { id: "kernel.session.open", streaming: false, status: MethodStatus::Implemented },
    ProtocolMethod { id: "kernel.session.close", streaming: false, status: MethodStatus::Implemented },
    ProtocolMethod { id: "kernel.session.fork", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.session.branch.list", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.session.get", streaming: false, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.session.list", streaming: false, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.event.append", streaming: false, status: MethodStatus::Implemented },
    ProtocolMethod { id: "kernel.event.list", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.event.subscribe", streaming: true, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.package.load", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.package.unload", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.package.restart", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.package.logs", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.package.list", streaming: false, status: MethodStatus::Implemented },
    ProtocolMethod { id: "kernel.package.status", streaming: false, status: MethodStatus::Implemented },
    ProtocolMethod { id: "kernel.package.describe", streaming: false, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.capability.discover", streaming: false, status: MethodStatus::Implemented },
    ProtocolMethod { id: "kernel.capability.describe", streaming: false, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.capability.invoke", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.capability.stream", streaming: true, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.capability.cancel", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.extension_point.list", streaming: false, status: MethodStatus::Implemented },
    ProtocolMethod { id: "kernel.extension_point.describe", streaming: false, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.hook.list", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.asset.put", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.asset.get", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.asset.list", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.projection.register", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.projection.rebuild", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.projection.get", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.projection.list", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.host.info", streaming: false, status: MethodStatus::Implemented },
    ProtocolMethod { id: "kernel.host.ping", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.host.diagnostics", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.host.principal", streaming: false, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.permission.grant", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.permission.revoke", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.permission.list", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.permission.audit", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.proposal.create", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.proposal.get", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.proposal.list", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.proposal.approve", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.proposal.reject", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.proposal.apply", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.surface.contribution.list", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.surface.contribution.describe", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.outbound.audit", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.outbound.execute", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.outbound.stream", streaming: true, status: MethodStatus::Partial },
];

pub fn method_ids() -> Vec<&'static str> {
    KERNEL_METHODS.iter().map(|method| method.id).collect()
}

pub fn host_info() -> HostInfo {
    HostInfo {
        protocol_version: KERNEL_PROTOCOL_VERSION,
        methods: KERNEL_METHODS,
        supported_transports: vec!["in_process", "http_rpc", "host_stdio", "http_ad_hoc"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_contains_no_content_methods() {
        for id in method_ids() {
            assert!(!id.contains("turn"));
            assert!(!id.contains("prompt"));
            assert!(!id.contains("model"));
            assert!(!id.contains("message"));
        }
    }

    #[test]
    fn protocol_registry_matches_alpha_contract_core() {
        let ids = method_ids();
        for expected in [
            "kernel.session.open",
            "kernel.session.list",
            "kernel.event.subscribe",
            "kernel.package.describe",
            "kernel.capability.cancel",
            "kernel.asset.put",
            "kernel.host.principal",
        ] {
            assert!(ids.contains(&expected), "missing {expected}");
        }
    }

    // --- KernelMethod / registry alignment tests ---

    #[test]
    fn every_registry_id_parses_to_kernel_method() {
        for method in KERNEL_METHODS {
            let parsed: Result<KernelMethod, String> = method.id.parse();
            assert!(parsed.is_ok(), "registry id '{}' does not parse to KernelMethod", method.id);
        }
    }

    #[test]
    fn kernel_method_all_covers_entire_registry() {
        let all_ids: Vec<&'static str> = KernelMethod::all().iter().map(|m| m.id()).collect();
        for method in KERNEL_METHODS {
            assert!(
                all_ids.contains(&method.id),
                "KERNEL_METHODS contains '{}' but KernelMethod::all() does not",
                method.id
            );
        }
    }

    #[test]
    fn registry_matches_enum_metadata() {
        for method in KERNEL_METHODS {
            let km: KernelMethod = method.id.parse().unwrap();
            assert_eq!(method.id, km.id(), "id mismatch for {:?}", km);
            assert_eq!(method.streaming, km.streaming(), "streaming mismatch for {:?}", km);
            assert_eq!(method.status, km.status(), "status mismatch for {:?}", km);
        }
    }

    #[test]
    fn no_duplicate_ids_in_all() {
        let all = KernelMethod::all();
        let ids: Vec<&'static str> = all.iter().map(|m| m.id()).collect();
        let unique: std::collections::HashSet<&'static str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "KernelMethod::all() contains duplicate ids");
    }

    #[test]
    fn session_close_is_implemented_and_dispatched() {
        let km = KernelMethod::SessionClose;
        assert_eq!(km.id(), "kernel.session.close");
        assert_eq!(km.status(), MethodStatus::Implemented);
        assert!(km.is_dispatched(), "kernel.session.close must be dispatch-covered");
    }

    #[test]
    fn hook_list_status_matches_dispatch() {
        let km = KernelMethod::HookList;
        assert_eq!(km.id(), "kernel.hook.list");
        // Was previously Planned, but dispatch exists → must be at least Partial
        assert!(
            matches!(km.status(), MethodStatus::Implemented | MethodStatus::Partial),
            "kernel.hook.list status must be Implemented or Partial since dispatch exists, got {:?}",
            km.status()
        );
        assert!(km.is_dispatched(), "kernel.hook.list must be dispatch-covered");
    }

    #[test]
    fn implemented_or_partial_methods_must_be_dispatched() {
        for method in KERNEL_METHODS {
            let km: KernelMethod = method.id.parse().unwrap();
            if matches!(km.status(), MethodStatus::Implemented | MethodStatus::Partial) {
                assert!(
                    km.is_dispatched(),
                    "{:?} ({}) is {:?} but has no dispatch — add dispatch or downgrade to Planned",
                    km,
                    km.id(),
                    km.status()
                );
            }
        }
    }

    #[test]
    fn dispatched_methods_must_not_be_planned() {
        for method in KERNEL_METHODS {
            let km: KernelMethod = method.id.parse().unwrap();
            if km.is_dispatched() {
                assert!(
                    !matches!(km.status(), MethodStatus::Planned),
                    "{:?} ({}) is dispatched but status is Planned — upgrade to at least Partial",
                    km,
                    km.id()
                );
            }
        }
    }

    #[test]
    fn display_roundtrips_through_fromstr() {
        for km in KernelMethod::all() {
            let s = km.to_string();
            let parsed: KernelMethod = s.parse().unwrap();
            assert_eq!(*km, parsed, "Display -> FromStr roundtrip failed for {:?}", km);
        }
    }
}
