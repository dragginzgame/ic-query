use crate::nns::node_provider::report::NnsNodeProviderInfoReport;

#[must_use]
pub fn nns_node_provider_info_report_text(report: &NnsNodeProviderInfoReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!("input: {}", report.input));
    lines.push(format!("resolved_from: {}", report.resolved_from));
    lines.push(format!(
        "node_provider_principal: {}",
        report.node_provider_principal
    ));
    lines.push(format!(
        "node_count: {}",
        optional_node_count_text(report.node_count)
    ));
    lines.push(format!(
        "reward_account_hex: {}",
        text_or_dash(report.reward_account_hex.as_deref())
    ));
    lines.push(format!(
        "governance_canister_id: {}",
        report.governance_canister_id
    ));
    lines.push(format!(
        "registry_canister_id: {}",
        report.registry_canister_id
    ));
    lines.push(format!("registry_version: {}", report.registry_version));
    lines.push(format!("network: {}", report.network));
    lines.push(format!("fetched_at: {}", report.fetched_at));
    lines.push(format!("source_endpoint: {}", report.source_endpoint));
    lines.push(format!("fetched_by: {}", report.fetched_by));
    lines.join("\n")
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
