mod info;
mod list;
#[cfg(feature = "host")]
use super::NnsDataCenterRefreshReport;

pub use info::nns_data_center_info_report_text;
pub use list::{nns_data_center_list_report_text, nns_data_center_list_report_verbose_text};

#[cfg(feature = "host")]
#[must_use]
pub fn nns_data_center_refresh_report_text(report: &NnsDataCenterRefreshReport) -> String {
    nns_leaf_refresh_report_text!(report, None, "data_center_count", data_center_count)
}
