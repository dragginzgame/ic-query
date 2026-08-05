use super::args::{
    geo_arg, kind_arg, range_limit_arg, range_offset_arg, show_ranges_arg, specialization_arg,
    verbose_arg,
};
use crate::{
    cli::common::{COLLECTION_MODE_CACHE_REFRESH_MISSING_OR_INVALID, collection_help},
    nns::leaf,
};
use clap::Command as ClapCommand;
use ic_query::subnet_catalog::DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT;

const LIST_HELP_AFTER: &str = "\
Examples:
  icq nns subnet list
  icq nns subnet list --verbose
  icq --network ic nns subnet list --json
  icq nns subnet list --kind application --specialization fiduciary

Refresh stale cache:
  icq nns subnet refresh";

pub(in crate::nns) fn list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq nns subnet list")
        .about("List cached mainnet IC subnets")
        .arg(kind_arg())
        .arg(specialization_arg())
        .arg(geo_arg())
        .arg(leaf::json_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT)
                .help("IC API endpoint used if the subnet catalog cache is missing or invalid"),
        )
        .arg(show_ranges_arg())
        .arg(verbose_arg())
        .arg(range_limit_arg())
        .arg(range_offset_arg())
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_REFRESH_MISSING_OR_INVALID,
            LIST_HELP_AFTER,
        ))
}
