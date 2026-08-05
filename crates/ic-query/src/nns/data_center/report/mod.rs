#[cfg(feature = "nns-host")]
mod build;
#[cfg(feature = "nns-host")]
mod cache;
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
use crate::nns::{NnsInventoryCacheRequest, NnsInventoryInfoRequest, NnsInventoryListRequest};

#[cfg(feature = "nns-host")]
pub use build::{
    build_nns_data_center_info_report, build_nns_data_center_info_report_with_source,
    build_nns_data_center_list_report, build_nns_data_center_list_report_with_source,
};
#[cfg(feature = "nns-host")]
pub use cache::{nns_data_center_cache_path, nns_data_center_refresh_lock_path};
#[cfg(feature = "nns-host")]
pub use refresh::{refresh_nns_data_center_report, refresh_nns_data_center_report_with_source};
#[cfg(all(test, feature = "nns-host"))]
use resolve::resolve_data_center;
#[cfg(feature = "nns-host")]
pub use source::NnsDataCenterSource;

#[cfg(feature = "nns-host")]
pub use model::{NnsDataCenterHostError, NnsDataCenterRefreshReport};
pub use model::{NnsDataCenterInfoReport, NnsDataCenterListReport, NnsDataCenterRow};
#[cfg(feature = "nns-host")]
pub use text::nns_data_center_refresh_report_text;
pub use text::{
    nns_data_center_info_report_text, nns_data_center_list_report_text,
    nns_data_center_list_report_verbose_text,
};

pub const DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "nns-host")]
pub const DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
#[cfg(feature = "nns-host")]
pub const NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
pub const NNS_DATA_CENTER_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
pub const NNS_DATA_CENTER_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
const NNS_DATA_CENTER_CACHE_DIR: &str = "data-center";
#[cfg(feature = "nns-host")]
const NNS_DATA_CENTER_CACHE_FILE: &str = "data-centers.json";

#[cfg(feature = "nns-host")]
impl_nns_mainnet_network_enforcer!(NnsDataCenterHostError);

#[cfg(all(test, feature = "nns-host"))]
mod tests;
