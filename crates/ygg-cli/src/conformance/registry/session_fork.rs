use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- session fork ---
        c!(
            "session.fork_branch",
            ["runtime", "session"],
            crate::conformance::core::session_fork_branch
        ),
    ]
}
