use super::{
    commands::{
        topology_capacity_usage, topology_command, topology_coverage_usage, topology_gaps_usage,
        topology_health_usage, topology_providers_usage, topology_refresh_usage,
        topology_regions_usage, topology_summary_usage, topology_usage, topology_versions_usage,
    },
    options::{
        TopologyCapacityOptions, TopologyCoverageOptions, TopologyGapsOptions,
        TopologyHealthOptions, TopologyProvidersOptions, TopologyReadOptions,
        TopologyRefreshOptions, TopologyRegionsOptions, TopologySummaryOptions,
        TopologyVersionsOptions,
    },
    report::{
        NnsTopologyCapacityReport, NnsTopologyCapacityRequest, NnsTopologyCoverageReport,
        NnsTopologyCoverageRequest, NnsTopologyGapsReport, NnsTopologyGapsRequest,
        NnsTopologyHealthReport, NnsTopologyHealthRequest, NnsTopologyHostError,
        NnsTopologyProvidersReport, NnsTopologyProvidersRequest, NnsTopologyRefreshRequest,
        NnsTopologyRegionsReport, NnsTopologyRegionsRequest, NnsTopologySummaryReport,
        NnsTopologySummaryRequest, NnsTopologyVersionsReport, NnsTopologyVersionsRequest,
        build_nns_topology_capacity_report, build_nns_topology_coverage_report,
        build_nns_topology_gaps_report, build_nns_topology_health_report,
        build_nns_topology_providers_report, build_nns_topology_regions_report,
        build_nns_topology_summary_report, build_nns_topology_versions_report,
        nns_topology_capacity_report_text, nns_topology_coverage_report_text,
        nns_topology_gaps_report_text, nns_topology_health_report_text,
        nns_topology_providers_report_text, nns_topology_refresh_report_text,
        nns_topology_regions_report_text, nns_topology_summary_report_text,
        nns_topology_versions_report_text, refresh_nns_topology_report,
    },
};
use crate::{
    cli::{
        clap::parse_required_subcommand,
        help::{print_help_or_version, print_help_or_version_flag},
    },
    nns::{NnsCommandError, now_unix_secs, write_text_or_json},
    project::icp_root,
    version_text,
};
use serde::Serialize;
use std::ffi::OsString;

trait TopologyReadRunner {
    type Options: TopologyReadOptions<Self::Request>;
    type Request;
    type Report: Serialize;
    type HostError: Into<NnsCommandError>;

    fn usage() -> String;
    fn build_report(request: &Self::Request) -> Result<Self::Report, Self::HostError>;
    fn render_text(report: &Self::Report) -> String;
}

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version_flag(&args, topology_usage, version_text()) {
        return Ok(());
    }
    let (command, args) = parse_required_subcommand(topology_command(), args)
        .map_err(|_| NnsCommandError::Usage(topology_usage()))?;

    match command.as_str() {
        "summary" => run_topology_summary(args),
        "coverage" => run_topology_coverage(args),
        "versions" => run_topology_versions(args),
        "health" => run_topology_health(args),
        "gaps" => run_topology_gaps(args),
        "capacity" => run_topology_capacity(args),
        "regions" => run_topology_regions(args),
        "providers" => run_topology_providers(args),
        "refresh" => run_topology_refresh(args),
        _ => unreachable!("nns topology dispatch command only defines known commands"),
    }
}

macro_rules! topology_read_runner {
    (
        $runner:ident,
        $name:ident,
        $options:ty,
        $request:ty,
        $report:ty,
        $usage:ident,
        $build:ident,
        $render:ident
    ) => {
        struct $runner;

        impl TopologyReadRunner for $runner {
            type Options = $options;
            type Request = $request;
            type Report = $report;
            type HostError = NnsTopologyHostError;

            fn usage() -> String {
                $usage()
            }

            fn build_report(request: &Self::Request) -> Result<Self::Report, Self::HostError> {
                $build(request)
            }

            fn render_text(report: &Self::Report) -> String {
                $render(report)
            }
        }

        fn $name<I>(args: I) -> Result<(), NnsCommandError>
        where
            I: IntoIterator<Item = OsString>,
        {
            run_topology_read::<_, $runner>(args)
        }
    };
}

topology_read_runner!(
    TopologySummaryRunner,
    run_topology_summary,
    TopologySummaryOptions,
    NnsTopologySummaryRequest,
    NnsTopologySummaryReport,
    topology_summary_usage,
    build_nns_topology_summary_report,
    nns_topology_summary_report_text
);
topology_read_runner!(
    TopologyCoverageRunner,
    run_topology_coverage,
    TopologyCoverageOptions,
    NnsTopologyCoverageRequest,
    NnsTopologyCoverageReport,
    topology_coverage_usage,
    build_nns_topology_coverage_report,
    nns_topology_coverage_report_text
);
topology_read_runner!(
    TopologyVersionsRunner,
    run_topology_versions,
    TopologyVersionsOptions,
    NnsTopologyVersionsRequest,
    NnsTopologyVersionsReport,
    topology_versions_usage,
    build_nns_topology_versions_report,
    nns_topology_versions_report_text
);
topology_read_runner!(
    TopologyHealthRunner,
    run_topology_health,
    TopologyHealthOptions,
    NnsTopologyHealthRequest,
    NnsTopologyHealthReport,
    topology_health_usage,
    build_nns_topology_health_report,
    nns_topology_health_report_text
);
topology_read_runner!(
    TopologyGapsRunner,
    run_topology_gaps,
    TopologyGapsOptions,
    NnsTopologyGapsRequest,
    NnsTopologyGapsReport,
    topology_gaps_usage,
    build_nns_topology_gaps_report,
    nns_topology_gaps_report_text
);
topology_read_runner!(
    TopologyCapacityRunner,
    run_topology_capacity,
    TopologyCapacityOptions,
    NnsTopologyCapacityRequest,
    NnsTopologyCapacityReport,
    topology_capacity_usage,
    build_nns_topology_capacity_report,
    nns_topology_capacity_report_text
);
topology_read_runner!(
    TopologyRegionsRunner,
    run_topology_regions,
    TopologyRegionsOptions,
    NnsTopologyRegionsRequest,
    NnsTopologyRegionsReport,
    topology_regions_usage,
    build_nns_topology_regions_report,
    nns_topology_regions_report_text
);
topology_read_runner!(
    TopologyProvidersRunner,
    run_topology_providers,
    TopologyProvidersOptions,
    NnsTopologyProvidersRequest,
    NnsTopologyProvidersReport,
    topology_providers_usage,
    build_nns_topology_providers_report,
    nns_topology_providers_report_text
);

fn run_topology_read<I, Runner>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
    Runner: TopologyReadRunner,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, Runner::usage, version_text()) {
        return Ok(());
    }
    let options = Runner::Options::parse_args(args)?;
    let format = options.format();
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = options.into_request(icp_root, now_unix_secs()?);
    let report = Runner::build_report(&request).map_err(Into::into)?;
    write_text_or_json(format, &report, Runner::render_text)
}

fn run_topology_refresh<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, topology_refresh_usage, version_text()) {
        return Ok(());
    }
    let options = TopologyRefreshOptions::parse(args)?;
    let format = options.format;
    let icp_root = icp_root().map_err(|err| NnsCommandError::Usage(err.to_string()))?;
    let request = NnsTopologyRefreshRequest {
        icp_root,
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
        lock_stale_after_seconds: options.lock_stale_after_seconds,
        dry_run: options.dry_run,
    };
    let report = refresh_nns_topology_report(&request)?;
    write_text_or_json(format, &report, nns_topology_refresh_report_text)
}
