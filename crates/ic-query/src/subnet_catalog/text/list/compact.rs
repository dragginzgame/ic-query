use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::{
    escape::terminal_text,
    table::{Alignment, render},
};

use super::ranges::append_compact_range_lines;
use crate::{
    subnet_catalog::{SubnetCatalogListReport, text::principal::compact_principal},
    text_value::yes_no,
};

/// Render catalog provenance and a compact filtered Subnet table.
#[must_use]
pub fn subnet_catalog_list_report_text(report: &SubnetCatalogListReport) -> String {
    let headers = [
        "SUBNET", "KIND", "SPEC", "GEO", "NODES", "CHG", "RANGES", "STALE",
    ];
    let rows = report
        .subnets
        .iter()
        .map(|subnet| {
            [
                compact_principal(&subnet.subnet_principal),
                subnet.subnet_kind.as_str().to_string(),
                subnet.subnet_specialization.as_str().to_string(),
                subnet.geographic_scope.as_str().to_string(),
                subnet
                    .node_count
                    .map_or_else(|| "unknown".to_string(), |count| count.to_string()),
                yes_no(subnet.charges_apply_by_default).to_string(),
                subnet.range_count.to_string(),
                yes_no(report.catalog_stale).to_string(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        Alignment::Left,
        Alignment::Left,
        Alignment::Left,
        Alignment::Left,
        Alignment::Right,
        Alignment::Left,
        Alignment::Right,
        Alignment::Left,
    ];
    let mut lines = Vec::new();
    lines.push(format!(
        "catalog: {} version {} stale {} assurance {} cache {}",
        terminal_text(&report.network),
        report.registry_version,
        yes_no(report.catalog_stale),
        report.assurance.as_str(),
        report.cache_disposition.as_str(),
    ));
    if rows.is_empty() {
        lines.push("subnets: none".to_string());
        return lines.join("\n");
    }
    lines.push(String::new());
    lines.push(render(&headers, &rows, &alignments, REPORT_TABLE_OPTIONS));
    append_compact_range_lines(report, &mut lines);
    lines.join("\n")
}
