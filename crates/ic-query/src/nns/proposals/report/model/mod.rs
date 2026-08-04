//! Module: nns::proposals::report::model
//!
//! Responsibility: expose NNS proposal request, report, and selection contracts.
//! Does not own: live Governance transport, command parsing, or text rendering.
//! Boundary: preserves one stable facade while cohesive child modules own each contract.

mod reports;
mod requests;
pub(super) mod selection;

pub use reports::{
    NnsProposalBallotRow, NnsProposalListReport, NnsProposalReport, NnsProposalRow,
    NnsProposalTally,
};
pub use requests::{NnsProposalListRequest, NnsProposalRequest};
pub use selection::{
    NnsProposalListSort, NnsProposalRewardStatus, NnsProposalRewardStatusFilter,
    NnsProposalSortDirection, NnsProposalStatus, NnsProposalStatusFilter, NnsProposalTopicFilter,
};
