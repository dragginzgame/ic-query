//! Module: sns::report::model::reports::proposals
//!
//! Responsibility: group SNS proposal report DTOs.
//! Does not own: live governance calls, proposal conversion, cache storage, or rendering.
//! Boundary: re-exports serializable proposal report models.

mod attempt;
mod cache;
mod refresh;
mod report;
mod row;

pub use attempt::SnsProposalsRefreshAttemptStatus;
pub use cache::{
    SnsProposalsCacheListReport, SnsProposalsCacheStatusReport, SnsProposalsCacheSummary,
};
pub use refresh::SnsProposalsRefreshReport;
pub use report::{SnsProposalReport, SnsProposalsReport};
pub use row::{SnsProposalBallotRow, SnsProposalFailureReason, SnsProposalRow, SnsProposalTally};
