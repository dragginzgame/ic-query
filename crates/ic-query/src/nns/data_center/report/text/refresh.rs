use crate::nns::data_center::report::NnsDataCenterRefreshReport;

#[must_use]
pub fn nns_data_center_refresh_report_text(report: &NnsDataCenterRefreshReport) -> String {
    nns_leaf_refresh_report_text!(report, None, "data_center_count", data_center_count)
}
