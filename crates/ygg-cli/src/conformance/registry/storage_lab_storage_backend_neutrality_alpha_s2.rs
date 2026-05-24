use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- storage-lab (Storage Backend Neutrality Alpha S2) ---
        c!(
            "storage_lab.contract_shape_no_kernel_database_terms",
            ["storage_lab", "storage", "official"],
            crate::conformance::storage_lab::contract_shape_no_kernel_database_terms
        ),
        c!(
            "storage_lab.backend_classes_no_credentials",
            ["storage_lab", "storage"],
            crate::conformance::storage_lab::backend_classes_no_secret_config
        ),
        c!(
            "storage_lab.package_state_plan_scoped",
            ["storage_lab", "storage"],
            crate::conformance::storage_lab::package_state_plan_scoped
        ),
        c!(
            "storage_lab.put_document_preview_no_write",
            ["storage_lab", "storage"],
            crate::conformance::storage_lab::put_document_preview_no_write
        ),
        c!(
            "storage_lab.get_document_preview_no_read",
            ["storage_lab", "storage"],
            crate::conformance::storage_lab::get_document_preview_no_read
        ),
        c!(
            "storage_lab.query_prefix_preview_no_query_execution",
            ["storage_lab", "storage"],
            crate::conformance::storage_lab::query_prefix_preview_no_query_execution
        ),
        c!(
            "storage_lab.delete_tombstone_preview_no_delete",
            ["storage_lab", "storage"],
            crate::conformance::storage_lab::delete_tombstone_preview_no_delete
        ),
        c!(
            "storage_lab.export_snapshot_preview_redacted",
            ["storage_lab", "storage"],
            crate::conformance::storage_lab::export_snapshot_preview_redacted
        ),
        c!(
            "storage_lab.raw_secret_rejected",
            ["storage_lab", "storage", "secret"],
            crate::conformance::storage_lab::raw_secret_rejected
        ),
        c!(
            "storage_lab.unsafe_id_rejected",
            ["storage_lab", "storage"],
            crate::conformance::storage_lab::unsafe_id_rejected
        ),
    ]
}
