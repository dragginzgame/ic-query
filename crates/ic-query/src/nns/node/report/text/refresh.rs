use crate::nns::node::report::NnsNodeRefreshReport;

#[must_use]
pub fn nns_node_refresh_report_text(report: &NnsNodeRefreshReport) -> String {
    nns_leaf_refresh_report_text!(report, None, "node_count", node_count)
}
