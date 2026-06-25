//! Module: sns::report::live::fetch::proposals
//!
//! Responsibility: fetch SNS governance proposals.
//! Does not own: lookup resolution, cache storage, report assembly, or rendering.
//! Boundary: queries direct, bounded, and paged proposal data from governance.

mod list;
mod single;

use super::block_on_sns;
use crate::sns::report::{
    SnsHostError, SnsProposalTopicFilter,
    source::{
        MainnetSns, MainnetSnsProposal, MainnetSnsProposalPage, MainnetSnsProposals,
        SnsFetchRequest,
    },
};
use list::{fetch_mainnet_sns_proposal_page_async, fetch_mainnet_sns_proposals_async};
use single::fetch_mainnet_sns_proposal_async;

/// Fetch one SNS governance proposal by id.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_proposal(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    proposal_id: u64,
) -> Result<MainnetSnsProposal, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_proposal_async(request, sns, proposal_id))
}

/// Fetch a bounded SNS governance proposal listing.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_proposals(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    before_proposal_id: Option<u64>,
    include_status: &[i32],
    topic: SnsProposalTopicFilter,
) -> Result<MainnetSnsProposals, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_proposals_async(
        request,
        sns,
        limit,
        before_proposal_id,
        include_status,
        topic,
    ))
}

/// Fetch one proposal page for complete proposal snapshot refresh.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_proposal_page(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    before_proposal_id: Option<u64>,
) -> Result<MainnetSnsProposalPage, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_proposal_page_async(
        request,
        sns,
        limit,
        before_proposal_id,
    ))
}
