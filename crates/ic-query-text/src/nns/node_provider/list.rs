use crate::text_value::table;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

use crate::nns::render::identity;
use crate::nns::render::optional_node_count_text;
use ic_query::nns::node_provider::NnsNodeProviderListReport;

#[must_use]
/// Render the NNS node provider list report as human-facing text.
pub fn nns_node_provider_list_report_text(
    report: &NnsNodeProviderListReport,
    options: crate::TextOptions,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "node_providers: {} count {} fetched_at {}",
        terminal_text(&report.provenance.network),
        report.node_providers.len(),
        terminal_text(&report.provenance.fetched_at)
    ));
    if options.verbose {
        lines.extend(crate::nns::render::registry_provenance_lines(
            &report.provenance,
        ));
    }
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
                identity(&provider.node_provider_principal, options.verbose),
                optional_node_count_text(provider.node_count),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [Alignment::Left, Alignment::Right];
    lines.push(String::new());
    lines.push(table(&headers, &rows, &alignments));
    lines.join("\n")
}
