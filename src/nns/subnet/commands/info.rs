use super::args::{INFO_INPUT_HELP, INFO_INPUT_VALUE_NAME};
use crate::{
    cli::clap::value_arg,
    nns::leaf,
    subnet_catalog::{DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT, ResolveAs},
};
use clap::Command as ClapCommand;

const INFO_HELP_AFTER: &str = "\
Examples:
  icq nns subnet info ryjl3-tyaaa-aaaaa-aaaba-cai
  icq nns subnet info <subnet-prefix>

Refresh stale cache:
  icq nns subnet refresh";

pub(in crate::nns::subnet) fn info_command() -> ClapCommand {
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
