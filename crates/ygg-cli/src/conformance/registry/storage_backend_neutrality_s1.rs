use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- storage backend neutrality S1 ---
        c!(
            "storage_backend.in_memory_event_store_contract_append_range",
            ["storage", "substrate"],
            crate::conformance::storage_backend::in_memory_event_store_contract_append_range
        ),
        c!(
            "storage_backend.sqlite_event_store_contract_append_range",
            ["storage", "substrate"],
            crate::conformance::storage_backend::sqlite_event_store_contract_append_range
        ),
        c!(
            "storage_backend.backend_parity_kind_prefix",
            ["storage", "substrate"],
            crate::conformance::storage_backend::backend_parity_kind_prefix
        ),
        c!(
            "storage_backend.backend_parity_concurrent_append",
            ["storage", "substrate"],
            crate::conformance::storage_backend::backend_parity_concurrent_append
        ),
        c!(
            "storage_backend.backend_parity_subscription",
            ["storage", "substrate"],
            crate::conformance::storage_backend::backend_parity_subscription
        ),
        c!(
            "storage_backend.rehydrate_parity",
            ["storage", "substrate"],
            crate::conformance::storage_backend::storage_backend_rehydrate_parity
        ),
    ]
}
