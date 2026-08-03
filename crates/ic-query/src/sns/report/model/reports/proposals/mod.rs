//! Module: sns::report::model::reports::proposals
//!
//! Responsibility: group SNS proposal report DTOs.
//! Does not own: live governance calls, proposal conversion, cache storage, or rendering.
//! Boundary: re-exports serializable proposal report models.

#[cfg(feature = "host")]
mod refresh;
mod report;
mod row;

#[cfg(feature = "host")]
pub use refresh::SnsProposalsRefreshReport;
pub use report::{SnsProposalReport, SnsProposalsReport};
pub use row::{
    SnsProposalBallotRow, SnsProposalDecisionState, SnsProposalFailureReason, SnsProposalRow,
    SnsProposalTally,
};
