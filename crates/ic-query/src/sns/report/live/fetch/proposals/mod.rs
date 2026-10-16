//! Module: sns::report::live::fetch::proposals
//!
//! Responsibility: fetch SNS governance proposals.
//! Does not own: lookup resolution, cache storage, report assembly, or rendering.
//! Boundary: queries direct, bounded, and paged proposal data from governance.

mod list;
mod single;
use crate::sns::report::{
    SnsHostError, SnsProposalRow, SnsProposalTopicFilter,
    source::{MainnetSns, SnsSourceRequest},
};
use list::{fetch_mainnet_sns_proposal_page_async, fetch_mainnet_sns_proposals_async};
use single::fetch_mainnet_sns_proposal_async;

/// Fetch one SNS governance proposal by id.
pub(in crate::sns::report::live) async fn fetch_mainnet_sns_proposal(
    source: &crate::sns::LiveSnsSource,
    request: &SnsSourceRequest,
    sns: &MainnetSns,
    proposal_id: u64,
) -> Result<SnsProposalRow, SnsHostError> {
    fetch_mainnet_sns_proposal_async(source, request, sns, proposal_id).await
}

/// Fetch a bounded SNS governance proposal listing.
pub(in crate::sns::report::live) async fn fetch_mainnet_sns_proposals(
    source: &crate::sns::LiveSnsSource,
    request: &SnsSourceRequest,
    sns: &MainnetSns,
    limit: u32,
    before_proposal_id: Option<u64>,
    include_status: &[i32],
    topic: SnsProposalTopicFilter,
) -> Result<Vec<SnsProposalRow>, SnsHostError> {
    fetch_mainnet_sns_proposals_async(
        source,
        request,
        sns,
        limit,
        before_proposal_id,
        include_status,
        topic,
    )
    .await
}

/// Fetch one proposal page for complete proposal snapshot refresh.
pub(in crate::sns::report::live) async fn fetch_mainnet_sns_proposal_page(
    source: &crate::sns::LiveSnsSource,
    request: &SnsSourceRequest,
    sns: &MainnetSns,
    limit: u32,
    before_proposal_id: Option<u64>,
) -> Result<Vec<SnsProposalRow>, SnsHostError> {
    fetch_mainnet_sns_proposal_page_async(source, request, sns, limit, before_proposal_id).await
}
