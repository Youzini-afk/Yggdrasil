use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- project lifecycle (Round 10A.2 Wave 3) ---
        c!(
            "project.detect_native_yaml",
            ["project", "install"],
            crate::conformance::project_lifecycle::detect_native_yaml
        ),
        c!(
            "project.detect_no_yaml",
            ["project", "install"],
            crate::conformance::project_lifecycle::detect_no_yaml
        ),
        c!(
            "project.detect_invalid_yaml_rejected",
            ["project", "install"],
            crate::conformance::project_lifecycle::detect_invalid_yaml_rejected
        ),
        c!(
            "project.register_creates_project_dir",
            ["project", "install"],
            crate::conformance::project_lifecycle::register_creates_project_dir
        ),
        c!(
            "project.list_returns_registered",
            ["project"],
            crate::conformance::project_lifecycle::list_returns_registered
        ),
        c!(
            "project.state_transitions",
            ["project"],
            crate::conformance::project_lifecycle::state_transitions
        ),
        c!(
            "project.archive_keeps_data",
            ["project", "uninstall"],
            crate::conformance::project_lifecycle::archive_keeps_data
        ),
    ]
}
