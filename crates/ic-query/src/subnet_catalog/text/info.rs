use crate::{
    human_quantity::cycle_count_text,
    subnet_catalog::SubnetCatalogInfoReport,
    text_value::{sanitize_text, yes_no},
};

#[must_use]
pub fn subnet_catalog_info_report_text(report: &SubnetCatalogInfoReport) -> String {
    let safe_line = |label: &str, value: &str| format!("{label}: {}", sanitize_text(value));
    let mut lines = Vec::new();
    lines.push(safe_line("input_principal", &report.input_principal));
    lines.push(safe_line("resolved_as", &report.resolved_as));
    lines.push(safe_line("resolved_from", &report.resolved_from));
    lines.push(format!("subnet_principal: {}", report.subnet_principal));
    lines.push(format!(
        "registry_subnet_type: {}",
        report.registry_subnet_type
    ));
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
    lines.push(safe_line("subnet_label", &report.subnet_label));
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
        sanitize_text(&report.charge_applicability_reason)
    ));
    append_catalog_evidence(&mut lines, report);
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
            .map_or_else(|| "not_applicable".to_string(), cycle_count_text)
    ));
    if let Some(rate_source) = &report.rate_source {
        lines.push(safe_line("rate_source", rate_source));
    }
    if let Some(formula_version) = &report.formula_version {
        lines.push(safe_line("formula_version", formula_version));
    }
    lines.join("\n")
}

fn append_catalog_evidence(lines: &mut Vec<String>, report: &SubnetCatalogInfoReport) {
    lines.push(format!(
        "registry_canister_id: {}",
        report.registry_canister_id
    ));
    lines.push(format!("registry_version: {}", report.registry_version));
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
        "catalog_schema_version: {}",
        report.catalog_schema_version
    ));
    lines.push(format!(
        "catalog_path: {}",
        sanitize_text(&report.catalog_path)
    ));
    lines.push(format!("fetched_at: {}", sanitize_text(&report.fetched_at)));
    lines.push(format!("catalog_stale: {}", yes_no(report.catalog_stale)));
    lines.push(format!(
        "stale_reason: {}",
        sanitize_text(&report.stale_reason)
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
}
