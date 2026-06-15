mod list;
mod single;

use super::block_on_sns;
use crate::sns::report::{
    SnsHostError,
    source::{MainnetSns, MainnetSnsProposal, MainnetSnsProposals, SnsFetchRequest},
};
use list::fetch_mainnet_sns_proposals_async;
use single::fetch_mainnet_sns_proposal_async;

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
