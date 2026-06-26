#[cfg(feature = "host")]
mod build;
#[cfg(feature = "host")]
mod cache;
#[cfg(feature = "host")]
mod filters;
mod model;
#[cfg(feature = "host")]
mod refresh;
#[cfg(feature = "host")]
mod resolve;
#[cfg(feature = "host")]
mod source;
mod text;

#[cfg(all(test, feature = "host"))]
use build::build_nns_node_list_report_with_source;
#[cfg(feature = "host")]
pub use build::{build_nns_node_info_report, build_nns_node_list_report};
#[cfg(feature = "host")]
pub use cache::{nns_node_cache_path, nns_node_refresh_lock_path};
#[cfg(all(test, feature = "host"))]
use filters::filter_node_list_report;
#[cfg(feature = "host")]
pub use refresh::refresh_nns_node_report;
#[cfg(all(test, feature = "host"))]
use resolve::resolve_node;
#[cfg(all(test, feature = "host"))]
use source::NnsNodeSource;

#[cfg(all(test, feature = "host"))]
use crate::ic_registry::{MainnetNodeList, MainnetRegistryFetchRequest};

pub use model::{
    NnsNodeCacheRequest, NnsNodeInfoReport, NnsNodeInfoRequest, NnsNodeListFilters,
    NnsNodeListReport, NnsNodeListRequest, NnsNodeRow,
};
#[cfg(feature = "host")]
pub use model::{NnsNodeHostError, NnsNodeRefreshReport, NnsNodeRefreshRequest};
#[cfg(feature = "host")]
pub use text::nns_node_refresh_report_text;
pub use text::{
    nns_node_info_report_text, nns_node_list_report_text, nns_node_list_report_verbose_text,
};

pub const DEFAULT_NNS_NODE_SOURCE_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "host")]
pub const DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
#[cfg(feature = "host")]
pub const NNS_NODE_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
pub const NNS_NODE_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
pub const NNS_NODE_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_SUBNET_KIND_APPLICATION: &str = "application";
pub const NNS_NODE_SUBNET_KIND_CLOUD_ENGINE: &str = "cloud_engine";
pub const NNS_NODE_SUBNET_KIND_SYSTEM: &str = "system";
pub const NNS_NODE_SUBNET_KIND_UNKNOWN: &str = "unknown";
#[cfg(feature = "host")]
const NNS_NODE_CACHE_DIR: &str = "node";
#[cfg(feature = "host")]
const NNS_NODE_CACHE_FILE: &str = "nodes.json";

#[cfg(feature = "host")]
impl_nns_mainnet_network_enforcer!(NnsNodeHostError);

#[cfg(all(test, feature = "host"))]
mod tests;
