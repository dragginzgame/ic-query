//! Module: sns::commands::options
//!
//! Responsibility: expose SNS command option DTOs parsed from clap matches.
//! Does not own: clap command definitions, command dispatch, or reports.
//! Boundary: keeps parsed command inputs scoped to SNS runtime code.

mod list;
mod lookup;
mod metrics;
mod neurons;
mod proposals;
mod reward;

pub(super) use list::{SnsCatalogRefreshOptions, SnsListOptions};
pub(super) use lookup::SnsLookupOptions;
pub(super) use metrics::SnsMetricsOptions;
pub(super) use neurons::{
    SnsNeuronOptions, SnsNeuronsCacheListOptions, SnsNeuronsCacheStatusOptions, SnsNeuronsOptions,
    SnsNeuronsRefreshOptions,
};
pub(super) use proposals::{
    SnsProposalOptions, SnsProposalsCacheListOptions, SnsProposalsCacheStatusOptions,
    SnsProposalsOptions, SnsProposalsRefreshOptions,
};
pub(super) use reward::{SnsRewardCheckpointOptions, SnsRewardDiffOptions};
