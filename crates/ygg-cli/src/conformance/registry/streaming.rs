use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- streaming ---
        c!(
            "stream.normal_lifecycle",
            ["stream"],
            crate::conformance::streaming::stream_normal_lifecycle
        ),
        c!(
            "stream.cancel_blocks_chunks",
            ["stream"],
            crate::conformance::streaming::stream_cancel_blocks_chunks
        ),
        c!(
            "stream.timeout_blocks_chunks",
            ["stream"],
            crate::conformance::streaming::stream_timeout_blocks_chunks
        ),
        c!(
            "stream.error_terminal",
            ["stream"],
            crate::conformance::streaming::stream_error_terminal
        ),
        c!(
            "stream.non_streaming_rejected",
            ["stream"],
            crate::conformance::streaming::stream_non_streaming_rejected
        ),
        c!(
            "stream.no_model_agent_methods",
            ["stream"],
            crate::conformance::streaming::stream_no_model_agent_methods
        ),
        c!(
            "stream.protocol_dispatch",
            ["stream", "protocol"],
            crate::conformance::streaming::stream_protocol_dispatch
        ),
    ]
}
