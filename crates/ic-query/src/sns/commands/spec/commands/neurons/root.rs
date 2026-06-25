//! Module: sns::commands::spec::commands::neurons::root
//!
//! Responsibility: build the clap spec for `icq sns neurons`.
//! Does not own: neuron list execution, cache selection, or report output.
//! Boundary: defines list options, owner filtering input, and examples.

use crate::{
    cli::{
        clap::{flag_arg, value_arg},
        common::{format_arg, source_endpoint_arg},
        globals::internal_network_arg,
    },
    sns::{
        commands::spec::commands::{
            args::{principal_value_parser, sns_lookup_input_arg},
            nested_dispatch_command,
            neurons::sort::neurons_sort_arg,
        },
        report::DEFAULT_SNS_SOURCE_ENDPOINT,
    },
};
use clap::{Command as ClapCommand, builder::RangedU64ValueParser};

const SNS_NEURONS_DEFAULT_LIMIT: &str = "25";

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

pub(in crate::sns::commands) fn sns_neurons_dispatch_command() -> ClapCommand {
    nested_dispatch_command(
        "neurons",
        "icq sns neurons",
        "Force-refresh and cache a complete SNS governance neuron snapshot",
        "Inspect local complete SNS governance neuron snapshots",
    )
}
