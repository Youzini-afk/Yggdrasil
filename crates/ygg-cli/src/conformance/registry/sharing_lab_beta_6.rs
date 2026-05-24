use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- sharing lab Beta 6 ---
        c!(
            "sharing_lab.contract_shape",
            ["sharing"],
            crate::conformance::sharing_lab::sharing_contract
        ),
        c!(
            "sharing_lab.export_composition_bundle",
            ["sharing", "composition"],
            crate::conformance::sharing_lab::sharing_export_bundle
        ),
        c!(
            "sharing_lab.import_composition_bundle",
            ["sharing", "composition"],
            crate::conformance::sharing_lab::sharing_import_bundle
        ),
        c!(
            "sharing_lab.branch_session_bundle",
            ["sharing"],
            crate::conformance::sharing_lab::sharing_branch_session_bundle
        ),
        c!(
            "sharing_lab.package_set_lockfile",
            ["sharing"],
            crate::conformance::sharing_lab::sharing_package_set_lockfile
        ),
        c!(
            "sharing_lab.compatibility_report",
            ["sharing"],
            crate::conformance::sharing_lab::sharing_compatibility_report
        ),
        c!(
            "sharing_lab.ai_disclosure_bundle",
            ["sharing"],
            crate::conformance::sharing_lab::sharing_ai_disclosure_bundle
        ),
        c!(
            "sharing_lab.read_only_share_manifest",
            ["sharing"],
            crate::conformance::sharing_lab::sharing_read_only_manifest
        ),
        c!(
            "sharing_lab.async_fork_share_plan",
            ["sharing"],
            crate::conformance::sharing_lab::sharing_async_fork_plan
        ),
        c!(
            "sharing_lab.no_marketplace_no_raw_secrets",
            ["sharing", "secret"],
            crate::conformance::sharing_lab::sharing_no_marketplace_no_raw_secrets
        ),
    ]
}
