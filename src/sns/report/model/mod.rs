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
pub(in crate::sns::report) use reports::{
    SNS_PROPOSAL_DECISION_DECIDED, SNS_PROPOSAL_DECISION_EXECUTED, SNS_PROPOSAL_DECISION_FAILED,
    SNS_PROPOSAL_DECISION_OPEN,
};
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
pub(in crate::sns::report) use sorts::{
    SNS_PROPOSAL_STATUS_ADOPTED_CODE, SNS_PROPOSAL_STATUS_REJECTED_CODE,
};
#[cfg(test)]
pub(in crate::sns::report) use sorts::{
    SNS_PROPOSAL_STATUS_EXECUTED_CODE, SNS_PROPOSAL_STATUS_OPEN_CODE,
};
pub use sorts::{
    SnsListSort, SnsNeuronsSort, SnsProposalSortDirection, SnsProposalStatusFilter,
    SnsProposalTopicFilter, SnsProposalsSort,
};
