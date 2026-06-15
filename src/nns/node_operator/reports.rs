use crate::nns::{
    leaf,
    node_operator::report::{
        NnsNodeOperatorCacheRequest, NnsNodeOperatorHostError, NnsNodeOperatorInfoReport,
        NnsNodeOperatorInfoRequest, NnsNodeOperatorListReport, NnsNodeOperatorListRequest,
        NnsNodeOperatorRefreshReport, NnsNodeOperatorRefreshRequest,
        build_nns_node_operator_info_report, build_nns_node_operator_list_report,
        nns_node_operator_info_report_text, nns_node_operator_list_report_text,
        nns_node_operator_list_report_verbose_text, nns_node_operator_refresh_report_text,
        refresh_nns_node_operator_report,
    },
};

impl_nns_leaf_reports!(
    NnsNodeOperatorReports,
    cache = NnsNodeOperatorCacheRequest,
    list_request = NnsNodeOperatorListRequest,
    info_request = NnsNodeOperatorInfoRequest,
    refresh_request = NnsNodeOperatorRefreshRequest,
    list_report = NnsNodeOperatorListReport,
    info_report = NnsNodeOperatorInfoReport,
    refresh_report = NnsNodeOperatorRefreshReport,
    host_error = NnsNodeOperatorHostError,
    build_list = build_nns_node_operator_list_report,
    build_info = build_nns_node_operator_info_report,
    refresh = refresh_nns_node_operator_report,
    list_text = nns_node_operator_list_report_text,
    list_verbose_text = nns_node_operator_list_report_verbose_text,
    info_text = nns_node_operator_info_report_text,
    refresh_text = nns_node_operator_refresh_report_text,
);

impl_cached_leaf_requests!(
    NnsNodeOperatorCacheRequest,
    NnsNodeOperatorListRequest,
    NnsNodeOperatorInfoRequest,
    NnsNodeOperatorRefreshRequest
);
