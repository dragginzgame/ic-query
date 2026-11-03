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
pub(in crate::sns::report) mod neurons;
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
pub use cache::{
    SnsCacheAdmission, SnsCacheListReport, SnsCacheStatusReport, SnsCacheSummary,
    SnsValidCacheSummary,
};
pub use canisters::{
    MainnetSnsCanisterInventory, SnsCanisterCycleBalanceStatus, SnsCanisterGap, SnsCanisterGapKind,
    SnsCanisterHealthQueryGap, SnsCanisterReport, SnsCanisterRole, SnsCanisterRow,
    SnsCanisterStatus,
};
pub use governance::{
    SnsCustomProposalCriticality, SnsDefaultFollowees, SnsDefaultFolloweesRow,
    SnsGovernanceParameters, SnsNeuronPermissionList, SnsVotingRewardsParameters,
};
pub use invocation::{SnsCanisterCallType, SnsCanisterMethod};
pub use list::{SnsInfoReport, SnsLifecycleOutcome, SnsListReport, SnsListRow};
pub use metrics::{SnsMetricsReport, SnsTreasuryMetricRow, SnsVotingPowerMetrics};
#[cfg(feature = "sns-host")]
pub use neurons::SnsNeuronsReport;
pub use neurons::{
    SnsMaturityDisbursementRow, SnsNeuronAccount, SnsNeuronDetail, SnsNeuronDetailReport,
    SnsNeuronDissolveState, SnsNeuronFolloweeRow, SnsNeuronFolloweesRow, SnsNeuronPermissionRow,
    SnsNeuronRow, SnsNeuronTopicFolloweesRow, SnsPolicyObservationStatus,
    sns_neuron_permission_name,
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
pub use upgrade::{SnsPendingUpgrade, SnsUpgradeQueryGap, SnsUpgradeReport, SnsVersion};

pub use metrics::MainnetSnsMetrics;

pub use swap::MainnetSnsSwap;

pub use proposals::sns_proposal_action_label;
