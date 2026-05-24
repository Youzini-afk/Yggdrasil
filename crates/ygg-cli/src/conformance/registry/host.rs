use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- host ---
        c!(
            "host.diagnostics",
            ["runtime", "host"],
            crate::conformance::core::host_diagnostics
        ),
        c!(
            "host.profile_autoload",
            ["runtime", "host", "slow"],
            crate::conformance::core::host_profile_autoload
        ),
    ]
}
