use ic_host_text::escape::terminal_text;

use crate::text_value::yes_no;
use ic_query::subnet_catalog::SubnetCatalogRefreshReport;

/// Render collection, validation, and catalog publication outcomes as text.
#[must_use]
pub fn subnet_catalog_refresh_report_text(report: &SubnetCatalogRefreshReport) -> String {
    crate::text_value::report_fields!(report;
        network,
        catalog_path,
        refresh_lock_path,
        registry_canister_id = report.metadata.registry_canister_id,
        registry_version = report.metadata.registry_version,
        assurance = report.metadata.assurance.as_str(),
        source_endpoints = report
                .metadata
                .source_endpoints
                .iter()
                .map(|endpoint| terminal_text(endpoint))
                .collect::<Vec<_>>()
                .join(","),
        agreement_digest = report.metadata.agreement_digest.as_deref().unwrap_or("-"),
        registry_query_call_count = report.metadata.registry_query_call_count,
        routing_source = report.metadata.routing_source.as_str(),
        registry_record_count = report.metadata.registry_records.len(),
        catalog_digest = report.metadata.catalog_digest,
        fetched_at = report.metadata.fetched_at,
        fetched_by,
        collector_version = report.metadata.collector_version,
        classification_schema_version = report.metadata.classification_schema_version,
        classification_policy_digest = report.metadata.classification_policy_digest,
        resolver_schema_version = report.metadata.resolver_schema_version,
        resolver_backend = report.metadata.resolver_backend,
        dry_run = yes_no(report.dry_run),
        wrote_catalog = yes_no(report.wrote_catalog),
        replaced_existing_catalog = yes_no(report.replaced_existing_catalog),
        subnet_count,
        routing_range_count,
    )
    .join("\n")
}
