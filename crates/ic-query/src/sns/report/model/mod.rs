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
pub use reports::{
    SnsCacheListReport, SnsCacheStatusReport, SnsCacheSummary, SnsNeuronsRefreshReport,
    SnsNeuronsReport, SnsProposalsRefreshReport, SnsRefreshAttemptStatus,
};
pub use reports::{
    SnsCanisterGap, SnsCanisterGapKind, SnsCanisterReport, SnsCanisterRole, SnsCanisterRow,
    SnsCanisterStatus, SnsCustomProposalCriticality, SnsDefaultFollowees, SnsDefaultFolloweesRow,
    SnsGovernanceParameters, SnsInfoReport, SnsListReport, SnsListRow, SnsMaturityDisbursementRow,
    SnsMetricsReport, SnsNeuronAccount, SnsNeuronDetail, SnsNeuronDetailReport,
    SnsNeuronDissolveState, SnsNeuronFolloweeRow, SnsNeuronFolloweesRow, SnsNeuronPermissionList,
    SnsNeuronPermissionRow, SnsNeuronPermissionValue, SnsNeuronRow, SnsNeuronTopicFolloweesRow,
    SnsParamsReport, SnsPendingUpgrade, SnsPolicyObservationStatus, SnsProposalBallotRow,
    SnsProposalDecisionState, SnsProposalFailureReason, SnsProposalReport, SnsProposalRow,
    SnsProposalTally, SnsProposalsReport, SnsRewardAllocationStatus, SnsRewardCheckpointReport,
    SnsRewardCheckpointRow, SnsRewardCheckpointValidationError, SnsRewardCollectionStatus,
    SnsRewardDiffCheckpointRef, SnsRewardDiffInvalidReason, SnsRewardDiffInvalidReasonKind,
    SnsRewardDiffReport, SnsRewardDiffRow, SnsRewardEvent, SnsRewardProposalId,
    SnsRunningVersionResponse, SnsSwapComponent, SnsSwapDerivedState, SnsSwapLifecycle,
    SnsSwapNeuronBasketConstructionParameters, SnsSwapQueryGap, SnsSwapReport,
    SnsSwapSaleParameters, SnsTokenMetadataRow, SnsTokenReport, SnsTokenStandardRow,
    SnsTreasuryKind, SnsTreasuryMetricRow, SnsUpgradeQueryGap, SnsUpgradeReport, SnsVersion,
    SnsVotingPowerMetrics, SnsVotingRewardsParameters, sns_neuron_permission_name,
    validate_sns_reward_checkpoint_report,
};
pub(in crate::sns::report) use reports::{
    SnsRewardCheckpointSummary, recompute_reward_checkpoint_summary,
};
#[cfg(feature = "host")]
pub(in crate::sns::report) use reports::{
    validate_sns_reward_checkpoint_parameter_evidence, validate_sns_reward_event_evidence,
    validate_sns_reward_running_version_evidence,
};
pub use requests::{
    DEFAULT_SNS_METRICS_TIME_WINDOW_SECONDS, MAX_SNS_METRICS_TIME_WINDOW_SECONDS, SnsListRequest,
    SnsLookupRequest, SnsMetricsRequest, SnsProposalRequest, SnsProposalsRequest,
};
#[cfg(feature = "host")]
pub use requests::{
    SnsCacheListRequest, SnsCacheStatusRequest, SnsNeuronRequest, SnsNeuronsRefreshRequest,
    SnsNeuronsRequest, SnsProposalsRefreshRequest, SnsRewardCheckpointRequest,
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
