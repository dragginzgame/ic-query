//! Module: sns::commands::spec::commands::neurons::refresh
//!
//! Responsibility: build the clap spec for SNS neuron snapshot refresh.
//! Does not own: refresh execution, progress reporting, or cache storage.
//! Boundary: defines refresh limits, source endpoint option, and examples.

use crate::{
    cli::{
        clap::value_arg, common::format_arg, common::source_endpoint_arg,
        globals::internal_network_arg,
    },
    sns::{
        commands::spec::commands::args::sns_lookup_input_arg, report::DEFAULT_SNS_SOURCE_ENDPOINT,
    },
};
use clap::{Command as ClapCommand, builder::RangedU64ValueParser};

const SNS_NEURONS_REFRESH_DEFAULT_PAGE_SIZE: &str = "100";
const SNS_NEURONS_REFRESH_MAX_PAGE_SIZE: u64 = 100;

const SNS_NEURONS_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq sns neurons refresh 1
  icq sns neurons refresh 23ten-uaaaa-aaaaq-aabia-cai
  icq sns neurons refresh 1 --page-size 100
  icq --network ic sns neurons refresh 1 --format json";

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
