use crate::text_value::field;
use crate::text_value::table;
mod ranges;

use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

use crate::subnet_catalog::compact_principal;

use crate::text_value::yes_no;
use ic_query::subnet_catalog::SubnetCatalogListReport;

/// Render catalog provenance and a compact filtered Subnet table.
#[must_use]
pub fn subnet_catalog_list_report_text(
    report: &SubnetCatalogListReport,
    options: crate::TextOptions,
) -> String {
    let headers = [
        "SUBNET", "KIND", "SPEC", "GEO", "NODES", "CHG", "RANGES", "STALE",
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
            ] = crate::subnet_catalog::list::subnet_cells(subnet);
            [
                if options.verbose {
                    terminal_text(&subnet.subnet_principal).into_owned()
                } else {
                    compact_principal(&subnet.subnet_principal)
                },
                kind,
                specialization,
                geographic_scope,
                node_count,
                charges,
                range_count,
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
        report.metadata.registry_version,
        yes_no(report.catalog_stale),
        report.metadata.assurance.as_str(),
        report.cache_disposition.as_str(),
    ));
    if options.verbose {
        append_catalog_evidence(report, &mut lines);
    }
    if rows.is_empty() {
        lines.push("subnets: none".to_string());
        return lines.join("\n");
    }
    lines.push(String::new());
    lines.push(table(&headers, &rows, &alignments));
    ranges::append_range_lines(report, &mut lines, options.verbose);
    lines.join("\n")
}

fn subnet_cells(subnet: &ic_query::subnet_catalog::SubnetCatalogSubnetRow) -> [String; 6] {
    [
        subnet.classification.subnet_kind.as_str().to_string(),
        subnet
            .classification
            .subnet_specialization
            .as_str()
            .to_string(),
        subnet.classification.geographic_scope.as_str().to_string(),
        subnet
            .node_count
            .map_or_else(|| "unknown".to_string(), |count| count.to_string()),
        crate::text_value::yes_no(subnet.charges_apply_by_default).to_string(),
        subnet.range_count.to_string(),
    ]
}

fn append_catalog_evidence(report: &SubnetCatalogListReport, lines: &mut Vec<String>) {
    lines.push(field("catalog_path", &report.catalog_path));
    crate::subnet_catalog::append_catalog_authority(
        lines,
        &report.metadata,
        report.cache_disposition,
    );
    crate::subnet_catalog::append_catalog_policy(lines, &report.metadata);
    lines.push(field("stale_reason", &report.stale_reason));
}
