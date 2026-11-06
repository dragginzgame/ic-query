#[cfg(feature = "nns-host")]
use crate::nns::NnsInventoryError;
#[cfg(feature = "nns-host")]
mod build;
mod model;

#[cfg(feature = "nns-host")]
use crate::nns::{NnsInventoryInfoRequest, NnsInventoryListRequest};

#[cfg(all(test, feature = "nns-host"))]
use build::resolve_data_center;
#[cfg(feature = "nns-host")]
pub use build::{
    build_nns_data_center_info_report, build_nns_data_center_info_report_with_source_and_progress,
    build_nns_data_center_list_report, build_nns_data_center_list_report_with_source_and_progress,
};

pub use model::{NnsDataCenterInfoReport, NnsDataCenterListReport, NnsDataCenterRow};

pub const DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "nns-host")]
pub const NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
pub const NNS_DATA_CENTER_INFO_REPORT_SCHEMA_VERSION: u32 = 1;

#[cfg(all(test, feature = "nns-host"))]
mod tests;

#[cfg(feature = "nns-host")]
const COLLECTION: crate::nns::NnsInventoryCollection =
    crate::nns::NnsInventoryCollection::DataCenter;
