use crate::nns::leaf;
use ic_query::nns::data_center::{
    NnsDataCenterHostError, NnsDataCenterInfoReport, NnsDataCenterListReport,
    NnsDataCenterRefreshReport, build_nns_data_center_info_report,
    build_nns_data_center_list_report, nns_data_center_cache_path,
    nns_data_center_info_report_text, nns_data_center_list_report_text,
    nns_data_center_list_report_verbose_text, nns_data_center_refresh_report_text,
    refresh_nns_data_center_report,
};

impl_nns_leaf_reports!(
    NnsDataCenterReports,
    list_report = NnsDataCenterListReport,
    info_report = NnsDataCenterInfoReport,
    refresh_report = NnsDataCenterRefreshReport,
    host_error = NnsDataCenterHostError,
    build_list = build_nns_data_center_list_report,
    build_info = build_nns_data_center_info_report,
    refresh = refresh_nns_data_center_report,
    cache_path = nns_data_center_cache_path,
    list_text = nns_data_center_list_report_text,
    list_verbose_text = nns_data_center_list_report_verbose_text,
    info_text = nns_data_center_info_report_text,
    refresh_text = nns_data_center_refresh_report_text,
);
