mod read;
mod refresh;

use super::commands::topology_command;
use crate::nns::NnsCommandError;
use clap::ArgMatches;

pub(in crate::nns) fn command() -> clap::Command {
    topology_command()
}

pub(in crate::nns) fn run(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("summary", matches)) => read::run_topology_summary(matches, network),
        Some(("coverage", matches)) => read::run_topology_coverage(matches, network),
        Some(("versions", matches)) => read::run_topology_versions(matches, network),
        Some(("health", matches)) => read::run_topology_health(matches, network),
        Some(("gaps", matches)) => read::run_topology_gaps(matches, network),
        Some(("capacity", matches)) => read::run_topology_capacity(matches, network),
        Some(("regions", matches)) => read::run_topology_regions(matches, network),
        Some(("providers", matches)) => read::run_topology_providers(matches, network),
        Some(("refresh", matches)) => refresh::run_topology_refresh(matches, network),
        _ => unreachable!("clap requires a known NNS topology subcommand"),
    }
}
