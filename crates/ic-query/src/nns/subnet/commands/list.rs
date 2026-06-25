use super::args::{
    geo_arg, kind_arg, range_limit_arg, range_offset_arg, show_ranges_arg, specialization_arg,
    verbose_arg,
};
use crate::{nns::leaf, subnet_catalog::DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT};
use clap::Command as ClapCommand;

const LIST_HELP_AFTER: &str = "\
Examples:
  icq nns subnet list
  icq nns subnet list --verbose
  icq --network ic nns subnet list --format json
  icq nns subnet list --kind application --specialization fiduciary

Refresh stale cache:
  icq nns subnet refresh";

pub(in crate::nns::subnet) fn list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq nns subnet list")
        .about("List cached mainnet IC subnets")
        .disable_help_flag(true)
        .arg(kind_arg())
        .arg(specialization_arg())
        .arg(geo_arg())
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT)
                .help("IC API endpoint used if the subnet catalog cache is missing"),
        )
        .arg(show_ranges_arg())
        .arg(verbose_arg())
        .arg(range_limit_arg())
        .arg(range_offset_arg())
        .arg(leaf::network_arg())
        .after_help(LIST_HELP_AFTER)
}
