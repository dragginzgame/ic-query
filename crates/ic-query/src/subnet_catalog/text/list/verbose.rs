use super::ranges::append_range_lines;
use crate::{
    subnet_catalog::SubnetCatalogListReport,
    table::{ColumnAlign, render_table},
    text_value::{sanitize_text, yes_no},
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
    append_catalog_evidence(report, &mut lines);
    if rows.is_empty() {
        lines.push("subnets: none".to_string());
        return lines.join("\n");
    }
    lines.push(String::new());
    lines.push(render_table(&headers, &rows, &alignments));
    append_range_lines(report, &mut lines);
    lines.join("\n")
}

fn append_catalog_evidence(report: &SubnetCatalogListReport, lines: &mut Vec<String>) {
    lines.push(format!(
        "catalog_path: {}",
        sanitize_text(&report.catalog_path)
    ));
    lines.push(format!("assurance: {}", report.assurance.as_str()));
    lines.push(format!(
        "cache_disposition: {}",
        report.cache_disposition.as_str()
    ));
    lines.push(format!(
        "catalog_digest: {}",
        sanitize_text(&report.catalog_digest)
    ));
    lines.push(format!(
        "source_endpoints: {}",
        report
            .source_endpoints
            .iter()
            .map(|endpoint| sanitize_text(endpoint))
            .collect::<Vec<_>>()
            .join(",")
    ));
    lines.push(format!(
        "agreement_digest: {}",
        report.agreement_digest.as_deref().unwrap_or("-")
    ));
    lines.push(format!(
        "registry_query_call_count: {}",
        report.registry_query_call_count
    ));
    lines.push(format!(
        "collector_version: {}",
        sanitize_text(&report.collector_version)
    ));
    lines.push(format!(
        "classification_schema_version: {}",
        report.classification_schema_version
    ));
    lines.push(format!(
        "classification_policy_digest: {}",
        sanitize_text(&report.classification_policy_digest)
    ));
    lines.push(format!(
        "resolver_schema_version: {}",
        report.resolver_schema_version
    ));
    lines.push(format!(
        "resolver_backend: {}",
        sanitize_text(&report.resolver_backend)
    ));
    lines.push(format!(
        "stale_reason: {}",
        sanitize_text(&report.stale_reason)
    ));
}
