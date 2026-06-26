use crate::nns::node_operator::report::NnsNodeOperatorInfoReport;

#[must_use]
pub fn nns_node_operator_info_report_text(report: &NnsNodeOperatorInfoReport) -> String {
    [
        format!("input: {}", report.input),
        format!("resolved_from: {}", report.resolved_from),
        format!(
            "node_operator_principal: {}",
            report.node_operator_principal
        ),
        format!(
            "node_provider_principal: {}",
            report.node_provider_principal
        ),
        format!(
            "node_count: {}",
            optional_node_count_text(report.node_count)
        ),
        format!("node_allowance: {}", report.node_allowance),
        format!(
            "data_center_id: {}",
            text_or_dash(Some(&report.data_center_id))
        ),
        format!("registry_canister_id: {}", report.registry_canister_id),
        format!("registry_version: {}", report.registry_version),
        format!("network: {}", report.network),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
        format!("fetched_by: {}", report.fetched_by),
    ]
    .join("\n")
}

fn optional_node_count_text(value: Option<u32>) -> String {
    value.map_or_else(|| "unknown".to_string(), |count| count.to_string())
}

const fn text_or_dash(value: Option<&str>) -> &str {
    match value {
        Some(text) if !text.is_empty() => text,
        _ => "-",
    }
}
