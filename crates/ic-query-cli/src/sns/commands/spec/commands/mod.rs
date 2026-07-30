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
        clap::{flag_arg, passthrough_subcommand, value_arg},
        common::{COLLECTION_MODE_LIVE, collection_help, format_arg, source_endpoint_arg},
        globals::internal_network_arg,
    },
    sns::commands::spec::values::SnsListSortArg,
};
use clap::Command as ClapCommand;
use ic_query::sns::DEFAULT_SNS_SOURCE_ENDPOINT;

pub(in crate::sns::commands) use canisters::{sns_canister_command, sns_canister_list_command};
pub(in crate::sns::commands) use lookup::{
    sns_info_command, sns_params_command, sns_token_command,
};
pub(in crate::sns::commands) use neurons::{
    sns_neuron_cache_command, sns_neuron_cache_list_command, sns_neuron_cache_status_command,
    sns_neuron_command, sns_neuron_list_command, sns_neuron_refresh_command,
};
pub(in crate::sns::commands) use proposals::{
    sns_proposal_cache_command, sns_proposal_cache_list_command, sns_proposal_cache_status_command,
    sns_proposal_command, sns_proposal_info_command, sns_proposal_list_command,
    sns_proposal_refresh_command,
};

const SNS_LIST_HELP_AFTER: &str = "\
Examples:
  icq sns list
  icq sns list --sort name
  icq sns list --verbose
  icq --network ic sns list --format json
  icq sns list --source-endpoint https://icp-api.io";

pub(in crate::sns::commands) fn sns_command() -> ClapCommand {
    ClapCommand::new("sns")
        .bin_name("icq sns")
        .about("Inspect SNS metadata")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about("List deployed mainnet SNS instances"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("info").about("Resolve a deployed SNS by list id or root principal"),
        ))
        .subcommand(passthrough_subcommand(ClapCommand::new("token").about(
            "Show SNS ledger token metadata by list id or root principal",
        )))
        .subcommand(passthrough_subcommand(ClapCommand::new("params").about(
            "Show SNS governance nervous system parameters by list id or root principal",
        )))
        .subcommand(passthrough_subcommand(ClapCommand::new("canister").about(
            "Inspect SNS Root canister inventory and operational health",
        )))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("proposal")
                .about("List, inspect, and refresh SNS governance proposals"),
        ))
        .subcommand(passthrough_subcommand(ClapCommand::new("neuron").about(
            "List and refresh SNS governance neurons by SNS list id or root principal",
        )))
}

pub(in crate::sns::commands) fn sns_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq sns list")
        .about("List deployed mainnet SNS instances")
        .disable_help_flag(true)
        .arg(format_arg())
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
        .arg(internal_network_arg().default_value("ic"))
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
