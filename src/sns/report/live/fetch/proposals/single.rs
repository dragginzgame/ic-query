use super::super::governance_canister;
use crate::sns::report::{
    SnsHostError,
    live::{
        convert::sns_proposal_row,
        query::{query_canister, sns_agent},
        types::{GetProposalRequest, GetProposalResponse, GetProposalResult, SnsProposalId},
    },
    source::{MainnetSns, MainnetSnsProposal, SnsFetchRequest},
};

pub(super) async fn fetch_mainnet_sns_proposal_async(
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
