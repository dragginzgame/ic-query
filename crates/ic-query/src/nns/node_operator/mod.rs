//! NNS node-operator inventory requests, reports, host builders.

mod report;

pub use report::{
    DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT, NnsNodeOperatorInfoReport,
    NnsNodeOperatorListReport, NnsNodeOperatorRow,
};
#[cfg(feature = "nns-host")]
pub use report::{
    NNS_NODE_OPERATOR_INFO_REPORT_SCHEMA_VERSION, NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION,
    build_nns_node_operator_info_report,
    build_nns_node_operator_info_report_with_source_and_progress,
    build_nns_node_operator_list_report,
    build_nns_node_operator_list_report_with_source_and_progress,
};
