use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- real TDB Rust adapter subprocess proof ---
        c!(
            "tdb_rust_adapter.subprocess_adapter_shell_invokes_disabled_smoke",
            ["tdb", "retrieval", "subprocess", "slow"],
            crate::conformance::tdb_rust_adapter::subprocess_adapter_shell_invokes_disabled_smoke
        ),
        c!(
            "tdb_rust_adapter.subprocess_adapter_rejects_secret_and_raw_path",
            ["tdb", "retrieval", "subprocess", "secret", "slow"],
            crate::conformance::tdb_rust_adapter::subprocess_adapter_rejects_secret_and_raw_path
        ),
        c!(
            "tdb_rust_adapter.real_crate_smoke_opt_in",
            ["tdb", "tdb_real", "retrieval", "slow"],
            crate::conformance::tdb_rust_adapter::real_crate_smoke_opt_in
        ),
    ]
}
