use super::*;

#[test]
fn nns_proposal_topic_labels_cover_common_values() {
    assert_eq!(
        nns_topic_text(NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE),
        NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL
    );
    assert_eq!(
        nns_topic_text(NNS_PROPOSAL_TOPIC_PROTOCOL_CANISTER_MANAGEMENT_CODE),
        NNS_PROPOSAL_TOPIC_PROTOCOL_CANISTER_MANAGEMENT_LABEL
    );
}
