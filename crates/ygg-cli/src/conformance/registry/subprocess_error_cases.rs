use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- subprocess error cases ---
        c!(
            "subprocess.bad_handshake",
            ["subprocess", "slow"],
            crate::conformance::subprocess::subprocess_bad_handshake
        ),
        c!(
            "subprocess.invoke_timeout",
            ["subprocess", "slow"],
            crate::conformance::subprocess::subprocess_timeout
        ),
        c!(
            "subprocess.invalid_output_schema",
            ["subprocess", "slow"],
            crate::conformance::subprocess::subprocess_invalid_output_schema
        ),
        c!(
            "subprocess.unload_removes_capability",
            ["subprocess", "slow"],
            crate::conformance::subprocess::subprocess_unload_removes_capability
        ),
    ]
}
