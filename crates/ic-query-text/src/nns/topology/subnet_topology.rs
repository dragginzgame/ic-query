use crate::text_value::field;
use crate::text_value::table;
use ic_host_text::table::Alignment;

use crate::nns::render::compact_text;
use ic_query::nns::topology::NnsSubnetTopologyReport;
use ic_query::nns::topology::NnsSubnetTopologyRow;

const COMPACT_PRINCIPAL_CHARS: usize = 12;

/// Render an exact-version Subnet topology report as compact human-facing text.
#[must_use]
pub fn nns_subnet_topology_report_text(report: &NnsSubnetTopologyReport) -> String {
    let mut lines = vec![
        format!(
            "subnet_topology: {} registry_version {} subnets {} nodes {}",
            report.network,
            report.registry_version,
            report.subnets.len(),
            report
                .node_count()
                .map_or_else(|_| "overflow".to_string(), |count| count.to_string())
        ),
        String::new(),
        render_subnet_provider_table(&report.subnets),
    ];
    lines.push(String::new());
    lines.push(field("fetched_at", &(report.fetched_at)));
    lines.push(field("source_endpoint", &(report.source_endpoint)));
    lines.join("\n")
}

fn render_subnet_provider_table(subnets: &[NnsSubnetTopologyRow]) -> String {
    let headers = ["SUBNET", "KIND", "NODES", "NODE_PROVIDER", "PROVIDER_NODES"];
    let mut rows = Vec::new();
    for subnet in subnets {
        let node_count = subnet
            .node_count()
            .map_or_else(|_| "overflow".to_string(), |count| count.to_string());
        if subnet.node_providers.is_empty() {
            rows.push([
                compact_text(&subnet.subnet_principal, COMPACT_PRINCIPAL_CHARS),
                subnet.subnet_kind.as_str().to_string(),
                node_count.clone(),
                "-".to_string(),
                "0".to_string(),
            ]);
            continue;
        }
        for provider in &subnet.node_providers {
            rows.push([
                compact_text(&subnet.subnet_principal, COMPACT_PRINCIPAL_CHARS),
                subnet.subnet_kind.as_str().to_string(),
                node_count.clone(),
                compact_text(&provider.node_provider_principal, COMPACT_PRINCIPAL_CHARS),
                provider.node_count.to_string(),
            ]);
        }
    }
    let alignments = [
        Alignment::Left,
        Alignment::Left,
        Alignment::Right,
        Alignment::Left,
        Alignment::Right,
    ];
    table(&headers, &rows, &alignments)
}
