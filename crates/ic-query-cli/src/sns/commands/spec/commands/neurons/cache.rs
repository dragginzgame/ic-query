//! Module: sns::commands::spec::commands::neurons::cache
//!
//! Responsibility: build clap specs for SNS neuron cache inspection commands.
//! Does not own: cache discovery, cache status reports, or command execution.
//! Boundary: defines local-only cache command shape and examples.

use crate::cli::clap::with_options;
use crate::cli::common::{COLLECTION_MODE_CACHE_ONLY, collection_help, json_arg};
use clap::Command as ClapCommand;

const SNS_NEURONS_CACHE_HELP_AFTER: &str = "\
Examples:
  icq sns neuron cache list
  icq sns neuron cache status <governance-canister-id>
  icq sns neuron cache status <governance-canister-id>
  icq sns neuron cache status <governance-canister-id> --json";

const SNS_NEURONS_CACHE_LIST_HELP_AFTER: &str = "\
Examples:
  icq sns neuron cache list
  icq sns neuron cache list --json";

const SNS_NEURONS_CACHE_STATUS_HELP_AFTER: &str = "\
Examples:
  icq sns neuron cache status <governance-canister-id>
  icq sns neuron cache status <governance-canister-id>
  icq sns neuron cache status <governance-canister-id> --json";

pub(in crate::sns::commands) fn sns_neuron_cache_command() -> ClapCommand {
    ClapCommand::new("cache")
        .about("Inspect local complete SNS governance neuron snapshots")
        .subcommand(sns_neuron_cache_list_command())
        .subcommand(sns_neuron_cache_status_command())
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            SNS_NEURONS_CACHE_HELP_AFTER,
        ))
}

pub(in crate::sns::commands) fn sns_neuron_cache_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .about("List local complete SNS neuron snapshots")
        .arg(json_arg())
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            SNS_NEURONS_CACHE_LIST_HELP_AFTER,
        ))
}

pub(in crate::sns::commands) fn sns_neuron_cache_status_command() -> ClapCommand {
    with_options::<crate::sns::commands::options::SnsCacheStatusOptions>(
        ClapCommand::new("status")
            .about("Show local SNS neuron snapshot and refresh-attempt status"),
    )
    .after_help(collection_help(
        COLLECTION_MODE_CACHE_ONLY,
        SNS_NEURONS_CACHE_STATUS_HELP_AFTER,
    ))
}
