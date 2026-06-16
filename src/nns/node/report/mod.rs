use crate::ic_registry::DEFAULT_MAINNET_ENDPOINT;
use crate::nns::leaf::nns_leaf_cache_path;
use std::path::{Path, PathBuf};

mod build;
mod cache;
mod filters;
mod model;
mod refresh;
mod resolve;
mod source;
mod text;

#[cfg(test)]
use build::build_nns_node_list_report_with_source;
pub use build::{build_nns_node_info_report, build_nns_node_list_report};
#[cfg(test)]
use filters::filter_node_list_report;
pub use refresh::refresh_nns_node_report;
#[cfg(test)]
use resolve::resolve_node;
#[cfg(test)]
use source::NnsNodeSource;

#[cfg(test)]
use crate::ic_registry::{MainnetNodeList, MainnetRegistryFetchRequest};

pub use model::{
    NnsNodeCacheRequest, NnsNodeHostError, NnsNodeInfoReport, NnsNodeInfoRequest,
    NnsNodeListFilters, NnsNodeListReport, NnsNodeListRequest, NnsNodeRefreshReport,
    NnsNodeRefreshRequest, NnsNodeRow,
};
pub use text::{
    nns_node_info_report_text, nns_node_list_report_text, nns_node_list_report_verbose_text,
    nns_node_refresh_report_text,
};

pub const DEFAULT_NNS_NODE_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
pub const DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
pub const NNS_NODE_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_SUBNET_KIND_APPLICATION: &str = "application";
pub const NNS_NODE_SUBNET_KIND_CLOUD_ENGINE: &str = "cloud_engine";
pub const NNS_NODE_SUBNET_KIND_SYSTEM: &str = "system";
pub const NNS_NODE_SUBNET_KIND_UNKNOWN: &str = "unknown";
const NNS_NODE_CACHE_DIR: &str = "node";
const NNS_NODE_CACHE_FILE: &str = "nodes.json";

#[must_use]
pub fn nns_node_cache_path(icp_root: &Path, network: &str) -> PathBuf {
    nns_leaf_cache_path(icp_root, NNS_NODE_CACHE_DIR, network, NNS_NODE_CACHE_FILE)
}

impl_nns_load_json_cache_error_mapper!(NnsNodeCacheErrors, NnsNodeHostError);
impl_nns_cache_error_mapper!(node_cache_error, NnsNodeHostError);
impl_nns_mainnet_network_enforcer!(NnsNodeHostError);

#[cfg(test)]
mod tests;
