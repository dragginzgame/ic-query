use crate::nns::leaf;
use ic_query::nns::{
    NnsInventoryCacheRequest, NnsInventoryInfoRequest, NnsInventoryListRequest,
    NnsInventoryRefreshRequest,
    node_operator::{
        NnsNodeOperatorHostError, NnsNodeOperatorInfoReport, NnsNodeOperatorListReport,
        NnsNodeOperatorRefreshReport, build_nns_node_operator_info_report,
        build_nns_node_operator_list_report, nns_node_operator_cache_path,
        nns_node_operator_info_report_text, nns_node_operator_list_report_text,
        nns_node_operator_list_report_verbose_text, nns_node_operator_refresh_report_text,
        refresh_nns_node_operator_report,
    },
};

impl_nns_leaf_reports!(
    NnsNodeOperatorReports,
    cache = NnsInventoryCacheRequest,
    list_request = NnsInventoryListRequest,
    info_request = NnsInventoryInfoRequest,
    refresh_request = NnsInventoryRefreshRequest,
    list_report = NnsNodeOperatorListReport,
    info_report = NnsNodeOperatorInfoReport,
    refresh_report = NnsNodeOperatorRefreshReport,
    host_error = NnsNodeOperatorHostError,
    build_list = build_nns_node_operator_list_report,
    build_info = build_nns_node_operator_info_report,
    refresh = refresh_nns_node_operator_report,
    cache_path = nns_node_operator_cache_path,
    list_text = nns_node_operator_list_report_text,
    list_verbose_text = nns_node_operator_list_report_verbose_text,
    info_text = nns_node_operator_info_report_text,
    refresh_text = nns_node_operator_refresh_report_text,
);
