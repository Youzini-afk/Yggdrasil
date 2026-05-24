use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- core ---
        c!(
            "session.open_empty",
            ["runtime", "session"],
            crate::conformance::core::session_open
        ),
        c!(
            "event.append_authorized",
            ["runtime", "event"],
            crate::conformance::core::event_append_authorized
        ),
        c!(
            "event.append_without_permission_denied",
            ["runtime", "event"],
            crate::conformance::core::event_append_without_permission_denied
        ),
        c!(
            "event.kernel_namespace_denied",
            ["runtime", "event"],
            crate::conformance::core::kernel_namespace_denied
        ),
        c!(
            "event.read_without_permission_denied",
            ["runtime", "event"],
            crate::conformance::core::event_read_without_permission_denied
        ),
        c!(
            "event.closed_session_rejects_append",
            ["runtime", "event"],
            crate::conformance::core::closed_session_rejects_append
        ),
        c!(
            "event.range_replay",
            ["runtime", "event"],
            crate::conformance::core::event_range_replay
        ),
        c!(
            "capability.invoke_rust_inproc",
            ["runtime", "capability"],
            crate::conformance::core::capability_invoke
        ),
        c!(
            "capability.handle_invoke",
            ["runtime", "capability", "handle"],
            crate::conformance::core::capability_handle_invoke
        ),
        c!(
            "capability.handle_attenuate_invoke",
            ["runtime", "capability", "handle"],
            crate::conformance::core::capability_handle_attenuate_invoke
        ),
        c!(
            "capability.handle_revoke_blocks_invoke",
            ["runtime", "capability", "handle"],
            crate::conformance::core::capability_handle_revoke_blocks_invoke
        ),
        c!(
            "capability.auto_mint_legacy_invoke",
            ["runtime", "capability", "handle"],
            crate::conformance::core::capability_auto_mint_legacy_invoke
        ),
        c!(
            "capability.invoke_events_completed",
            ["runtime", "capability", "audit"],
            crate::conformance::core::capability_invoke_events_completed
        ),
        c!(
            "capability.invoke_events_failed",
            ["runtime", "capability", "audit"],
            crate::conformance::core::capability_invoke_events_failed
        ),
        c!(
            "package.audit_report",
            ["runtime", "audit"],
            crate::conformance::audit::package_audit_report
        ),
        c!(
            "capability.ambiguous_provider_denied",
            ["runtime", "capability"],
            crate::conformance::core::ambiguous_provider_denied
        ),
        c!(
            "capability.explicit_provider_selected",
            ["runtime", "capability"],
            crate::conformance::core::explicit_provider_selected
        ),
        c!(
            "package.unload_removes_capability",
            ["runtime", "package"],
            crate::conformance::core::unload_removes_capability
        ),
        c!(
            "official.no_privilege",
            ["official"],
            crate::conformance::core::official_no_privilege
        ),
        c!(
            "schema.capability_input_rejects_invalid",
            ["runtime", "schema"],
            crate::conformance::core::capability_schema_rejects_invalid
        ),
        c!(
            "schema.event_payload_rejects_invalid",
            ["runtime", "schema"],
            crate::conformance::core::event_schema_rejects_invalid
        ),
    ]
}
