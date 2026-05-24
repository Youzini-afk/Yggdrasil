use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- network ---
        c!(
            "network.no_permission_denied",
            ["network", "outbound"],
            crate::conformance::network::no_network_permission_denied
        ),
        c!(
            "network.allowlisted_host_method_allowed",
            ["network", "outbound"],
            crate::conformance::network::allowlisted_host_method_allowed
        ),
        c!(
            "network.host_method_mismatch_denied",
            ["network", "outbound"],
            crate::conformance::network::host_method_mismatch_denied
        ),
        c!(
            "network.official_no_network_bypass",
            ["network", "outbound"],
            crate::conformance::network::official_no_network_bypass
        ),
        c!(
            "network.audit_no_raw_secrets",
            ["network", "outbound"],
            crate::conformance::network::audit_no_raw_secrets
        ),
        c!(
            "network.policy_pure_function",
            ["network", "outbound"],
            crate::conformance::network::network_policy_pure_function
        ),
    ]
}
