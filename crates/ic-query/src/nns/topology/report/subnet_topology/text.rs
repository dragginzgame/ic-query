use super::{NnsSubnetTopologyReport, NnsSubnetTopologyRow};
use crate::{
    nns::render::compact_text,
    table::{ColumnAlign, render_table},
};

const COMPACT_PRINCIPAL_CHARS: usize = 12;

/// Render an exact-version Subnet topology report as compact human-facing text.
#[must_use]
pub fn nns_subnet_topology_report_text(report: &NnsSubnetTopologyReport) -> String {
    let mut lines = vec![
        format!(
            "subnet_topology: {} registry_version {} subnets {} nodes {}",
            report.network, report.registry_version, report.subnet_count, report.node_count
        ),
        String::new(),
        render_subnet_provider_table(&report.subnets),
    ];
    lines.push(String::new());
    lines.push(format!("fetched_at: {}", report.fetched_at));
    lines.push(format!("source_endpoint: {}", report.source_endpoint));
    lines.join("\n")
}

fn render_subnet_provider_table(subnets: &[NnsSubnetTopologyRow]) -> String {
    let headers = ["SUBNET", "KIND", "NODES", "NODE_PROVIDER", "PROVIDER_NODES"];
    let mut rows = Vec::new();
    for subnet in subnets {
        if subnet.node_providers.is_empty() {
            rows.push([
                compact_text(&subnet.subnet_principal, COMPACT_PRINCIPAL_CHARS),
                subnet.subnet_kind.as_str().to_string(),
                subnet.node_count.to_string(),
                "-".to_string(),
                "0".to_string(),
            ]);
            continue;
        }
        for provider in &subnet.node_providers {
            rows.push([
                compact_text(&subnet.subnet_principal, COMPACT_PRINCIPAL_CHARS),
                subnet.subnet_kind.as_str().to_string(),
                subnet.node_count.to_string(),
                compact_text(&provider.node_provider_principal, COMPACT_PRINCIPAL_CHARS),
                provider.node_count.to_string(),
            ]);
        }
    }
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Left,
        ColumnAlign::Right,
    ];
    render_table(&headers, &rows, &alignments)
}
