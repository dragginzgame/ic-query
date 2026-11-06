use super::args::SubnetListOptions;
use crate::cli::clap::with_options;
use crate::cli::common::{COLLECTION_MODE_CACHE_REFRESH_MISSING_OR_INVALID, collection_help};
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
    with_options::<SubnetListOptions>(ClapCommand::new("list"))
        .about("List cached mainnet IC subnets")
        .mut_arg("source-endpoint", |arg| {
            arg.default_value(DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT)
                .help("IC API endpoint used if the subnet catalog cache is missing or invalid")
        })
        .after_help(collection_help(
            COLLECTION_MODE_CACHE_REFRESH_MISSING_OR_INVALID,
            LIST_HELP_AFTER,
        ))
}
