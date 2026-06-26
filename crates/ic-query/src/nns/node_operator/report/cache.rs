use super::{
    NNS_NODE_OPERATOR_CACHE_DIR, NNS_NODE_OPERATOR_CACHE_FILE,
    NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION, NnsNodeOperatorCacheRequest,
    NnsNodeOperatorHostError, NnsNodeOperatorListReport, enforce_mainnet_network,
};
use crate::{
    cache_file::CachedJsonReport,
    nns::leaf::{NnsLeafCachePaths, load_nns_leaf_json_cache},
};
use std::path::{Path, PathBuf};

#[must_use]
pub fn nns_node_operator_cache_path(icp_root: &Path, network: &str) -> PathBuf {
    nns_node_operator_cache_paths(icp_root, network).cache_path
}

#[must_use]
pub fn nns_node_operator_refresh_lock_path(icp_root: &Path, network: &str) -> PathBuf {
    nns_node_operator_cache_paths(icp_root, network).lock_path
}

pub(super) fn load_cached_nns_node_operator_report(
    request: &NnsNodeOperatorCacheRequest,
) -> Result<CachedJsonReport<NnsNodeOperatorListReport>, NnsNodeOperatorHostError> {
    enforce_mainnet_network(&request.network)?;
    load_nns_leaf_json_cache(
        request,
        NNS_NODE_OPERATOR_CACHE_DIR,
        NNS_NODE_OPERATOR_CACHE_FILE,
        NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION,
    )
    .map_err(Into::into)
}

fn nns_node_operator_cache_paths(icp_root: &Path, network: &str) -> NnsLeafCachePaths {
    NnsLeafCachePaths::for_component(
        icp_root,
        NNS_NODE_OPERATOR_CACHE_DIR,
        network,
        NNS_NODE_OPERATOR_CACHE_FILE,
    )
}
