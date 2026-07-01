pub mod report;

#[cfg(feature = "cli")]
mod commands;
#[cfg(feature = "cli")]
mod options;
#[cfg(feature = "cli")]
mod run;

pub use report::{
    DEFAULT_NNS_NODE_SOURCE_ENDPOINT, NNS_NODE_SUBNET_KIND_APPLICATION,
    NNS_NODE_SUBNET_KIND_CLOUD_ENGINE, NNS_NODE_SUBNET_KIND_SYSTEM, NNS_NODE_SUBNET_KIND_UNKNOWN,
    NnsNodeCacheRequest, NnsNodeInfoReport, NnsNodeInfoRequest, NnsNodeListFilters,
    NnsNodeListReport, NnsNodeListRequest, NnsNodeRow, nns_node_info_report_text,
    nns_node_list_report_text, nns_node_list_report_verbose_text,
};
#[cfg(feature = "host")]
pub use report::{
    DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS, LiveNnsNodeSource, NnsNodeHostError,
    NnsNodeRefreshReport, NnsNodeRefreshRequest, NnsNodeSource, NnsNodeSourceRequest,
    build_nns_node_info_report, build_nns_node_info_report_with_source, build_nns_node_list_report,
    build_nns_node_list_report_with_source, nns_node_cache_path, nns_node_refresh_lock_path,
    nns_node_refresh_report_text, refresh_nns_node_report, refresh_nns_node_report_with_source,
};

#[cfg(all(test, feature = "cli"))]
pub(super) use commands::{node_info_usage, node_list_usage, node_refresh_usage, node_usage};
#[cfg(all(test, feature = "cli"))]
pub(super) use options::{node_info_options, node_list_options, node_refresh_options};
#[cfg(feature = "cli")]
pub(super) use run::run;
