//! Module: sns::report::live::convert::proposals::labels
//!
//! Responsibility: label SNS governance proposal enum/code values.
//! Does not own: governance transport, filtering, or rendering.
//! Boundary: converts raw vote/action codes into stable report labels.

/// Convert an SNS governance ballot vote code into a stable label.
pub(super) fn ballot_vote_text(vote: i32) -> String {
    match vote {
        0 => "unspecified".to_string(),
        1 => "yes".to_string(),
        2 => "no".to_string(),
        other => format!("unknown:{other}"),
    }
}

/// Convert an SNS governance proposal action id into a stable label.
pub(super) fn proposal_action_text(action: u64) -> String {
    match action {
        0 => "unspecified".to_string(),
        1 => "motion".to_string(),
        2 => "manage_nervous_system_parameters".to_string(),
        3 => "upgrade_sns_controlled_canister".to_string(),
        4 => "add_generic_nervous_system_function".to_string(),
        5 => "remove_generic_nervous_system_function".to_string(),
        6 => "execute_generic_nervous_system_function".to_string(),
        7 => "upgrade_sns_to_next_version".to_string(),
        8 => "manage_sns_metadata".to_string(),
        9 => "transfer_sns_treasury_funds".to_string(),
        10 => "register_dapp_canisters".to_string(),
        11 => "deregister_dapp_canisters".to_string(),
        12 => "mint_sns_tokens".to_string(),
        13 => "manage_ledger_parameters".to_string(),
        14 => "manage_dapp_canister_settings".to_string(),
        15 => "advance_sns_target_version".to_string(),
        16 => "set_topics_for_custom_proposals".to_string(),
        17 => "register_extension".to_string(),
        18 => "execute_extension_operation".to_string(),
        19 => "upgrade_extension".to_string(),
        id if id >= 1_000 => format!("generic:{id}"),
        id => format!("unknown:{id}"),
    }
}
