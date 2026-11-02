#[cfg(feature = "nns-host")]
use crate::nns::NnsInventoryError;
#[cfg(feature = "nns-host")]
mod build;
#[cfg(feature = "nns-host")]
mod cache;
mod model;
#[cfg(feature = "nns-host")]
mod refresh;
#[cfg(feature = "nns-host")]
mod source;

#[cfg(feature = "nns-host")]
use crate::nns::NnsInventoryRefreshRequest;
#[cfg(feature = "nns-host")]
use crate::nns::{NnsInventoryCacheRequest, NnsInventoryInfoRequest, NnsInventoryListRequest};

#[cfg(all(test, feature = "nns-host"))]
use build::resolve_node_operator;
#[cfg(feature = "nns-host")]
pub use build::{
    build_nns_node_operator_info_report,
    build_nns_node_operator_info_report_with_source_and_progress,
    build_nns_node_operator_list_report,
    build_nns_node_operator_list_report_with_source_and_progress,
};
#[cfg(feature = "nns-host")]
pub use cache::{nns_node_operator_cache_path, nns_node_operator_refresh_lock_path};
#[cfg(feature = "nns-host")]
pub use refresh::{refresh_nns_node_operator_report, refresh_nns_node_operator_report_with_source};
#[cfg(feature = "nns-host")]
pub use source::NnsNodeOperatorSource;

pub use model::{NnsNodeOperatorInfoReport, NnsNodeOperatorListReport, NnsNodeOperatorRow};

pub const DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "nns-host")]
#[cfg(feature = "nns-host")]
pub const NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
pub const NNS_NODE_OPERATOR_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
const NNS_NODE_OPERATOR_CACHE_DIR: &str = "node-operator";
#[cfg(feature = "nns-host")]
const NNS_NODE_OPERATOR_CACHE_FILE: &str = "operators.json";

#[cfg(feature = "nns-host")]
impl_nns_mainnet_network_enforcer!(NnsInventoryError, COLLECTION);

#[cfg(all(test, feature = "nns-host"))]
mod tests;

#[cfg(feature = "nns-host")]
const COLLECTION: crate::nns::NnsInventoryCollection =
    crate::nns::NnsInventoryCollection::NodeOperator;
