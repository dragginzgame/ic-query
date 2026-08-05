use crate::{
    nns::topology::options as topology_options,
    nns::write_text_or_json,
    nns::{
        NnsCommandError, command_cache_root, now_unix_secs, topology::options::TopologyReadOptions,
    },
};
use clap::ArgMatches;
use ic_query::nns::topology::{self, NnsTopologyHostError};
use serde::Serialize;

macro_rules! topology_read_runner {
    (
        $name:ident,
        $options:ty,
        $report:ty,
        $build:path,
        $render:path
    ) => {
        pub(in crate::nns::topology::run) fn $name(
            matches: &ArgMatches,
            network: &str,
        ) -> Result<(), NnsCommandError> {
            run_topology_read::<$options, $report>(matches, network, $build, $render)
        }
    };
}

topology_read_runner!(
    run_topology_summary,
    topology_options::TopologySummaryOptions,
    topology::NnsTopologySummaryReport,
    topology::build_nns_topology_summary_report,
    topology::nns_topology_summary_report_text
);
topology_read_runner!(
    run_topology_coverage,
    topology_options::TopologyCoverageOptions,
    topology::NnsTopologyCoverageReport,
    topology::build_nns_topology_coverage_report,
    topology::nns_topology_coverage_report_text
);
topology_read_runner!(
    run_topology_versions,
    topology_options::TopologyVersionsOptions,
    topology::NnsTopologyVersionsReport,
    topology::build_nns_topology_versions_report,
    topology::nns_topology_versions_report_text
);
topology_read_runner!(
    run_topology_check,
    topology_options::TopologyCheckOptions,
    topology::NnsTopologyCheckReport,
    topology::build_nns_topology_check_report,
    topology::nns_topology_check_report_text
);
topology_read_runner!(
    run_topology_gaps,
    topology_options::TopologyGapsOptions,
    topology::NnsTopologyGapsReport,
    topology::build_nns_topology_gaps_report,
    topology::nns_topology_gaps_report_text
);
topology_read_runner!(
    run_topology_capacity,
    topology_options::TopologyCapacityOptions,
    topology::NnsTopologyCapacityReport,
    topology::build_nns_topology_capacity_report,
    topology::nns_topology_capacity_report_text
);
topology_read_runner!(
    run_topology_regions,
    topology_options::TopologyRegionsOptions,
    topology::NnsTopologyRegionsReport,
    topology::build_nns_topology_regions_report,
    topology::nns_topology_regions_report_text
);
topology_read_runner!(
    run_topology_providers,
    topology_options::TopologyProvidersOptions,
    topology::NnsTopologyProvidersReport,
    topology::build_nns_topology_providers_report,
    topology::nns_topology_providers_report_text
);

fn run_topology_read<Options, Report>(
    matches: &ArgMatches,
    network: &str,
    build_report: fn(&topology::NnsTopologyReadRequest) -> Result<Report, NnsTopologyHostError>,
    render_text: fn(&Report) -> String,
) -> Result<(), NnsCommandError>
where
    Options: TopologyReadOptions,
    Report: Serialize,
{
    let options = Options::from_matches(matches, network);
    let format = options.format();
    let cache_root = command_cache_root()?;
    let request = options.into_request(cache_root, now_unix_secs()?);
    let report = build_report(&request)?;
    write_text_or_json(format, &report, render_text)
}
