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

#[cfg(feature = "host")]
pub use build::{
    build_nns_node_operator_info_report, build_nns_node_operator_info_report_with_source,
    build_nns_node_operator_list_report, build_nns_node_operator_list_report_with_source,
};
#[cfg(feature = "host")]
pub use cache::{nns_node_operator_cache_path, nns_node_operator_refresh_lock_path};
#[cfg(feature = "host")]
pub use refresh::{refresh_nns_node_operator_report, refresh_nns_node_operator_report_with_source};
#[cfg(all(test, feature = "host"))]
use resolve::resolve_node_operator;
#[cfg(feature = "host")]
pub use source::{LiveNnsNodeOperatorSource, NnsNodeOperatorSource, NnsNodeOperatorSourceRequest};

pub use model::{
    NnsNodeOperatorCacheRequest, NnsNodeOperatorInfoReport, NnsNodeOperatorInfoRequest,
    NnsNodeOperatorListReport, NnsNodeOperatorListRequest, NnsNodeOperatorRow,
};
#[cfg(feature = "host")]
pub use model::{
    NnsNodeOperatorHostError, NnsNodeOperatorRefreshReport, NnsNodeOperatorRefreshRequest,
};
#[cfg(feature = "host")]
pub use text::nns_node_operator_refresh_report_text;
pub use text::{
    nns_node_operator_info_report_text, nns_node_operator_list_report_text,
    nns_node_operator_list_report_verbose_text,
};

pub const DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "host")]
pub const DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
#[cfg(feature = "host")]
pub const NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
pub const NNS_NODE_OPERATOR_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
pub const NNS_NODE_OPERATOR_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const NNS_NODE_OPERATOR_CACHE_DIR: &str = "node-operator";
#[cfg(feature = "host")]
const NNS_NODE_OPERATOR_CACHE_FILE: &str = "operators.json";

#[cfg(feature = "host")]
impl_nns_mainnet_network_enforcer!(NnsNodeOperatorHostError);

#[cfg(all(test, feature = "host"))]
mod tests;
