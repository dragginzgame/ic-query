mod read;
mod refresh;

use crate::nns::NnsCommandError;
use clap::ArgMatches;

pub(in crate::nns) async fn run(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("summary", matches)) => read::run_topology_summary(matches, network).await,
        Some(("coverage", matches)) => read::run_topology_coverage(matches, network).await,
        Some(("versions", matches)) => read::run_topology_versions(matches, network).await,
        Some(("check", matches)) => read::run_topology_check(matches, network).await,
        Some(("gaps", matches)) => read::run_topology_gaps(matches, network).await,
        Some(("capacity", matches)) => read::run_topology_capacity(matches, network).await,
        Some(("regions", matches)) => read::run_topology_regions(matches, network).await,
        Some(("providers", matches)) => read::run_topology_providers(matches, network).await,
        Some(("refresh", matches)) => refresh::run_topology_refresh(matches, network).await,
        _ => unreachable!("clap requires a known NNS topology subcommand"),
    }
}
