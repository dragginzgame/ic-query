//! NNS data-center inventory requests, reports, host builders, and renderers.

mod report;

#[cfg(feature = "host")]
pub use report::{
    DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS, NNS_DATA_CENTER_INFO_REPORT_SCHEMA_VERSION,
    NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION, NNS_DATA_CENTER_REFRESH_REPORT_SCHEMA_VERSION,
    NnsDataCenterHostError, NnsDataCenterRefreshReport, NnsDataCenterSource,
    build_nns_data_center_info_report, build_nns_data_center_info_report_with_source,
    build_nns_data_center_list_report, build_nns_data_center_list_report_with_source,
    nns_data_center_cache_path, nns_data_center_refresh_lock_path,
    nns_data_center_refresh_report_text, refresh_nns_data_center_report,
    refresh_nns_data_center_report_with_source,
};
pub use report::{
    DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT, NnsDataCenterInfoReport, NnsDataCenterListReport,
    NnsDataCenterRow, nns_data_center_info_report_text, nns_data_center_list_report_text,
    nns_data_center_list_report_verbose_text,
};
