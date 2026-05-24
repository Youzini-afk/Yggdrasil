use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- live model providers ---
        c!(
            "outbound.openai_chat_loopback",
            ["outbound", "network", "live"],
            crate::conformance::live_model::openai_chat_loopback
        ),
        c!(
            "outbound.openai_responses_loopback",
            ["outbound", "network", "live"],
            crate::conformance::live_model::openai_responses_loopback
        ),
        c!(
            "outbound.anthropic_messages_loopback",
            ["outbound", "network", "live"],
            crate::conformance::live_model::anthropic_messages_loopback
        ),
        c!(
            "outbound.gemini_generate_content_loopback",
            ["outbound", "network", "live"],
            crate::conformance::live_model::gemini_generate_content_loopback
        ),
        c!(
            "outbound.missing_secret_fails_closed",
            ["outbound", "network", "live", "secret"],
            crate::conformance::live_model::missing_secret_fails_closed
        ),
        c!(
            "outbound.provider_normalize_request_alignment",
            ["outbound", "network", "live"],
            crate::conformance::live_model::provider_normalize_request_alignment
        ),
        c!(
            "outbound.no_raw_secret_leak_all_providers",
            ["outbound", "network", "live", "secret"],
            crate::conformance::live_model::no_raw_secret_leak_all_providers
        ),
        c!(
            "outbound.static_headers_safe_allowlist",
            ["outbound", "network", "live"],
            crate::conformance::live_model::static_headers_safe_allowlist
        ),
        c!(
            "outbound.static_headers_block_secrets",
            ["outbound", "network", "live", "secret"],
            crate::conformance::live_model::static_headers_block_secrets
        ),
    ]
}
