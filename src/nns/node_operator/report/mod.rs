use crate::ic_registry::DEFAULT_MAINNET_ENDPOINT;
use crate::nns::leaf::nns_leaf_cache_path;
use std::path::{Path, PathBuf};

mod build;
mod cache;
mod model;
mod refresh;
mod resolve;
mod source;
mod text;

#[cfg(test)]
use build::build_nns_node_operator_list_report_with_source;
pub use build::{build_nns_node_operator_info_report, build_nns_node_operator_list_report};
pub use refresh::refresh_nns_node_operator_report;
#[cfg(test)]
use resolve::resolve_node_operator;
#[cfg(test)]
use source::NnsNodeOperatorSource;

#[cfg(test)]
use crate::ic_registry::{MainnetNodeOperatorList, MainnetRegistryFetchRequest};

pub use model::*;
pub use text::{
    nns_node_operator_info_report_text, nns_node_operator_list_report_text,
    nns_node_operator_list_report_verbose_text, nns_node_operator_refresh_report_text,
};

pub const DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
pub const DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
pub const NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_OPERATOR_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_OPERATOR_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_NODE_OPERATOR_CACHE_DIR: &str = "node-operator";
const NNS_NODE_OPERATOR_CACHE_FILE: &str = "operators.json";

#[must_use]
pub fn nns_node_operator_cache_path(icp_root: &Path, network: &str) -> PathBuf {
    nns_leaf_cache_path(
        icp_root,
        NNS_NODE_OPERATOR_CACHE_DIR,
        network,
        NNS_NODE_OPERATOR_CACHE_FILE,
    )
}

impl_nns_load_json_cache_error_mapper!(NnsNodeOperatorCacheErrors, NnsNodeOperatorHostError);
impl_nns_cache_error_mapper!(node_operator_cache_error, NnsNodeOperatorHostError);
impl_nns_mainnet_network_enforcer!(NnsNodeOperatorHostError);

#[cfg(test)]
mod tests;
