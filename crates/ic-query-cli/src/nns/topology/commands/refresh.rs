use crate::cli::clap::with_options;
use crate::cli::common::{COLLECTION_MODE_FORCE_REFRESH, collection_help};
use ic_query::nns::topology::DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT;

pub(in crate::nns::topology) const DRY_RUN_ARG: &str = "dry-run";

const TOPOLOGY_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns topology refresh
  icq nns topology refresh --dry-run
  icq --network ic nns topology refresh --json
  icq nns topology refresh --source-endpoint https://icp-api.io";
const TOPOLOGY_REFRESH_SOURCE_HELP: &str =
    "IC API endpoint used for NNS topology component refreshes";

///
/// TopologyRefreshOptions
///
/// Typed arguments for the topology_refresh_command operation.
///

#[derive(clap::Args)]
#[group(skip)]
pub(in crate::nns) struct TopologyRefreshOptions {
    #[command(flatten)]
    pub(in crate::nns) source: crate::cli::common::SourceOutputOptions,
    #[arg(id = DRY_RUN_ARG, long = DRY_RUN_ARG, help = "Fetch and validate without replacing topology component caches")]
    pub(in crate::nns) dry_run: bool,
}

pub(in crate::nns) fn topology_refresh_command() -> clap::Command {
    with_options::<TopologyRefreshOptions>(
        clap::Command::new("refresh")
            .about("Refresh cached mainnet NNS topology component reports")
            .after_help(collection_help(
                COLLECTION_MODE_FORCE_REFRESH,
                TOPOLOGY_REFRESH_HELP_AFTER,
            )),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT)
            .help(TOPOLOGY_REFRESH_SOURCE_HELP)
    })
}
