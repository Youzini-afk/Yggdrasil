use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- project-scoped secrets (Round 10A.2 Wave 2C) ---
        c!(
            "project_secret.put_then_resolve_via_project_ref",
            ["project", "secret"],
            crate::conformance::project_secret::put_then_resolve_via_project_ref
        ),
        c!(
            "project_secret.fallback_to_platform_when_missing",
            ["project", "secret"],
            crate::conformance::project_secret::fallback_to_platform_when_missing
        ),
        c!(
            "project_secret.no_fallback_when_disabled",
            ["project", "secret"],
            crate::conformance::project_secret::no_fallback_when_disabled
        ),
        c!(
            "project_secret.require_per_project_blocks_fallback",
            ["project", "secret"],
            crate::conformance::project_secret::require_per_project_blocks_fallback
        ),
        c!(
            "project_secret.isolation_between_projects",
            ["project", "secret"],
            crate::conformance::project_secret::isolation_between_projects
        ),
        c!(
            "project_secret.no_session_context_fails_closed",
            ["project", "secret", "outbound"],
            crate::conformance::project_secret::no_session_context_fails_closed
        ),
        c!(
            "project_secret.list_returns_names_not_values",
            ["project", "secret"],
            crate::conformance::project_secret::list_returns_names_not_values
        ),
    ]
}
