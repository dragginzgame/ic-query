mod model;
mod text;

#[cfg(feature = "host")]
mod cache;
#[cfg(feature = "host")]
mod error;
#[cfg(feature = "host")]
mod source;

pub use model::{
    NnsSubnetNodeProviderRow, NnsSubnetTopologyFreshness, NnsSubnetTopologyReport,
    NnsSubnetTopologyRow, NnsSubnetTopologyValidationError,
};
pub use text::nns_subnet_topology_report_text;

#[cfg(feature = "host")]
pub use cache::{
    load_cached_nns_subnet_topology, load_or_refresh_missing_nns_subnet_topology,
    load_or_refresh_missing_nns_subnet_topology_with_source,
    load_or_refresh_stale_nns_subnet_topology,
    load_or_refresh_stale_nns_subnet_topology_with_source, nns_subnet_topology_cache_path,
    nns_subnet_topology_freshness, nns_subnet_topology_refresh_lock_path,
    refresh_nns_subnet_topology, refresh_nns_subnet_topology_with_source,
};
#[cfg(feature = "host")]
pub use error::NnsSubnetTopologyHostError;
#[cfg(feature = "host")]
pub use model::{
    CachedNnsSubnetTopologyReport, NnsSubnetTopologyCacheRequest, NnsSubnetTopologyRefreshRequest,
};
#[cfg(feature = "host")]
pub use source::NnsSubnetTopologySource;

/// Current serialized schema version for exact-version Subnet topology reports.
pub const NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
/// Default mainnet replica endpoint for live topology refreshes.
pub const DEFAULT_NNS_SUBNET_TOPOLOGY_SOURCE_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "host")]
/// Default age after which a cached topology report is stale.
pub const DEFAULT_NNS_SUBNET_TOPOLOGY_STALE_AFTER_SECONDS: u64 = 24 * 60 * 60;
#[cfg(feature = "host")]
/// Default age after which a topology refresh lock is stale.
pub const DEFAULT_NNS_SUBNET_TOPOLOGY_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;

#[cfg(all(test, feature = "host"))]
mod tests;
