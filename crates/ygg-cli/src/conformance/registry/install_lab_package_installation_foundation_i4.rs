use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- install-lab (Package Installation Foundation I4) ---
        c!(
            "install_lab.resolve_plan_local_source",
            ["official", "install", "package_install", "fixture"],
            crate::conformance::install_lab::resolve_plan_local_source
        ),
        c!(
            "install_lab.resolve_plan_runs_conformance",
            ["official", "install", "package_install", "fixture"],
            crate::conformance::install_lab::resolve_plan_runs_conformance
        ),
        c!(
            "install_lab.resolve_plan_blocks_when_strict",
            ["official", "install", "package_install", "fixture"],
            crate::conformance::install_lab::resolve_plan_blocks_when_strict
        ),
        c!(
            "install_lab.strict_conformance_blocks",
            ["official", "install", "package_install", "fixture"],
            crate::conformance::install_lab::strict_conformance_blocks
        ),
        c!(
            "install_lab.lenient_conformance_warns_not_blocks",
            ["official", "install", "package_install", "fixture"],
            crate::conformance::install_lab::lenient_conformance_warns_not_blocks
        ),
        c!(
            "install_lab.transitive_conformance_propagates",
            ["official", "install", "package_install", "fixture"],
            crate::conformance::install_lab::transitive_conformance_propagates
        ),
        c!(
            "install_lab.resolve_plan_with_transitive",
            ["official", "install", "package_install", "fixture"],
            crate::conformance::install_lab::resolve_plan_with_transitive
        ),
        c!(
            "install_lab.resolve_plan_cycle_detection",
            ["official", "install", "package_install", "fixture"],
            crate::conformance::install_lab::resolve_plan_cycle_detection
        ),
        c!(
            "install_lab.execute_plan_local",
            ["official", "install", "package_install", "fixture"],
            crate::conformance::install_lab::execute_plan_local
        ),
        c!(
            "install_lab.execute_plan_consent_mismatch",
            ["official", "install", "package_install", "fixture"],
            crate::conformance::install_lab::execute_plan_consent_mismatch
        ),
        c!(
            "install_lab.uninstall_removes_from_profile",
            ["official", "install", "package_install", "fixture"],
            crate::conformance::install_lab::uninstall_removes_from_profile
        ),
        c!(
            "install_lab.list_installed_reflects_lockfile",
            ["official", "install", "package_install", "fixture"],
            crate::conformance::install_lab::list_installed_reflects_lockfile
        ),
        c!(
            "install_lab.check_lockfile_drift_detection",
            ["official", "install", "package_install", "fixture"],
            crate::conformance::install_lab::check_lockfile_drift_detection
        ),
        c!(
            "install.real_github_smoke",
            ["install", "real-network", "opt-in"],
            crate::conformance::install_real_smoke::real_github_smoke
        ),
    ]
}
