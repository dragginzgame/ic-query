#[cfg(feature = "nns-host")]
use crate::nns::NnsInventoryError;
#[cfg(feature = "nns-host")]
mod build;
mod model;

#[cfg(feature = "nns-host")]
use crate::nns::{NnsInventoryInfoRequest, NnsInventoryListRequest};

#[cfg(all(test, feature = "nns-host"))]
use build::resolve_node_operator;
#[cfg(feature = "nns-host")]
pub use build::{
    build_nns_node_operator_info_report,
    build_nns_node_operator_info_report_with_source_and_progress,
    build_nns_node_operator_list_report,
    build_nns_node_operator_list_report_with_source_and_progress,
};

pub use model::{NnsNodeOperatorInfoReport, NnsNodeOperatorListReport, NnsNodeOperatorRow};

pub const DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "nns-host")]
pub const NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
pub const NNS_NODE_OPERATOR_INFO_REPORT_SCHEMA_VERSION: u32 = 1;

#[cfg(all(test, feature = "nns-host"))]
mod tests;

#[cfg(feature = "nns-host")]
const COLLECTION: crate::nns::NnsInventoryCollection =
    crate::nns::NnsInventoryCollection::NodeOperator;
