use super::{
    super::values::SnsNeuronsSortArg,
    args::{principal_value_parser, sns_lookup_input_arg},
};
use crate::{
    cli::{
        clap::{flag_arg, passthrough_subcommand, value_arg},
        common::{format_arg, source_endpoint_arg},
        globals::internal_network_arg,
    },
    sns::report::DEFAULT_SNS_SOURCE_ENDPOINT,
};
use clap::{Command as ClapCommand, builder::RangedU64ValueParser};

const SNS_NEURONS_DEFAULT_LIMIT: &str = "25";
const SNS_NEURONS_REFRESH_DEFAULT_PAGE_SIZE: &str = "100";
const SNS_NEURONS_REFRESH_MAX_PAGE_SIZE: u64 = 100;

const SNS_NEURONS_HELP_AFTER: &str = "\
Examples:
  icq sns neurons 1
  icq sns neurons 23ten-uaaaa-aaaaq-aabia-cai --limit 10
  icq sns neurons 1 --owner zqfso-syaaa-aaaaq-aaafq-cai
  icq sns neurons 1 --verbose
  icq sns neurons refresh 1
  icq sns neurons cache list
  icq sns neurons cache status 1
  icq sns neurons 1 --limit 500 --sort stake
  icq --network ic sns neurons 1 --format json";

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

const SNS_NEURONS_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq sns neurons refresh 1
  icq sns neurons refresh 23ten-uaaaa-aaaaq-aabia-cai
  icq sns neurons refresh 1 --page-size 100
  icq --network ic sns neurons refresh 1 --format json";

pub(in crate::sns::commands) fn sns_neurons_command() -> ClapCommand {
    ClapCommand::new("neurons")
        .bin_name("icq sns neurons")
        .about("List and refresh SNS governance neurons by SNS list id or root principal")
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
        .after_help(SNS_NEURONS_HELP_AFTER)
}

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

pub(in crate::sns::commands) fn sns_neurons_refresh_command() -> ClapCommand {
    ClapCommand::new("refresh")
        .bin_name("icq sns neurons refresh")
        .about("Force-refresh and cache a complete SNS governance neuron snapshot")
        .disable_help_flag(true)
        .arg(sns_lookup_input_arg())
        .arg(format_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for SNS-W and governance queries"),
        )
        .arg(
            value_arg("page-size")
                .long("page-size")
                .value_name("count")
                .default_value(SNS_NEURONS_REFRESH_DEFAULT_PAGE_SIZE)
                .value_parser(
                    RangedU64ValueParser::<u32>::new().range(1..=SNS_NEURONS_REFRESH_MAX_PAGE_SIZE),
                )
                .help("Maximum neurons to request per SNS governance page"),
        )
        .arg(
            value_arg("max-pages")
                .long("max-pages")
                .value_name("count")
                .value_parser(RangedU64ValueParser::<u32>::new().range(1..))
                .help("Stop before publishing if this page count is reached before API exhaustion"),
        )
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_NEURONS_REFRESH_HELP_AFTER)
}

fn neurons_sort_arg() -> clap::Arg {
    value_arg("sort")
        .long("sort")
        .value_name("api|id|stake|maturity|created")
        .default_value("api")
        .value_parser(clap::value_parser!(SnsNeuronsSortArg))
        .help("Row order; api uses a bounded live query, other sorts read the complete cache")
}
