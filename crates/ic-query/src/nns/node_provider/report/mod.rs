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
    build_nns_node_provider_info_report, build_nns_node_provider_info_report_with_source,
    build_nns_node_provider_list_report, build_nns_node_provider_list_report_with_source,
};
#[cfg(feature = "host")]
pub use cache::{nns_node_provider_cache_path, nns_node_provider_refresh_lock_path};
#[cfg(feature = "host")]
pub use refresh::{refresh_nns_node_provider_report, refresh_nns_node_provider_report_with_source};
#[cfg(all(test, feature = "host"))]
use resolve::resolve_node_provider;
#[cfg(feature = "host")]
pub use source::{LiveNnsNodeProviderSource, NnsNodeProviderSource, NnsNodeProviderSourceRequest};

pub use model::{
    NnsNodeProviderCacheRequest, NnsNodeProviderInfoReport, NnsNodeProviderInfoRequest,
    NnsNodeProviderListReport, NnsNodeProviderListRequest, NnsNodeProviderRow,
};
#[cfg(feature = "host")]
pub use model::{
    NnsNodeProviderHostError, NnsNodeProviderRefreshReport, NnsNodeProviderRefreshRequest,
};
#[cfg(feature = "host")]
pub use text::nns_node_provider_refresh_report_text;
pub use text::{
    nns_node_provider_info_report_text, nns_node_provider_list_report_text,
    nns_node_provider_list_report_verbose_text,
};

pub const DEFAULT_NNS_SOURCE_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "host")]
pub const DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
#[cfg(feature = "host")]
pub const NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
pub const NNS_NODE_PROVIDER_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
pub const NNS_NODE_PROVIDER_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const NNS_NODE_PROVIDER_CACHE_DIR: &str = "node-provider";
#[cfg(feature = "host")]
const NNS_NODE_PROVIDER_CACHE_FILE: &str = "providers.json";

#[cfg(feature = "host")]
impl_nns_mainnet_network_enforcer!(NnsNodeProviderHostError);

#[cfg(all(test, feature = "host"))]
mod tests;
