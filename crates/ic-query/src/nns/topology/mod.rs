//! NNS topology requests, reports, host builders, source adapters, and renderers.

mod report;

#[cfg(feature = "nns-topology-host")]
pub use report::{
    CachedNnsSubnetTopologyReport, DEFAULT_NNS_SUBNET_TOPOLOGY_REFRESH_LOCK_STALE_SECONDS,
    DEFAULT_NNS_SUBNET_TOPOLOGY_SOURCE_ENDPOINT, DEFAULT_NNS_SUBNET_TOPOLOGY_STALE_AFTER_SECONDS,
    NnsSubnetTopologyCacheRequest, NnsSubnetTopologyHostError, NnsSubnetTopologyRefreshRequest,
    NnsSubnetTopologySource, load_cached_nns_subnet_topology,
    load_or_refresh_missing_nns_subnet_topology,
    load_or_refresh_missing_nns_subnet_topology_with_source,
    load_or_refresh_stale_nns_subnet_topology,
    load_or_refresh_stale_nns_subnet_topology_with_source, nns_subnet_topology_cache_path,
    nns_subnet_topology_freshness, nns_subnet_topology_refresh_lock_path,
    refresh_nns_subnet_topology, refresh_nns_subnet_topology_with_source,
};
pub use report::{
    DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT, NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION,
    NNS_TOPOLOGY_CAPACITY_REPORT_SCHEMA_VERSION, NNS_TOPOLOGY_CHECK_REPORT_SCHEMA_VERSION,
    NNS_TOPOLOGY_COVERAGE_REPORT_SCHEMA_VERSION, NNS_TOPOLOGY_GAPS_REPORT_SCHEMA_VERSION,
    NNS_TOPOLOGY_PROVIDERS_REPORT_SCHEMA_VERSION, NNS_TOPOLOGY_REFRESH_REPORT_SCHEMA_VERSION,
    NNS_TOPOLOGY_REGIONS_REPORT_SCHEMA_VERSION, NNS_TOPOLOGY_SUMMARY_REPORT_SCHEMA_VERSION,
    NNS_TOPOLOGY_VERSIONS_REPORT_SCHEMA_VERSION, NnsSubnetNodeProviderRow,
    NnsSubnetTopologyFreshness, NnsSubnetTopologyReport, NnsSubnetTopologyRow,
    NnsSubnetTopologyValidationError, NnsTopologyAssessmentStatus, NnsTopologyCapacityReport,
    NnsTopologyCapacityRow, NnsTopologyCapacityStatus, NnsTopologyCheckReport, NnsTopologyCheckRow,
    NnsTopologyCoverageReport, NnsTopologyGapRelationKind, NnsTopologyGapRow,
    NnsTopologyGapSubjectKind, NnsTopologyGapsReport, NnsTopologyProviderRow,
    NnsTopologyProviderStatus, NnsTopologyProvidersReport, NnsTopologyReadRequest,
    NnsTopologyRefreshReport, NnsTopologyRefreshRequest, NnsTopologyRefreshRow,
    NnsTopologyRegionRow, NnsTopologyRegionsReport, NnsTopologyRegistryVersionRow,
    NnsTopologySummaryReport, NnsTopologyVersionsReport, nns_subnet_topology_report_text,
    nns_topology_capacity_report_text, nns_topology_check_report_text,
    nns_topology_coverage_report_text, nns_topology_gaps_report_text,
    nns_topology_providers_report_text, nns_topology_refresh_report_text,
    nns_topology_regions_report_text, nns_topology_summary_report_text,
    nns_topology_versions_report_text,
};
#[cfg(feature = "nns-host")]
pub use report::{
    NnsTopologyHostError, NnsTopologyRefreshSource, NnsTopologyRefreshSourceRequest,
    NnsTopologySource, NnsTopologySourceRequest, build_nns_topology_capacity_report,
    build_nns_topology_capacity_report_with_source, build_nns_topology_check_report,
    build_nns_topology_check_report_with_source, build_nns_topology_coverage_report,
    build_nns_topology_coverage_report_with_source, build_nns_topology_gaps_report,
    build_nns_topology_gaps_report_with_source, build_nns_topology_providers_report,
    build_nns_topology_providers_report_with_source, build_nns_topology_regions_report,
    build_nns_topology_regions_report_with_source, build_nns_topology_summary_report,
    build_nns_topology_summary_report_with_source, build_nns_topology_versions_report,
    build_nns_topology_versions_report_with_source, refresh_nns_topology_report,
    refresh_nns_topology_report_with_source,
};
