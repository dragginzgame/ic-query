use crate::{
    cli::clap::{flag_arg, passthrough_subcommand, render_help, value_arg},
    nns::leaf,
    subnet_catalog::{
        DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT, GeographicScope, ResolveAs, SubnetKind,
        SubnetSpecialization,
    },
};
use clap::Command as ClapCommand;

#[cfg(test)]
pub(in crate::nns) const DEFAULT_RANGE_LIMIT: usize = 50;
const DEFAULT_RANGE_LIMIT_ARG: &str = "50";
const INFO_INPUT_VALUE_NAME: &str = "subnet|canister|subnet-prefix";
const INFO_INPUT_HELP: &str = "Subnet/canister principal or unique subnet prefix";
const LIST_HELP_AFTER: &str = "\
Examples:
  icq nns subnet list
  icq nns subnet list --verbose
  icq --network ic nns subnet list --format json
  icq nns subnet list --kind application --specialization fiduciary

Refresh stale cache:
  icq nns subnet refresh";
const INFO_HELP_AFTER: &str = "\
Examples:
  icq nns subnet info ryjl3-tyaaa-aaaaa-aaaba-cai
  icq nns subnet info <subnet-prefix>

Refresh stale cache:
  icq nns subnet refresh";
const REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns subnet refresh
  icq --network ic nns subnet refresh --format json
  icq nns subnet refresh --dry-run --output .icq/subnet-catalog/ic/catalog.preview.json";

pub(super) fn subnet_command() -> ClapCommand {
    ClapCommand::new("subnet")
        .bin_name("icq nns subnet")
        .about("Inspect and refresh NNS subnet metadata")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about("List cached mainnet IC subnets"),
        ))
        .subcommand(passthrough_subcommand(ClapCommand::new("info").about(
            "Resolve a subnet, canister, or subnet prefix to cached subnet info",
        )))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("refresh").about("Force-refresh and cache NNS subnet metadata"),
        ))
}

pub(super) fn list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq nns subnet list")
        .about("List cached mainnet IC subnets")
        .disable_help_flag(true)
        .arg(
            value_arg("kind")
                .long("kind")
                .value_name("kind")
                .value_parser(clap::value_parser!(SubnetKind))
                .help("Filter by subnet kind: application, cloud_engine, system, or unknown"),
        )
        .arg(
            value_arg("specialization")
                .long("specialization")
                .value_name("specialization")
                .value_parser(clap::value_parser!(SubnetSpecialization))
                .help("Filter by specialization: none, fiduciary, european, or unknown"),
        )
        .arg(
            value_arg("geo")
                .long("geo")
                .value_name("scope")
                .value_parser(clap::value_parser!(GeographicScope))
                .help("Filter by geographic scope: global, europe, or unknown"),
        )
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT)
                .help("IC API endpoint used if the subnet catalog cache is missing"),
        )
        .arg(
            flag_arg("show-ranges")
                .long("show-ranges")
                .help("Show cached routing ranges after the subnet table"),
        )
        .arg(
            flag_arg("verbose")
                .long("verbose")
                .help("Show full subnet principals and catalog metadata in text output"),
        )
        .arg(
            value_arg("range-limit")
                .long("range-limit")
                .value_name("n")
                .default_value(DEFAULT_RANGE_LIMIT_ARG)
                .value_parser(clap::builder::RangedU64ValueParser::<usize>::new().range(1u64..))
                .help("Maximum routing ranges to show per subnet in text output"),
        )
        .arg(
            value_arg("range-offset")
                .long("range-offset")
                .value_name("n")
                .default_value("0")
                .value_parser(clap::value_parser!(usize))
                .help("Routing range offset for text output"),
        )
        .arg(leaf::network_arg())
        .after_help(LIST_HELP_AFTER)
}

pub(super) fn info_command() -> ClapCommand {
    ClapCommand::new("info")
        .bin_name("icq nns subnet info")
        .about("Resolve a subnet, canister, or subnet prefix to cached subnet info")
        .disable_help_flag(true)
        .arg(
            value_arg("input")
                .value_name(INFO_INPUT_VALUE_NAME)
                .required(true)
                .help(INFO_INPUT_HELP),
        )
        .arg(
            value_arg("as")
                .long("as")
                .value_name("subnet|canister")
                .value_parser(clap::value_parser!(ResolveAs))
                .help("Force principal interpretation"),
        )
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT)
                .help("IC API endpoint used if the subnet catalog cache is missing"),
        )
        .arg(leaf::network_arg())
        .after_help(INFO_HELP_AFTER)
}

pub(super) fn refresh_command() -> ClapCommand {
    ClapCommand::new("refresh")
        .bin_name("icq nns subnet refresh")
        .about("Force-refresh and cache NNS subnet metadata")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT)
                .help("IC API endpoint used for the NNS registry query"),
        )
        .arg(leaf::refresh_lock_stale_after_arg())
        .arg(
            flag_arg("dry-run")
                .long("dry-run")
                .help("Fetch and validate without replacing the cached catalog"),
        )
        .arg(leaf::output_path_arg().help("Also write the fetched catalog JSON to this path"))
        .arg(leaf::network_arg())
        .after_help(REFRESH_HELP_AFTER)
}

pub(in crate::nns) fn subnet_usage() -> String {
    render_help(subnet_command())
}

pub(in crate::nns) fn list_usage() -> String {
    render_help(list_command())
}

pub(in crate::nns) fn info_usage() -> String {
    render_help(info_command())
}

pub(in crate::nns) fn refresh_usage() -> String {
    render_help(refresh_command())
}
