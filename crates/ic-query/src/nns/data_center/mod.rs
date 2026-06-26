pub mod report;

#[cfg(feature = "cli")]
mod reports;
#[cfg(feature = "cli")]
mod run;
#[cfg(feature = "cli")]
mod spec;
#[cfg(all(test, feature = "cli"))]
mod test_helpers;

pub use report::{
    DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT, NnsDataCenterCacheRequest, NnsDataCenterInfoReport,
    NnsDataCenterInfoRequest, NnsDataCenterListReport, NnsDataCenterListRequest, NnsDataCenterRow,
    nns_data_center_info_report_text, nns_data_center_list_report_text,
    nns_data_center_list_report_verbose_text,
};
#[cfg(feature = "host")]
pub use report::{
    NnsDataCenterHostError, NnsDataCenterRefreshReport, NnsDataCenterRefreshRequest,
    build_nns_data_center_info_report, build_nns_data_center_list_report,
    nns_data_center_refresh_report_text, refresh_nns_data_center_report,
};

#[cfg(feature = "cli")]
pub(super) use run::run;
#[cfg(all(test, feature = "cli"))]
pub(super) use test_helpers::{
    data_center_info_options, data_center_info_usage, data_center_list_options,
    data_center_list_usage, data_center_refresh_options, data_center_refresh_usage,
    data_center_usage,
};
