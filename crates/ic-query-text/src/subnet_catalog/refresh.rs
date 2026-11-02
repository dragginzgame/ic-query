use ic_host_text::escape::terminal_text;

use crate::text_value::yes_no;
use ic_query::subnet_catalog::SubnetCatalogRefreshReport;

/// Render collection, validation, and catalog publication outcomes as text.
#[must_use]
pub fn subnet_catalog_refresh_report_text(report: &SubnetCatalogRefreshReport) -> String {
    [
        format!("network: {}", terminal_text(&report.network)),
        format!("catalog_path: {}", terminal_text(&report.catalog_path)),
        format!(
            "refresh_lock_path: {}",
            terminal_text(&report.refresh_lock_path)
        ),
        format!(
            "registry_canister_id: {}",
            report.metadata.registry_canister_id
        ),
        format!("registry_version: {}", report.metadata.registry_version),
        format!("assurance: {}", report.metadata.assurance.as_str()),
        format!(
            "source_endpoints: {}",
            report
                .metadata
                .source_endpoints
                .iter()
                .map(|endpoint| terminal_text(endpoint))
                .collect::<Vec<_>>()
                .join(",")
        ),
        format!(
            "agreement_digest: {}",
            report.metadata.agreement_digest.as_deref().unwrap_or("-")
        ),
        format!(
            "registry_query_call_count: {}",
            report.metadata.registry_query_call_count
        ),
        format!(
            "routing_source: {}",
            report.metadata.routing_source.as_str()
        ),
        format!(
            "registry_record_count: {}",
            report.metadata.registry_records.len()
        ),
        format!("catalog_digest: {}", report.metadata.catalog_digest),
        format!("fetched_at: {}", terminal_text(&report.metadata.fetched_at)),
        format!("fetched_by: {}", terminal_text(&report.fetched_by)),
        format!(
            "collector_version: {}",
            terminal_text(&report.metadata.collector_version)
        ),
        format!(
            "classification_schema_version: {}",
            report.metadata.classification_schema_version
        ),
        format!(
            "classification_policy_digest: {}",
            report.metadata.classification_policy_digest
        ),
        format!(
            "resolver_schema_version: {}",
            report.metadata.resolver_schema_version
        ),
        format!(
            "resolver_backend: {}",
            terminal_text(&report.metadata.resolver_backend)
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
