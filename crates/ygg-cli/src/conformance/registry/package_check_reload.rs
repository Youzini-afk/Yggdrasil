use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- package check / reload ---
        c!(
            "package.check_diagnostics",
            ["subprocess", "package", "slow"],
            crate::conformance::subprocess::package_check_diagnostics
        ),
        c!(
            "package.reload_smoke",
            ["subprocess", "package", "slow"],
            crate::conformance::subprocess::package_reload_smoke
        ),
    ]
}
