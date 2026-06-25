use super::ranges::append_range_lines;
use crate::{
    nns::render::yes_no,
    subnet_catalog::SubnetCatalogListReport,
    table::{ColumnAlign, render_table},
};

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
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Left,
        ColumnAlign::Left,
    ];
    let mut lines = Vec::new();
    lines.push(format!("catalog_path: {}", report.catalog_path));
    lines.push(format!("stale_reason: {}", report.stale_reason));
    if rows.is_empty() {
        lines.push("subnets: none".to_string());
        return lines.join("\n");
    }
    lines.push(render_table(&headers, &rows, &alignments));
    append_range_lines(report, &mut lines);
    lines.join("\n")
}
