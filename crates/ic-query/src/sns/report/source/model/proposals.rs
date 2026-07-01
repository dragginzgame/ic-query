//! Module: sns::report::source::model::proposals
//!
//! Responsibility: source-layer SNS proposal models.
//! Does not own: governance transport, proposal conversion, or rendering.
//! Boundary: carries converted proposal rows from sources to builders.

use crate::sns::report::SnsProposalRow;

///
/// MainnetSnsProposals
///
/// Source-layer bounded SNS proposal listing.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsProposals {
    pub proposals: Vec<SnsProposalRow>,
}

///
/// MainnetSnsProposalPage
///
/// Source-layer SNS proposal page used by complete snapshot refresh.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsProposalPage {
    pub proposals: Vec<SnsProposalRow>,
    pub last_cursor: Option<u64>,
}

///
/// MainnetSnsProposal
///
/// Source-layer SNS proposal detail result.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsProposal {
    pub proposal: SnsProposalRow,
}
