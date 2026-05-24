use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- official foundation / labs ---
        c!(
            "official.foundation_packages",
            ["official", "slow"],
            crate::conformance::official_foundation::foundation_packages
        ),
        c!(
            "official.assistant_lab_proposal",
            ["official", "slow"],
            crate::conformance::official_labs::assistant_lab_proposal
        ),
        c!(
            "play_creation.blank_loop",
            ["official", "slow"],
            crate::conformance::official_play_creation::blank_play_creation_loop
        ),
    ]
}
