use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- agentic forge Phase D ---
        c!(
            "agentic_forge.explain_tool_call_scoped_no_ambient_authority",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_explain_tool_call_scoped
        ),
        c!(
            "agentic_forge.record_observation_untrusted_large_output_redaction",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_record_observation_untrusted
        ),
        c!(
            "agentic_forge.tool_risk_injection_exfiltration_outbound",
            ["agentic", "network"],
            crate::conformance::agentic_forge::agentic_forge_tool_risk_categories
        ),
        c!(
            "agentic_forge.replay_tool_plan_mismatch_flagged",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_replay_tool_mismatch
        ),
        c!(
            "agentic_forge.plan_toolchain_requires_explicit_provider_nested_delegation_blocked",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_plan_toolchain_requires_provider
        ),
    ]
}
