#[cfg(feature = "host")]
mod build;
#[cfg(feature = "host")]
mod capacity;
#[cfg(feature = "host")]
mod coverage;
#[cfg(feature = "host")]
mod error;
#[cfg(feature = "host")]
mod gaps;
#[cfg(feature = "host")]
mod health;
mod model;
mod percent;
#[cfg(feature = "host")]
mod providers;
#[cfg(feature = "host")]
mod refresh;
#[cfg(feature = "host")]
mod regions;
#[cfg(feature = "host")]
mod relations;
mod request;
#[cfg(feature = "host")]
mod summary;
mod text;
#[cfg(feature = "host")]
mod versions;

#[cfg(feature = "host")]
pub use build::{
    build_nns_topology_capacity_report, build_nns_topology_coverage_report,
    build_nns_topology_gaps_report, build_nns_topology_health_report,
    build_nns_topology_providers_report, build_nns_topology_regions_report,
    build_nns_topology_summary_report, build_nns_topology_versions_report,
    refresh_nns_topology_report,
};
#[cfg(feature = "host")]
pub use error::NnsTopologyHostError;
pub use model::{
    NnsTopologyCapacityReport, NnsTopologyCapacityRow, NnsTopologyCoverageReport,
    NnsTopologyGapRow, NnsTopologyGapsReport, NnsTopologyHealthCheckRow, NnsTopologyHealthReport,
    NnsTopologyProviderRow, NnsTopologyProvidersReport, NnsTopologyRefreshReport,
    NnsTopologyRefreshRow, NnsTopologyRegionRow, NnsTopologyRegionsReport,
    NnsTopologyRegistryVersionRow, NnsTopologySummaryReport, NnsTopologyVersionsReport,
};
pub use request::{
    NnsTopologyCapacityRequest, NnsTopologyCoverageRequest, NnsTopologyGapsRequest,
    NnsTopologyHealthRequest, NnsTopologyProvidersRequest, NnsTopologyRefreshRequest,
    NnsTopologyRegionsRequest, NnsTopologySummaryRequest, NnsTopologyVersionsRequest,
};
pub use text::{
    nns_topology_capacity_report_text, nns_topology_coverage_report_text,
    nns_topology_gaps_report_text, nns_topology_health_report_text,
    nns_topology_providers_report_text, nns_topology_refresh_report_text,
    nns_topology_regions_report_text, nns_topology_summary_report_text,
    nns_topology_versions_report_text,
};

pub const DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT: &str =
    crate::nns::node::report::DEFAULT_NNS_NODE_SOURCE_ENDPOINT;
pub const NNS_TOPOLOGY_SUMMARY_REPORT_SCHEMA_VERSION: u32 = 3;
pub const NNS_TOPOLOGY_COVERAGE_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_VERSIONS_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_HEALTH_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_GAPS_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_CAPACITY_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_REGIONS_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_PROVIDERS_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_TOPOLOGY_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
const COMPACT_PRINCIPAL_CHARS: usize = 12;

#[cfg(all(test, feature = "host"))]
use capacity::topology_capacity_report_from_report;
#[cfg(all(test, feature = "host"))]
use coverage::topology_coverage_report_from_summary;
#[cfg(all(test, feature = "host"))]
use gaps::topology_gaps_report_from_reports;
#[cfg(all(test, feature = "host"))]
use health::topology_health_report_from_summary;
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
impl_nns_mainnet_network_enforcer!(NnsTopologyHostError);

#[cfg(all(test, feature = "host"))]
mod tests;
