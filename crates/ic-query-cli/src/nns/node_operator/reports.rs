use crate::nns::leaf;
use ic_query::nns::node_operator::{
    NnsNodeOperatorHostError, NnsNodeOperatorInfoReport, NnsNodeOperatorListReport,
    NnsNodeOperatorRefreshReport, build_nns_node_operator_info_report_with_progress,
    build_nns_node_operator_list_report_with_progress, nns_node_operator_info_report_text,
    nns_node_operator_list_report_text, nns_node_operator_list_report_verbose_text,
    nns_node_operator_refresh_report_text, refresh_nns_node_operator_report,
};

impl_nns_leaf_reports!(
    NnsNodeOperatorReports,
    list_report = NnsNodeOperatorListReport,
    info_report = NnsNodeOperatorInfoReport,
    refresh_report = NnsNodeOperatorRefreshReport,
    host_error = NnsNodeOperatorHostError,
    build_list = build_nns_node_operator_list_report_with_progress,
    build_info = build_nns_node_operator_info_report_with_progress,
    refresh = refresh_nns_node_operator_report,
    list_text = nns_node_operator_list_report_text,
    list_verbose_text = nns_node_operator_list_report_verbose_text,
    info_text = nns_node_operator_info_report_text,
    refresh_text = nns_node_operator_refresh_report_text,
);
