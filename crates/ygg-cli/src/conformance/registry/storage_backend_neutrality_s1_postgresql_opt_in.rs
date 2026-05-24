#![allow(unused_imports, unused_macros)]

use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- storage backend neutrality S1 — PostgreSQL (opt-in) ---
        #[cfg(feature = "postgres")]
        c!(
            "storage_backend.postgres_event_store_contract_append_range",
            ["storage", "substrate", "postgres"],
            crate::conformance::storage_backend::postgres_event_store_contract_append_range
        ),
        #[cfg(feature = "postgres")]
        c!(
            "storage_backend.postgres_backend_parity_kind_prefix",
            ["storage", "substrate", "postgres"],
            crate::conformance::storage_backend::postgres_backend_parity_kind_prefix
        ),
        #[cfg(feature = "postgres")]
        c!(
            "storage_backend.postgres_backend_parity_concurrent_append",
            ["storage", "substrate", "postgres"],
            crate::conformance::storage_backend::postgres_backend_parity_concurrent_append
        ),
        #[cfg(feature = "postgres")]
        c!(
            "storage_backend.postgres_backend_parity_subscription",
            ["storage", "substrate", "postgres"],
            crate::conformance::storage_backend::postgres_backend_parity_subscription
        ),
        #[cfg(feature = "postgres")]
        c!(
            "storage_backend.postgres_rehydrate_parity",
            ["storage", "substrate", "postgres"],
            crate::conformance::storage_backend::postgres_rehydrate_parity
        ),
    ]
}
