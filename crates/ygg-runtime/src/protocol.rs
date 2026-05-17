use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    ProtocolMethod { id: "kernel.capability.stream", streaming: true, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.capability.cancel", streaming: false, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.extension_point.list", streaming: false, status: MethodStatus::Implemented },
    ProtocolMethod { id: "kernel.extension_point.describe", streaming: false, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.hook.list", streaming: false, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.asset.put", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.asset.get", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.asset.list", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.projection.register", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.projection.rebuild", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.projection.get", streaming: false, status: MethodStatus::Partial },
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
}
