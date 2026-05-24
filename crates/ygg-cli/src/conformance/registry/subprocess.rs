use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- subprocess ---
        c!(
            "subprocess.load_ready",
            ["subprocess", "slow"],
            crate::conformance::subprocess::subprocess_load_ready
        ),
        c!(
            "subprocess.invoke_echo",
            ["subprocess", "slow"],
            crate::conformance::subprocess::subprocess_invoke_echo
        ),
        c!(
            "subprocess.bindings_injected_invokable",
            ["subprocess", "binding", "slow"],
            crate::conformance::subprocess::subprocess_bindings_injected_and_invokable
        ),
        c!(
            "package.lifecycle_timeline",
            ["subprocess", "package", "slow"],
            crate::conformance::subprocess::package_lifecycle_timeline
        ),
        c!(
            "package.path_b_self_contained",
            ["subprocess", "package", "path_b", "slow"],
            crate::conformance::subprocess::path_b_self_contained_contract_none
        ),
        c!(
            "package.logs_capture",
            ["subprocess", "package", "slow"],
            crate::conformance::subprocess::package_logs_capture
        ),
        c!(
            "package.restart_subprocess",
            ["subprocess", "package", "slow"],
            crate::conformance::subprocess::package_restart_subprocess
        ),
    ]
}
