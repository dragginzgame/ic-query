//! NNS data-center inventory requests, reports, host builders.

mod report;

pub use report::{
    DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT, NnsDataCenterInfoReport, NnsDataCenterListReport,
    NnsDataCenterRow,
};
#[cfg(feature = "nns-host")]
pub use report::{
    NNS_DATA_CENTER_INFO_REPORT_SCHEMA_VERSION, NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION,
    build_nns_data_center_info_report, build_nns_data_center_info_report_with_source_and_progress,
    build_nns_data_center_list_report, build_nns_data_center_list_report_with_source_and_progress,
};
