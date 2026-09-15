use crate::{
    subnet_catalog::SubnetCatalogRefreshReport,
    text_value::{sanitize_text, yes_no},
};

#[must_use]
pub fn subnet_catalog_refresh_report_text(report: &SubnetCatalogRefreshReport) -> String {
    [
        format!("network: {}", sanitize_text(&report.network)),
        format!("catalog_path: {}", sanitize_text(&report.catalog_path)),
        format!(
            "refresh_lock_path: {}",
            sanitize_text(&report.refresh_lock_path)
        ),
        format!("registry_canister_id: {}", report.registry_canister_id),
        format!("registry_version: {}", report.registry_version),
        format!("assurance: {}", report.assurance.as_str()),
        format!(
            "source_endpoints: {}",
            report
                .source_endpoints
                .iter()
                .map(|endpoint| sanitize_text(endpoint))
                .collect::<Vec<_>>()
                .join(",")
        ),
        format!(
            "agreement_digest: {}",
            report.agreement_digest.as_deref().unwrap_or("-")
        ),
        format!(
            "registry_query_call_count: {}",
            report.registry_query_call_count
        ),
        format!("routing_source: {}", report.routing_source.as_str()),
        format!("registry_record_count: {}", report.registry_records.len()),
        format!("catalog_digest: {}", report.catalog_digest),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!("fetched_by: {}", sanitize_text(&report.fetched_by)),
        format!(
            "collector_version: {}",
            sanitize_text(&report.collector_version)
        ),
        format!(
            "classification_schema_version: {}",
            report.classification_schema_version
        ),
        format!(
            "classification_policy_digest: {}",
            report.classification_policy_digest
        ),
        format!(
            "resolver_schema_version: {}",
            report.resolver_schema_version
        ),
        format!(
            "resolver_backend: {}",
            sanitize_text(&report.resolver_backend)
        ),
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
