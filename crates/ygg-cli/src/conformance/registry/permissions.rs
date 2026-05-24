use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- permissions ---
        c!(
            "protocol.structured_permission_error",
            ["protocol", "permission"],
            crate::conformance::permissions::structured_permission_error
        ),
        c!(
            "permission.grant_revoke_audit",
            ["runtime", "permission"],
            crate::conformance::permissions::permission_grant_revoke_audit
        ),
        c!(
            "permission.assistant_capability_grant",
            ["runtime", "permission"],
            crate::conformance::permissions::assistant_capability_grant
        ),
        c!(
            "principal.package_cannot_self_assert_writer",
            ["runtime", "permission"],
            crate::conformance::permissions::principal_cannot_self_assert_writer
        ),
        c!(
            "principal.package_cannot_self_assert_capability_caller",
            ["runtime", "permission"],
            crate::conformance::permissions::principal_cannot_self_assert_capability_caller
        ),
    ]
}
