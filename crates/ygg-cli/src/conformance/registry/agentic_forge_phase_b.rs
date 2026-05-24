use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- agentic forge Phase B ---
        c!(
            "agentic_forge.create_candidate_branch_aware",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_create_candidate
        ),
        c!(
            "agentic_forge.compare_candidate_stale_detection",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_compare_candidate
        ),
        c!(
            "agentic_forge.draft_promote_proposal_no_mutation",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_draft_promote_proposal
        ),
        c!(
            "agentic_forge.stale_promote_blocked",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_stale_promote_blocked
        ),
        c!(
            "agentic_forge.archive_candidate_target_unchanged",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_archive_candidate
        ),
    ]
}
