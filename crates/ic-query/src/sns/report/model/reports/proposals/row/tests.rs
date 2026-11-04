use super::*;

#[test]
fn proposal_decision_state_labels_round_trip() {
    for (state, label) in [
        (SnsProposalDecisionState::Open, "open"),
        (SnsProposalDecisionState::Decided, "decided"),
        (SnsProposalDecisionState::Executed, "executed"),
        (SnsProposalDecisionState::Failed, "failed"),
    ] {
        assert_eq!(
            serde_json::to_string(&state).unwrap(),
            format!("\"{label}\"")
        );
        assert_eq!(
            serde_json::from_str::<SnsProposalDecisionState>(&format!("\"{label}\"")).unwrap(),
            state
        );
    }
    assert!(serde_json::from_str::<SnsProposalDecisionState>("\"unknown\"").is_err());
}

#[test]
fn proposal_action_labels_cover_native_generic_and_unknown_ids() {
    for (id, label) in [
        (0, "unspecified"),
        (1, "motion"),
        (2, "manage_nervous_system_parameters"),
        (3, "upgrade_sns_controlled_canister"),
        (4, "add_generic_nervous_system_function"),
        (5, "remove_generic_nervous_system_function"),
        (6, "execute_generic_nervous_system_function"),
        (7, "upgrade_sns_to_next_version"),
        (8, "manage_sns_metadata"),
        (9, "transfer_sns_treasury_funds"),
        (10, "register_dapp_canisters"),
        (11, "deregister_dapp_canisters"),
        (12, "mint_sns_tokens"),
        (13, "manage_ledger_parameters"),
        (14, "manage_dapp_canister_settings"),
        (15, "advance_sns_target_version"),
        (16, "set_topics_for_custom_proposals"),
        (17, "register_extension"),
        (18, "execute_extension_operation"),
        (19, "upgrade_extension"),
        (20, "unknown:20"),
        (1_000, "generic:1000"),
        (u64::MAX, "generic:18446744073709551615"),
    ] {
        assert_eq!(sns_proposal_action_label(id), label);
    }
}
