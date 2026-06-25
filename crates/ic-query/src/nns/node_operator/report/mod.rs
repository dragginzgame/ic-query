use crate::ic_registry::DEFAULT_MAINNET_ENDPOINT;

mod build;
mod cache;
mod model;
mod refresh;
mod resolve;
mod source;
mod text;

#[cfg(test)]
use build::build_nns_node_operator_list_report_with_source;
pub use build::{build_nns_node_operator_info_report, build_nns_node_operator_list_report};
pub use refresh::refresh_nns_node_operator_report;
#[cfg(test)]
use resolve::resolve_node_operator;
#[cfg(test)]
use source::NnsNodeOperatorSource;

#[cfg(test)]
use crate::ic_registry::{MainnetNodeOperatorList, MainnetRegistryFetchRequest};

pub use model::{
    NnsNodeOperatorCacheRequest, NnsNodeOperatorHostError, NnsNodeOperatorInfoReport,
    NnsNodeOperatorInfoRequest, NnsNodeOperatorListReport, NnsNodeOperatorListRequest,
    NnsNodeOperatorRefreshReport, NnsNodeOperatorRefreshRequest, NnsNodeOperatorRow,
};
pub use text::{
    nns_node_operator_info_report_text, nns_node_operator_list_report_text,
    nns_node_operator_list_report_verbose_text, nns_node_operator_refresh_report_text,
};

pub const DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
pub const DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
pub const NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_OPERATOR_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_OPERATOR_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_NODE_OPERATOR_CACHE_DIR: &str = "node-operator";
const NNS_NODE_OPERATOR_CACHE_FILE: &str = "operators.json";

impl_nns_mainnet_network_enforcer!(NnsNodeOperatorHostError);

#[cfg(test)]
mod tests;
