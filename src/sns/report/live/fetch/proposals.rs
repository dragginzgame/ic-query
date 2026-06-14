use super::{block_on_sns, governance_canister};
use crate::sns::report::{
    SnsHostError,
    live::{
        convert::sns_proposal_row,
        query::{query_canister, sns_agent},
        types::{
            GetProposalRequest, GetProposalResponse, GetProposalResult, ListProposalsRequest,
            ListProposalsResponse, SnsProposalId,
        },
    },
    source::{MainnetSns, MainnetSnsProposal, MainnetSnsProposals, SnsFetchRequest},
};

pub(in crate::sns::report::live) fn fetch_mainnet_sns_proposal(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    proposal_id: u64,
) -> Result<MainnetSnsProposal, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_proposal_async(request, sns, proposal_id))
}

pub(in crate::sns::report::live) fn fetch_mainnet_sns_proposals(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    before_proposal_id: Option<u64>,
    include_status: &[i32],
) -> Result<MainnetSnsProposals, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_proposals_async(
        request,
        sns,
        limit,
        before_proposal_id,
        include_status,
    ))
}

async fn fetch_mainnet_sns_proposal_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    proposal_id: u64,
) -> Result<MainnetSnsProposal, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let governance_canister = governance_canister(sns)?;
    let response: GetProposalResponse = query_canister(
        &agent,
        &governance_canister,
        "get_proposal",
        "GetProposalRequest",
        "GetProposalResponse",
        &GetProposalRequest {
            proposal_id: Some(SnsProposalId { id: proposal_id }),
        },
    )
    .await?;
    match response.result {
        Some(GetProposalResult::Proposal(proposal)) => Ok(MainnetSnsProposal {
            proposal: sns_proposal_row(*proposal),
        }),
        Some(GetProposalResult::Error(err)) => Err(SnsHostError::GovernanceError {
            method: "get_proposal",
            error_type: err.error_type,
            message: err.error_message,
        }),
        None => Err(SnsHostError::MissingGovernanceResult {
            method: "get_proposal",
        }),
    }
}

async fn fetch_mainnet_sns_proposals_async(
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
