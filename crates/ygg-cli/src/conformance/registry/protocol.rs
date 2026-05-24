use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- protocol ---
        c!(
            "protocol.call_host_info",
            ["protocol"],
            crate::conformance::protocol::call_host_info
        ),
        c!(
            "protocol.call_capability_in_process",
            ["protocol"],
            crate::conformance::protocol::call_capability_in_process
        ),
        c!(
            "protocol.project_list_returns_registered_projects",
            ["protocol", "project"],
            crate::conformance::protocol_project::project_list_returns_registered_projects
        ),
        c!(
            "protocol.project_get_returns_full_descriptor",
            ["protocol", "project"],
            crate::conformance::protocol_project::project_get_returns_full_descriptor
        ),
        c!(
            "protocol.project_start_transitions_state",
            ["protocol", "project"],
            crate::conformance::protocol_project::project_start_transitions_state
        ),
        c!(
            "project.start_returns_session_id",
            ["protocol", "project", "session"],
            crate::conformance::protocol_project::project_start_returns_session_id
        ),
        c!(
            "project.start_idempotent_returns_existing_session",
            ["protocol", "project", "session"],
            crate::conformance::protocol_project::project_start_idempotent_returns_existing_session
        ),
        c!(
            "project.session_metadata_carries_project_id",
            ["protocol", "project", "session"],
            crate::conformance::protocol_project::project_session_metadata_carries_project_id
        ),
        c!(
            "project.stop_closes_session",
            ["protocol", "project", "session"],
            crate::conformance::protocol_project::project_stop_closes_session
        ),
        c!(
            "project.get_returns_running_session_id",
            ["protocol", "project", "session"],
            crate::conformance::protocol_project::project_get_returns_running_session_id
        ),
        c!(
            "protocol.project_methods_require_admin_principal",
            ["protocol", "project", "permission"],
            crate::conformance::protocol_project::project_methods_require_admin_principal
        ),
        c!(
            "protocol.project_lifecycle_event_emitted_on_start",
            ["protocol", "project", "event"],
            crate::conformance::protocol_project::project_lifecycle_event_emitted_on_start
        ),
        c!(
            "surface.resolve_via_dev_path",
            ["protocol", "surface"],
            crate::conformance::protocol_project::surface_resolve_via_dev_path
        ),
        c!(
            "surface.resolve_via_installed_project",
            ["protocol", "surface", "project"],
            crate::conformance::protocol_project::surface_resolve_via_installed_project
        ),
        c!(
            "surface.resolve_unknown_fails",
            ["protocol", "surface"],
            crate::conformance::protocol_project::surface_resolve_unknown_fails
        ),
        c!(
            "surface.resolve_admin_principal_required",
            ["protocol", "surface", "permission"],
            crate::conformance::protocol_project::surface_resolve_admin_principal_required
        ),
    ]
}
