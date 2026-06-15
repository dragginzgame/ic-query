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

pub use build::{build_nns_node_provider_info_report, build_nns_node_provider_list_report};
#[cfg(test)]
use build::{
    build_nns_node_provider_info_report_with_source,
    build_nns_node_provider_list_report_with_source,
};
pub use refresh::refresh_nns_node_provider_report;
#[cfg(test)]
use refresh::refresh_nns_node_provider_report_with_source;
#[cfg(test)]
use resolve::resolve_node_provider;
#[cfg(test)]
use source::NnsNodeProviderSource;

#[cfg(test)]
use crate::ic_registry::{MainnetNodeProviderList, MainnetRegistryFetchRequest};

pub use model::*;
pub use text::{
    nns_node_provider_info_report_text, nns_node_provider_list_report_text,
    nns_node_provider_list_report_verbose_text, nns_node_provider_refresh_report_text,
};

pub const DEFAULT_NNS_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
pub const DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
pub const NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_PROVIDER_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_PROVIDER_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_NODE_PROVIDER_CACHE_DIR: &str = "node-provider";
const NNS_NODE_PROVIDER_CACHE_FILE: &str = "providers.json";

#[must_use]
pub fn nns_node_provider_cache_path(icp_root: &Path, network: &str) -> PathBuf {
    nns_leaf_cache_path(
        icp_root,
        NNS_NODE_PROVIDER_CACHE_DIR,
        network,
        NNS_NODE_PROVIDER_CACHE_FILE,
    )
}

impl_nns_load_json_cache_error_mapper!(NnsNodeProviderCacheErrors, NnsNodeProviderHostError);
impl_nns_cache_error_mapper!(node_provider_cache_error, NnsNodeProviderHostError);
impl_nns_mainnet_network_enforcer!(NnsNodeProviderHostError);

#[cfg(test)]
mod tests;
