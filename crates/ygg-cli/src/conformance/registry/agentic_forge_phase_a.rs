use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- agentic forge Phase A ---
        c!(
            "agentic_forge.describe_contract",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_describe_contract
        ),
        c!(
            "agentic_forge.start_run_plan_graph_working_state",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_start_run
        ),
        c!(
            "agentic_forge.inspect_cancel_summarize",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_inspect_cancel_summarize
        ),
        c!(
            "agentic_forge.raw_secret_blocked",
            ["agentic", "secret"],
            crate::conformance::agentic_forge::agentic_forge_raw_secret_blocked
        ),
        c!(
            "agentic_forge.no_kernel_agent_namespace",
            ["agentic"],
            crate::conformance::agentic_forge::agentic_forge_no_kernel_agent_namespace
        ),
    ]
}
