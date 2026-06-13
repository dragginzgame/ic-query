use super::{SubnetCatalogInfoReport, SubnetCatalogListReport, SubnetCatalogRefreshReport};
use crate::{
    nns::render::yes_no,
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

#[must_use]
pub fn subnet_catalog_info_report_text(report: &SubnetCatalogInfoReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!("input_principal: {}", report.input_principal));
    lines.push(format!("resolved_as: {}", report.resolved_as));
    lines.push(format!("resolved_from: {}", report.resolved_from));
    lines.push(format!("subnet_principal: {}", report.subnet_principal));
    lines.push(format!("subnet_kind: {}", report.subnet_kind.as_str()));
    lines.push(format!(
        "subnet_kind_source: {}",
        report.subnet_kind_source.as_str()
    ));
    lines.push(format!(
        "subnet_specialization: {}",
        report.subnet_specialization.as_str()
    ));
    lines.push(format!(
        "subnet_specialization_source: {}",
        report.subnet_specialization_source.as_str()
    ));
    lines.push(format!(
        "geographic_scope: {}",
        report.geographic_scope.as_str()
    ));
    lines.push(format!(
        "geographic_scope_source: {}",
        report.geographic_scope_source.as_str()
    ));
    lines.push(format!("subnet_label: {}", report.subnet_label));
    lines.push(format!(
        "subnet_label_source: {}",
        report.subnet_label_source.as_str()
    ));
    lines.push(format!(
        "node_count: {}",
        report
            .node_count
            .map_or_else(|| "unknown".to_string(), |count| count.to_string())
    ));
    lines.push(format!(
        "charges_apply_to_subject: {}",
        yes_no(report.charges_apply_to_subject)
    ));
    lines.push(format!(
        "charge_applicability_reason: {}",
        report.charge_applicability_reason
    ));
    lines.push(format!(
        "registry_canister_id: {}",
        report.registry_canister_id
    ));
    lines.push(format!("registry_version: {}", report.registry_version));
    lines.push(format!(
        "catalog_schema_version: {}",
        report.catalog_schema_version
    ));
    lines.push(format!("catalog_path: {}", report.catalog_path));
    lines.push(format!("fetched_at: {}", report.fetched_at));
    lines.push(format!("catalog_stale: {}", yes_no(report.catalog_stale)));
    lines.push(format!("stale_reason: {}", report.stale_reason));
    lines.push(format!("resolver_backend: {}", report.resolver_backend));
    if let Some(canister) = &report.matched_canister_principal {
        lines.push(format!("matched_canister_principal: {canister}"));
    }
    if let Some(range) = &report.matched_routing_range {
        lines.push(format!(
            "matched_routing_range: {}..{}",
            range.start_canister_id, range.end_canister_id
        ));
    }
    lines.push(format!(
        "cycles_per_billion_instructions: {}",
        report
            .cycles_per_billion_instructions
            .map_or_else(|| "not_applicable".to_string(), |cycles| cycles.to_string())
    ));
    if let Some(rate_source) = &report.rate_source {
        lines.push(format!("rate_source: {rate_source}"));
    }
    if let Some(formula_version) = &report.formula_version {
        lines.push(format!("formula_version: {formula_version}"));
    }
    lines.join("\n")
}

#[must_use]
pub fn subnet_catalog_refresh_report_text(report: &SubnetCatalogRefreshReport) -> String {
    [
        format!("network: {}", report.network),
        format!("catalog_path: {}", report.catalog_path),
        format!("refresh_lock_path: {}", report.refresh_lock_path),
        format!("registry_canister_id: {}", report.registry_canister_id),
        format!("registry_version: {}", report.registry_version),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
        format!("fetched_by: {}", report.fetched_by),
        format!("dry_run: {}", yes_no(report.dry_run)),
        format!("wrote_catalog: {}", yes_no(report.wrote_catalog)),
        format!(
            "replaced_existing_catalog: {}",
            yes_no(report.replaced_existing_catalog)
        ),
        format!("subnet_count: {}", report.subnet_count),
        format!("routing_range_count: {}", report.routing_range_count),
    ]
    .join("\n")
}

fn append_range_lines(report: &SubnetCatalogListReport, lines: &mut Vec<String>) {
    for subnet in &report.subnets {
        if subnet.ranges.is_empty() {
            continue;
        }
        lines.push(format!("ranges for {}:", subnet.subnet_principal));
        for range in &subnet.ranges {
            lines.push(format!(
                "  {}..{}",
                range.start_canister_id, range.end_canister_id
            ));
        }
        if subnet.ranges_shown < subnet.range_count {
            lines.push(format!(
                "  showing {} of {} ranges; use --range-limit or --format json",
                subnet.ranges_shown, subnet.range_count
            ));
        }
    }
}

fn append_compact_range_lines(report: &SubnetCatalogListReport, lines: &mut Vec<String>) {
    for subnet in &report.subnets {
        if subnet.ranges.is_empty() {
            continue;
        }
        lines.push(format!(
            "ranges for {}:",
            compact_principal(&subnet.subnet_principal)
        ));
        for range in &subnet.ranges {
            lines.push(format!(
                "  {}..{}",
                compact_principal(&range.start_canister_id),
                compact_principal(&range.end_canister_id)
            ));
        }
        if subnet.ranges_shown < subnet.range_count {
            lines.push(format!(
                "  showing {} of {} ranges; use --range-limit or --format json",
                subnet.ranges_shown, subnet.range_count
            ));
        }
    }
}

pub fn compact_principal(value: &str) -> String {
    value.chars().take(5).collect()
}
