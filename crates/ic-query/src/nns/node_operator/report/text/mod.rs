mod info;
mod list;
#[cfg(feature = "nns-host")]
use super::NnsNodeOperatorRefreshReport;

pub use info::nns_node_operator_info_report_text;
pub use list::{nns_node_operator_list_report_text, nns_node_operator_list_report_verbose_text};

#[cfg(feature = "nns-host")]
#[must_use]
pub fn nns_node_operator_refresh_report_text(report: &NnsNodeOperatorRefreshReport) -> String {
    nns_leaf_refresh_report_text!(report, None, "node_operator_count", node_operator_count)
}
