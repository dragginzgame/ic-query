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
    SnsGovernanceParameters, SnsNeuronPermissionList, SnsNeuronRow, SnsNeuronsCacheListReport,
    SnsNeuronsCacheStatusReport, SnsNeuronsCacheSummary, SnsNeuronsRefreshAttemptStatus,
    SnsNeuronsRefreshReport, SnsNeuronsReport, SnsParamsReport, SnsProposalBallotRow,
    SnsProposalFailureReason, SnsProposalReport, SnsProposalRow, SnsProposalTally,
    SnsProposalsCacheListReport, SnsProposalsCacheStatusReport, SnsProposalsCacheSummary,
    SnsProposalsRefreshAttemptStatus, SnsProposalsRefreshReport, SnsProposalsReport,
};
pub use reports::{
    SnsInfoReport, SnsListReport, SnsListRow, SnsTokenMetadataRow, SnsTokenReport,
    SnsTokenStandardRow,
};
pub use requests::{SnsInfoRequest, SnsListRequest, SnsLookupRequest, SnsTokenRequest};
#[cfg(feature = "host")]
pub use requests::{
    SnsNeuronsCacheListRequest, SnsNeuronsCacheStatusRequest, SnsNeuronsRefreshRequest,
    SnsNeuronsRequest, SnsParamsRequest, SnsProposalRequest, SnsProposalsCacheListRequest,
    SnsProposalsCacheStatusRequest, SnsProposalsRefreshRequest, SnsProposalsRequest,
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
