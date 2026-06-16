use super::super::governance_canister;
use crate::sns::report::{
    SnsHostError, SnsProposalTopicFilter,
    live::{
        convert::sns_proposal_row,
        query::{query_canister, sns_agent},
        types::{
            ListProposalsRequest, ListProposalsResponse, SnsProposalId, SnsTopic, SnsTopicSelector,
        },
    },
    source::{MainnetSns, MainnetSnsProposals, SnsFetchRequest},
};

pub(super) async fn fetch_mainnet_sns_proposals_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    before_proposal_id: Option<u64>,
    include_status: &[i32],
    topic: SnsProposalTopicFilter,
) -> Result<MainnetSnsProposals, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let governance_canister = governance_canister(sns)?;
    let response: ListProposalsResponse = query_canister(
        &agent,
        &governance_canister,
        "list_proposals",
        "ListProposalsRequest",
        "ListProposalsResponse",
        &ListProposalsRequest {
            include_reward_status: Vec::new(),
            before_proposal: before_proposal_id.map(|id| SnsProposalId { id }),
            limit,
            exclude_type: Vec::new(),
            include_status: include_status.to_vec(),
            include_topics: sns_topic_selectors(topic),
        },
    )
    .await?;
    Ok(MainnetSnsProposals {
        proposals: response
            .proposals
            .into_iter()
            .map(sns_proposal_row)
            .collect(),
    })
}

fn sns_topic_selectors(topic: SnsProposalTopicFilter) -> Option<Vec<SnsTopicSelector>> {
    sns_topic(topic).map(|topic| vec![SnsTopicSelector { topic: Some(topic) }])
}

const fn sns_topic(topic: SnsProposalTopicFilter) -> Option<SnsTopic> {
    match topic {
        SnsProposalTopicFilter::Any => None,
        SnsProposalTopicFilter::DaoCommunitySettings => Some(SnsTopic::DaoCommunitySettings),
        SnsProposalTopicFilter::SnsFrameworkManagement => Some(SnsTopic::SnsFrameworkManagement),
        SnsProposalTopicFilter::DappCanisterManagement => Some(SnsTopic::DappCanisterManagement),
        SnsProposalTopicFilter::ApplicationBusinessLogic => {
            Some(SnsTopic::ApplicationBusinessLogic)
        }
        SnsProposalTopicFilter::Governance => Some(SnsTopic::Governance),
        SnsProposalTopicFilter::TreasuryAssetManagement => Some(SnsTopic::TreasuryAssetManagement),
        SnsProposalTopicFilter::CriticalDappOperations => Some(SnsTopic::CriticalDappOperations),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SnsProposalTopicFilter, SnsTopic, SnsTopicSelector, sns_topic, sns_topic_selectors,
    };

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
}
