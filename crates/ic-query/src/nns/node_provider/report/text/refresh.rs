use crate::nns::node_provider::report::NnsNodeProviderRefreshReport;

#[must_use]
pub fn nns_node_provider_refresh_report_text(report: &NnsNodeProviderRefreshReport) -> String {
    nns_leaf_refresh_report_text!(
        report,
        Some(&report.governance_canister_id),
        "node_provider_count",
        node_provider_count
    )
}
