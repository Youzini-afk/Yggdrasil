use serde::{Deserialize, Serialize};

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

pub const KERNEL_PROTOCOL_VERSION: &str = "0.1.0";

pub const KERNEL_METHODS: &[ProtocolMethod] = &[
    ProtocolMethod { id: "kernel.session.open", streaming: false, status: MethodStatus::Implemented },
    ProtocolMethod { id: "kernel.session.close", streaming: false, status: MethodStatus::Implemented },
    ProtocolMethod { id: "kernel.session.get", streaming: false, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.session.list", streaming: false, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.event.append", streaming: false, status: MethodStatus::Implemented },
    ProtocolMethod { id: "kernel.event.list", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.event.subscribe", streaming: true, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.package.load", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.package.unload", streaming: false, status: MethodStatus::Partial },
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
    ProtocolMethod { id: "kernel.asset.put", streaming: false, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.asset.get", streaming: false, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.asset.list", streaming: false, status: MethodStatus::Planned },
    ProtocolMethod { id: "kernel.host.info", streaming: false, status: MethodStatus::Implemented },
    ProtocolMethod { id: "kernel.host.ping", streaming: false, status: MethodStatus::Partial },
    ProtocolMethod { id: "kernel.host.principal", streaming: false, status: MethodStatus::Planned },
];

pub fn method_ids() -> Vec<&'static str> {
    KERNEL_METHODS.iter().map(|method| method.id).collect()
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
