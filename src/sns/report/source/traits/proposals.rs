//! Module: sns::report::source::traits::proposals
//!
//! Responsibility: SNS proposal source contracts.
//! Does not own: live governance transport, proposal conversion, or rendering.
//! Boundary: extends deployed SNS lookup sources with proposal fetching.

use super::super::{MainnetSns, MainnetSnsProposal, MainnetSnsProposals, SnsFetchRequest};
use super::list::SnsListSource;
use crate::sns::report::{SnsHostError, SnsProposalTopicFilter};

///
/// SnsProposalSource
///
/// Source contract for fetching one SNS proposal by id.
///

pub(in crate::sns::report) trait SnsProposalSource: SnsListSource {
    /// Fetch one SNS governance proposal for one resolved SNS.
    fn fetch_sns_proposal(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        proposal_id: u64,
    ) -> Result<MainnetSnsProposal, SnsHostError>;
}

///
/// SnsProposalsSource
///
/// Source contract for fetching bounded SNS proposal listings.
///

pub(in crate::sns::report) trait SnsProposalsSource: SnsListSource {
    /// Fetch a bounded SNS governance proposal page for one resolved SNS.
    fn fetch_sns_proposals(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &[i32],
        topic: SnsProposalTopicFilter,
    ) -> Result<MainnetSnsProposals, SnsHostError>;
}
