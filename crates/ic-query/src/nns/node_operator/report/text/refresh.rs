use crate::nns::node_operator::report::NnsNodeOperatorRefreshReport;

#[must_use]
pub fn nns_node_operator_refresh_report_text(report: &NnsNodeOperatorRefreshReport) -> String {
    nns_leaf_refresh_report_text!(report, None, "node_operator_count", node_operator_count)
}
