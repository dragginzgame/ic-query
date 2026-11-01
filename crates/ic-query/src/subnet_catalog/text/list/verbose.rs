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
            let [
                kind,
                specialization,
                geographic_scope,
                node_count,
                charges,
                range_count,
            ] = super::subnet_cells(subnet);
            [
                subnet.subnet_principal.clone(),
                kind,
                specialization,
                geographic_scope,
                node_count,
                charges,
                range_count,
                report.metadata.registry_version.to_string(),
                report.metadata.fetched_at.clone(),
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
    crate::subnet_catalog::text::append_catalog_authority(
        lines,
        &report.metadata,
        report.cache_disposition,
    );
    crate::subnet_catalog::text::append_catalog_policy(lines, &report.metadata);
    lines.push(format!(
        "stale_reason: {}",
        terminal_text(&report.stale_reason)
    ));
}
