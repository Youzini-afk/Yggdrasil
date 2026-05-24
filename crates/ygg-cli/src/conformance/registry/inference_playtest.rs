use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- inference playtest ---
        c!(
            "official.inference_playtest_lab_draft",
            ["official", "slow"],
            crate::conformance::inference_playtest::inference_playtest_draft
        ),
        c!(
            "official.inference_playtest_lab_inspect",
            ["official", "slow"],
            crate::conformance::inference_playtest::inference_playtest_inspect
        ),
        c!(
            "official.inference_playtest_lab_reject_apply_denied",
            ["official", "slow"],
            crate::conformance::inference_playtest::inference_playtest_reject_apply_denied
        ),
        c!(
            "official.inference_playtest_lab_apply_and_branch",
            ["official", "slow"],
            crate::conformance::inference_playtest::inference_playtest_apply_and_branch
        ),
        c!(
            "official.inference_playtest_lab_no_chat_kernel_terms",
            ["official", "slow"],
            crate::conformance::inference_playtest::inference_playtest_no_chat_kernel_terms
        ),
    ]
}
