use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- creator loop Beta 5 ---
        c!(
            "creator_loop.playable_board_template",
            ["experience", "generated"],
            crate::conformance::creator_loop::creator_loop_playable_board_template
        ),
        c!(
            "creator_loop.playable_experience_template",
            ["experience", "generated"],
            crate::conformance::creator_loop::creator_loop_playable_experience_template
        ),
        c!(
            "creator_loop.experience_surface_warnings",
            ["experience", "generated"],
            crate::conformance::creator_loop::creator_loop_experience_surface_warnings
        ),
        c!(
            "creator_loop.missing_checkpoint_warning",
            ["experience", "generated"],
            crate::conformance::creator_loop::creator_loop_missing_checkpoint_warning
        ),
        c!(
            "creator_loop.dangerous_permissions_warning",
            ["experience", "generated"],
            crate::conformance::creator_loop::creator_loop_dangerous_permissions_warning
        ),
        c!(
            "creator_loop.network_nondeterministic_hint",
            ["experience", "generated", "network"],
            crate::conformance::creator_loop::creator_loop_network_nondeterministic_hint
        ),
        c!(
            "creator_loop.composition_experience_diagnostics",
            ["experience", "composition", "generated"],
            crate::conformance::creator_loop::creator_loop_composition_experience_diagnostics
        ),
        c!(
            "creator_loop.walkthrough_reference",
            ["experience", "generated"],
            crate::conformance::creator_loop::creator_loop_walkthrough_reference
        ),
        c!(
            "creator_loop.thirdparty_no_privilege",
            ["experience", "replacement", "generated"],
            crate::conformance::creator_loop::creator_loop_thirdparty_no_privilege
        ),
    ]
}
