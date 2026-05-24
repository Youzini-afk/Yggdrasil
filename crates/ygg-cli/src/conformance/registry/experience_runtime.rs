use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- experience runtime ---
        c!(
            "experience_runtime.describe_contract_shape",
            ["experience"],
            crate::conformance::experience_runtime::experience_runtime_describe_contract
        ),
        c!(
            "experience_runtime.checkpoint_recovery_shape",
            ["experience"],
            crate::conformance::experience_runtime::experience_runtime_checkpoint_shape
        ),
        c!(
            "experience_runtime.recovery_shape",
            ["experience"],
            crate::conformance::experience_runtime::experience_runtime_recovery_shape
        ),
        c!(
            "experience_runtime.no_kernel_experience_namespace",
            ["experience"],
            crate::conformance::experience_runtime::experience_runtime_no_kernel_namespace
        ),
        c!(
            "experience_runtime.template_generation",
            ["experience", "generated"],
            crate::conformance::experience_runtime::experience_runtime_template_generation
        ),
        c!(
            "experience_runtime.bind_agent_run_shape",
            ["experience", "agentic"],
            crate::conformance::experience_runtime::experience_runtime_bind_agent_run
        ),
    ]
}
