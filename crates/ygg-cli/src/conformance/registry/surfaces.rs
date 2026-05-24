use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- surfaces ---
        c!(
            "surface.contribution_list",
            ["surface"],
            crate::conformance::surfaces::contribution_list
        ),
    ]
}
