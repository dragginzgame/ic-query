//! Module: sns::report::model::reports
//!
//! Responsibility: group SNS report DTOs by report family.
//! Does not own: report construction, source fetching, cache IO, or rendering.
//! Boundary: re-exports serializable report models used by SNS output writers.

#[cfg(feature = "host")]
mod attempt;
#[cfg(feature = "host")]
mod governance;
mod list;
#[cfg(feature = "host")]
mod neurons;
#[cfg(feature = "host")]
mod params;
#[cfg(feature = "host")]
mod proposals;
mod token;

#[cfg(all(test, feature = "host"))]
pub use governance::{SnsCustomProposalCriticality, SnsVotingRewardsParameters};
#[cfg(feature = "host")]
pub use governance::{SnsGovernanceParameters, SnsNeuronPermissionList};
pub use list::{SnsInfoReport, SnsListReport, SnsListRow};
#[cfg(feature = "host")]
pub use neurons::{
    SnsNeuronRow, SnsNeuronsCacheListReport, SnsNeuronsCacheStatusReport, SnsNeuronsCacheSummary,
    SnsNeuronsRefreshAttemptStatus, SnsNeuronsRefreshReport, SnsNeuronsReport,
};
#[cfg(feature = "host")]
pub use params::SnsParamsReport;
#[cfg(feature = "host")]
pub(in crate::sns::report) use proposals::{
    SNS_PROPOSAL_DECISION_DECIDED, SNS_PROPOSAL_DECISION_EXECUTED, SNS_PROPOSAL_DECISION_FAILED,
    SNS_PROPOSAL_DECISION_OPEN,
};
#[cfg(feature = "host")]
pub use proposals::{
    SnsProposalBallotRow, SnsProposalFailureReason, SnsProposalReport, SnsProposalRow,
    SnsProposalTally, SnsProposalsCacheListReport, SnsProposalsCacheStatusReport,
    SnsProposalsCacheSummary, SnsProposalsRefreshAttemptStatus, SnsProposalsRefreshReport,
    SnsProposalsReport,
};
pub use token::{SnsTokenMetadataRow, SnsTokenReport, SnsTokenStandardRow};
