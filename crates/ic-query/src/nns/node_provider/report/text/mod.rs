mod info;
mod list;
#[cfg(feature = "host")]
use super::NnsNodeProviderRefreshReport;

pub use info::nns_node_provider_info_report_text;
pub use list::{nns_node_provider_list_report_text, nns_node_provider_list_report_verbose_text};

#[cfg(feature = "host")]
#[must_use]
pub fn nns_node_provider_refresh_report_text(report: &NnsNodeProviderRefreshReport) -> String {
    nns_leaf_refresh_report_text!(
        report,
        Some(&report.governance_canister_id),
        "node_provider_count",
        node_provider_count
    )
}
