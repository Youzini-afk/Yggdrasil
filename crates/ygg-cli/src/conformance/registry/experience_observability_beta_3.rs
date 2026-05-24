use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
    // --- experience observability Beta 3 ---
    c!(
        "experience_observability.contract_shape",
        ["experience"],
        crate::conformance::experience_observability::experience_observability_contract
    ),
    c!(
        "experience_observability.session_health",
        ["experience"],
        crate::conformance::experience_observability::experience_observability_session_health
    ),
    c!(
        "experience_observability.package_health",
        ["experience"],
        crate::conformance::experience_observability::experience_observability_package_health
    ),
    c!(
        "experience_observability.agent_run_health",
        ["experience", "agentic"],
        crate::conformance::experience_observability::experience_observability_agent_run_health
    ),
    c!(
        "experience_observability.proposal_causality",
        ["experience"],
        crate::conformance::experience_observability::experience_observability_proposal_causality
    ),
    c!(
        "experience_observability.cost_latency_summary",
        ["experience"],
        crate::conformance::experience_observability::experience_observability_cost_latency
    ),
    c!(
        "experience_observability.failure_breadcrumbs",
        ["experience"],
        crate::conformance::experience_observability::experience_observability_failure_breadcrumbs
    ),
    c!(
        "experience_observability.guardrail_audit_summary",
        ["experience"],
        crate::conformance::experience_observability::experience_observability_guardrail_summary
    ),
    c!(
        "experience_observability.no_forbidden_namespace",
        ["experience"],
        crate::conformance::experience_observability::experience_observability_no_forbidden_namespace
    ),
    c!(
        "experience_observability.no_raw_secrets",
        ["experience", "secret"],
        crate::conformance::experience_observability::experience_observability_no_raw_secrets
    ),
    ]
}
