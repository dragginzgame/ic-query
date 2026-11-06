//! Module: nns::topology::report
//!
//! Responsibility: assemble reusable NNS topology reports and refresh operations.
//! Does not own: CLI parsing, registry transport internals, or process output.
//! Boundary: groups topology requests, sources, projections, and text renderers.

#[cfg(feature = "nns-host")]
mod build;
#[cfg(feature = "nns-host")]
mod capacity;
#[cfg(feature = "nns-host")]
mod check;
#[cfg(feature = "nns-host")]
mod coverage;
#[cfg(feature = "nns-host")]
mod error;
#[cfg(feature = "nns-host")]
mod gaps;
mod model;
#[cfg(feature = "nns-host")]
mod providers;
#[cfg(feature = "nns-host")]
mod regions;
#[cfg(feature = "nns-host")]
mod registry_versions;
#[cfg(feature = "nns-host")]
mod relations;
mod request;
#[cfg(feature = "nns-host")]
mod source;
mod subnet_topology;
#[cfg(feature = "nns-host")]
mod summary;

#[cfg(all(test, feature = "nns-host"))]
mod tests;

#[cfg(all(test, feature = "nns-host"))]
use capacity::topology_capacity_report_from_rows;
#[cfg(all(test, feature = "nns-host"))]
use check::topology_check_report_from_reports;
#[cfg(all(test, feature = "nns-host"))]
use coverage::topology_coverage_report_from_reports;
#[cfg(all(test, feature = "nns-host"))]
use gaps::topology_gaps_report_from_reports;
#[cfg(all(test, feature = "nns-host"))]
use providers::topology_providers_report_from_reports;
#[cfg(all(test, feature = "nns-host"))]
use regions::topology_regions_report_from_rows;
#[cfg(all(test, feature = "nns-host"))]
use summary::topology_summary_report_from_reports;

#[cfg(feature = "nns-host")]
use crate::nns::{NnsRegistryInventorySnapshot, node_provider::NnsNodeProviderListReport};

///
/// TopologyInventoryReports
///
/// Complete component inventories collected in diagnostic read order.
///

#[cfg(feature = "nns-host")]
struct TopologyInventoryReports {
    registry: NnsRegistryInventorySnapshot,
    node_provider: NnsNodeProviderListReport,
}

#[cfg(feature = "nns-host")]
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
#[cfg(feature = "nns-host")]
pub use error::NnsTopologyHostError;
pub use model::{
    NnsTopologyAssessmentStatus, NnsTopologyCapacityReport, NnsTopologyCapacityRow,
    NnsTopologyCapacityStatus, NnsTopologyCheckReport, NnsTopologyCheckRow,
    NnsTopologyCoverageReport, NnsTopologyGapRelationKind, NnsTopologyGapRow,
    NnsTopologyGapSubjectKind, NnsTopologyGapsReport, NnsTopologyJoinCoverageCounts,
    NnsTopologyProviderRow, NnsTopologyProviderStatus, NnsTopologyProvidersReport,
    NnsTopologyRefreshReport, NnsTopologyRefreshRow, NnsTopologyRegionRow,
    NnsTopologyRegionsReport, NnsTopologyRegistryVersionRow, NnsTopologySummaryReport,
    NnsTopologyVersionsReport,
};
pub use request::NnsTopologyRefreshRequest;
#[cfg(feature = "nns-host")]
pub use source::{NnsTopologyRefreshSource, NnsTopologySource};
#[cfg(feature = "nns-topology-host")]
pub use subnet_topology::{
    CachedNnsSubnetTopologyReport, DEFAULT_NNS_SUBNET_TOPOLOGY_SOURCE_ENDPOINT,
    DEFAULT_NNS_SUBNET_TOPOLOGY_STALE_AFTER_SECONDS, NnsSubnetTopologyCacheRequest,
    NnsSubnetTopologyHostError, NnsSubnetTopologyRefreshRequest, NnsSubnetTopologySource,
    load_cached_nns_subnet_topology, load_or_refresh_missing_nns_subnet_topology,
    load_or_refresh_missing_nns_subnet_topology_with_source,
    load_or_refresh_stale_nns_subnet_topology,
    load_or_refresh_stale_nns_subnet_topology_with_source, nns_subnet_topology_cache_path,
    nns_subnet_topology_freshness, nns_subnet_topology_refresh_lock_path,
    refresh_nns_subnet_topology, refresh_nns_subnet_topology_with_source,
};
pub use subnet_topology::{
    NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION, NnsSubnetNodeProviderRow,
    NnsSubnetTopologyFreshness, NnsSubnetTopologyReport, NnsSubnetTopologyRow,
    NnsSubnetTopologyValidationError,
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

#[cfg(feature = "nns-host")]
impl_nns_mainnet_network_enforcer!(NnsTopologyHostError);
