//! Module: sns::commands::spec::commands::neurons
//!
//! Responsibility: expose clap specs for SNS neuron list, refresh, and cache commands.
//! Does not own: option parsing, cache policy, or report construction.
//! Boundary: groups neuron command spec leaves under one command family.

mod cache;
mod refresh;
mod root;
mod sort;

pub(in crate::sns::commands) use cache::{
    sns_neuron_cache_command, sns_neuron_cache_list_command, sns_neuron_cache_status_command,
};
pub(in crate::sns::commands) use refresh::sns_neuron_refresh_command;
pub(in crate::sns::commands) use root::{sns_neuron_command, sns_neuron_list_command};
