//! Module: sns::commands::spec::commands::neurons::cache
//!
//! Responsibility: build clap specs for SNS neuron cache inspection commands.
//! Does not own: cache discovery, cache status reports, or command execution.
//! Boundary: defines local-only cache command shape and examples.

use crate::{
    cli::{clap::passthrough_subcommand, common::format_arg, globals::internal_network_arg},
    sns::commands::spec::commands::args::sns_lookup_input_arg,
};
use clap::Command as ClapCommand;

const SNS_NEURONS_CACHE_HELP_AFTER: &str = "\
Examples:
  icq sns neurons cache list
  icq sns neurons cache status 1
  icq sns neurons cache status 23ten-uaaaa-aaaaq-aabia-cai
  icq sns neurons cache status 1 --format json";

const SNS_NEURONS_CACHE_LIST_HELP_AFTER: &str = "\
Examples:
  icq sns neurons cache list
  icq sns neurons cache list --format json";

const SNS_NEURONS_CACHE_STATUS_HELP_AFTER: &str = "\
Examples:
  icq sns neurons cache status 1
  icq sns neurons cache status 23ten-uaaaa-aaaaq-aabia-cai
  icq sns neurons cache status 1 --format json";

pub(in crate::sns::commands) fn sns_neurons_cache_command() -> ClapCommand {
    ClapCommand::new("cache")
        .bin_name("icq sns neurons cache")
        .about("Inspect local complete SNS governance neuron snapshots")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about("List local complete SNS neuron snapshots"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("status")
                .about("Show local SNS neuron snapshot and refresh-attempt status"),
        ))
        .after_help(SNS_NEURONS_CACHE_HELP_AFTER)
}

pub(in crate::sns::commands) fn sns_neurons_cache_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq sns neurons cache list")
        .about("List local complete SNS neuron snapshots")
        .disable_help_flag(true)
        .arg(format_arg())
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_NEURONS_CACHE_LIST_HELP_AFTER)
}

pub(in crate::sns::commands) fn sns_neurons_cache_status_command() -> ClapCommand {
    ClapCommand::new("status")
        .bin_name("icq sns neurons cache status")
        .about("Show local SNS neuron snapshot and refresh-attempt status")
        .disable_help_flag(true)
        .arg(sns_lookup_input_arg())
        .arg(format_arg())
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_NEURONS_CACHE_STATUS_HELP_AFTER)
}
