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
    build_nns_node_operator_info_report, build_nns_node_operator_info_report_with_source,
    build_nns_node_operator_list_report, build_nns_node_operator_list_report_with_source,
};
#[cfg(feature = "nns-host")]
pub use cache::{nns_node_operator_cache_path, nns_node_operator_refresh_lock_path};
#[cfg(feature = "nns-host")]
pub use refresh::{refresh_nns_node_operator_report, refresh_nns_node_operator_report_with_source};
#[cfg(all(test, feature = "nns-host"))]
use resolve::resolve_node_operator;
#[cfg(feature = "nns-host")]
pub use source::NnsNodeOperatorSource;

#[cfg(feature = "nns-host")]
pub use model::{NnsNodeOperatorHostError, NnsNodeOperatorRefreshReport};
pub use model::{NnsNodeOperatorInfoReport, NnsNodeOperatorListReport, NnsNodeOperatorRow};
#[cfg(feature = "nns-host")]
pub use text::nns_node_operator_refresh_report_text;
pub use text::{
    nns_node_operator_info_report_text, nns_node_operator_list_report_text,
    nns_node_operator_list_report_verbose_text,
};

pub const DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "nns-host")]
pub const DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
#[cfg(feature = "nns-host")]
pub const NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
pub const NNS_NODE_OPERATOR_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
pub const NNS_NODE_OPERATOR_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
const NNS_NODE_OPERATOR_CACHE_DIR: &str = "node-operator";
#[cfg(feature = "nns-host")]
const NNS_NODE_OPERATOR_CACHE_FILE: &str = "operators.json";

#[cfg(feature = "nns-host")]
impl_nns_mainnet_network_enforcer!(NnsNodeOperatorHostError);

#[cfg(all(test, feature = "nns-host"))]
mod tests;
