use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- storage-lab S5 (Retrieval / Vector / Multimodal Provider Contract) ---
        c!(
            "storage_lab.retrieval_contract_shape",
            ["storage_lab", "storage", "retrieval"],
            crate::conformance::storage_lab::retrieval_contract_shape
        ),
        c!(
            "storage_lab.multimodal_index_plan_no_embedding_no_storage",
            ["storage_lab", "storage", "retrieval"],
            crate::conformance::storage_lab::multimodal_index_plan_no_embedding_no_storage
        ),
        c!(
        "storage_lab.multimodal_index_rejects_invalid_modality_or_too_many_refs",
        ["storage_lab", "storage", "retrieval"],
        crate::conformance::storage_lab::multimodal_index_rejects_invalid_modality_or_too_many_refs
    ),
        c!(
            "storage_lab.vector_search_plan_no_execution",
            ["storage_lab", "storage", "retrieval"],
            crate::conformance::storage_lab::vector_search_plan_no_execution
        ),
        c!(
            "storage_lab.backend_fit_mentions_tdb_future_only",
            ["storage_lab", "storage", "retrieval", "tdb"],
            crate::conformance::storage_lab::backend_fit_mentions_tdb_future_only
        ),
        c!(
            "storage_lab.retrieval_rejects_raw_secret",
            ["storage_lab", "storage", "retrieval", "secret"],
            crate::conformance::storage_lab::retrieval_rejects_raw_secret
        ),
        c!(
            "storage_lab.retrieval_no_kernel_vector_namespace_or_credentials",
            ["storage_lab", "storage", "retrieval"],
            crate::conformance::storage_lab::retrieval_no_kernel_vector_namespace_or_secret_config
        ),
        c!(
            "tdb_retrieval_lab.contract_shape",
            ["tdb", "retrieval", "storage", "official"],
            crate::conformance::tdb_retrieval_lab::contract_shape
        ),
        c!(
            "tdb_retrieval_lab.index_plan_no_execution",
            ["tdb", "retrieval", "storage"],
            crate::conformance::tdb_retrieval_lab::index_plan_no_execution
        ),
        c!(
            "tdb_retrieval_lab.query_plan_no_execution",
            ["tdb", "retrieval", "storage"],
            crate::conformance::tdb_retrieval_lab::query_plan_no_execution
        ),
        c!(
            "tdb_retrieval_lab.backend_fit_boundary",
            ["tdb", "retrieval", "storage"],
            crate::conformance::tdb_retrieval_lab::backend_fit_boundary
        ),
        c!(
            "tdb_retrieval_lab.invalid_input_rejected",
            ["tdb", "retrieval", "storage"],
            crate::conformance::tdb_retrieval_lab::invalid_input_rejected
        ),
        c!(
            "tdb_retrieval_lab.raw_secret_and_unsafe_id_rejected",
            ["tdb", "retrieval", "storage", "secret"],
            crate::conformance::tdb_retrieval_lab::raw_secret_and_unsafe_id_rejected
        ),
        c!(
            "tdb_retrieval_lab.real_tdb_opt_in_seam_crate_adapter_available",
            ["tdb", "retrieval", "storage"],
            crate::conformance::tdb_retrieval_lab::real_tdb_opt_in_seam_crate_adapter_available
        ),
    ]
}
