use crate::{
    nns::node_provider::report::NnsNodeProviderListReport,
    table::{ColumnAlign, render_table},
};

const COMPACT_PRINCIPAL_CHARS: usize = 5;

#[must_use]
pub fn nns_node_provider_list_report_text(report: &NnsNodeProviderListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "node_providers: {} count {} fetched_at {}",
        report.network, report.node_provider_count, report.fetched_at
    ));
    if report.node_providers.is_empty() {
        lines.push("node providers: none".to_string());
        return lines.join("\n");
    }

    let headers = ["NODE_PROVIDER", "NODES"];
    let rows = report
        .node_providers
        .iter()
        .map(|provider| {
            [
                compact_text(&provider.node_provider_principal, COMPACT_PRINCIPAL_CHARS),
                optional_node_count_text(provider.node_count),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [ColumnAlign::Left, ColumnAlign::Right];
    lines.push(render_table(&headers, &rows, &alignments));
    lines.join("\n")
}

fn compact_text(value: &str, chars: usize) -> String {
    value.chars().take(chars).collect()
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

#[must_use]
pub fn nns_node_provider_list_report_verbose_text(report: &NnsNodeProviderListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!("source_endpoint: {}", report.source_endpoint));
    lines.push(format!("fetched_by: {}", report.fetched_by));
    if report.node_providers.is_empty() {
        lines.push("node providers: none".to_string());
        return lines.join("\n");
    }

    let headers = [
        "NODE_PROVIDER",
        "NODES",
        "REWARD_ACCOUNT",
        "REGISTRY_VERSION",
        "FETCHED_AT",
    ];
    let rows = report
        .node_providers
        .iter()
        .map(|provider| {
            [
                provider.node_provider_principal.clone(),
                optional_node_count_text(provider.node_count),
                text_or_dash(provider.reward_account_hex.as_deref()).to_string(),
                report.registry_version.to_string(),
                report.fetched_at.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Left,
    ];
    lines.push(render_table(&headers, &rows, &alignments));
    lines.join("\n")
}
