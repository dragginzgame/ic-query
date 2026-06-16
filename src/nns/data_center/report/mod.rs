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
use build::build_nns_data_center_list_report_with_source;
pub use build::{build_nns_data_center_info_report, build_nns_data_center_list_report};
pub use refresh::refresh_nns_data_center_report;
#[cfg(test)]
use resolve::resolve_data_center;
#[cfg(test)]
use source::NnsDataCenterSource;

#[cfg(test)]
use crate::ic_registry::{MainnetDataCenterList, MainnetRegistryFetchRequest};

pub use model::{
    NnsDataCenterCacheRequest, NnsDataCenterHostError, NnsDataCenterInfoReport,
    NnsDataCenterInfoRequest, NnsDataCenterListReport, NnsDataCenterListRequest,
    NnsDataCenterRefreshReport, NnsDataCenterRefreshRequest, NnsDataCenterRow,
};
pub use text::{
    nns_data_center_info_report_text, nns_data_center_list_report_text,
    nns_data_center_list_report_verbose_text, nns_data_center_refresh_report_text,
};

pub const DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
pub const DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
pub const NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_DATA_CENTER_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_DATA_CENTER_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_DATA_CENTER_CACHE_DIR: &str = "data-center";
const NNS_DATA_CENTER_CACHE_FILE: &str = "data-centers.json";

#[must_use]
pub fn nns_data_center_cache_path(icp_root: &Path, network: &str) -> PathBuf {
    nns_leaf_cache_path(
        icp_root,
        NNS_DATA_CENTER_CACHE_DIR,
        network,
        NNS_DATA_CENTER_CACHE_FILE,
    )
}

impl_nns_load_json_cache_error_mapper!(NnsDataCenterCacheErrors, NnsDataCenterHostError);
impl_nns_cache_error_mapper!(data_center_cache_error, NnsDataCenterHostError);
impl_nns_mainnet_network_enforcer!(NnsDataCenterHostError);

#[cfg(test)]
mod tests;
