//! Module: sns::commands::spec::commands::neurons::cache
//!
//! Responsibility: build clap specs for SNS neuron cache inspection commands.
//! Does not own: cache discovery, cache status reports, or command execution.
//! Boundary: defines local-only cache command shape and examples.

use crate::{
    cli::{
        clap::passthrough_subcommand,
        common::{COLLECTION_MODE_CACHE_ONLY, collection_help, json_arg},
        globals::internal_network_arg,
    },
    sns::commands::spec::commands::args::sns_lookup_input_arg,
};
use clap::Command as ClapCommand;

const SNS_NEURONS_CACHE_HELP_AFTER: &str = "\
Examples:
  icq sns neuron cache list
  icq sns neuron cache status 1
  icq sns neuron cache status 23ten-uaaaa-aaaaq-aabia-cai
  icq sns neuron cache status 1 --json";

const SNS_NEURONS_CACHE_LIST_HELP_AFTER: &str = "\
Examples:
  icq sns neuron cache list
  icq sns neuron cache list --json";

const SNS_NEURONS_CACHE_STATUS_HELP_AFTER: &str = "\
Examples:
  icq sns neuron cache status 1
  icq sns neuron cache status 23ten-uaaaa-aaaaq-aabia-cai
  icq sns neuron cache status 1 --json";

pub(in crate::sns::commands) fn sns_neuron_cache_command() -> ClapCommand {
    ClapCommand::new("cache")
        .bin_name("icq sns neuron cache")
        .about("Inspect local complete SNS governance neuron snapshots")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about("List local complete SNS neuron snapshots"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("status")
                .about("Show local SNS neuron snapshot and refresh-attempt status"),
        ))
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            SNS_NEURONS_CACHE_HELP_AFTER,
        ))
}

pub(in crate::sns::commands) fn sns_neuron_cache_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq sns neuron cache list")
        .about("List local complete SNS neuron snapshots")
        .disable_help_flag(true)
        .arg(json_arg())
        .arg(internal_network_arg().default_value("ic"))
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            SNS_NEURONS_CACHE_LIST_HELP_AFTER,
        ))
}

pub(in crate::sns::commands) fn sns_neuron_cache_status_command() -> ClapCommand {
    ClapCommand::new("status")
        .bin_name("icq sns neuron cache status")
        .about("Show local SNS neuron snapshot and refresh-attempt status")
        .disable_help_flag(true)
        .arg(sns_lookup_input_arg())
        .arg(json_arg())
        .arg(internal_network_arg().default_value("ic"))
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_ONLY,
            SNS_NEURONS_CACHE_STATUS_HELP_AFTER,
        ))
}
