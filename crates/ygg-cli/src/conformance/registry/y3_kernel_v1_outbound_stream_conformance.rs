use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- Y3: kernel.v1.outbound.stream conformance ---
        c!(
            "outbound_stream.profile_default_deny_all",
            ["outbound", "network", "stream", "profile"],
            crate::conformance::network::outbound_stream_profile_default_deny_all
        ),
        c!(
            "outbound_stream.fake_executor_emits_canned_frames",
            ["outbound", "network", "stream"],
            crate::conformance::network::outbound_stream_fake_executor_emits_canned_frames
        ),
        c!(
            "outbound_stream.secret_ref_undeclared_fails",
            ["outbound", "network", "stream", "secret", "manifest"],
            crate::conformance::network::outbound_stream_secret_ref_undeclared_fails
        ),
        c!(
            "outbound_stream.secret_ref_declared_resolves",
            ["outbound", "network", "stream", "secret", "manifest"],
            crate::conformance::network::outbound_stream_secret_ref_declared_resolves
        ),
        c!(
            "outbound_stream.capability_namespace_enforced",
            ["outbound", "network", "stream", "permission"],
            crate::conformance::network::outbound_stream_capability_namespace_enforced
        ),
        c!(
            "outbound_stream.https_only",
            ["outbound", "network", "stream", "profile"],
            crate::conformance::network::outbound_stream_https_only
        ),
        c!(
            "subprocess.reverse_kernel_call_dispatched",
            ["subprocess", "outbound", "network"],
            crate::conformance::network::subprocess_reverse_kernel_call_dispatched
        ),
        c!(
            "subprocess.reverse_kernel_call_principal_locked",
            ["subprocess", "outbound", "network"],
            crate::conformance::network::subprocess_reverse_kernel_call_principal_locked
        ),
        c!(
            "subprocess.reverse_stream_chunks_piped",
            ["subprocess", "outbound", "network", "stream"],
            crate::conformance::network::subprocess_reverse_stream_chunks_piped
        ),
        c!(
            "sse_parser.basic_smoke",
            ["outbound", "network", "stream", "sse"],
            crate::conformance::network::sse_parser_basic_smoke
        ),
        c!(
            "sse_parser.partial_chunks",
            ["outbound", "network", "stream", "sse"],
            crate::conformance::network::sse_parser_partial_chunks
        ),
    ]
}
