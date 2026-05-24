use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- live model ---
        c!(
            "outbound.secret_headers_parsed",
            ["outbound", "network", "live", "secret"],
            crate::conformance::live_model::outbound_secret_headers_parsed
        ),
        c!(
            "outbound.live_loopback_secret_injection",
            ["outbound", "network", "live", "secret"],
            crate::conformance::live_model::outbound_live_loopback_secret_injection
        ),
        c!(
            "stream.sse_normalize_deepseek_canary",
            ["stream", "live"],
            crate::conformance::live_model::stream_sse_normalize_deepseek_canary
        ),
        c!(
            "live_model.default_disabled_when_env_unset",
            ["live", "outbound", "network"],
            crate::conformance::live_model::live_model_default_disabled_when_env_unset
        ),
        c!(
            "live_model.smoke_skipped_in_default_run",
            ["live", "outbound", "network"],
            crate::conformance::live_model::live_model_smoke_skipped_in_default_run
        ),
        c!(
            "outbound.live_deepseek_opt_in",
            ["outbound", "network", "live"],
            crate::conformance::live_model::outbound_live_deepseek_opt_in
        ),
        c!(
            "canary.deepseek_profile_shape",
            ["live", "outbound"],
            crate::conformance::live_model::canary_deepseek_profile_shape
        ),
    ]
}
