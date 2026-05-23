use std::fmt;
use std::str::FromStr;

use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, Schema, SchemaObject, SingleOrVec},
    JsonSchema,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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
    ProjectList,
    ProjectGet,
    ProjectStart,
    ProjectStop,
    ProjectStatus,
    CapabilityDiscover,
    CapabilityDescribe,
    CapabilityInvoke,
    CapabilityHandleAttenuate,
    CapabilityHandleRevoke,
    CapabilityHandleListFor,
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
    AuditPackage,
    ProposalCreate,
    ProposalGet,
    ProposalList,
    ProposalApprove,
    ProposalReject,
    ProposalApply,
    SurfaceResolveBundle,
    SurfaceContributionList,
    SurfaceContributionDescribe,
    OutboundAudit,
    OutboundExecute,
    OutboundStream,
    OutboundWebSocketOpen,
    OutboundWebSocketSend,
    OutboundWebSocketClose,
}

impl KernelMethod {
    /// Canonical dotted method identifier (e.g. `"kernel.v1.session.open"`).
    pub const fn id(&self) -> &'static str {
        match self {
            Self::SessionOpen => "kernel.v1.session.open",
            Self::SessionClose => "kernel.v1.session.close",
            Self::SessionFork => "kernel.v1.session.fork",
            Self::SessionBranchList => "kernel.v1.session.branch.list",
            Self::SessionGet => "kernel.v1.session.get",
            Self::SessionList => "kernel.v1.session.list",
            Self::EventAppend => "kernel.v1.event.append",
            Self::EventList => "kernel.v1.event.list",
            Self::EventSubscribe => "kernel.v1.event.subscribe",
            Self::PackageLoad => "kernel.v1.package.load",
            Self::PackageUnload => "kernel.v1.package.unload",
            Self::PackageRestart => "kernel.v1.package.restart",
            Self::PackageLogs => "kernel.v1.package.logs",
            Self::PackageList => "kernel.v1.package.list",
            Self::PackageStatus => "kernel.v1.package.status",
            Self::PackageDescribe => "kernel.v1.package.describe",
            Self::ProjectList => "kernel.v1.project.list",
            Self::ProjectGet => "kernel.v1.project.get",
            Self::ProjectStart => "kernel.v1.project.start",
            Self::ProjectStop => "kernel.v1.project.stop",
            Self::ProjectStatus => "kernel.v1.project.status",
            Self::CapabilityDiscover => "kernel.v1.capability.discover",
            Self::CapabilityDescribe => "kernel.v1.capability.describe",
            Self::CapabilityInvoke => "kernel.v1.capability.invoke",
            Self::CapabilityHandleAttenuate => "kernel.v1.cap.attenuate",
            Self::CapabilityHandleRevoke => "kernel.v1.cap.revoke",
            Self::CapabilityHandleListFor => "kernel.v1.cap.list_for",
            Self::CapabilityStream => "kernel.v1.capability.stream",
            Self::CapabilityCancel => "kernel.v1.capability.cancel",
            Self::ExtensionPointList => "kernel.v1.extension_point.list",
            Self::ExtensionPointDescribe => "kernel.v1.extension_point.describe",
            Self::HookList => "kernel.v1.hook.list",
            Self::AssetPut => "kernel.v1.asset.put",
            Self::AssetGet => "kernel.v1.asset.get",
            Self::AssetList => "kernel.v1.asset.list",
            Self::ProjectionRegister => "kernel.v1.projection.register",
            Self::ProjectionRebuild => "kernel.v1.projection.rebuild",
            Self::ProjectionGet => "kernel.v1.projection.get",
            Self::ProjectionList => "kernel.v1.projection.list",
            Self::HostInfo => "kernel.v1.host.info",
            Self::HostPing => "kernel.v1.host.ping",
            Self::HostDiagnostics => "kernel.v1.host.diagnostics",
            Self::HostPrincipal => "kernel.v1.host.principal",
            Self::PermissionGrant => "kernel.v1.permission.grant",
            Self::PermissionRevoke => "kernel.v1.permission.revoke",
            Self::PermissionList => "kernel.v1.permission.list",
            Self::PermissionAudit => "kernel.v1.permission.audit",
            Self::AuditPackage => "kernel.v1.audit.package",
            Self::ProposalCreate => "kernel.v1.proposal.create",
            Self::ProposalGet => "kernel.v1.proposal.get",
            Self::ProposalList => "kernel.v1.proposal.list",
            Self::ProposalApprove => "kernel.v1.proposal.approve",
            Self::ProposalReject => "kernel.v1.proposal.reject",
            Self::ProposalApply => "kernel.v1.proposal.apply",
            Self::SurfaceResolveBundle => "kernel.v1.surface.resolve_bundle",
            Self::SurfaceContributionList => "kernel.v1.surface.contribution.list",
            Self::SurfaceContributionDescribe => "kernel.v1.surface.contribution.describe",
            Self::OutboundAudit => "kernel.v1.outbound.audit",
            Self::OutboundExecute => "kernel.v1.outbound.execute",
            Self::OutboundStream => "kernel.v1.outbound.stream",
            Self::OutboundWebSocketOpen => "kernel.v1.outbound.websocket.open",
            Self::OutboundWebSocketSend => "kernel.v1.outbound.websocket.send",
            Self::OutboundWebSocketClose => "kernel.v1.outbound.websocket.close",
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
            Self::ProjectList => MethodStatus::Implemented,
            Self::ProjectGet => MethodStatus::Implemented,
            Self::ProjectStart => MethodStatus::Implemented,
            Self::ProjectStop => MethodStatus::Implemented,
            Self::ProjectStatus => MethodStatus::Implemented,
            Self::CapabilityDiscover => MethodStatus::Implemented,
            Self::CapabilityDescribe => MethodStatus::Planned,
            Self::CapabilityInvoke => MethodStatus::Partial,
            Self::CapabilityHandleAttenuate => MethodStatus::Partial,
            Self::CapabilityHandleRevoke => MethodStatus::Partial,
            Self::CapabilityHandleListFor => MethodStatus::Partial,
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
            Self::AuditPackage => MethodStatus::Partial,
            Self::ProposalCreate => MethodStatus::Partial,
            Self::ProposalGet => MethodStatus::Partial,
            Self::ProposalList => MethodStatus::Partial,
            Self::ProposalApprove => MethodStatus::Partial,
            Self::ProposalReject => MethodStatus::Partial,
            Self::ProposalApply => MethodStatus::Partial,
            Self::SurfaceResolveBundle => MethodStatus::Partial,
            Self::SurfaceContributionList => MethodStatus::Partial,
            Self::SurfaceContributionDescribe => MethodStatus::Partial,
            Self::OutboundAudit => MethodStatus::Partial,
            Self::OutboundExecute => MethodStatus::Partial,
            Self::OutboundStream => MethodStatus::Partial,
            Self::OutboundWebSocketOpen => MethodStatus::Partial,
            Self::OutboundWebSocketSend => MethodStatus::Partial,
            Self::OutboundWebSocketClose => MethodStatus::Partial,
        }
    }

    /// Whether this method returns a streaming response.
    pub const fn streaming(&self) -> bool {
        match self {
            Self::EventSubscribe
            | Self::CapabilityStream
            | Self::OutboundStream
            | Self::OutboundWebSocketOpen => true,
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
            Self::ProjectList,
            Self::ProjectGet,
            Self::ProjectStart,
            Self::ProjectStop,
            Self::ProjectStatus,
            Self::CapabilityDiscover,
            Self::CapabilityDescribe,
            Self::CapabilityInvoke,
            Self::CapabilityHandleAttenuate,
            Self::CapabilityHandleRevoke,
            Self::CapabilityHandleListFor,
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
            Self::AuditPackage,
            Self::ProposalCreate,
            Self::ProposalGet,
            Self::ProposalList,
            Self::ProposalApprove,
            Self::ProposalReject,
            Self::ProposalApply,
            Self::SurfaceResolveBundle,
            Self::SurfaceContributionList,
            Self::SurfaceContributionDescribe,
            Self::OutboundAudit,
            Self::OutboundExecute,
            Self::OutboundStream,
            Self::OutboundWebSocketOpen,
            Self::OutboundWebSocketSend,
            Self::OutboundWebSocketClose,
        ]
    }

    /// Convert to the serialisable descriptor used in the public registry.
    pub fn to_protocol_method(&self) -> ProtocolMethod {
        ProtocolMethod {
            id: self.id(),
            streaming: self.streaming(),
            status: self.status(),
        }
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
            | Self::ProjectList
            | Self::ProjectGet
            | Self::ProjectStart
            | Self::ProjectStop
            | Self::ProjectStatus
            | Self::CapabilityDiscover
            | Self::CapabilityInvoke
            | Self::CapabilityHandleAttenuate
            | Self::CapabilityHandleRevoke
            | Self::CapabilityHandleListFor
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
            | Self::AuditPackage
            | Self::ProposalCreate
            | Self::ProposalGet
            | Self::ProposalList
            | Self::ProposalApprove
            | Self::ProposalReject
            | Self::ProposalApply
            | Self::SurfaceResolveBundle
            | Self::SurfaceContributionList
            | Self::SurfaceContributionDescribe
            | Self::OutboundAudit
            | Self::OutboundExecute
            | Self::OutboundStream
            | Self::OutboundWebSocketOpen
            | Self::OutboundWebSocketSend
            | Self::OutboundWebSocketClose => true,
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
            "kernel.v1.session.open" => Ok(Self::SessionOpen),
            "kernel.v1.session.close" => Ok(Self::SessionClose),
            "kernel.v1.session.fork" => Ok(Self::SessionFork),
            "kernel.v1.session.branch.list" => Ok(Self::SessionBranchList),
            "kernel.v1.session.get" => Ok(Self::SessionGet),
            "kernel.v1.session.list" => Ok(Self::SessionList),
            "kernel.v1.event.append" => Ok(Self::EventAppend),
            "kernel.v1.event.list" => Ok(Self::EventList),
            "kernel.v1.event.subscribe" => Ok(Self::EventSubscribe),
            "kernel.v1.package.load" => Ok(Self::PackageLoad),
            "kernel.v1.package.unload" => Ok(Self::PackageUnload),
            "kernel.v1.package.restart" => Ok(Self::PackageRestart),
            "kernel.v1.package.logs" => Ok(Self::PackageLogs),
            "kernel.v1.package.list" => Ok(Self::PackageList),
            "kernel.v1.package.status" => Ok(Self::PackageStatus),
            "kernel.v1.package.describe" => Ok(Self::PackageDescribe),
            "kernel.v1.project.list" => Ok(Self::ProjectList),
            "kernel.v1.project.get" => Ok(Self::ProjectGet),
            "kernel.v1.project.start" => Ok(Self::ProjectStart),
            "kernel.v1.project.stop" => Ok(Self::ProjectStop),
            "kernel.v1.project.status" => Ok(Self::ProjectStatus),
            "kernel.v1.capability.discover" => Ok(Self::CapabilityDiscover),
            "kernel.v1.capability.describe" => Ok(Self::CapabilityDescribe),
            "kernel.v1.capability.invoke" => Ok(Self::CapabilityInvoke),
            "kernel.v1.cap.attenuate" => Ok(Self::CapabilityHandleAttenuate),
            "kernel.v1.cap.revoke" => Ok(Self::CapabilityHandleRevoke),
            "kernel.v1.cap.list_for" => Ok(Self::CapabilityHandleListFor),
            "kernel.v1.capability.stream" => Ok(Self::CapabilityStream),
            "kernel.v1.capability.cancel" => Ok(Self::CapabilityCancel),
            "kernel.v1.extension_point.list" => Ok(Self::ExtensionPointList),
            "kernel.v1.extension_point.describe" => Ok(Self::ExtensionPointDescribe),
            "kernel.v1.hook.list" => Ok(Self::HookList),
            "kernel.v1.asset.put" => Ok(Self::AssetPut),
            "kernel.v1.asset.get" => Ok(Self::AssetGet),
            "kernel.v1.asset.list" => Ok(Self::AssetList),
            "kernel.v1.projection.register" => Ok(Self::ProjectionRegister),
            "kernel.v1.projection.rebuild" => Ok(Self::ProjectionRebuild),
            "kernel.v1.projection.get" => Ok(Self::ProjectionGet),
            "kernel.v1.projection.list" => Ok(Self::ProjectionList),
            "kernel.v1.host.info" => Ok(Self::HostInfo),
            "kernel.v1.host.ping" => Ok(Self::HostPing),
            "kernel.v1.host.diagnostics" => Ok(Self::HostDiagnostics),
            "kernel.v1.host.principal" => Ok(Self::HostPrincipal),
            "kernel.v1.permission.grant" => Ok(Self::PermissionGrant),
            "kernel.v1.permission.revoke" => Ok(Self::PermissionRevoke),
            "kernel.v1.permission.list" => Ok(Self::PermissionList),
            "kernel.v1.permission.audit" => Ok(Self::PermissionAudit),
            "kernel.v1.audit.package" => Ok(Self::AuditPackage),
            "kernel.v1.proposal.create" => Ok(Self::ProposalCreate),
            "kernel.v1.proposal.get" => Ok(Self::ProposalGet),
            "kernel.v1.proposal.list" => Ok(Self::ProposalList),
            "kernel.v1.proposal.approve" => Ok(Self::ProposalApprove),
            "kernel.v1.proposal.reject" => Ok(Self::ProposalReject),
            "kernel.v1.proposal.apply" => Ok(Self::ProposalApply),
            "kernel.v1.surface.resolve_bundle" => Ok(Self::SurfaceResolveBundle),
            "kernel.v1.surface.contribution.list" => Ok(Self::SurfaceContributionList),
            "kernel.v1.surface.contribution.describe" => Ok(Self::SurfaceContributionDescribe),
            "kernel.v1.outbound.audit" => Ok(Self::OutboundAudit),
            "kernel.v1.outbound.execute" => Ok(Self::OutboundExecute),
            "kernel.v1.outbound.stream" => Ok(Self::OutboundStream),
            "kernel.v1.outbound.websocket.open" => Ok(Self::OutboundWebSocketOpen),
            "kernel.v1.outbound.websocket.send" => Ok(Self::OutboundWebSocketSend),
            "kernel.v1.outbound.websocket.close" => Ok(Self::OutboundWebSocketClose),
            other => Err(format!("unknown kernel method: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Public protocol types (API-compatible)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProtocolMethod {
    pub id: &'static str,
    pub streaming: bool,
    pub status: MethodStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MethodStatus {
    Implemented,
    Partial,
    Planned,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProtocolPrincipal {
    HostAdmin,
    HostDev,
    Package {
        package_id: String,
    },
    Human {
        user_id: String,
    },
    Assistant {
        assistant_id: String,
        delegated_user_id: Option<String>,
    },
    Anonymous,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProtocolContext {
    pub principal: ProtocolPrincipal,
    pub transport: String,
    /// Optional kernel session id this call is operating under.
    /// Used by outbound dispatch to scope secret resolution to the
    /// session's project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default)]
    #[schemars(schema_with = "optional_uuid_schema")]
    pub correlation_id: Option<Uuid>,
    #[serde(default)]
    #[schemars(schema_with = "optional_uuid_schema")]
    pub parent_invocation_id: Option<Uuid>,
}

impl ProtocolContext {
    pub fn host_dev(transport: impl Into<String>) -> Self {
        Self {
            principal: ProtocolPrincipal::HostDev,
            transport: transport.into(),
            session_id: None,
            correlation_id: Some(Uuid::new_v4()),
            parent_invocation_id: None,
        }
    }

    pub fn package(package_id: impl Into<String>, transport: impl Into<String>) -> Self {
        Self {
            principal: ProtocolPrincipal::Package {
                package_id: package_id.into(),
            },
            transport: transport.into(),
            session_id: None,
            correlation_id: Some(Uuid::new_v4()),
            parent_invocation_id: None,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn effective_correlation_id(&self) -> Uuid {
        self.correlation_id.unwrap_or_else(Uuid::new_v4)
    }
}

fn optional_uuid_schema(_gen: &mut SchemaGenerator) -> Schema {
    let mut schema = SchemaObject::default();
    schema.instance_type = Some(SingleOrVec::Vec(vec![
        InstanceType::String,
        InstanceType::Null,
    ]));
    schema.format = Some("uuid".to_string());
    schema.metadata = Some(Box::new(Metadata::default()));
    Schema::Object(schema)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ProtocolRequest {
    pub id: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ProtocolResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ProtocolError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ProtocolError {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub details: Value,
}

impl ProtocolError {
    pub fn new(code: impl Into<String>, message: impl Into<String>, details: Value) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details,
        }
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new("kernel/v1/error/invalid_request", message, Value::Null)
    }

    pub fn from_anyhow(error: anyhow::Error) -> Self {
        let message = error.to_string();
        let code = if message.contains("not allowed") || message.contains("permission") {
            "kernel/v1/error/permission_denied"
        } else if message.contains("ambiguous") {
            "kernel/v1/error/ambiguous_route"
        } else if message.contains("schema")
            || message.contains("required")
            || message.contains("does not match")
        {
            "kernel/v1/error/schema_invalid"
        } else if message.contains("not loaded")
            || message.contains("not found")
            || message.contains("no provider")
        {
            "kernel/v1/error/not_found"
        } else if message.contains("closed")
            || message.contains("not ready")
            || message.contains("cannot execute")
        {
            "kernel/v1/error/package_state"
        } else {
            "kernel/v1/error/internal"
        };
        Self::new(code, message, Value::Null)
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq)]
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
    ProtocolMethod {
        id: "kernel.v1.session.open",
        streaming: false,
        status: MethodStatus::Implemented,
    },
    ProtocolMethod {
        id: "kernel.v1.session.close",
        streaming: false,
        status: MethodStatus::Implemented,
    },
    ProtocolMethod {
        id: "kernel.v1.session.fork",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.session.branch.list",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.session.get",
        streaming: false,
        status: MethodStatus::Planned,
    },
    ProtocolMethod {
        id: "kernel.v1.session.list",
        streaming: false,
        status: MethodStatus::Planned,
    },
    ProtocolMethod {
        id: "kernel.v1.event.append",
        streaming: false,
        status: MethodStatus::Implemented,
    },
    ProtocolMethod {
        id: "kernel.v1.event.list",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.event.subscribe",
        streaming: true,
        status: MethodStatus::Planned,
    },
    ProtocolMethod {
        id: "kernel.v1.package.load",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.package.unload",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.package.restart",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.package.logs",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.package.list",
        streaming: false,
        status: MethodStatus::Implemented,
    },
    ProtocolMethod {
        id: "kernel.v1.package.status",
        streaming: false,
        status: MethodStatus::Implemented,
    },
    ProtocolMethod {
        id: "kernel.v1.package.describe",
        streaming: false,
        status: MethodStatus::Planned,
    },
    ProtocolMethod {
        id: "kernel.v1.project.list",
        streaming: false,
        status: MethodStatus::Implemented,
    },
    ProtocolMethod {
        id: "kernel.v1.project.get",
        streaming: false,
        status: MethodStatus::Implemented,
    },
    ProtocolMethod {
        id: "kernel.v1.project.start",
        streaming: false,
        status: MethodStatus::Implemented,
    },
    ProtocolMethod {
        id: "kernel.v1.project.stop",
        streaming: false,
        status: MethodStatus::Implemented,
    },
    ProtocolMethod {
        id: "kernel.v1.project.status",
        streaming: false,
        status: MethodStatus::Implemented,
    },
    ProtocolMethod {
        id: "kernel.v1.capability.discover",
        streaming: false,
        status: MethodStatus::Implemented,
    },
    ProtocolMethod {
        id: "kernel.v1.capability.describe",
        streaming: false,
        status: MethodStatus::Planned,
    },
    ProtocolMethod {
        id: "kernel.v1.capability.invoke",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.cap.attenuate",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.cap.revoke",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.cap.list_for",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.capability.stream",
        streaming: true,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.capability.cancel",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.extension_point.list",
        streaming: false,
        status: MethodStatus::Implemented,
    },
    ProtocolMethod {
        id: "kernel.v1.extension_point.describe",
        streaming: false,
        status: MethodStatus::Planned,
    },
    ProtocolMethod {
        id: "kernel.v1.hook.list",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.asset.put",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.asset.get",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.asset.list",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.projection.register",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.projection.rebuild",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.projection.get",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.projection.list",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.host.info",
        streaming: false,
        status: MethodStatus::Implemented,
    },
    ProtocolMethod {
        id: "kernel.v1.host.ping",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.host.diagnostics",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.host.principal",
        streaming: false,
        status: MethodStatus::Planned,
    },
    ProtocolMethod {
        id: "kernel.v1.permission.grant",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.permission.revoke",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.permission.list",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.permission.audit",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.audit.package",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.proposal.create",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.proposal.get",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.proposal.list",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.proposal.approve",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.proposal.reject",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.proposal.apply",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.surface.resolve_bundle",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.surface.contribution.list",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.surface.contribution.describe",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.outbound.audit",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.outbound.execute",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.outbound.stream",
        streaming: true,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.outbound.websocket.open",
        streaming: true,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.outbound.websocket.send",
        streaming: false,
        status: MethodStatus::Partial,
    },
    ProtocolMethod {
        id: "kernel.v1.outbound.websocket.close",
        streaming: false,
        status: MethodStatus::Partial,
    },
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
            "kernel.v1.session.open",
            "kernel.v1.session.list",
            "kernel.v1.event.subscribe",
            "kernel.v1.package.describe",
            "kernel.v1.capability.cancel",
            "kernel.v1.asset.put",
            "kernel.v1.host.principal",
        ] {
            assert!(ids.contains(&expected), "missing {expected}");
        }
    }

    #[test]
    fn protocol_context_serializes_session_id_when_set() {
        let ctx = ProtocolContext {
            principal: ProtocolPrincipal::HostAdmin,
            transport: "http".into(),
            session_id: Some("session-abc".into()),
            correlation_id: None,
            parent_invocation_id: None,
        };
        let json = serde_json::to_string(&ctx).unwrap();
        assert!(json.contains("\"session_id\":\"session-abc\""));
    }

    #[test]
    fn protocol_context_omits_session_id_when_none() {
        let ctx = ProtocolContext {
            principal: ProtocolPrincipal::HostAdmin,
            transport: "http".into(),
            session_id: None,
            correlation_id: None,
            parent_invocation_id: None,
        };
        let json = serde_json::to_string(&ctx).unwrap();
        assert!(!json.contains("session_id"));
    }

    #[test]
    fn protocol_context_deserializes_without_session_id() {
        let json = r#"{"principal":{"kind":"host_admin"},"transport":"http"}"#;
        let ctx: ProtocolContext = serde_json::from_str(json).unwrap();
        assert!(ctx.session_id.is_none());
    }

    // --- KernelMethod / registry alignment tests ---

    #[test]
    fn every_registry_id_parses_to_kernel_method() {
        for method in KERNEL_METHODS {
            let parsed: Result<KernelMethod, String> = method.id.parse();
            assert!(
                parsed.is_ok(),
                "registry id '{}' does not parse to KernelMethod",
                method.id
            );
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
            assert_eq!(
                method.streaming,
                km.streaming(),
                "streaming mismatch for {:?}",
                km
            );
            assert_eq!(method.status, km.status(), "status mismatch for {:?}", km);
        }
    }

    #[test]
    fn no_duplicate_ids_in_all() {
        let all = KernelMethod::all();
        let ids: Vec<&'static str> = all.iter().map(|m| m.id()).collect();
        let unique: std::collections::HashSet<&'static str> = ids.iter().copied().collect();
        assert_eq!(
            ids.len(),
            unique.len(),
            "KernelMethod::all() contains duplicate ids"
        );
    }

    #[test]
    fn session_close_is_implemented_and_dispatched() {
        let km = KernelMethod::SessionClose;
        assert_eq!(km.id(), "kernel.v1.session.close");
        assert_eq!(km.status(), MethodStatus::Implemented);
        assert!(
            km.is_dispatched(),
            "kernel.v1.session.close must be dispatch-covered"
        );
    }

    #[test]
    fn hook_list_status_matches_dispatch() {
        let km = KernelMethod::HookList;
        assert_eq!(km.id(), "kernel.v1.hook.list");
        // Was previously Planned, but dispatch exists → must be at least Partial
        assert!(
            matches!(km.status(), MethodStatus::Implemented | MethodStatus::Partial),
            "kernel.v1.hook.list status must be Implemented or Partial since dispatch exists, got {:?}",
            km.status()
        );
        assert!(
            km.is_dispatched(),
            "kernel.v1.hook.list must be dispatch-covered"
        );
    }

    #[test]
    fn implemented_or_partial_methods_must_be_dispatched() {
        for method in KERNEL_METHODS {
            let km: KernelMethod = method.id.parse().unwrap();
            if matches!(
                km.status(),
                MethodStatus::Implemented | MethodStatus::Partial
            ) {
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
            assert_eq!(
                *km, parsed,
                "Display -> FromStr roundtrip failed for {:?}",
                km
            );
        }
    }
}
