//! Module: sns::report::live::client::proposals
//!
//! Responsibility: live SNS proposal source implementations.
//! Does not own: governance query construction, report assembly, or rendering.
//! Boundary: delegates proposal source traits to live fetch helpers.

use super::LiveSnsSource;
use crate::sns::report::{
    MainnetSns, MainnetSnsProposal, MainnetSnsProposals, SnsFetchRequest, SnsHostError,
    SnsProposalSource, SnsProposalTopicFilter, SnsProposalsSource,
    live::fetch::{fetch_mainnet_sns_proposal, fetch_mainnet_sns_proposals},
};

impl SnsProposalSource for LiveSnsSource {
    fn fetch_sns_proposal(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        proposal_id: u64,
    ) -> Result<MainnetSnsProposal, SnsHostError> {
        fetch_mainnet_sns_proposal(request, sns, proposal_id)
    }
}

impl SnsProposalsSource for LiveSnsSource {
    fn fetch_sns_proposals(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &[i32],
        topic: SnsProposalTopicFilter,
    ) -> Result<MainnetSnsProposals, SnsHostError> {
        fetch_mainnet_sns_proposals(
            request,
            sns,
            limit,
            before_proposal_id,
            include_status,
            topic,
        )
    }
}
