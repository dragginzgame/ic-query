use crate::{
    nns::topology::{commands as topology_commands, options as topology_options},
    nns::write_text_or_json,
    nns::{
        NnsCommandError, command_args, command_cache_root, now_unix_secs,
        topology::options::TopologyReadOptions,
    },
};
use ic_query::nns::topology::{self, NnsTopologyHostError};
use serde::Serialize;
use std::ffi::OsString;

macro_rules! topology_read_runner {
    (
        $name:ident,
        $options:ty,
        $report:ty,
        $usage:path,
        $build:path,
        $render:path
    ) => {
        pub(in crate::nns::topology::run) fn $name(
            args: Vec<OsString>,
        ) -> Result<(), NnsCommandError> {
            run_topology_read::<$options, $report>(args, $usage, $build, $render)
        }
    };
}

topology_read_runner!(
    run_topology_summary,
    topology_options::TopologySummaryOptions,
    topology::NnsTopologySummaryReport,
    topology_commands::topology_summary_usage,
    topology::build_nns_topology_summary_report,
    topology::nns_topology_summary_report_text
);
topology_read_runner!(
    run_topology_coverage,
    topology_options::TopologyCoverageOptions,
    topology::NnsTopologyCoverageReport,
    topology_commands::topology_coverage_usage,
    topology::build_nns_topology_coverage_report,
    topology::nns_topology_coverage_report_text
);
topology_read_runner!(
    run_topology_versions,
    topology_options::TopologyVersionsOptions,
    topology::NnsTopologyVersionsReport,
    topology_commands::topology_versions_usage,
    topology::build_nns_topology_versions_report,
    topology::nns_topology_versions_report_text
);
topology_read_runner!(
    run_topology_health,
    topology_options::TopologyHealthOptions,
    topology::NnsTopologyHealthReport,
    topology_commands::topology_health_usage,
    topology::build_nns_topology_health_report,
    topology::nns_topology_health_report_text
);
topology_read_runner!(
    run_topology_gaps,
    topology_options::TopologyGapsOptions,
    topology::NnsTopologyGapsReport,
    topology_commands::topology_gaps_usage,
    topology::build_nns_topology_gaps_report,
    topology::nns_topology_gaps_report_text
);
topology_read_runner!(
    run_topology_capacity,
    topology_options::TopologyCapacityOptions,
    topology::NnsTopologyCapacityReport,
    topology_commands::topology_capacity_usage,
    topology::build_nns_topology_capacity_report,
    topology::nns_topology_capacity_report_text
);
topology_read_runner!(
    run_topology_regions,
    topology_options::TopologyRegionsOptions,
    topology::NnsTopologyRegionsReport,
    topology_commands::topology_regions_usage,
    topology::build_nns_topology_regions_report,
    topology::nns_topology_regions_report_text
);
topology_read_runner!(
    run_topology_providers,
    topology_options::TopologyProvidersOptions,
    topology::NnsTopologyProvidersReport,
    topology_commands::topology_providers_usage,
    topology::build_nns_topology_providers_report,
    topology::nns_topology_providers_report_text
);

fn run_topology_read<Options, Report>(
    args: Vec<OsString>,
    usage: fn() -> String,
    build_report: fn(&topology::NnsTopologyReadRequest) -> Result<Report, NnsTopologyHostError>,
    render_text: fn(&Report) -> String,
) -> Result<(), NnsCommandError>
where
    Options: TopologyReadOptions,
    Report: Serialize,
{
    let Some(args) = command_args(args, usage) else {
        return Ok(());
    };
    let options = Options::parse_args(args)?;
    let format = options.format();
    let cache_root = command_cache_root()?;
    let request = options.into_request(cache_root, now_unix_secs()?);
    let report = build_report(&request)?;
    write_text_or_json(format, &report, render_text)
}
