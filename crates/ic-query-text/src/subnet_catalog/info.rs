use crate::text_value::field;

use crate::human_quantity::cycle_count_text;
use crate::text_value::yes_no;
use ic_query::subnet_catalog::SubnetCatalogInfoReport;

/// Render resolved subject, catalog provenance, and execution-rate details as text.
#[must_use]
pub fn subnet_catalog_info_report_text(report: &SubnetCatalogInfoReport) -> String {
    let mut lines = Vec::from(crate::text_value::report_fields!(report;
        input_principal,
        resolved_as,
        resolved_from,
        subnet_principal,
        registry_subnet_type = report.classification.registry_subnet_type,
        subnet_kind = report.classification.subnet_kind.as_str(),
        subnet_kind_source = report.classification.subnet_kind_source.as_str(),
        subnet_specialization = report.classification.subnet_specialization.as_str(),
        subnet_specialization_source = report.classification.subnet_specialization_source.as_str(),
        geographic_scope = report.classification.geographic_scope.as_str(),
        geographic_scope_source = report.classification.geographic_scope_source.as_str(),
        subnet_label = report.classification.subnet_label,
        subnet_label_source = report.classification.subnet_label_source.as_str(),
        node_count = report
                .node_count
                .map_or_else(|| "unknown".to_string(), |count| count.to_string()),
        charges_apply_to_subject = yes_no(report.charges_apply_to_subject),
        charge_applicability_reason,
    ));
    append_catalog_evidence(&mut lines, report);
    if let Some(canister) = &report.matched_canister_principal {
        lines.push(field("matched_canister_principal", &(canister)));
    }
    if let Some(range) = &report.matched_routing_range {
        lines.push(format!(
            "matched_routing_range: {}..{}",
            range.start_canister_id, range.end_canister_id
        ));
    }
    lines.push(field(
        "cycles_per_billion_instructions",
        &(report
            .cycles_per_billion_instructions
            .map_or_else(|| "not_applicable".to_string(), cycle_count_text)),
    ));
    if let Some(rate_source) = &report.rate_source {
        lines.push(field("rate_source", rate_source));
    }
    if let Some(formula_version) = &report.formula_version {
        lines.push(field("formula_version", formula_version));
    }
    lines.join("\n")
}

fn append_catalog_evidence(lines: &mut Vec<String>, report: &SubnetCatalogInfoReport) {
    lines.push(field(
        "registry_canister_id",
        &(report.metadata.registry_canister_id),
    ));
    lines.push(field(
        "registry_version",
        &(report.metadata.registry_version),
    ));
    crate::subnet_catalog::append_catalog_authority(
        lines,
        &report.metadata,
        report.cache_disposition,
    );
    lines.push(field(
        "catalog_schema_version",
        &(report.catalog_schema_version),
    ));
    lines.push(field("catalog_path", &report.catalog_path));
    lines.push(field("fetched_at", &report.metadata.fetched_at));
    lines.push(field("catalog_stale", &(yes_no(report.catalog_stale))));
    lines.push(field("stale_reason", &report.stale_reason));
    crate::subnet_catalog::append_catalog_policy(lines, &report.metadata);
}
