//! Module: sns::report::model::reports
//!
//! Responsibility: group SNS report DTOs by report family.
//! Does not own: report construction, source fetching, cache IO, or rendering.
//! Boundary: re-exports serializable report models used by SNS output writers.

#[cfg(feature = "host")]
mod attempt;
#[cfg(feature = "host")]
mod cache;
mod canisters;
mod governance;
mod list;
mod metrics;
mod neurons;
mod params;
mod proposals;
mod swap;
mod token;
mod upgrade;

#[cfg(feature = "host")]
pub use attempt::SnsRefreshAttemptStatus;
#[cfg(feature = "host")]
pub use cache::{SnsCacheListReport, SnsCacheStatusReport, SnsCacheSummary};
pub use canisters::{
    SnsCanisterCallType, SnsCanisterGap, SnsCanisterGapKind, SnsCanisterReport, SnsCanisterRole,
    SnsCanisterRow, SnsCanisterStatus,
};
pub use governance::{
    SnsCustomProposalCriticality, SnsDefaultFollowees, SnsDefaultFolloweesRow,
    SnsGovernanceParameters, SnsNeuronPermissionList, SnsVotingRewardsParameters,
};
pub use list::{SnsInfoReport, SnsListReport, SnsListRow};
pub use metrics::{SnsMetricsReport, SnsTreasuryKind, SnsTreasuryMetricRow, SnsVotingPowerMetrics};
pub use neurons::{
    SnsMaturityDisbursementRow, SnsNeuronAccount, SnsNeuronDetail, SnsNeuronDetailReport,
    SnsNeuronDissolveState, SnsNeuronFolloweeRow, SnsNeuronFolloweesRow, SnsNeuronPermissionRow,
    SnsNeuronPermissionValue, SnsNeuronRow, SnsNeuronTopicFolloweesRow, SnsPolicyObservationStatus,
    SnsRewardAllocationStatus, SnsRewardCheckpointReport, SnsRewardCheckpointRow,
    SnsRewardCheckpointValidationError, SnsRewardCollectionStatus, SnsRewardDiffCheckpointRef,
    SnsRewardDiffInvalidReason, SnsRewardDiffInvalidReasonKind, SnsRewardDiffReport,
    SnsRewardDiffRow, SnsRewardEvent, SnsRewardProposalId, sns_neuron_permission_name,
    validate_sns_reward_checkpoint_report,
};
#[cfg(feature = "host")]
pub use neurons::{SnsNeuronsRefreshReport, SnsNeuronsReport};
pub(in crate::sns::report) use neurons::{
    SnsRewardCheckpointSummary, recompute_reward_checkpoint_summary,
};
#[cfg(feature = "host")]
pub(in crate::sns::report) use neurons::{
    validate_sns_reward_checkpoint_parameter_evidence, validate_sns_reward_event_evidence,
    validate_sns_reward_running_version_evidence,
};
pub use params::SnsParamsReport;
#[cfg(feature = "host")]
pub use proposals::SnsProposalsRefreshReport;
pub use proposals::{
    SnsProposalBallotRow, SnsProposalDecisionState, SnsProposalFailureReason, SnsProposalReport,
    SnsProposalRow, SnsProposalTally, SnsProposalsReport,
};
pub use swap::{
    SnsSwapComponent, SnsSwapDerivedState, SnsSwapLifecycle,
    SnsSwapNeuronBasketConstructionParameters, SnsSwapQueryGap, SnsSwapReport,
    SnsSwapSaleParameters,
};
pub use token::{SnsTokenMetadataRow, SnsTokenReport, SnsTokenStandardRow};
pub use upgrade::{
    SnsPendingUpgrade, SnsRunningVersionResponse, SnsUpgradeQueryGap, SnsUpgradeReport, SnsVersion,
};
