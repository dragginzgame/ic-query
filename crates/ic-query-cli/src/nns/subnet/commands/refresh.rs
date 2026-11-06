use crate::cli::clap::with_options;
use crate::cli::common::{COLLECTION_MODE_FORCE_REFRESH, collection_help};
use clap::Command as ClapCommand;
use ic_query::subnet_catalog::DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT;

const REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns subnet refresh
  icq --network ic nns subnet refresh --json
  icq nns subnet refresh --dry-run --output /tmp/icq-subnet-catalog.preview.json";

pub(in crate::nns) fn refresh_command() -> ClapCommand {
    with_options::<crate::nns::leaf::NnsLeafRefreshOptions>(
        ClapCommand::new("refresh")
            .about("Force-refresh and cache NNS subnet metadata")
            .after_help(collection_help(
                COLLECTION_MODE_FORCE_REFRESH,
                REFRESH_HELP_AFTER,
            )),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT)
            .help("IC API endpoint used for the NNS registry query")
    })
    .mut_arg("dry-run", |arg| {
        arg.help("Fetch and validate without replacing the cached catalog")
    })
    .mut_arg("output", |arg| {
        arg.help("Also write the fetched catalog JSON to this path")
    })
}
