use crate::nns::leaf;
use ic_query::nns::node_provider::{
    NnsNodeProviderHostError, NnsNodeProviderInfoReport, NnsNodeProviderListReport,
    NnsNodeProviderRefreshReport, build_nns_node_provider_info_report_with_progress,
    build_nns_node_provider_list_report_with_progress, nns_node_provider_info_report_text,
    nns_node_provider_list_report_text, nns_node_provider_list_report_verbose_text,
    nns_node_provider_refresh_report_text, refresh_nns_node_provider_report,
};

impl_nns_leaf_reports!(
    NnsNodeProviderReports,
    list_report = NnsNodeProviderListReport,
    info_report = NnsNodeProviderInfoReport,
    refresh_report = NnsNodeProviderRefreshReport,
    host_error = NnsNodeProviderHostError,
    build_list = build_nns_node_provider_list_report_with_progress,
    build_info = build_nns_node_provider_info_report_with_progress,
    refresh = refresh_nns_node_provider_report,
    list_text = nns_node_provider_list_report_text,
    list_verbose_text = nns_node_provider_list_report_verbose_text,
    info_text = nns_node_provider_info_report_text,
    refresh_text = nns_node_provider_refresh_report_text,
);
