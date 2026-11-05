mod info;
mod list;

pub use crate::nns::node_provider::info::nns_node_provider_info_report_text;
pub use crate::nns::node_provider::list::nns_node_provider_list_report_text;

/// Render the independent Registry and Governance refresh outcomes.
#[cfg(feature = "nns-host")]
#[must_use]
pub fn nns_node_provider_refresh_report_text(
    report: &ic_query::nns::node_provider::NnsNodeProviderRefreshReport,
) -> String {
    format!(
        "Registry inventory:\n{}\n\nGovernance provider metadata:\n{}",
        crate::nns::nns_inventory_refresh_report_text(&report.inventory),
        crate::nns::nns_inventory_refresh_report_text(&report.metadata)
    )
}
