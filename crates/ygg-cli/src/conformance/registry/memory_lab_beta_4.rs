use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- memory lab Beta 4 ---
        c!(
            "memory_lab.contract_shape",
            ["memory"],
            crate::conformance::memory_lab::memory_lab_contract
        ),
        c!(
            "memory_lab.record_memory",
            ["memory"],
            crate::conformance::memory_lab::memory_lab_record_memory
        ),
        c!(
            "memory_lab.retrieve_memory",
            ["memory"],
            crate::conformance::memory_lab::memory_lab_retrieve_memory
        ),
        c!(
            "memory_lab.trace_retrieval",
            ["memory"],
            crate::conformance::memory_lab::memory_lab_trace_retrieval
        ),
        c!(
            "memory_lab.draft_update_proposal_only",
            ["memory"],
            crate::conformance::memory_lab::memory_lab_draft_update
        ),
        c!(
            "memory_lab.correction_proposal_gated",
            ["memory"],
            crate::conformance::memory_lab::memory_lab_correction
        ),
        c!(
            "memory_lab.forget_redaction_plan",
            ["memory"],
            crate::conformance::memory_lab::memory_lab_forget_redaction
        ),
        c!(
            "memory_lab.branch_view",
            ["memory"],
            crate::conformance::memory_lab::memory_lab_branch_view
        ),
        c!(
            "memory_lab.no_forbidden_namespace",
            ["memory"],
            crate::conformance::memory_lab::memory_lab_no_forbidden_namespace
        ),
        c!(
            "memory_lab.no_raw_secrets",
            ["memory", "secret"],
            crate::conformance::memory_lab::memory_lab_no_raw_secrets
        ),
    ]
}
