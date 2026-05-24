use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- composition ---
        c!(
            "composition.check_descriptor",
            ["composition"],
            crate::conformance::generated::composition_descriptor
        ),
        c!(
            "composition.check_descriptor_v2",
            ["composition"],
            crate::conformance::generated::composition_descriptor_v2
        ),
        c!(
            "official.composition_lab",
            ["official", "composition", "slow"],
            crate::conformance::official_labs::composition_lab
        ),
        c!(
            "official.composition_lab_diagnostics",
            ["official", "composition", "slow"],
            crate::conformance::official_labs::composition_lab_diagnostics
        ),
        c!(
            "official.asset_lab",
            ["official", "slow"],
            crate::conformance::official_labs::asset_lab
        ),
        c!(
            "official.projection_lab",
            ["official", "slow"],
            crate::conformance::official_labs::projection_lab
        ),
        c!(
            "official.playable_seed",
            ["official", "slow"],
            crate::conformance::official_labs::playable_seed
        ),
        c!(
            "official.persona_lab",
            ["official", "slow"],
            crate::conformance::official_labs::persona_lab
        ),
        c!(
            "official.knowledge_lab",
            ["official", "slow"],
            crate::conformance::official_labs::knowledge_lab
        ),
        c!(
            "official.context_lab",
            ["official", "slow"],
            crate::conformance::official_labs::context_lab
        ),
        c!(
            "official.text_transform_lab",
            ["official", "slow"],
            crate::conformance::official_labs::text_transform_lab
        ),
        c!(
            "official.model_connector_lab",
            ["official", "slow"],
            crate::conformance::official_labs::model_connector_lab
        ),
        c!(
            "official.model_provider_lab",
            ["official", "slow"],
            crate::conformance::official_labs::model_provider_lab
        ),
        c!(
            "official.model_provider_lab_invoke_core",
            ["official", "slow"],
            crate::conformance::official_labs::model_provider_lab_invoke_core
        ),
        c!(
            "official.model_provider_lab_normalize_stream",
            ["official", "slow"],
            crate::conformance::official_labs::model_provider_lab_normalize_stream
        ),
        c!(
            "official.model_routing_lab",
            ["official", "slow"],
            crate::conformance::official_labs::model_routing_lab
        ),
        c!(
            "official.pi_agent_runtime_lab",
            ["official", "agentic", "slow"],
            crate::conformance::official_labs::pi_agent_runtime_lab
        ),
        c!(
            "official.capability_tool_bridge_lab",
            ["official", "agentic", "slow"],
            crate::conformance::official_labs::capability_tool_bridge_lab
        ),
    ]
}
