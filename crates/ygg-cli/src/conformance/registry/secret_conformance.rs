use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- secret conformance ---
        c!(
            "substrate.permission_grant_rehydrate",
            ["substrate", "secret"],
            crate::conformance::secret_conformance::permission_grant_rehydrate
        ),
        c!(
            "secret.ref_validation",
            ["secret"],
            crate::conformance::secret_conformance::secret_ref_validation
        ),
        c!(
            "secret.raw_blocked_in_proposal",
            ["secret"],
            crate::conformance::secret_conformance::raw_secret_blocked_in_proposal
        ),
        c!(
            "secret.raw_blocked_in_asset_metadata",
            ["secret"],
            crate::conformance::secret_conformance::raw_secret_blocked_in_asset_metadata
        ),
        c!(
            "official.no_secret_bypass",
            ["official", "secret"],
            crate::conformance::secret_conformance::no_secret_bypass
        ),
        c!(
            "secret.env_resolver_allowed",
            ["secret"],
            crate::conformance::secret_conformance::env_resolver_allowed
        ),
        c!(
            "secret.env_resolver_denied",
            ["secret"],
            crate::conformance::secret_conformance::env_resolver_denied
        ),
        c!(
            "secret.env_resolver_missing_no_leak",
            ["secret"],
            crate::conformance::secret_conformance::env_resolver_missing_no_leak
        ),
    ]
}
