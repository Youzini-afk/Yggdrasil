use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- outbound ---
        c!(
            "outbound.no_permission_executor_not_called",
            ["outbound", "network"],
            crate::conformance::network::outbound_no_permission_executor_not_called
        ),
        c!(
            "outbound.policy_executor_mismatch_denied",
            ["outbound", "network"],
            crate::conformance::network::outbound_policy_executor_mismatch_denied
        ),
        c!(
            "outbound.allowlisted_fake_executor",
            ["outbound", "network"],
            crate::conformance::network::outbound_allowlisted_fake_executor
        ),
        c!(
            "outbound.raw_body_not_audited",
            ["outbound", "network"],
            crate::conformance::network::outbound_raw_body_not_audited
        ),
        c!(
            "outbound.secret_refs_only",
            ["outbound", "network", "secret"],
            crate::conformance::network::outbound_secret_refs_only
        ),
        c!(
            "outbound.host_mismatch_redirect_denied",
            ["outbound", "network"],
            crate::conformance::network::outbound_host_mismatch_redirect_denied
        ),
        c!(
            "outbound.model_provider_shape_fake_executor",
            ["outbound", "network"],
            crate::conformance::network::outbound_model_provider_shape_fake_executor
        ),
    ]
}
