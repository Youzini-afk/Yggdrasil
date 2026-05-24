use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- storage-lab S4 (Projection / Index Materialization Contract Proof) ---
        c!(
            "storage_lab.projection_contract_shape",
            ["storage_lab", "storage", "projection"],
            crate::conformance::storage_lab::projection_contract_shape
        ),
        c!(
            "storage_lab.projection_materialization_plan_only",
            ["storage_lab", "storage", "projection"],
            crate::conformance::storage_lab::projection_materialization_plan_only
        ),
        c!(
            "storage_lab.projection_query_preview_no_execution",
            ["storage_lab", "storage", "projection"],
            crate::conformance::storage_lab::projection_query_preview_no_execution
        ),
        c!(
            "storage_lab.projection_migration_plan_no_rewrite",
            ["storage_lab", "storage", "projection"],
            crate::conformance::storage_lab::projection_migration_plan_no_rewrite
        ),
        c!(
            "storage_lab.projection_rejects_raw_secret",
            ["storage_lab", "storage", "projection", "secret"],
            crate::conformance::storage_lab::projection_rejects_raw_secret
        ),
        c!(
            "storage_lab.projection_no_db_table_leakage",
            ["storage_lab", "storage", "projection"],
            crate::conformance::storage_lab::projection_no_db_table_leakage
        ),
    ]
}
