use super::core::{TopologyReadRunner, run_topology_read};
use crate::{
    nns::NnsCommandError,
    nns::topology::{
        commands::{
            topology_capacity_usage, topology_coverage_usage, topology_gaps_usage,
            topology_health_usage, topology_providers_usage, topology_regions_usage,
            topology_summary_usage, topology_versions_usage,
        },
        options::{
            TopologyCapacityOptions, TopologyCoverageOptions, TopologyGapsOptions,
            TopologyHealthOptions, TopologyProvidersOptions, TopologyRegionsOptions,
            TopologySummaryOptions, TopologyVersionsOptions,
        },
        report::{
            NnsTopologyCapacityReport, NnsTopologyCapacityRequest, NnsTopologyCoverageReport,
            NnsTopologyCoverageRequest, NnsTopologyGapsReport, NnsTopologyGapsRequest,
            NnsTopologyHealthReport, NnsTopologyHealthRequest, NnsTopologyHostError,
            NnsTopologyProvidersReport, NnsTopologyProvidersRequest, NnsTopologyRegionsReport,
            NnsTopologyRegionsRequest, NnsTopologySummaryReport, NnsTopologySummaryRequest,
            NnsTopologyVersionsReport, NnsTopologyVersionsRequest,
            build_nns_topology_capacity_report, build_nns_topology_coverage_report,
            build_nns_topology_gaps_report, build_nns_topology_health_report,
            build_nns_topology_providers_report, build_nns_topology_regions_report,
            build_nns_topology_summary_report, build_nns_topology_versions_report,
            nns_topology_capacity_report_text, nns_topology_coverage_report_text,
            nns_topology_gaps_report_text, nns_topology_health_report_text,
            nns_topology_providers_report_text, nns_topology_regions_report_text,
            nns_topology_summary_report_text, nns_topology_versions_report_text,
        },
    },
};
use std::ffi::OsString;

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

        pub(in crate::nns::topology::run) fn $name<I>(args: I) -> Result<(), NnsCommandError>
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
