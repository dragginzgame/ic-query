use crate::nns::{
    leaf,
    node_provider::report::{
        NnsNodeProviderCacheRequest, NnsNodeProviderHostError, NnsNodeProviderInfoReport,
        NnsNodeProviderInfoRequest, NnsNodeProviderListReport, NnsNodeProviderListRequest,
        NnsNodeProviderRefreshReport, NnsNodeProviderRefreshRequest,
        build_nns_node_provider_info_report, build_nns_node_provider_list_report,
        nns_node_provider_info_report_text, nns_node_provider_list_report_text,
        nns_node_provider_list_report_verbose_text, nns_node_provider_refresh_report_text,
        refresh_nns_node_provider_report,
    },
};

impl_nns_leaf_reports!(
    NnsNodeProviderReports,
    cache = NnsNodeProviderCacheRequest,
    list_request = NnsNodeProviderListRequest,
    info_request = NnsNodeProviderInfoRequest,
    refresh_request = NnsNodeProviderRefreshRequest,
    list_report = NnsNodeProviderListReport,
    info_report = NnsNodeProviderInfoReport,
    refresh_report = NnsNodeProviderRefreshReport,
    host_error = NnsNodeProviderHostError,
    build_list = build_nns_node_provider_list_report,
    build_info = build_nns_node_provider_info_report,
    refresh = refresh_nns_node_provider_report,
    list_text = nns_node_provider_list_report_text,
    list_verbose_text = nns_node_provider_list_report_verbose_text,
    info_text = nns_node_provider_info_report_text,
    refresh_text = nns_node_provider_refresh_report_text,
);

impl_cached_leaf_cli_requests!(
    NnsNodeProviderCacheRequest,
    NnsNodeProviderListRequest,
    NnsNodeProviderInfoRequest
);
