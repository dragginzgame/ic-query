//! Module: sns::report::model::reports
//!
//! Responsibility: group SNS report DTOs by report family.
//! Does not own: report construction, source fetching, cache IO, or rendering.
//! Boundary: re-exports serializable report models used by SNS output writers.

mod attempt;
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
pub(in crate::sns::report) use proposals::{
    SNS_PROPOSAL_DECISION_DECIDED, SNS_PROPOSAL_DECISION_EXECUTED, SNS_PROPOSAL_DECISION_FAILED,
    SNS_PROPOSAL_DECISION_OPEN,
};
pub use proposals::{
    SnsProposalBallotRow, SnsProposalFailureReason, SnsProposalReport, SnsProposalRow,
    SnsProposalTally, SnsProposalsCacheListReport, SnsProposalsCacheStatusReport,
    SnsProposalsCacheSummary, SnsProposalsRefreshAttemptStatus, SnsProposalsRefreshReport,
    SnsProposalsReport,
};
pub use token::{SnsTokenMetadataRow, SnsTokenReport, SnsTokenStandardRow};
