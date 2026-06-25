use super::ranges::append_compact_range_lines;
use crate::{
    nns::render::yes_no,
    subnet_catalog::{SubnetCatalogListReport, text::principal::compact_principal},
    table::{ColumnAlign, render_table},
};

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
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Left,
    ];
    let mut lines = Vec::new();
    lines.push(format!(
        "catalog: {} version {} stale {}",
        report.network,
        report.registry_version,
        yes_no(report.catalog_stale)
    ));
    if rows.is_empty() {
        lines.push("subnets: none".to_string());
        return lines.join("\n");
    }
    lines.push(render_table(&headers, &rows, &alignments));
    append_compact_range_lines(report, &mut lines);
    lines.join("\n")
}
