//! Module: sns::commands::spec::commands::neurons::root
//!
//! Responsibility: build the clap specs for `icq sns neuron` and its list view.
//! Does not own: neuron list execution, cache selection, or report output.
//! Boundary: defines list options, owner filtering input, and examples.

use crate::{
    cli::{
        clap::{flag_arg, passthrough_subcommand, value_arg},
        common::{
            COLLECTION_MODE_LIVE_OR_CACHE_BY_VIEW, collection_help, format_arg, source_endpoint_arg,
        },
        globals::internal_network_arg,
    },
    sns::commands::spec::commands::{
        args::{principal_value_parser, sns_lookup_input_arg},
        neurons::sort::neurons_sort_arg,
    },
};
use clap::{Command as ClapCommand, builder::RangedU64ValueParser};
use ic_query::sns::DEFAULT_SNS_SOURCE_ENDPOINT;

const SNS_NEURONS_DEFAULT_LIMIT: &str = "25";

const SNS_NEURONS_HELP_AFTER: &str = "\
Examples:
  icq sns neuron list 1
  icq sns neuron list 23ten-uaaaa-aaaaq-aabia-cai --limit 10
  icq sns neuron list 1 --owner zqfso-syaaa-aaaaq-aaafq-cai
  icq sns neuron list 1 --verbose
  icq sns neuron refresh 1
  icq sns neuron cache list
  icq sns neuron cache status 1
  icq sns neuron list 1 --limit 500 --sort stake
  icq --network ic sns neuron list 1 --format json";

pub(in crate::sns::commands) fn sns_neuron_command() -> ClapCommand {
    ClapCommand::new("neuron")
        .bin_name("icq sns neuron")
        .about("List and refresh SNS governance neurons by SNS list id or root principal")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about("List SNS governance neurons"),
        ))
        .subcommand(passthrough_subcommand(ClapCommand::new("refresh").about(
            "Force-refresh and cache a complete SNS governance neuron snapshot",
        )))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("cache").about("Inspect local complete SNS neuron snapshots"),
        ))
}

pub(in crate::sns::commands) fn sns_neuron_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq sns neuron list")
        .about("List SNS governance neurons by SNS list id or root principal")
        .disable_help_flag(true)
        .arg(sns_lookup_input_arg())
        .arg(format_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for SNS-W and governance queries"),
        )
        .arg(
            value_arg("limit")
                .long("limit")
                .value_name("count")
                .default_value(SNS_NEURONS_DEFAULT_LIMIT)
                .value_parser(RangedU64ValueParser::<u32>::new().range(1..))
                .help("Maximum rows to show; --sort api can request at most 100 live neurons"),
        )
        .arg(
            value_arg("owner")
                .long("owner")
                .value_name("principal")
                .value_parser(principal_value_parser())
                .help("Filter neurons by controlling principal"),
        )
        .arg(
            flag_arg("verbose")
                .long("verbose")
                .help("Show full neuron IDs in text output"),
        )
        .arg(neurons_sort_arg())
        .arg(internal_network_arg().default_value("ic"))
        .after_help(collection_help(
            COLLECTION_MODE_LIVE_OR_CACHE_BY_VIEW,
            SNS_NEURONS_HELP_AFTER,
        ))
}
