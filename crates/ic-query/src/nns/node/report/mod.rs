#[cfg(feature = "nns-host")]
use crate::nns::NnsInventoryError;
#[cfg(feature = "nns-host")]
mod build;
#[cfg(feature = "nns-host")]
mod filters;
mod model;

#[cfg(feature = "nns-host")]
use crate::nns::NnsInventoryInfoRequest;

#[cfg(all(test, feature = "nns-host"))]
use build::resolve_node;
#[cfg(feature = "nns-host")]
pub use build::{
    build_nns_node_info_report, build_nns_node_info_report_with_source_and_progress,
    build_nns_node_list_report, build_nns_node_list_report_with_source_and_progress,
};
#[cfg(all(test, feature = "nns-host"))]
use filters::filter_node_list_report;

pub use model::{
    NnsNodeInfoReport, NnsNodeListFilters, NnsNodeListReport, NnsNodeListRequest, NnsNodeRow,
};

pub const DEFAULT_NNS_NODE_SOURCE_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "nns-host")]
#[cfg(feature = "nns-host")]
pub const NNS_NODE_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
pub const NNS_NODE_INFO_REPORT_SCHEMA_VERSION: u32 = 1;

#[cfg(all(test, feature = "nns-host"))]
mod tests;

#[cfg(feature = "nns-host")]
const COLLECTION: crate::nns::NnsInventoryCollection = crate::nns::NnsInventoryCollection::Node;
