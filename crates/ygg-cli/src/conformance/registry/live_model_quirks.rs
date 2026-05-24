use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- live model quirks ---
        c!(
            "outbound.openrouter_loopback_headers",
            ["outbound", "network", "live"],
            crate::conformance::live_model::openrouter_loopback_headers
        ),
        c!(
            "outbound.xai_loopback",
            ["outbound", "network", "live"],
            crate::conformance::live_model::xai_loopback
        ),
        c!(
            "outbound.fireworks_loopback",
            ["outbound", "network", "live"],
            crate::conformance::live_model::fireworks_loopback
        ),
        c!(
            "stream.deepseek_reasoning_stream",
            ["stream", "live"],
            crate::conformance::live_model::deepseek_reasoning_stream
        ),
        c!(
            "stream.openrouter_midstream_error",
            ["stream", "live"],
            crate::conformance::live_model::openrouter_midstream_error
        ),
        c!(
            "outbound.provider_quirk_fixtures_no_secrets",
            ["outbound", "network", "live", "secret"],
            crate::conformance::live_model::provider_quirk_fixtures_no_secrets
        ),
        c!(
            "outbound.static_headers_openrouter_safe",
            ["outbound", "network", "live"],
            crate::conformance::live_model::static_headers_openrouter_safe
        ),
    ]
}
