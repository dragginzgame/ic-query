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
#[cfg(all(test, feature = "host"))]
pub use reports::{SnsCustomProposalCriticality, SnsVotingRewardsParameters};
#[cfg(feature = "host")]
pub use reports::{
    SnsGovernanceParameters, SnsInfoReport, SnsListReport, SnsListRow, SnsNeuronPermissionList,
    SnsNeuronRow, SnsNeuronsCacheListReport, SnsNeuronsCacheStatusReport, SnsNeuronsCacheSummary,
    SnsNeuronsRefreshAttemptStatus, SnsNeuronsRefreshReport, SnsNeuronsReport, SnsParamsReport,
    SnsProposalBallotRow, SnsProposalFailureReason, SnsProposalReport, SnsProposalRow,
    SnsProposalTally, SnsProposalsCacheListReport, SnsProposalsCacheStatusReport,
    SnsProposalsCacheSummary, SnsProposalsRefreshAttemptStatus, SnsProposalsRefreshReport,
    SnsProposalsReport, SnsTokenMetadataRow, SnsTokenReport, SnsTokenStandardRow,
};
#[cfg(not(feature = "host"))]
pub use reports::{SnsListReport, SnsListRow};
#[cfg(not(feature = "host"))]
pub use requests::SnsListRequest;
#[cfg(feature = "host")]
pub use requests::{
    SnsInfoRequest, SnsListRequest, SnsLookupRequest, SnsNeuronsCacheListRequest,
    SnsNeuronsCacheStatusRequest, SnsNeuronsRefreshRequest, SnsNeuronsRequest, SnsParamsRequest,
    SnsProposalRequest, SnsProposalsCacheListRequest, SnsProposalsCacheStatusRequest,
    SnsProposalsRefreshRequest, SnsProposalsRequest, SnsTokenRequest,
};
#[cfg(not(feature = "host"))]
pub use sorts::SnsListSort;
#[cfg(feature = "host")]
pub(in crate::sns::report) use sorts::{
    SNS_PROPOSAL_STATUS_ADOPTED_CODE, SNS_PROPOSAL_STATUS_REJECTED_CODE,
};
#[cfg(all(test, feature = "host"))]
pub(in crate::sns::report) use sorts::{
    SNS_PROPOSAL_STATUS_EXECUTED_CODE, SNS_PROPOSAL_STATUS_OPEN_CODE,
};
#[cfg(feature = "host")]
pub use sorts::{
    SnsListSort, SnsNeuronsSort, SnsProposalEligibilityFilter, SnsProposalSortDirection,
    SnsProposalStatusFilter, SnsProposalTopicFilter, SnsProposalsSort,
};
