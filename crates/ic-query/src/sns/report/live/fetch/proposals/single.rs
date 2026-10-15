//! Module: sns::report::live::fetch::proposals::single
//!
//! Responsibility: fetch one SNS governance proposal.
//! Does not own: proposal conversion policy outside direct lookup results.
//! Boundary: maps get_proposal Candid responses into source-layer proposal data.

use crate::candid_call::query_canister;
use crate::sns::report::{
    SnsCanisterMethod, SnsHostError,
    live::{
        convert::sns_proposal_row,
        fetch::governance_canister,
        query::sns_agent,
        types::{GetProposalRequest, GetProposalResponse, GetProposalResult, SnsProposalId},
    },
    source::{MainnetSns, MainnetSnsProposal, SnsSourceRequest},
};

/// Fetch one SNS governance proposal by id from a resolved governance canister.
pub(super) async fn fetch_mainnet_sns_proposal_async(
    source: &crate::sns::LiveSnsSource,
    request: &SnsSourceRequest,
    sns: &MainnetSns,
    proposal_id: u64,
) -> Result<MainnetSnsProposal, SnsHostError> {
    let agent = sns_agent(source, request)?;
    let governance_canister = governance_canister(sns)?;
    let method = SnsCanisterMethod::GetProposal.as_str();
    let response: GetProposalResponse = query_canister::<_, _, SnsHostError>(
        agent,
        &governance_canister,
        method,
        "GetProposalRequest",
        "GetProposalResponse",
        &GetProposalRequest {
            proposal_id: Some(SnsProposalId { id: proposal_id }),
        },
    )
    .await?;
    match response.result {
        Some(GetProposalResult::Proposal(proposal)) => Ok(MainnetSnsProposal {
            proposal: sns_proposal_row(*proposal)?,
        }),
        Some(GetProposalResult::Error(err)) => Err(SnsHostError::GovernanceError {
            method,
            error_type: err.error_type,
            message: err.error_message,
        }),
        None => Err(SnsHostError::MissingGovernanceResult { method }),
    }
}
