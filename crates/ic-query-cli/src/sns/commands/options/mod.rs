//! Module: sns::commands::options
//!
//! Responsibility: construct SNS requests from clap matches.
//! Does not own: clap command definitions, command dispatch, or reports.
//! Boundary: keeps parsed command inputs scoped to SNS runtime code.

mod list;
mod lookup;
mod metrics;
mod neurons;
mod proposals;
mod reward;

pub(super) use list::list_request;
pub(super) use lookup::{
    cache_list_request, cache_status_request, lookup_request, snapshot_refresh_request,
};
pub(super) use metrics::metrics_request;
pub(super) use neurons::{neuron_request, neurons_request};
pub(super) use proposals::{proposal_request, proposals_request};
pub(super) use reward::reward_checkpoint_request;
