use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- proposals ---
        c!(
            "proposal.lifecycle_apply",
            ["runtime", "proposal"],
            crate::conformance::proposals::lifecycle_apply
        ),
        c!(
            "proposal.reject_and_apply_denied",
            ["runtime", "proposal"],
            crate::conformance::proposals::reject_and_apply_denied
        ),
    ]
}
