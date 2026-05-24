use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- agentic forge Phase F ---
        c!(
            "agentic_forge.thirdparty_replacement_shape_no_official_priority",
            ["agentic", "replacement"],
            crate::conformance::agentic_forge::agentic_forge_thirdparty_replacement_shape
        ),
        c!(
            "agentic_forge.no_official_priority_ordinary_package",
            ["agentic", "official"],
            crate::conformance::agentic_forge::agentic_forge_no_official_priority
        ),
        c!(
            "agentic_forge.hostile_injection_secret_blocked_cross_package",
            ["agentic", "secret"],
            crate::conformance::agentic_forge::agentic_forge_hostile_injection_secret_blocked
        ),
        c!(
            "agentic_forge.budget_deadline_contract_cancellation_consistent",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_budget_deadline_contract
        ),
        c!(
            "agentic_forge.cross_package_replay_mismatch_flagged",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_cross_package_replay_consistency
        ),
    ]
}
