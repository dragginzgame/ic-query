use super::*;

#[test]
fn nns_proposal_labels_cover_common_values() {
    assert_eq!(
        nns_proposal_status_text(NNS_PROPOSAL_STATUS_OPEN_CODE),
        NNS_PROPOSAL_STATUS_OPEN_LABEL
    );
    assert_eq!(
        nns_proposal_status_text(NNS_PROPOSAL_STATUS_EXECUTED_CODE),
        NNS_PROPOSAL_STATUS_EXECUTED_LABEL
    );
    assert_eq!(
        nns_reward_status_text(NNS_PROPOSAL_REWARD_STATUS_SETTLED_CODE),
        NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL
    );
    assert_eq!(
        nns_topic_text(NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE),
        NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL
    );
    assert_eq!(
        nns_topic_text(NNS_PROPOSAL_TOPIC_PROTOCOL_CANISTER_MANAGEMENT_CODE),
        NNS_PROPOSAL_TOPIC_PROTOCOL_CANISTER_MANAGEMENT_LABEL
    );
}
