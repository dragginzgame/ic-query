use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::{
    escape::terminal_text,
    table::{Alignment, render},
};

use super::ranges::append_range_lines;
use crate::{subnet_catalog::SubnetCatalogListReport, text_value::yes_no};

/// Render catalog provenance and expanded filtered Subnet and routing tables.
#[must_use]
pub fn subnet_catalog_list_report_verbose_text(report: &SubnetCatalogListReport) -> String {
    let headers = [
        "SUBNET",
        "KIND",
        "SPECIALIZATION",
        "GEO",
        "NODES",
        "CHARGES",
        "RANGES",
        "VERSION",
        "FETCHED_AT",
        "STALE",
    ];
    let rows = report
        .subnets
        .iter()
        .map(|subnet| {
            [
                subnet.subnet_principal.clone(),
                subnet.subnet_kind.as_str().to_string(),
                subnet.subnet_specialization.as_str().to_string(),
                subnet.geographic_scope.as_str().to_string(),
                subnet
                    .node_count
                    .map_or_else(|| "unknown".to_string(), |count| count.to_string()),
                yes_no(subnet.charges_apply_by_default).to_string(),
                subnet.range_count.to_string(),
                report.registry_version.to_string(),
                report.fetched_at.clone(),
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
        Alignment::Right,
        Alignment::Left,
        Alignment::Left,
    ];
    let mut lines = Vec::new();
    append_catalog_evidence(report, &mut lines);
    if rows.is_empty() {
        lines.push("subnets: none".to_string());
        return lines.join("\n");
    }
    lines.push(String::new());
    lines.push(render(&headers, &rows, &alignments, REPORT_TABLE_OPTIONS));
    append_range_lines(report, &mut lines);
    lines.join("\n")
}

fn append_catalog_evidence(report: &SubnetCatalogListReport, lines: &mut Vec<String>) {
    lines.push(format!(
        "catalog_path: {}",
        terminal_text(&report.catalog_path)
    ));
    crate::subnet_catalog::text::append_catalog_authority!(lines, report);
    crate::subnet_catalog::text::append_catalog_policy!(lines, report);
    lines.push(format!(
        "stale_reason: {}",
        terminal_text(&report.stale_reason)
    ));
}
