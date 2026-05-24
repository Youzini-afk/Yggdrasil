use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- inproc ---
        c!(
            "inproc.non_official_preview_rejected",
            ["runtime"],
            crate::conformance::inproc::non_official_preview_rejected
        ),
        c!(
            "inproc.unknown_capability_errors",
            ["runtime"],
            crate::conformance::inproc::unknown_inproc_capability_errors
        ),
        c!(
            "inproc.bindings_init",
            ["runtime", "binding"],
            crate::conformance::inproc::inproc_bindings_init_receives_manifest_bindings
        ),
    ]
}
