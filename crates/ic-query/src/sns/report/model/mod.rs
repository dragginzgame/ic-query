//! Module: sns::report::model
//!
//! Responsibility: root SNS report model exports.
//! Does not own: command parsing, live source calls, cache IO, or text output.
//! Boundary: exposes report DTOs, request DTOs, errors, and selectors.

#[cfg(feature = "host")]
mod errors;
mod reports;
mod requests;
mod sorts;

#[cfg(feature = "host")]
pub use errors::SnsHostError;
#[cfg(feature = "host")]
pub(in crate::sns::report) use reports::{
    SNS_PROPOSAL_DECISION_DECIDED, SNS_PROPOSAL_DECISION_EXECUTED, SNS_PROPOSAL_DECISION_FAILED,
    SNS_PROPOSAL_DECISION_OPEN,
};
pub use reports::{
    SnsCustomProposalCriticality, SnsGovernanceParameters, SnsInfoReport, SnsListReport,
    SnsListRow, SnsNeuronPermissionList, SnsParamsReport, SnsProposalBallotRow,
    SnsProposalFailureReason, SnsProposalReport, SnsProposalRow, SnsProposalTally,
    SnsProposalsReport, SnsTokenMetadataRow, SnsTokenReport, SnsTokenStandardRow,
    SnsVotingRewardsParameters,
};
#[cfg(feature = "host")]
pub use reports::{
    SnsNeuronRow, SnsNeuronsCacheListReport, SnsNeuronsCacheStatusReport, SnsNeuronsCacheSummary,
    SnsNeuronsRefreshAttemptStatus, SnsNeuronsRefreshReport, SnsNeuronsReport,
    SnsProposalsCacheListReport, SnsProposalsCacheStatusReport, SnsProposalsCacheSummary,
    SnsProposalsRefreshAttemptStatus, SnsProposalsRefreshReport,
};
pub use requests::{
    SnsInfoRequest, SnsListRequest, SnsLookupRequest, SnsParamsRequest, SnsProposalRequest,
    SnsProposalsRequest, SnsTokenRequest,
};
#[cfg(feature = "host")]
pub use requests::{
    SnsNeuronsCacheListRequest, SnsNeuronsCacheStatusRequest, SnsNeuronsRefreshRequest,
    SnsNeuronsRequest, SnsProposalsCacheListRequest, SnsProposalsCacheStatusRequest,
    SnsProposalsRefreshRequest,
};
#[cfg(feature = "host")]
pub use sorts::SnsNeuronsSort;
#[cfg(feature = "host")]
pub(in crate::sns::report) use sorts::{
    SNS_PROPOSAL_STATUS_ADOPTED_CODE, SNS_PROPOSAL_STATUS_REJECTED_CODE,
};
#[cfg(all(test, feature = "host"))]
pub(in crate::sns::report) use sorts::{
    SNS_PROPOSAL_STATUS_EXECUTED_CODE, SNS_PROPOSAL_STATUS_OPEN_CODE,
};
pub use sorts::{
    SnsListSort, SnsProposalEligibilityFilter, SnsProposalSortDirection, SnsProposalStatusFilter,
    SnsProposalTopicFilter, SnsProposalsSort,
};
