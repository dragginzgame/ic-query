//! Module: nns::topology::report
//!
//! Responsibility: assemble reusable NNS topology reports and refresh operations.
//! Does not own: CLI parsing, registry transport internals, or process output.
//! Boundary: groups topology requests, sources, projections, and text renderers.

#[cfg(feature = "host")]
mod build;
#[cfg(feature = "host")]
mod capacity;
#[cfg(feature = "host")]
mod check;
#[cfg(feature = "host")]
mod coverage;
#[cfg(feature = "host")]
mod error;
#[cfg(feature = "host")]
mod gaps;
mod model;
mod percent;
#[cfg(feature = "host")]
mod providers;
#[cfg(feature = "host")]
mod refresh;
#[cfg(feature = "host")]
mod regions;
#[cfg(feature = "host")]
mod registry_versions;
#[cfg(feature = "host")]
mod relations;
mod request;
#[cfg(feature = "host")]
mod source;
mod subnet_topology;
#[cfg(feature = "host")]
mod summary;
mod text;
#[cfg(feature = "host")]
mod versions;

#[cfg(all(test, feature = "host"))]
mod tests;

#[cfg(all(test, feature = "host"))]
use capacity::topology_capacity_report_from_report;
#[cfg(all(test, feature = "host"))]
use check::topology_check_report_from_summary;
#[cfg(all(test, feature = "host"))]
use coverage::topology_coverage_report_from_summary;
#[cfg(all(test, feature = "host"))]
use gaps::topology_gaps_report_from_reports;
#[cfg(all(test, feature = "host"))]
use providers::topology_providers_report_from_reports;
#[cfg(all(test, feature = "host"))]
use refresh::{NnsTopologyRefreshComponentReports, topology_refresh_report_from_reports};
#[cfg(all(test, feature = "host"))]
use regions::topology_regions_report_from_report;
#[cfg(all(test, feature = "host"))]
use summary::topology_summary_report_from_reports;
#[cfg(all(test, feature = "host"))]
use versions::topology_versions_report_from_summary;

#[cfg(feature = "host")]
pub use build::{
    build_nns_topology_capacity_report, build_nns_topology_capacity_report_with_source,
    build_nns_topology_check_report, build_nns_topology_check_report_with_source,
    build_nns_topology_coverage_report, build_nns_topology_coverage_report_with_source,
    build_nns_topology_gaps_report, build_nns_topology_gaps_report_with_source,
    build_nns_topology_providers_report, build_nns_topology_providers_report_with_source,
    build_nns_topology_regions_report, build_nns_topology_regions_report_with_source,
    build_nns_topology_summary_report, build_nns_topology_summary_report_with_source,
    build_nns_topology_versions_report, build_nns_topology_versions_report_with_source,
    refresh_nns_topology_report, refresh_nns_topology_report_with_source,
};
#[cfg(feature = "host")]
pub use error::NnsTopologyHostError;
pub use model::{
    NnsTopologyAssessmentStatus, NnsTopologyCapacityReport, NnsTopologyCapacityRow,
    NnsTopologyCapacityStatus, NnsTopologyCheckReport, NnsTopologyCheckRow,
    NnsTopologyCoverageReport, NnsTopologyGapRelationKind, NnsTopologyGapRow,
    NnsTopologyGapSubjectKind, NnsTopologyGapsReport, NnsTopologyProviderRow,
    NnsTopologyProviderStatus, NnsTopologyProvidersReport, NnsTopologyRefreshReport,
    NnsTopologyRefreshRow, NnsTopologyRegionRow, NnsTopologyRegionsReport,
    NnsTopologyRegistryVersionRow, NnsTopologySummaryReport, NnsTopologyVersionsReport,
};
pub use request::{NnsTopologyReadRequest, NnsTopologyRefreshRequest};
#[cfg(feature = "host")]
pub use source::{
    NnsTopologyRefreshSource, NnsTopologyRefreshSourceRequest, NnsTopologySource,
    NnsTopologySourceRequest,
};
#[cfg(feature = "nns-topology-host")]
pub use subnet_topology::{
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
pub use subnet_topology::{
    NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION, NnsSubnetNodeProviderRow,
    NnsSubnetTopologyFreshness, NnsSubnetTopologyReport, NnsSubnetTopologyRow,
    NnsSubnetTopologyValidationError, nns_subnet_topology_report_text,
};
pub use text::{
    nns_topology_capacity_report_text, nns_topology_check_report_text,
    nns_topology_coverage_report_text, nns_topology_gaps_report_text,
    nns_topology_providers_report_text, nns_topology_refresh_report_text,
    nns_topology_regions_report_text, nns_topology_summary_report_text,
    nns_topology_versions_report_text,
};

pub const DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT: &str =
    crate::nns::node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT;
pub const NNS_TOPOLOGY_SUMMARY_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_COVERAGE_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_VERSIONS_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_CHECK_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_GAPS_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_CAPACITY_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_REGIONS_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_PROVIDERS_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
const COMPACT_PRINCIPAL_CHARS: usize = 12;

#[cfg(feature = "host")]
impl_nns_mainnet_network_enforcer!(NnsTopologyHostError);
