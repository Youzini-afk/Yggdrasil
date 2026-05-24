use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- kernel.v1.outbound.execute ---
        c!(
            "outbound.execute_package_allowed",
            ["outbound", "network"],
            crate::conformance::network::outbound_execute_package_allowed
        ),
        c!(
            "outbound.execute_spoofed_package_id_rejected",
            ["outbound", "network"],
            crate::conformance::network::outbound_execute_spoofed_package_id_rejected
        ),
        c!(
            "outbound.execute_no_permission_denied",
            ["outbound", "network"],
            crate::conformance::network::outbound_execute_no_permission_denied
        ),
        c!(
            "outbound.execute_no_raw_secret_in_response",
            ["outbound", "network", "secret"],
            crate::conformance::network::outbound_execute_no_raw_secret_in_response
        ),
    ]
}
