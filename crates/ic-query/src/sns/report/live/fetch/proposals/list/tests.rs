use super::{SnsProposalTopicFilter, SnsTopic, SnsTopicSelector, sns_topic, sns_topic_selectors};

#[test]
fn any_topic_filter_omits_candid_topic_selector() {
    assert_eq!(sns_topic(SnsProposalTopicFilter::Any), None);
    assert_eq!(sns_topic_selectors(SnsProposalTopicFilter::Any), None);
}

#[test]
fn concrete_topic_filters_build_matching_candid_topic_selectors() {
    let cases = [
        (
            SnsProposalTopicFilter::DaoCommunitySettings,
            SnsTopic::DaoCommunitySettings,
        ),
        (
            SnsProposalTopicFilter::SnsFrameworkManagement,
            SnsTopic::SnsFrameworkManagement,
        ),
        (
            SnsProposalTopicFilter::DappCanisterManagement,
            SnsTopic::DappCanisterManagement,
        ),
        (
            SnsProposalTopicFilter::ApplicationBusinessLogic,
            SnsTopic::ApplicationBusinessLogic,
        ),
        (SnsProposalTopicFilter::Governance, SnsTopic::Governance),
        (
            SnsProposalTopicFilter::TreasuryAssetManagement,
            SnsTopic::TreasuryAssetManagement,
        ),
        (
            SnsProposalTopicFilter::CriticalDappOperations,
            SnsTopic::CriticalDappOperations,
        ),
    ];

    for (filter, topic) in cases {
        assert_eq!(sns_topic(filter), Some(topic.clone()));
        assert_eq!(
            sns_topic_selectors(filter),
            Some(vec![SnsTopicSelector { topic: Some(topic) }])
        );
    }
}
