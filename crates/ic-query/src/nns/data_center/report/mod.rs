#[cfg(feature = "host")]
mod build;
#[cfg(feature = "host")]
mod cache;
mod model;
#[cfg(feature = "host")]
mod refresh;
#[cfg(feature = "host")]
mod resolve;
#[cfg(feature = "host")]
mod source;
mod text;

#[cfg(all(test, feature = "host"))]
use build::build_nns_data_center_list_report_with_source;
#[cfg(feature = "host")]
pub use build::{build_nns_data_center_info_report, build_nns_data_center_list_report};
#[cfg(feature = "host")]
pub use refresh::refresh_nns_data_center_report;
#[cfg(all(test, feature = "host"))]
use resolve::resolve_data_center;
#[cfg(all(test, feature = "host"))]
use source::NnsDataCenterSource;

#[cfg(all(test, feature = "host"))]
use crate::ic_registry::{MainnetDataCenterList, MainnetRegistryFetchRequest};

pub use model::{
    NnsDataCenterCacheRequest, NnsDataCenterInfoReport, NnsDataCenterInfoRequest,
    NnsDataCenterListReport, NnsDataCenterListRequest, NnsDataCenterRow,
};
#[cfg(feature = "host")]
pub use model::{NnsDataCenterHostError, NnsDataCenterRefreshReport, NnsDataCenterRefreshRequest};
#[cfg(feature = "host")]
pub use text::nns_data_center_refresh_report_text;
pub use text::{
    nns_data_center_info_report_text, nns_data_center_list_report_text,
    nns_data_center_list_report_verbose_text,
};

pub const DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "host")]
pub const DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
#[cfg(feature = "host")]
pub const NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
pub const NNS_DATA_CENTER_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
pub const NNS_DATA_CENTER_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const NNS_DATA_CENTER_CACHE_DIR: &str = "data-center";
#[cfg(feature = "host")]
const NNS_DATA_CENTER_CACHE_FILE: &str = "data-centers.json";

#[cfg(feature = "host")]
impl_nns_mainnet_network_enforcer!(NnsDataCenterHostError);

#[cfg(all(test, feature = "host"))]
mod tests;
