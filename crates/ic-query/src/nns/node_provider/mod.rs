pub mod report;

#[cfg(feature = "cli")]
mod reports;
#[cfg(feature = "cli")]
mod run;
#[cfg(feature = "cli")]
mod spec;
#[cfg(all(test, feature = "cli"))]
mod test_helpers;

pub use report::{
    DEFAULT_NNS_SOURCE_ENDPOINT, NnsNodeProviderCacheRequest, NnsNodeProviderInfoReport,
    NnsNodeProviderInfoRequest, NnsNodeProviderListReport, NnsNodeProviderListRequest,
    NnsNodeProviderRow, nns_node_provider_info_report_text, nns_node_provider_list_report_text,
    nns_node_provider_list_report_verbose_text,
};
#[cfg(feature = "host")]
pub use report::{
    DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS, NnsNodeProviderHostError,
    NnsNodeProviderRefreshReport, NnsNodeProviderRefreshRequest,
    build_nns_node_provider_info_report, build_nns_node_provider_list_report,
    nns_node_provider_cache_path, nns_node_provider_refresh_lock_path,
    nns_node_provider_refresh_report_text, refresh_nns_node_provider_report,
};

#[cfg(feature = "cli")]
pub(super) use run::run;
#[cfg(all(test, feature = "cli"))]
pub(super) use test_helpers::{
    node_provider_info_options, node_provider_info_usage, node_provider_list_options,
    node_provider_list_usage, node_provider_refresh_options, node_provider_refresh_usage,
    node_provider_usage,
};
