use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- Y2: manifest permissions.secret_refs conformance ---
        c!(
            "outbound_execute.secret_ref_undeclared_fails",
            ["outbound", "network", "secret", "manifest"],
            crate::conformance::network::outbound_execute_secret_ref_undeclared_fails
        ),
        c!(
            "outbound_execute.secret_ref_declared_resolves",
            ["outbound", "network", "secret", "manifest"],
            crate::conformance::network::outbound_execute_secret_ref_declared_resolves
        ),
    ]
}
