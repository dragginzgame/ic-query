//! Module: sns::report::model
//!
//! Responsibility: root SNS report model exports.
//! Does not own: command parsing, live source calls, cache IO, or text output.
//! Boundary: exposes report DTOs, request DTOs, errors, and selectors.

mod errors;
mod reports;
mod requests;
mod sorts;

pub use errors::SnsHostError;
#[cfg(test)]
pub use reports::{SnsCustomProposalCriticality, SnsVotingRewardsParameters};
pub use reports::{
    SnsGovernanceParameters, SnsInfoReport, SnsListReport, SnsListRow, SnsNeuronPermissionList,
    SnsNeuronRow, SnsNeuronsCacheListReport, SnsNeuronsCacheStatusReport, SnsNeuronsCacheSummary,
    SnsNeuronsRefreshAttemptStatus, SnsNeuronsRefreshReport, SnsNeuronsReport, SnsParamsReport,
    SnsProposalBallotRow, SnsProposalFailureReason, SnsProposalReport, SnsProposalRow,
    SnsProposalTally, SnsProposalsCacheListReport, SnsProposalsCacheStatusReport,
    SnsProposalsCacheSummary, SnsProposalsRefreshAttemptStatus, SnsProposalsRefreshReport,
    SnsProposalsReport, SnsTokenMetadataRow, SnsTokenReport, SnsTokenStandardRow,
};
pub use requests::{
    SnsInfoRequest, SnsListRequest, SnsLookupRequest, SnsNeuronsCacheListRequest,
    SnsNeuronsCacheStatusRequest, SnsNeuronsRefreshRequest, SnsNeuronsRequest, SnsParamsRequest,
    SnsProposalRequest, SnsProposalsCacheListRequest, SnsProposalsCacheStatusRequest,
    SnsProposalsRefreshRequest, SnsProposalsRequest, SnsTokenRequest,
};
pub use sorts::{SnsListSort, SnsNeuronsSort, SnsProposalStatusFilter, SnsProposalTopicFilter};
