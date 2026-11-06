#[cfg(feature = "nns-host")]
use crate::nns::NnsInventoryError;
#[cfg(feature = "nns-host")]
mod build;
#[cfg(feature = "nns-host")]
mod cache;
mod model;
#[cfg(feature = "nns-host")]
mod refresh;
#[cfg(feature = "nns-host")]
mod source;

#[cfg(feature = "nns-host")]
use crate::nns::NnsInventoryRefreshRequest;
#[cfg(feature = "nns-host")]
use crate::nns::{NnsInventoryInfoRequest, NnsInventoryListRequest};

#[cfg(all(test, feature = "nns-host"))]
use build::resolve_node_provider;
#[cfg(feature = "nns-host")]
pub use build::{
    build_nns_node_provider_info_report,
    build_nns_node_provider_info_report_with_source_and_progress,
    build_nns_node_provider_list_report,
    build_nns_node_provider_list_report_with_source_and_progress,
};
#[cfg(feature = "nns-host")]
pub use cache::{nns_node_provider_cache_path, nns_node_provider_refresh_lock_path};
#[cfg(feature = "nns-host")]
pub use refresh::{
    NnsNodeProviderRefreshReport, refresh_nns_node_provider_report,
    refresh_nns_node_provider_report_with_source,
};
#[cfg(feature = "nns-host")]
pub use source::NnsNodeProviderSource;

pub use model::{
    NnsNodeProviderInfoReport, NnsNodeProviderListReport, NnsNodeProviderMetadata,
    NnsNodeProviderRow,
};

pub const DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "nns-host")]
pub const NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
pub const NNS_NODE_PROVIDER_INFO_REPORT_SCHEMA_VERSION: u32 = 1;

#[cfg(feature = "nns-host")]
impl_nns_mainnet_network_enforcer!(NnsInventoryError, COLLECTION);

#[cfg(all(test, feature = "nns-host"))]
mod tests;

#[cfg(feature = "nns-host")]
const COLLECTION: crate::nns::NnsInventoryCollection =
    crate::nns::NnsInventoryCollection::NodeProvider;

#[cfg(feature = "nns-host")]
pub(in crate::nns) use build::node_provider_report_from_snapshot;
#[cfg(feature = "nns-host")]
pub(in crate::nns) use cache::load_or_refresh_metadata;
