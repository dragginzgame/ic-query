mod build;
mod capacity;
mod coverage;
mod error;
mod gaps;
mod health;
mod model;
mod percent;
mod providers;
mod refresh;
mod regions;
mod relations;
mod request;
mod summary;
mod text;
mod versions;

pub use build::{
    build_nns_topology_capacity_report, build_nns_topology_coverage_report,
    build_nns_topology_gaps_report, build_nns_topology_health_report,
    build_nns_topology_providers_report, build_nns_topology_regions_report,
    build_nns_topology_summary_report, build_nns_topology_versions_report,
    refresh_nns_topology_report,
};
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

#[cfg(test)]
use capacity::topology_capacity_report_from_report;
#[cfg(test)]
use coverage::topology_coverage_report_from_summary;
#[cfg(test)]
use gaps::topology_gaps_report_from_reports;
#[cfg(test)]
use health::topology_health_report_from_summary;
#[cfg(test)]
use providers::topology_providers_report_from_reports;
#[cfg(test)]
use refresh::{NnsTopologyRefreshComponentReports, topology_refresh_report_from_reports};
#[cfg(test)]
use regions::topology_regions_report_from_report;
#[cfg(test)]
use summary::topology_summary_report_from_reports;
#[cfg(test)]
use versions::topology_versions_report_from_summary;

impl_nns_mainnet_network_enforcer!(NnsTopologyHostError);

#[cfg(test)]
mod tests;
