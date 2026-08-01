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
#[cfg(feature = "host")]
pub use reports::{
    SnsCacheListReport, SnsCacheStatusReport, SnsCacheSummary, SnsNeuronRow,
    SnsNeuronsRefreshReport, SnsNeuronsReport, SnsProposalsRefreshReport, SnsRefreshAttemptStatus,
};
pub use reports::{
    SnsCanisterGap, SnsCanisterGapKind, SnsCanisterReport, SnsCanisterRole, SnsCanisterRow,
    SnsCanisterStatus, SnsCustomProposalCriticality, SnsGovernanceParameters, SnsInfoReport,
    SnsListReport, SnsListRow, SnsMetricsReport, SnsNeuronPermissionList, SnsParamsReport,
    SnsPendingUpgrade, SnsProposalBallotRow, SnsProposalFailureReason, SnsProposalReport,
    SnsProposalRow, SnsProposalTally, SnsProposalsReport, SnsSwapComponent, SnsSwapDerivedState,
    SnsSwapLifecycle, SnsSwapNeuronBasketConstructionParameters, SnsSwapQueryGap, SnsSwapReport,
    SnsSwapSaleParameters, SnsTokenMetadataRow, SnsTokenReport, SnsTokenStandardRow,
    SnsTreasuryKind, SnsTreasuryMetricRow, SnsUpgradeQueryGap, SnsUpgradeReport, SnsVersion,
    SnsVotingPowerMetrics, SnsVotingRewardsParameters,
};
pub use requests::{
    DEFAULT_SNS_METRICS_TIME_WINDOW_SECONDS, MAX_SNS_METRICS_TIME_WINDOW_SECONDS, SnsListRequest,
    SnsLookupRequest, SnsMetricsRequest, SnsProposalRequest, SnsProposalsRequest,
};
#[cfg(feature = "host")]
pub use requests::{
    SnsCacheListRequest, SnsCacheStatusRequest, SnsNeuronsRefreshRequest, SnsNeuronsRequest,
    SnsProposalsRefreshRequest,
};
#[cfg(feature = "host")]
pub(in crate::sns::report) use requests::{
    sns_metrics_lookup_request, validate_sns_metrics_request, validate_sns_metrics_time_window,
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
