//! Module: sns::report::model::reports
//!
//! Responsibility: group SNS report DTOs by report family.
//! Does not own: report construction, source fetching, cache IO, or rendering.
//! Boundary: re-exports serializable report models used by SNS output writers.

#[cfg(feature = "sns-host")]
mod attempt;
#[cfg(feature = "sns-host")]
mod cache;
mod canisters;
mod governance;
mod invocation;
mod list;
mod metrics;
mod neurons;
mod params;
mod proposals;
#[cfg(feature = "sns-host")]
mod refresh;
mod swap;
mod token;
mod upgrade;

#[cfg(feature = "sns-host")]
pub use attempt::SnsRefreshAttemptStatus;
#[cfg(feature = "sns-host")]
pub use cache::{SnsCacheListReport, SnsCacheStatusReport, SnsCacheSummary};
pub use canisters::{
    SnsCanisterCycleBalanceStatus, SnsCanisterGap, SnsCanisterGapKind, SnsCanisterHealthQueryGap,
    SnsCanisterReport, SnsCanisterRole, SnsCanisterRow, SnsCanisterStatus,
};
pub use governance::{
    SnsCustomProposalCriticality, SnsDefaultFollowees, SnsDefaultFolloweesRow,
    SnsGovernanceParameters, SnsNeuronPermissionList, SnsVotingRewardsParameters,
};
pub use invocation::{SnsCanisterCallType, SnsCanisterMethod};
pub use list::{SnsInfoReport, SnsListReport, SnsListRow};
pub use metrics::{SnsMetricsReport, SnsTreasuryMetricRow, SnsVotingPowerMetrics};
#[cfg(feature = "sns-host")]
pub use neurons::SnsNeuronsReport;
pub use neurons::{
    SnsMaturityDisbursementRow, SnsNeuronAccount, SnsNeuronDetail, SnsNeuronDetailReport,
    SnsNeuronDissolveState, SnsNeuronFolloweeRow, SnsNeuronFolloweesRow, SnsNeuronPermissionRow,
    SnsNeuronPermissionValue, SnsNeuronRow, SnsNeuronTopicFolloweesRow, SnsPolicyObservationStatus,
    SnsRewardAllocationStatus, SnsRewardCheckpointReport, SnsRewardCheckpointRow,
    SnsRewardCheckpointSummary, SnsRewardCheckpointValidationError, SnsRewardCollectionStatus,
    SnsRewardDiffCheckpointRef, SnsRewardDiffInvalidReason, SnsRewardDiffInvalidReasonKind,
    SnsRewardDiffReport, SnsRewardDiffRow, SnsRewardEvent, SnsRewardProposalId,
    sns_neuron_permission_name, validate_sns_reward_checkpoint_report,
};
pub(in crate::sns::report) use neurons::{
    recompute_reward_checkpoint_summary, validated_reward_checkpoint_summary,
};
#[cfg(feature = "sns-host")]
pub(in crate::sns::report) use neurons::{
    validate_maturity_disbursements, validate_neuron_permissions,
    validate_sns_reward_checkpoint_parameter_evidence, validate_sns_reward_event_evidence,
    validate_sns_reward_running_version_evidence,
};
pub use params::SnsParamsReport;
pub use proposals::{
    SnsProposalBallotRow, SnsProposalDecisionState, SnsProposalFailureReason, SnsProposalReport,
    SnsProposalRow, SnsProposalTally, SnsProposalsReport,
};
#[cfg(feature = "sns-host")]
pub use refresh::SnsSnapshotRefreshReport;
pub use swap::{
    SnsSwapComponent, SnsSwapDerivedState, SnsSwapLifecycle,
    SnsSwapNeuronBasketConstructionParameters, SnsSwapQueryGap, SnsSwapReport,
    SnsSwapSaleParameters,
};
pub use token::SnsTokenReport;
pub use upgrade::{
    SnsPendingUpgrade, SnsRunningVersionResponse, SnsUpgradeQueryGap, SnsUpgradeReport, SnsVersion,
};

pub(in crate::sns::report) use proposals::{sns_proposal_action_label, sns_proposal_vote_label};

pub use metrics::MainnetSnsMetrics;

pub use swap::MainnetSnsSwap;

pub(in crate::sns::report) use swap::sns_swap_lifecycle_name;

pub(in crate::sns::report) use metrics::sns_treasury_kind;
