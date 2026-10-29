mod read;
mod refresh;

use crate::nns::NnsCommandError;
use clap::ArgMatches;
use ic_query::nns::topology;

pub(in crate::nns) async fn run(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), NnsCommandError> {
    match matches.subcommand() {
        Some(("summary", matches)) => {
            read::run_topology_read(
                matches,
                network,
                topology::build_nns_topology_summary_report,
                topology::nns_topology_summary_report_text,
            )
            .await
        }
        Some(("coverage", matches)) => {
            read::run_topology_read(
                matches,
                network,
                topology::build_nns_topology_coverage_report,
                topology::nns_topology_coverage_report_text,
            )
            .await
        }
        Some(("versions", matches)) => {
            read::run_topology_read(
                matches,
                network,
                topology::build_nns_topology_versions_report,
                topology::nns_topology_versions_report_text,
            )
            .await
        }
        Some(("check", matches)) => {
            read::run_topology_read(
                matches,
                network,
                topology::build_nns_topology_check_report,
                topology::nns_topology_check_report_text,
            )
            .await
        }
        Some(("gaps", matches)) => {
            read::run_topology_read(
                matches,
                network,
                topology::build_nns_topology_gaps_report,
                topology::nns_topology_gaps_report_text,
            )
            .await
        }
        Some(("capacity", matches)) => {
            read::run_topology_read(
                matches,
                network,
                topology::build_nns_topology_capacity_report,
                topology::nns_topology_capacity_report_text,
            )
            .await
        }
        Some(("regions", matches)) => {
            read::run_topology_read(
                matches,
                network,
                topology::build_nns_topology_regions_report,
                topology::nns_topology_regions_report_text,
            )
            .await
        }
        Some(("providers", matches)) => {
            read::run_topology_read(
                matches,
                network,
                topology::build_nns_topology_providers_report,
                topology::nns_topology_providers_report_text,
            )
            .await
        }
        Some(("refresh", matches)) => refresh::run_topology_refresh(matches, network).await,
        _ => unreachable!("clap requires a known NNS topology subcommand"),
    }
}
