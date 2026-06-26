use super::{
    NNS_DATA_CENTER_CACHE_DIR, NNS_DATA_CENTER_CACHE_FILE,
    NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION, NnsDataCenterCacheRequest, NnsDataCenterHostError,
    NnsDataCenterListReport, enforce_mainnet_network,
};
use crate::{
    cache_file::CachedJsonReport,
    nns::leaf::{NnsLeafCachePaths, load_nns_leaf_json_cache},
};
use std::path::{Path, PathBuf};

#[must_use]
pub fn nns_data_center_cache_path(icp_root: &Path, network: &str) -> PathBuf {
    nns_data_center_cache_paths(icp_root, network).cache_path
}

#[must_use]
pub fn nns_data_center_refresh_lock_path(icp_root: &Path, network: &str) -> PathBuf {
    nns_data_center_cache_paths(icp_root, network).lock_path
}

pub(super) fn load_cached_nns_data_center_report(
    request: &NnsDataCenterCacheRequest,
) -> Result<CachedJsonReport<NnsDataCenterListReport>, NnsDataCenterHostError> {
    enforce_mainnet_network(&request.network)?;
    load_nns_leaf_json_cache(
        request,
        NNS_DATA_CENTER_CACHE_DIR,
        NNS_DATA_CENTER_CACHE_FILE,
        NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION,
    )
    .map_err(Into::into)
}

fn nns_data_center_cache_paths(icp_root: &Path, network: &str) -> NnsLeafCachePaths {
    NnsLeafCachePaths::for_component(
        icp_root,
        NNS_DATA_CENTER_CACHE_DIR,
        network,
        NNS_DATA_CENTER_CACHE_FILE,
    )
}
