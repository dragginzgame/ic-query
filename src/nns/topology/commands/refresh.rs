use super::super::report::DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT;
use crate::{cli::clap::flag_arg, nns::leaf};

pub(in crate::nns::topology) const DRY_RUN_ARG: &str = "dry-run";
pub(in crate::nns::topology) const LOCK_STALE_AFTER_ARG: &str = "lock-stale-after";

const TOPOLOGY_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns topology refresh
  icq nns topology refresh --dry-run
  icq --network ic nns topology refresh --format json
  icq nns topology refresh --source-endpoint https://icp-api.io";
const TOPOLOGY_REFRESH_SOURCE_HELP: &str =
    "IC API endpoint used for NNS topology component refreshes";

pub(in crate::nns::topology) fn topology_refresh_command() -> clap::Command {
    clap::Command::new("refresh")
        .bin_name("icq nns topology refresh")
        .about("Refresh cached mainnet NNS topology component reports")
        .disable_help_flag(true)
        .arg(leaf::format_arg())
        .arg(
            leaf::source_endpoint_arg(DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT)
                .help(TOPOLOGY_REFRESH_SOURCE_HELP),
        )
        .arg(leaf::refresh_lock_stale_after_arg())
        .arg(
            flag_arg(DRY_RUN_ARG)
                .long(DRY_RUN_ARG)
                .help("Fetch and validate without replacing topology component caches"),
        )
        .arg(leaf::network_arg())
        .after_help(TOPOLOGY_REFRESH_HELP_AFTER)
}
