use crate::{
    nns::topology::{commands as topology_commands, options as topology_options, report},
    nns::{
        NnsCommandError, command_args, command_icp_root, now_unix_secs,
        topology::options::TopologyReadOptions,
    },
    nns::{topology::report::NnsTopologyHostError, write_text_or_json},
};
use serde::Serialize;
use std::ffi::OsString;

macro_rules! topology_read_runner {
    (
        $name:ident,
        $options:ty,
        $request:ty,
        $report:ty,
        $usage:path,
        $build:path,
        $render:path
    ) => {
        pub(in crate::nns::topology::run) fn $name(
            args: Vec<OsString>,
        ) -> Result<(), NnsCommandError> {
            run_topology_read::<$options, $request, $report>(args, $usage, $build, $render)
        }
    };
}

topology_read_runner!(
    run_topology_summary,
    topology_options::TopologySummaryOptions,
    report::NnsTopologySummaryRequest,
    report::NnsTopologySummaryReport,
    topology_commands::topology_summary_usage,
    report::build_nns_topology_summary_report,
    report::nns_topology_summary_report_text
);
topology_read_runner!(
    run_topology_coverage,
    topology_options::TopologyCoverageOptions,
    report::NnsTopologyCoverageRequest,
    report::NnsTopologyCoverageReport,
    topology_commands::topology_coverage_usage,
    report::build_nns_topology_coverage_report,
    report::nns_topology_coverage_report_text
);
topology_read_runner!(
    run_topology_versions,
    topology_options::TopologyVersionsOptions,
    report::NnsTopologyVersionsRequest,
    report::NnsTopologyVersionsReport,
    topology_commands::topology_versions_usage,
    report::build_nns_topology_versions_report,
    report::nns_topology_versions_report_text
);
topology_read_runner!(
    run_topology_health,
    topology_options::TopologyHealthOptions,
    report::NnsTopologyHealthRequest,
    report::NnsTopologyHealthReport,
    topology_commands::topology_health_usage,
    report::build_nns_topology_health_report,
    report::nns_topology_health_report_text
);
topology_read_runner!(
    run_topology_gaps,
    topology_options::TopologyGapsOptions,
    report::NnsTopologyGapsRequest,
    report::NnsTopologyGapsReport,
    topology_commands::topology_gaps_usage,
    report::build_nns_topology_gaps_report,
    report::nns_topology_gaps_report_text
);
topology_read_runner!(
    run_topology_capacity,
    topology_options::TopologyCapacityOptions,
    report::NnsTopologyCapacityRequest,
    report::NnsTopologyCapacityReport,
    topology_commands::topology_capacity_usage,
    report::build_nns_topology_capacity_report,
    report::nns_topology_capacity_report_text
);
topology_read_runner!(
    run_topology_regions,
    topology_options::TopologyRegionsOptions,
    report::NnsTopologyRegionsRequest,
    report::NnsTopologyRegionsReport,
    topology_commands::topology_regions_usage,
    report::build_nns_topology_regions_report,
    report::nns_topology_regions_report_text
);
topology_read_runner!(
    run_topology_providers,
    topology_options::TopologyProvidersOptions,
    report::NnsTopologyProvidersRequest,
    report::NnsTopologyProvidersReport,
    topology_commands::topology_providers_usage,
    report::build_nns_topology_providers_report,
    report::nns_topology_providers_report_text
);

fn run_topology_read<Options, Request, Report>(
    args: Vec<OsString>,
    usage: fn() -> String,
    build_report: fn(&Request) -> Result<Report, NnsTopologyHostError>,
    render_text: fn(&Report) -> String,
) -> Result<(), NnsCommandError>
where
    Options: TopologyReadOptions<Request>,
    Report: Serialize,
{
    let Some(args) = command_args(args, usage) else {
        return Ok(());
    };
    let options = Options::parse_args(args)?;
    let format = options.format();
    let icp_root = command_icp_root()?;
    let request = options.into_request(icp_root, now_unix_secs()?);
    let report = build_report(&request)?;
    write_text_or_json(format, &report, render_text)
}
