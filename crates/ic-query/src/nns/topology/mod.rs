//! NNS topology requests, reports, host builders, source adapters, and renderers.

mod report;

pub use report::{
    DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT, NNS_TOPOLOGY_CAPACITY_REPORT_SCHEMA_VERSION,
    NNS_TOPOLOGY_COVERAGE_REPORT_SCHEMA_VERSION, NNS_TOPOLOGY_GAPS_REPORT_SCHEMA_VERSION,
    NNS_TOPOLOGY_HEALTH_REPORT_SCHEMA_VERSION, NNS_TOPOLOGY_PROVIDERS_REPORT_SCHEMA_VERSION,
    NNS_TOPOLOGY_REFRESH_REPORT_SCHEMA_VERSION, NNS_TOPOLOGY_REGIONS_REPORT_SCHEMA_VERSION,
    NNS_TOPOLOGY_SUMMARY_REPORT_SCHEMA_VERSION, NNS_TOPOLOGY_VERSIONS_REPORT_SCHEMA_VERSION,
    NnsTopologyCapacityReport, NnsTopologyCapacityRow, NnsTopologyCoverageReport,
    NnsTopologyGapRow, NnsTopologyGapsReport, NnsTopologyHealthCheckRow, NnsTopologyHealthReport,
    NnsTopologyProviderRow, NnsTopologyProvidersReport, NnsTopologyReadRequest,
    NnsTopologyRefreshReport, NnsTopologyRefreshRequest, NnsTopologyRefreshRow,
    NnsTopologyRegionRow, NnsTopologyRegionsReport, NnsTopologyRegistryVersionRow,
    NnsTopologySummaryReport, NnsTopologyVersionsReport, nns_topology_capacity_report_text,
    nns_topology_coverage_report_text, nns_topology_gaps_report_text,
    nns_topology_health_report_text, nns_topology_providers_report_text,
    nns_topology_refresh_report_text, nns_topology_regions_report_text,
    nns_topology_summary_report_text, nns_topology_versions_report_text,
};
#[cfg(feature = "host")]
pub use report::{
    LiveNnsTopologySource, NnsTopologyHostError, NnsTopologyRefreshSource,
    NnsTopologyRefreshSourceRequest, NnsTopologySource, NnsTopologySourceRequest,
    build_nns_topology_capacity_report, build_nns_topology_capacity_report_with_source,
    build_nns_topology_coverage_report, build_nns_topology_coverage_report_with_source,
    build_nns_topology_gaps_report, build_nns_topology_gaps_report_with_source,
    build_nns_topology_health_report, build_nns_topology_health_report_with_source,
    build_nns_topology_providers_report, build_nns_topology_providers_report_with_source,
    build_nns_topology_regions_report, build_nns_topology_regions_report_with_source,
    build_nns_topology_summary_report, build_nns_topology_summary_report_with_source,
    build_nns_topology_versions_report, build_nns_topology_versions_report_with_source,
    refresh_nns_topology_report, refresh_nns_topology_report_with_source,
};
