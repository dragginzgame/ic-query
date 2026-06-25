use crate::ic_registry::DEFAULT_MAINNET_ENDPOINT;

mod build;
mod cache;
mod model;
mod refresh;
mod resolve;
mod source;
mod text;

pub use build::{build_nns_node_provider_info_report, build_nns_node_provider_list_report};
#[cfg(test)]
use build::{
    build_nns_node_provider_info_report_with_source,
    build_nns_node_provider_list_report_with_source,
};
pub use refresh::refresh_nns_node_provider_report;
#[cfg(test)]
use refresh::refresh_nns_node_provider_report_with_source;
#[cfg(test)]
use resolve::resolve_node_provider;
#[cfg(test)]
use source::NnsNodeProviderSource;

#[cfg(test)]
use crate::ic_registry::{MainnetNodeProviderList, MainnetRegistryFetchRequest};

pub use model::{
    NnsNodeProviderCacheRequest, NnsNodeProviderHostError, NnsNodeProviderInfoReport,
    NnsNodeProviderInfoRequest, NnsNodeProviderListReport, NnsNodeProviderListRequest,
    NnsNodeProviderRefreshReport, NnsNodeProviderRefreshRequest, NnsNodeProviderRow,
};
pub use text::{
    nns_node_provider_info_report_text, nns_node_provider_list_report_text,
    nns_node_provider_list_report_verbose_text, nns_node_provider_refresh_report_text,
};

pub const DEFAULT_NNS_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
pub const DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
pub const NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_PROVIDER_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_PROVIDER_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_NODE_PROVIDER_CACHE_DIR: &str = "node-provider";
const NNS_NODE_PROVIDER_CACHE_FILE: &str = "providers.json";

impl_nns_mainnet_network_enforcer!(NnsNodeProviderHostError);

#[cfg(test)]
mod tests;
