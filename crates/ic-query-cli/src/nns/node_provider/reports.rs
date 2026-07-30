use crate::nns::leaf;
use ic_query::nns::{
    NnsInventoryCacheRequest, NnsInventoryInfoRequest, NnsInventoryListRequest,
    NnsInventoryRefreshRequest,
    node_provider::{
        NnsNodeProviderHostError, NnsNodeProviderInfoReport, NnsNodeProviderListReport,
        NnsNodeProviderRefreshReport, build_nns_node_provider_info_report,
        build_nns_node_provider_list_report, nns_node_provider_cache_path,
        nns_node_provider_info_report_text, nns_node_provider_list_report_text,
        nns_node_provider_list_report_verbose_text, nns_node_provider_refresh_report_text,
        refresh_nns_node_provider_report,
    },
};

impl_nns_leaf_reports!(
    NnsNodeProviderReports,
    cache = NnsInventoryCacheRequest,
    list_request = NnsInventoryListRequest,
    info_request = NnsInventoryInfoRequest,
    refresh_request = NnsInventoryRefreshRequest,
    list_report = NnsNodeProviderListReport,
    info_report = NnsNodeProviderInfoReport,
    refresh_report = NnsNodeProviderRefreshReport,
    host_error = NnsNodeProviderHostError,
    build_list = build_nns_node_provider_list_report,
    build_info = build_nns_node_provider_info_report,
    refresh = refresh_nns_node_provider_report,
    cache_path = nns_node_provider_cache_path,
    list_text = nns_node_provider_list_report_text,
    list_verbose_text = nns_node_provider_list_report_verbose_text,
    info_text = nns_node_provider_info_report_text,
    refresh_text = nns_node_provider_refresh_report_text,
);
