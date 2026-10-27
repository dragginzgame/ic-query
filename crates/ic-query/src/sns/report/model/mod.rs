//! Module: sns::report::model
//!
//! Responsibility: root SNS report model exports.
//! Does not own: command parsing, live source calls, cache IO, or text output.
//! Boundary: exposes report DTOs, request DTOs, errors, and selectors.

#[cfg(feature = "sns-host")]
mod errors;
mod identity;
pub(in crate::sns::report) mod reports;
mod requests;
mod sorts;

pub use identity::MainnetSnsCanisters;

#[cfg(feature = "sns-host")]
pub use errors::SnsHostError;
#[cfg(feature = "sns-host")]
pub use reports::{
    SnsCacheListReport, SnsCacheStatusReport, SnsCacheSummary, SnsNeuronsReport,
    SnsRefreshAttemptStatus, SnsSnapshotRefreshReport,
};
pub use reports::{
    SnsCanisterCallType, SnsCanisterCycleBalanceStatus, SnsCanisterGap, SnsCanisterGapKind,
    SnsCanisterHealthQueryGap, SnsCanisterMethod, SnsCanisterReport, SnsCanisterRole,
    SnsCanisterRow, SnsCanisterStatus, SnsCustomProposalCriticality, SnsDefaultFollowees,
    SnsDefaultFolloweesRow, SnsGovernanceParameters, SnsInfoReport, SnsListReport, SnsListRow,
    SnsMaturityDisbursementRow, SnsMetricsReport, SnsNeuronAccount, SnsNeuronDetail,
    SnsNeuronDetailReport, SnsNeuronDissolveState, SnsNeuronFolloweeRow, SnsNeuronFolloweesRow,
    SnsNeuronPermissionList, SnsNeuronPermissionRow, SnsNeuronPermissionValue, SnsNeuronRow,
    SnsNeuronTopicFolloweesRow, SnsParamsReport, SnsPendingUpgrade, SnsPolicyObservationStatus,
    SnsProposalBallotRow, SnsProposalDecisionState, SnsProposalFailureReason, SnsProposalReport,
    SnsProposalRow, SnsProposalTally, SnsProposalsReport, SnsRewardAllocationStatus,
    SnsRewardCheckpointReport, SnsRewardCheckpointRow, SnsRewardCheckpointSummary,
    SnsRewardCheckpointValidationError, SnsRewardCollectionStatus, SnsRewardDiffCheckpointRef,
    SnsRewardDiffInvalidReason, SnsRewardDiffInvalidReasonKind, SnsRewardDiffReport,
    SnsRewardDiffRow, SnsRewardEvent, SnsRewardProposalId, SnsRunningVersionResponse,
    SnsSwapComponent, SnsSwapDerivedState, SnsSwapLifecycle,
    SnsSwapNeuronBasketConstructionParameters, SnsSwapQueryGap, SnsSwapReport,
    SnsSwapSaleParameters, SnsTokenReport, SnsTreasuryMetricRow, SnsUpgradeQueryGap,
    SnsUpgradeReport, SnsVersion, SnsVotingPowerMetrics, SnsVotingRewardsParameters,
    sns_neuron_permission_name, validate_sns_reward_checkpoint_report,
};
pub use requests::{
    DEFAULT_SNS_METRICS_TIME_WINDOW_SECONDS, MAX_SNS_METRICS_TIME_WINDOW_SECONDS, SnsListRequest,
    SnsLookupRequest, SnsMetricsRequest, SnsProposalRequest, SnsProposalsRequest,
};
#[cfg(feature = "sns-host")]
pub use requests::{
    SnsCacheListRequest, SnsCacheStatusRequest, SnsNeuronRequest, SnsNeuronsRequest,
    SnsRewardCheckpointRequest, SnsSnapshotRefreshRequest,
};
#[cfg(feature = "sns-host")]
pub(in crate::sns::report) use requests::{
    validate_sns_metrics_request, validate_sns_metrics_time_window,
};
#[cfg(feature = "sns-host")]
pub use sorts::SnsNeuronsSort;
#[cfg(feature = "sns-host")]
pub(in crate::sns::report) use sorts::{
    SNS_PROPOSAL_STATUS_ADOPTED_CODE, SNS_PROPOSAL_STATUS_REJECTED_CODE,
};
#[cfg(all(test, feature = "sns-host"))]
pub(in crate::sns::report) use sorts::{
    SNS_PROPOSAL_STATUS_EXECUTED_CODE, SNS_PROPOSAL_STATUS_OPEN_CODE,
};
pub use sorts::{
    SnsListSort, SnsProposalEligibilityFilter, SnsProposalSortDirection, SnsProposalStatusFilter,
    SnsProposalTopicFilter, SnsProposalsSort,
};

pub(in crate::sns::report) use reports::{sns_proposal_action_label, sns_proposal_vote_label};

pub use reports::MainnetSnsMetrics;

pub use reports::MainnetSnsSwap;

pub(in crate::sns::report) use reports::sns_swap_lifecycle_name;

pub(in crate::sns::report) use reports::sns_treasury_kind;
