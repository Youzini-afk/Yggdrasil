use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- playable board Beta 2 ---
        c!(
            "playable_board.content_address_stable",
            ["experience"],
            crate::conformance::playable_creation_board::playable_board_content_address_stable
        ),
        c!(
            "playable_board.checkpoint_metadata",
            ["experience"],
            crate::conformance::playable_creation_board::playable_board_checkpoint_metadata
        ),
        c!(
            "playable_board.provenance_graph",
            ["experience"],
            crate::conformance::playable_creation_board::playable_board_provenance_graph
        ),
        c!(
            "playable_board.state_diff_preview",
            ["experience"],
            crate::conformance::playable_creation_board::playable_board_state_diff_preview
        ),
        c!(
            "playable_board.describe_asset_provenance",
            ["experience"],
            crate::conformance::playable_creation_board::playable_board_describe_asset_provenance
        ),
        c!(
            "playable_board.beta2_no_raw_secrets",
            ["experience", "secret"],
            crate::conformance::playable_creation_board::playable_board_beta2_no_raw_secrets
        ),
        c!(
            "official.asset_lab_content_address",
            ["official", "slow"],
            crate::conformance::official_labs::asset_lab_content_address
        ),
        c!(
            "official.asset_lab_provenance_graph",
            ["official", "slow"],
            crate::conformance::official_labs::asset_lab_provenance_graph
        ),
        c!(
            "official.projection_lab_state_snapshot",
            ["official", "slow"],
            crate::conformance::official_labs::projection_lab_state_snapshot
        ),
    ]
}
