use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
    // --- playable board Beta 1 ---
    c!(
        "playable_board.describe_contract_shape",
        ["experience"],
        crate::conformance::playable_creation_board::playable_board_describe_contract
    ),
    c!(
        "playable_board.launch_and_player_actions",
        ["experience"],
        crate::conformance::playable_creation_board::playable_board_launch_and_player_actions
    ),
    c!(
        "playable_board.checkpoint_recovery_shape",
        ["experience"],
        crate::conformance::playable_creation_board::playable_board_checkpoint_recovery
    ),
    c!(
        "playable_board.request_change_no_chat",
        ["experience"],
        crate::conformance::playable_creation_board::playable_board_request_change_no_chat
    ),
    c!(
        "playable_board.bind_agent_run_scoped",
        ["experience", "agentic"],
        crate::conformance::playable_creation_board::playable_board_bind_agent_run_scoped
    ),
    c!(
        "playable_board.candidate_proposal_no_target_mutation",
        ["experience"],
        crate::conformance::playable_creation_board::playable_board_candidate_proposal_no_target_mutation
    ),
    c!(
        "playable_board.reject_approve_fork_proof",
        ["experience"],
        crate::conformance::playable_creation_board::playable_board_reject_approve_fork_proof
    ),
    c!(
        "playable_board.thirdparty_no_official_priority",
        ["experience", "replacement"],
        crate::conformance::playable_creation_board::playable_board_thirdparty_no_official_priority
    ),
    c!(
        "playable_board.no_forbidden_namespace",
        ["experience"],
        crate::conformance::playable_creation_board::playable_board_no_forbidden_namespace
    ),
    c!(
        "playable_board.no_raw_secrets",
        ["experience", "secret"],
        crate::conformance::playable_creation_board::playable_board_no_raw_secrets
    ),
    ]
}
