use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- generated packages ---
        c!(
            "package.generated_subprocess_conformance",
            ["generated", "slow"],
            crate::conformance::generated::generated_subprocess_package
        ),
        c!(
            "package.generated_typescript_subprocess_conformance",
            ["generated", "slow"],
            crate::conformance::generated::generated_typescript_subprocess_package
        ),
        c!(
            "package.generated_experience_template",
            ["generated", "slow"],
            crate::conformance::generated::generated_experience_template
        ),
        c!(
            "package.generated_basic_template",
            ["generated", "slow"],
            crate::conformance::generated::generated_basic_template
        ),
        c!(
            "package.generated_explicit_experience_template",
            ["generated", "slow"],
            crate::conformance::generated::generated_explicit_experience_template
        ),
        c!(
            "package.generated_assistant_action_template",
            ["generated", "slow"],
            crate::conformance::generated::generated_assistant_action_template
        ),
        c!(
            "package.generated_asset_editor_template",
            ["generated", "slow"],
            crate::conformance::generated::generated_asset_editor_template
        ),
        c!(
            "package.generated_full_surface_template",
            ["generated", "slow"],
            crate::conformance::generated::generated_full_surface_template
        ),
        c!(
            "package.generated_networked_template",
            ["generated", "network", "slow"],
            crate::conformance::generated::generated_networked_template
        ),
        c!(
            "package.generated_streaming_template",
            ["generated", "stream", "slow"],
            crate::conformance::generated::generated_streaming_template
        ),
        c!(
            "package.generated_agent_runtime_template",
            ["generated", "agentic", "slow"],
            crate::conformance::generated::generated_agent_runtime_template
        ),
        c!(
            "package.generated_experience_runtime_template",
            ["generated", "experience", "slow"],
            crate::conformance::generated::generated_experience_runtime_template
        ),
        c!(
            "package.faux_model_readiness",
            ["generated", "slow"],
            crate::conformance::generated::faux_model_readiness_package
        ),
        c!(
            "package.faux_agent_readiness",
            ["generated", "agentic", "slow"],
            crate::conformance::generated::faux_agent_readiness_package
        ),
    ]
}
