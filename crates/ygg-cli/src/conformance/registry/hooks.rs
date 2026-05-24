use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- hooks ---
        c!(
            "hook.ordering_stable",
            ["runtime", "hook"],
            crate::conformance::hooks::ordering_stable
        ),
        c!(
            "hook.veto_blocks_event_append",
            ["runtime", "hook"],
            crate::conformance::hooks::veto_blocks_event_append
        ),
        c!(
            "hook.metadata_mutation_allowed",
            ["runtime", "hook"],
            crate::conformance::hooks::metadata_mutation_allowed
        ),
        c!(
            "hook.package_owned_handler",
            ["runtime", "hook"],
            crate::conformance::hooks::package_owned_handler
        ),
        c!(
            "hook.unload_removes_subscription",
            ["runtime", "hook"],
            crate::conformance::hooks::unload_removes_subscription
        ),
    ]
}
