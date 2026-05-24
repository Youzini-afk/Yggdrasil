use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- asset ---
        c!(
            "asset.put_get_list",
            ["runtime", "asset"],
            crate::conformance::core::asset_put_get_list
        ),
    ]
}
