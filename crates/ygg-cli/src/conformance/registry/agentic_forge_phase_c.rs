use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- agentic forge Phase C ---
        c!(
            "agentic_forge.inference_node_deterministic_candidate_seed",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_inference_node_deterministic
        ),
        c!(
            "agentic_forge.replay_match_mismatch_flagged",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_replay_match_mismatch
        ),
        c!(
            "agentic_forge.inference_output_privilege_escalation_rejected",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_inference_output_validation
        ),
        c!(
            "agentic_forge.cloud_adapter_needs_host_policy_no_network",
            ["agentic", "network"],
            crate::conformance::agentic_forge::agentic_forge_cloud_adapter_no_network
        ),
        c!(
            "agentic_forge.inference_failure_taxonomy_recovery_hints",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_inference_failure_taxonomy
        ),
    ]
}
