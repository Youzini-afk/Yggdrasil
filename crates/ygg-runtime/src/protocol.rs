use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolMethod {
    pub id: &'static str,
    pub streaming: bool,
}

pub const KERNEL_PROTOCOL_VERSION: &str = "0.1.0";

pub const KERNEL_METHODS: &[ProtocolMethod] = &[
    ProtocolMethod { id: "kernel.session.open", streaming: false },
    ProtocolMethod { id: "kernel.session.close", streaming: false },
    ProtocolMethod { id: "kernel.event.append", streaming: false },
    ProtocolMethod { id: "kernel.event.list", streaming: false },
    ProtocolMethod { id: "kernel.event.subscribe", streaming: true },
    ProtocolMethod { id: "kernel.package.load", streaming: false },
    ProtocolMethod { id: "kernel.package.unload", streaming: false },
    ProtocolMethod { id: "kernel.package.list", streaming: false },
    ProtocolMethod { id: "kernel.package.status", streaming: false },
    ProtocolMethod { id: "kernel.capability.discover", streaming: false },
    ProtocolMethod { id: "kernel.capability.invoke", streaming: false },
    ProtocolMethod { id: "kernel.extension_point.list", streaming: false },
    ProtocolMethod { id: "kernel.host.info", streaming: false },
    ProtocolMethod { id: "kernel.host.ping", streaming: false },
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
}
