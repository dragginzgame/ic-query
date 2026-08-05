#[cfg(feature = "nns-host")]
mod build;
#[cfg(feature = "nns-host")]
mod cache;
#[cfg(feature = "nns-host")]
mod filters;
mod model;
#[cfg(feature = "nns-host")]
mod refresh;
#[cfg(feature = "nns-host")]
mod resolve;
#[cfg(feature = "nns-host")]
mod source;
mod text;

#[cfg(feature = "nns-host")]
use crate::nns::NnsInventoryRefreshRequest;
#[cfg(feature = "nns-host")]
use crate::nns::{NnsInventoryCacheRequest, NnsInventoryInfoRequest};

#[cfg(feature = "nns-host")]
pub use build::{
    build_nns_node_info_report, build_nns_node_info_report_with_source, build_nns_node_list_report,
    build_nns_node_list_report_with_source,
};
#[cfg(feature = "nns-host")]
pub use cache::{nns_node_cache_path, nns_node_refresh_lock_path};
#[cfg(all(test, feature = "nns-host"))]
use filters::filter_node_list_report;
#[cfg(feature = "nns-host")]
pub use refresh::{refresh_nns_node_report, refresh_nns_node_report_with_source};
#[cfg(all(test, feature = "nns-host"))]
use resolve::resolve_node;
#[cfg(feature = "nns-host")]
pub use source::NnsNodeSource;

#[cfg(feature = "nns-host")]
pub use model::{NnsNodeHostError, NnsNodeRefreshReport};
pub use model::{
    NnsNodeInfoReport, NnsNodeListFilters, NnsNodeListReport, NnsNodeListRequest, NnsNodeRow,
};
#[cfg(feature = "nns-host")]
pub use text::nns_node_refresh_report_text;
pub use text::{
    nns_node_info_report_text, nns_node_list_report_text, nns_node_list_report_verbose_text,
};

pub const DEFAULT_NNS_NODE_SOURCE_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "nns-host")]
pub const DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
#[cfg(feature = "nns-host")]
pub const NNS_NODE_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
pub const NNS_NODE_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
pub const NNS_NODE_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
const NNS_NODE_CACHE_DIR: &str = "node";
#[cfg(feature = "nns-host")]
const NNS_NODE_CACHE_FILE: &str = "nodes.json";

#[cfg(feature = "nns-host")]
impl_nns_mainnet_network_enforcer!(NnsNodeHostError);

#[cfg(all(test, feature = "nns-host"))]
mod tests;
