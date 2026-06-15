use super::super::governance_canister;
use crate::sns::report::{
    SnsHostError,
    live::{
        convert::sns_proposal_row,
        query::{query_canister, sns_agent},
        types::{ListProposalsRequest, ListProposalsResponse, SnsProposalId},
    },
    source::{MainnetSns, MainnetSnsProposals, SnsFetchRequest},
};

pub(super) async fn fetch_mainnet_sns_proposals_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    before_proposal_id: Option<u64>,
    include_status: &[i32],
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
            include_topics: None,
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
