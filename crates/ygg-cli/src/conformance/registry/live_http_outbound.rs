use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- live http outbound ---
        c!(
            "outbound.live_http_default_disabled",
            ["outbound", "network", "live"],
            crate::conformance::network::outbound_live_http_default_disabled
        ),
        c!(
            "outbound.live_http_rejects_insecure_url",
            ["outbound", "network", "live"],
            crate::conformance::network::outbound_live_http_rejects_insecure_url
        ),
        c!(
            "outbound.live_http_redacted_shape",
            ["outbound", "network", "live"],
            crate::conformance::network::outbound_live_http_redacted_shape
        ),
    ]
}
