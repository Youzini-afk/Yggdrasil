use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- substrate ---
        c!(
            "substrate.sqlite_rehydrate",
            ["substrate", "slow"],
            crate::conformance::substrate::sqlite_rehydrate
        ),
    ]
}
