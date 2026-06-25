//! Module: sns::report::model::reports::proposals
//!
//! Responsibility: group SNS proposal report DTOs.
//! Does not own: live governance calls, proposal conversion, cache storage, or rendering.
//! Boundary: re-exports serializable proposal report models.

mod cache;
mod refresh;
mod report;
mod row;

pub type SnsProposalsRefreshAttemptStatus = super::attempt::SnsRefreshAttemptStatus;
pub use cache::{
    SnsProposalsCacheListReport, SnsProposalsCacheStatusReport, SnsProposalsCacheSummary,
};
pub use refresh::SnsProposalsRefreshReport;
pub use report::{SnsProposalReport, SnsProposalsReport};
pub(in crate::sns::report) use row::{
    SNS_PROPOSAL_DECISION_DECIDED, SNS_PROPOSAL_DECISION_EXECUTED, SNS_PROPOSAL_DECISION_FAILED,
    SNS_PROPOSAL_DECISION_OPEN,
};
pub use row::{SnsProposalBallotRow, SnsProposalFailureReason, SnsProposalRow, SnsProposalTally};
