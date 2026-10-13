use crate::cli::common::current_unix_secs;
use crate::{
    nns::write_text_or_json,
    nns::{NnsCommandError, command_cache_root, leaf::NnsCommonOptions},
};
use clap::ArgMatches;
use ic_query::nns::topology::{self, NnsTopologyHostError};
use serde::Serialize;

macro_rules! topology_read_runner {
    (
        $name:ident,
        $report:ty,
        $build:path,
        $render:path
    ) => {
        pub(in crate::nns::topology::run) async fn $name(
            matches: &ArgMatches,
            network: &str,
        ) -> Result<(), NnsCommandError> {
            run_topology_read::<$report>(matches, network, $build, $render).await
        }
    };
}

topology_read_runner!(
    run_topology_summary,
    topology::NnsTopologySummaryReport,
    topology::build_nns_topology_summary_report,
    topology::nns_topology_summary_report_text
);
topology_read_runner!(
    run_topology_coverage,
    topology::NnsTopologyCoverageReport,
    topology::build_nns_topology_coverage_report,
    topology::nns_topology_coverage_report_text
);
topology_read_runner!(
    run_topology_versions,
    topology::NnsTopologyVersionsReport,
    topology::build_nns_topology_versions_report,
    topology::nns_topology_versions_report_text
);
topology_read_runner!(
    run_topology_check,
    topology::NnsTopologyCheckReport,
    topology::build_nns_topology_check_report,
    topology::nns_topology_check_report_text
);
topology_read_runner!(
    run_topology_gaps,
    topology::NnsTopologyGapsReport,
    topology::build_nns_topology_gaps_report,
    topology::nns_topology_gaps_report_text
);
topology_read_runner!(
    run_topology_capacity,
    topology::NnsTopologyCapacityReport,
    topology::build_nns_topology_capacity_report,
    topology::nns_topology_capacity_report_text
);
topology_read_runner!(
    run_topology_regions,
    topology::NnsTopologyRegionsReport,
    topology::build_nns_topology_regions_report,
    topology::nns_topology_regions_report_text
);
topology_read_runner!(
    run_topology_providers,
    topology::NnsTopologyProvidersReport,
    topology::build_nns_topology_providers_report,
    topology::nns_topology_providers_report_text
);

async fn run_topology_read<Report>(
    matches: &ArgMatches,
    network: &str,
    build_report: impl AsyncFn(
        &topology::NnsTopologyReadRequest,
    ) -> Result<Report, NnsTopologyHostError>,
    render_text: fn(&Report) -> String,
) -> Result<(), NnsCommandError>
where
    Report: Serialize,
{
    let options = NnsCommonOptions::from_matches(matches, network);
    let format = options.format;
    let cache_root = command_cache_root()?;
    let request = topology::NnsTopologyReadRequest::new(
        cache_root,
        options.network,
        options.source_endpoint,
        current_unix_secs()?,
    );
    let report = build_report(&request).await?;
    write_text_or_json(format, &report, render_text)
}
