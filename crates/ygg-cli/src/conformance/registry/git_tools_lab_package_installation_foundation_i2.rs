use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- git-tools-lab (Package Installation Foundation I2) ---
        c!(
            "git_tools.url_validation_https_only",
            ["official", "git_tools", "install"],
            crate::conformance::git_tools::url_validation_https_only
        ),
        c!(
            "git_tools.url_validation_no_userinfo",
            ["official", "git_tools", "install", "secret"],
            crate::conformance::git_tools::url_validation_no_userinfo
        ),
        c!(
            "git_tools.path_validation_absolute",
            ["official", "git_tools", "install"],
            crate::conformance::git_tools::path_validation_absolute
        ),
        c!(
            "git_tools.path_validation_no_traversal",
            ["official", "git_tools", "install"],
            crate::conformance::git_tools::path_validation_no_traversal
        ),
        c!(
            "git_tools.read_signed_tag_unsigned",
            ["official", "git_tools", "install", "fixture"],
            crate::conformance::git_tools::read_signed_tag_unsigned
        ),
    ]
}
