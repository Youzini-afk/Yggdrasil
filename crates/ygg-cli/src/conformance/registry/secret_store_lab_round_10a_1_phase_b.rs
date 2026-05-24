use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- secret-store-lab (Round 10A.1 Phase B) ---
        c!(
            "secret_store.put_then_has_succeeds",
            ["official", "secret_store", "secret"],
            crate::conformance::secret_store::put_then_has_succeeds
        ),
        c!(
            "secret_store.list_returns_names_not_values",
            ["official", "secret_store", "secret"],
            crate::conformance::secret_store::list_returns_names_not_values
        ),
        c!(
            "secret_store.delete_removes",
            ["official", "secret_store", "secret"],
            crate::conformance::secret_store::delete_removes
        ),
        c!(
            "secret_store.put_invalid_name_rejected",
            ["official", "secret_store", "secret"],
            crate::conformance::secret_store::put_invalid_name_rejected
        ),
        c!(
            "secret_store.put_oversized_value_rejected",
            ["official", "secret_store", "secret"],
            crate::conformance::secret_store::put_oversized_value_rejected
        ),
        c!(
            "secret_store.health_reports_layout",
            ["official", "secret_store", "secret"],
            crate::conformance::secret_store::health_reports_layout
        ),
        c!(
            "secret_store_resolver.resolves_existing",
            ["official", "secret_store", "secret"],
            crate::conformance::secret_store::resolver_resolves_existing
        ),
        c!(
            "secret_store_resolver.missing_name_fails_closed",
            ["official", "secret_store", "secret"],
            crate::conformance::secret_store::resolver_missing_name_fails_closed
        ),
        c!(
            "secret_store_resolver.non_store_ref_rejected",
            ["official", "secret_store", "secret"],
            crate::conformance::secret_store::resolver_non_store_ref_rejected
        ),
        c!(
            "secret_store_resolver.error_does_not_leak_value",
            ["official", "secret_store", "secret"],
            crate::conformance::secret_store::resolver_error_does_not_leak_value
        ),
        c!(
            "secret_store_resolver.host_profile_installs_composite_resolver",
            ["official", "secret_store", "secret", "host"],
            crate::conformance::secret_store::host_profile_installs_composite_resolver
        ),
    ]
}
