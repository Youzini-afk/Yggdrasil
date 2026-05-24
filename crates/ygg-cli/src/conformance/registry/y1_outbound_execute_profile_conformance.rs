use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- Y1: outbound execute profile conformance ---
        c!(
            "outbound_execute.profile_default_deny_all",
            ["outbound", "network", "profile"],
            crate::conformance::network::outbound_execute_profile_default_deny_all
        ),
        c!(
            "outbound_execute.profile_fake_executor_works",
            ["outbound", "network", "profile"],
            crate::conformance::network::outbound_execute_profile_fake_executor_works
        ),
        c!(
            "outbound_execute.profile_live_disabled_returns_deny",
            ["outbound", "network", "profile"],
            crate::conformance::network::outbound_execute_profile_live_disabled_returns_deny
        ),
        c!(
            "outbound_websocket.profile_default_deny_all",
            ["outbound", "network", "websocket"],
            crate::conformance::network::outbound_websocket_profile_default_deny_all
        ),
        c!(
            "outbound_websocket.profile_fake_executor_works",
            ["outbound", "network", "websocket"],
            crate::conformance::network::outbound_websocket_profile_fake_executor_works
        ),
        c!(
            "outbound_websocket.profile_live_disabled_returns_deny",
            ["outbound", "network", "websocket"],
            crate::conformance::network::outbound_websocket_profile_live_disabled_returns_deny
        ),
        c!(
            "outbound_websocket.secret_ref_undeclared_fails",
            ["outbound", "network", "websocket", "secret", "manifest"],
            crate::conformance::network::outbound_websocket_secret_ref_undeclared_fails
        ),
        c!(
            "outbound_websocket.capability_namespace_enforced",
            ["outbound", "network", "websocket", "permission"],
            crate::conformance::network::outbound_websocket_capability_namespace_enforced
        ),
        c!(
            "outbound_websocket.wss_only_default",
            ["outbound", "network", "websocket"],
            crate::conformance::network::outbound_websocket_wss_only_default
        ),
        c!(
            "outbound_websocket.idle_timeout_emits_error_and_completed",
            ["outbound", "network", "websocket", "audit"],
            crate::conformance::network::outbound_websocket_idle_timeout_emits_error_and_completed
        ),
        c!(
            "outbound_websocket.max_total_bytes_inbound_terminates",
            ["outbound", "network", "websocket", "audit"],
            crate::conformance::network::outbound_websocket_max_total_bytes_inbound_terminates
        ),
        c!(
            "outbound_websocket.max_concurrent_connections_enforced",
            ["outbound", "network", "websocket"],
            crate::conformance::network::outbound_websocket_max_concurrent_connections_enforced
        ),
        c!(
            "outbound_websocket.cancel_via_capability_cancel",
            ["outbound", "network", "websocket"],
            crate::conformance::network::outbound_websocket_cancel_via_capability_cancel
        ),
        c!(
            "outbound.execute_completed_audit_emitted",
            ["outbound", "network", "audit"],
            crate::conformance::network::outbound_execute_completed_audit_emitted
        ),
        c!(
            "outbound.execute_correlation_id_propagates",
            ["outbound", "network", "audit"],
            crate::conformance::network::outbound_execute_correlation_id_propagates
        ),
        c!(
            "outbound.stream_completed_audit_emitted",
            ["outbound", "network", "stream", "audit"],
            crate::conformance::network::outbound_stream_completed_audit_emitted
        ),
        c!(
            "outbound.websocket_completed_audit_emitted",
            ["outbound", "network", "websocket", "audit"],
            crate::conformance::network::outbound_websocket_completed_audit_emitted
        ),
    ]
}
