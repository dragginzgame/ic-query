use crate::nns::{
    data_center::report::{
        NnsDataCenterCacheRequest, NnsDataCenterHostError, NnsDataCenterInfoReport,
        NnsDataCenterInfoRequest, NnsDataCenterListReport, NnsDataCenterListRequest,
        NnsDataCenterRefreshReport, NnsDataCenterRefreshRequest, build_nns_data_center_info_report,
        build_nns_data_center_list_report, nns_data_center_info_report_text,
        nns_data_center_list_report_text, nns_data_center_list_report_verbose_text,
        nns_data_center_refresh_report_text, refresh_nns_data_center_report,
    },
    leaf,
};

impl_nns_leaf_reports!(
    NnsDataCenterReports,
    cache = NnsDataCenterCacheRequest,
    list_request = NnsDataCenterListRequest,
    info_request = NnsDataCenterInfoRequest,
    refresh_request = NnsDataCenterRefreshRequest,
    list_report = NnsDataCenterListReport,
    info_report = NnsDataCenterInfoReport,
    refresh_report = NnsDataCenterRefreshReport,
    host_error = NnsDataCenterHostError,
    build_list = build_nns_data_center_list_report,
    build_info = build_nns_data_center_info_report,
    refresh = refresh_nns_data_center_report,
    list_text = nns_data_center_list_report_text,
    list_verbose_text = nns_data_center_list_report_verbose_text,
    info_text = nns_data_center_info_report_text,
    refresh_text = nns_data_center_refresh_report_text,
);

impl_cached_leaf_cli_requests!(
    NnsDataCenterCacheRequest,
    NnsDataCenterListRequest,
    NnsDataCenterInfoRequest
);
