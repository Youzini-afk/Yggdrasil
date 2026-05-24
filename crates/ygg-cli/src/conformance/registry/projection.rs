use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- projection ---
        c!(
            "projection.rebuild",
            ["runtime", "projection"],
            crate::conformance::core::projection_rebuild
        ),
    ]
}
