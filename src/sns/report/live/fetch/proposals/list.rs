//! Module: sns::report::live::fetch::proposals::list
//!
//! Responsibility: fetch bounded and paged SNS proposal listings.
//! Does not own: proposal conversion, cache refresh orchestration, or rendering.
//! Boundary: builds list_proposals requests and maps responses into source models.

use crate::sns::report::{
    SnsHostError, SnsProposalTopicFilter,
    live::{
        convert::sns_proposal_row,
        fetch::governance_canister,
        query::{query_canister, sns_agent},
        types::{
            ListProposalsRequest, ListProposalsResponse, SnsProposalId, SnsTopic, SnsTopicSelector,
        },
    },
    source::{MainnetSns, MainnetSnsProposalPage, MainnetSnsProposals, SnsFetchRequest},
};

/// Fetch a bounded SNS governance proposal listing from one resolved SNS.
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

/// Fetch one unfiltered proposal page for complete snapshot refresh.
pub(super) async fn fetch_mainnet_sns_proposal_page_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    before_proposal_id: Option<u64>,
) -> Result<MainnetSnsProposalPage, SnsHostError> {
    let proposals = fetch_mainnet_sns_proposals_async(
        request,
        sns,
        limit,
        before_proposal_id,
        &[],
        SnsProposalTopicFilter::Any,
    )
    .await?
    .proposals;
    let last_cursor = proposals
        .iter()
        .rev()
        .find_map(|proposal| proposal.proposal_id);
    Ok(MainnetSnsProposalPage {
        proposals,
        last_cursor,
    })
}

/// Build Candid topic selectors for a concrete SNS proposal topic filter.
fn sns_topic_selectors(topic: SnsProposalTopicFilter) -> Option<Vec<SnsTopicSelector>> {
    sns_topic(topic).map(|topic| vec![SnsTopicSelector { topic: Some(topic) }])
}

/// Convert a report topic filter into a Candid SNS topic.
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
