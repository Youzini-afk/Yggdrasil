use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- replacement ---
        c!(
            "replacement.thirdparty_seed_surfaces",
            ["replacement"],
            crate::conformance::replacement::thirdparty_seed_surfaces
        ),
        c!(
            "replacement.thirdparty_seed_invocation",
            ["replacement"],
            crate::conformance::replacement::thirdparty_seed_invocation
        ),
        c!(
            "replacement.ambiguous_no_official_priority",
            ["replacement"],
            crate::conformance::replacement::ambiguous_no_official_priority
        ),
        c!(
            "replacement.composition_thirdparty",
            ["replacement", "composition"],
            crate::conformance::replacement::composition_thirdparty
        ),
        c!(
            "replacement.thirdparty_agent_runtime_surfaces",
            ["replacement", "agentic"],
            crate::conformance::replacement::thirdparty_agent_runtime_surfaces
        ),
        c!(
            "replacement.thirdparty_agent_runtime_invocation",
            ["replacement", "agentic"],
            crate::conformance::replacement::thirdparty_agent_runtime_invocation
        ),
        c!(
            "replacement.composition_agent_runtime_replacement",
            ["replacement", "agentic", "composition"],
            crate::conformance::replacement::composition_agent_runtime_replacement
        ),
    ]
}
