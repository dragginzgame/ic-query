//! Module: sns::report::model::reports::proposals
//!
//! Responsibility: group SNS proposal report DTOs.
//! Does not own: live governance calls, proposal conversion, cache storage, or rendering.
//! Boundary: re-exports serializable proposal report models.

mod report;
mod row;

pub use report::{SnsProposalReport, SnsProposalsReport};
pub use row::{
    SnsProposalAction, SnsProposalBallotRow, SnsProposalDecisionState, SnsProposalFailureReason,
    SnsProposalRow, SnsProposalTally, SnsProposalVote,
};
