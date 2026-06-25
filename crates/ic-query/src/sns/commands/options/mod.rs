//! Module: sns::commands::options
//!
//! Responsibility: expose SNS command option DTOs parsed from clap matches.
//! Does not own: clap command definitions, command dispatch, or reports.
//! Boundary: keeps parsed command inputs scoped to SNS runtime code.

mod common;
mod list;
mod lookup;
mod neurons;
mod proposals;

pub(super) use list::SnsListOptions;
pub(super) use lookup::SnsLookupOptions;
pub(super) use neurons::{
    SnsNeuronsCacheListOptions, SnsNeuronsCacheStatusOptions, SnsNeuronsOptions,
    SnsNeuronsRefreshOptions,
};
pub(super) use proposals::{
    SnsProposalOptions, SnsProposalsCacheListOptions, SnsProposalsCacheStatusOptions,
    SnsProposalsOptions, SnsProposalsRefreshOptions,
};
