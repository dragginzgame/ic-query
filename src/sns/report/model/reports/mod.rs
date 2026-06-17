//! Module: sns::report::model::reports
//!
//! Responsibility: group SNS report DTOs by report family.
//! Does not own: report construction, source fetching, cache IO, or rendering.
//! Boundary: re-exports serializable report models used by SNS output writers.

mod governance;
mod list;
mod neurons;
mod params;
mod proposals;
mod token;

#[cfg(test)]
pub use governance::{SnsCustomProposalCriticality, SnsVotingRewardsParameters};
pub use governance::{SnsGovernanceParameters, SnsNeuronPermissionList};
pub use list::{SnsInfoReport, SnsListReport, SnsListRow};
pub use neurons::{
    SnsNeuronRow, SnsNeuronsCacheListReport, SnsNeuronsCacheStatusReport, SnsNeuronsCacheSummary,
    SnsNeuronsRefreshAttemptStatus, SnsNeuronsRefreshReport, SnsNeuronsReport,
};
pub use params::SnsParamsReport;
pub use proposals::{
    SnsProposalBallotRow, SnsProposalFailureReason, SnsProposalReport, SnsProposalRow,
    SnsProposalTally, SnsProposalsCacheListReport, SnsProposalsCacheStatusReport,
    SnsProposalsCacheSummary, SnsProposalsRefreshAttemptStatus, SnsProposalsRefreshReport,
    SnsProposalsReport,
};
pub use token::{SnsTokenMetadataRow, SnsTokenReport, SnsTokenStandardRow};
