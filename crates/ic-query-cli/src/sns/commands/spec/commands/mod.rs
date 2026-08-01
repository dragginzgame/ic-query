//! Module: sns::commands::spec::commands
//!
//! Responsibility: build clap command definitions for SNS commands.
//! Does not own: command execution, report requests, or text rendering.
//! Boundary: defines command shape and help examples only.

mod args;
mod canisters;
mod lookup;
mod neurons;
mod proposals;

use crate::{
    cli::{
        clap::{flag_arg, value_arg},
        common::{COLLECTION_MODE_LIVE, collection_help, json_arg, source_endpoint_arg},
    },
    sns::commands::spec::values::SnsListSortArg,
};
use clap::Command as ClapCommand;
use ic_query::sns::DEFAULT_SNS_SOURCE_ENDPOINT;

pub(in crate::sns::commands) use canisters::sns_canister_command;
#[cfg(test)]
pub(in crate::sns::commands) use canisters::sns_canister_list_command;
pub(in crate::sns::commands) use lookup::{
    sns_info_command, sns_params_command, sns_swap_command, sns_token_command, sns_upgrade_command,
};
pub(in crate::sns::commands) use neurons::sns_neuron_command;
#[cfg(test)]
pub(in crate::sns::commands) use neurons::{
    sns_neuron_cache_command, sns_neuron_cache_list_command, sns_neuron_cache_status_command,
    sns_neuron_list_command, sns_neuron_refresh_command,
};
pub(in crate::sns::commands) use proposals::sns_proposal_command;
#[cfg(test)]
pub(in crate::sns::commands) use proposals::{
    sns_proposal_cache_list_command, sns_proposal_cache_status_command, sns_proposal_info_command,
    sns_proposal_list_command, sns_proposal_refresh_command,
};

const SNS_LIST_HELP_AFTER: &str = "\
Examples:
  icq sns list
  icq sns list --sort name
  icq sns list --verbose
  icq --network ic sns list --json
  icq sns list --source-endpoint https://icp-api.io";

pub(in crate::sns::commands) fn sns_command() -> ClapCommand {
    ClapCommand::new("sns")
        .bin_name("icq sns")
        .about("Inspect SNS metadata")
        .subcommand_required(true)
        .subcommand(sns_list_command())
        .subcommand(sns_info_command())
        .subcommand(sns_token_command())
        .subcommand(sns_params_command())
        .subcommand(sns_swap_command())
        .subcommand(sns_upgrade_command())
        .subcommand(sns_canister_command())
        .subcommand(sns_proposal_command())
        .subcommand(sns_neuron_command())
}

pub(in crate::sns::commands) fn sns_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq sns list")
        .about("List deployed mainnet SNS instances")
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for SNS-W and governance metadata queries"),
        )
        .arg(
            flag_arg("verbose")
                .long("verbose")
                .help("Show full canister IDs in text output"),
        )
        .arg(sort_arg())
        .after_help(collection_help(COLLECTION_MODE_LIVE, SNS_LIST_HELP_AFTER))
}

fn sort_arg() -> clap::Arg {
    value_arg("sort")
        .long("sort")
        .value_name("id|name")
        .default_value("id")
        .value_parser(clap::value_parser!(SnsListSortArg))
        .help("Text/JSON row order; ids follow the SNS-W response order")
}
