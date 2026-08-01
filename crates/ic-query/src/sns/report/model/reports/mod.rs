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
#[cfg(feature = "host")]
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
    SnsCanisterGap, SnsCanisterGapKind, SnsCanisterReport, SnsCanisterRole, SnsCanisterRow,
    SnsCanisterStatus,
};
pub use governance::{
    SnsCustomProposalCriticality, SnsGovernanceParameters, SnsNeuronPermissionList,
    SnsVotingRewardsParameters,
};
pub use list::{SnsInfoReport, SnsListReport, SnsListRow};
pub use metrics::{SnsMetricsReport, SnsTreasuryKind, SnsTreasuryMetricRow, SnsVotingPowerMetrics};
#[cfg(feature = "host")]
pub use neurons::{
    SnsNeuronDissolveState, SnsNeuronRow, SnsNeuronsRefreshReport, SnsNeuronsReport,
};
pub use params::SnsParamsReport;
#[cfg(feature = "host")]
pub use proposals::SnsProposalsRefreshReport;
#[cfg(feature = "host")]
pub(in crate::sns::report) use proposals::{
    SNS_PROPOSAL_DECISION_DECIDED, SNS_PROPOSAL_DECISION_EXECUTED, SNS_PROPOSAL_DECISION_FAILED,
    SNS_PROPOSAL_DECISION_OPEN,
};
pub use proposals::{
    SnsProposalBallotRow, SnsProposalFailureReason, SnsProposalReport, SnsProposalRow,
    SnsProposalTally, SnsProposalsReport,
};
pub use swap::{
    SnsSwapComponent, SnsSwapDerivedState, SnsSwapLifecycle,
    SnsSwapNeuronBasketConstructionParameters, SnsSwapQueryGap, SnsSwapReport,
    SnsSwapSaleParameters,
};
pub use token::{SnsTokenMetadataRow, SnsTokenReport, SnsTokenStandardRow};
pub use upgrade::{SnsPendingUpgrade, SnsUpgradeQueryGap, SnsUpgradeReport, SnsVersion};
