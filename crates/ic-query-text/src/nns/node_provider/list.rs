use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;
use ic_host_text::table::render;

use crate::nns::render::compact_text;
use crate::nns::render::optional_node_count_text;
use crate::nns::render::text_or_dash;
use ic_query::nns::node_provider::NnsNodeProviderListReport;

const COMPACT_PRINCIPAL_CHARS: usize = 5;

#[must_use]
/// Render the NNS node provider list report as human-facing text.
pub fn nns_node_provider_list_report_text(report: &NnsNodeProviderListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "node_providers: {} count {} fetched_at {}",
        terminal_text(&report.provenance.network),
        report.node_provider_count,
        terminal_text(&report.provenance.fetched_at)
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
    let alignments = [Alignment::Left, Alignment::Right];
    lines.push(String::new());
    lines.push(render(&headers, &rows, &alignments, REPORT_TABLE_OPTIONS));
    lines.join("\n")
}

#[must_use]
/// Render the NNS node provider list report verbose as human-facing text.
pub fn nns_node_provider_list_report_verbose_text(report: &NnsNodeProviderListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "source_endpoint: {}",
        terminal_text(&report.provenance.source_endpoint)
    ));
    lines.push(format!(
        "fetched_by: {}",
        terminal_text(&report.provenance.fetched_by)
    ));
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
                text_or_dash(provider.reward_account_hex.as_deref()),
                report.provenance.registry_version.to_string(),
                report.provenance.fetched_at.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        Alignment::Left,
        Alignment::Right,
        Alignment::Left,
        Alignment::Right,
        Alignment::Left,
    ];
    lines.push(String::new());
    lines.push(render(&headers, &rows, &alignments, REPORT_TABLE_OPTIONS));
    lines.join("\n")
}
