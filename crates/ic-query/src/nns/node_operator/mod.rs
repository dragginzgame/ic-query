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
    DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT, NnsNodeOperatorCacheRequest,
    NnsNodeOperatorInfoReport, NnsNodeOperatorInfoRequest, NnsNodeOperatorListReport,
    NnsNodeOperatorListRequest, NnsNodeOperatorRow, nns_node_operator_info_report_text,
    nns_node_operator_list_report_text, nns_node_operator_list_report_verbose_text,
};
#[cfg(feature = "host")]
pub use report::{
    NnsNodeOperatorHostError, NnsNodeOperatorRefreshReport, NnsNodeOperatorRefreshRequest,
    build_nns_node_operator_info_report, build_nns_node_operator_list_report,
    nns_node_operator_refresh_report_text, refresh_nns_node_operator_report,
};

#[cfg(feature = "cli")]
pub(super) use run::run;
#[cfg(all(test, feature = "cli"))]
pub(super) use test_helpers::{
    node_operator_info_options, node_operator_info_usage, node_operator_list_options,
    node_operator_list_usage, node_operator_refresh_options, node_operator_refresh_usage,
    node_operator_usage,
};
