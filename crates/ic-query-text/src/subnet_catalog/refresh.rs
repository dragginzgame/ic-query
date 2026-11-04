use crate::text_value::field;
use ic_host_text::escape::terminal_text;

use crate::text_value::yes_no;
use ic_query::subnet_catalog::SubnetCatalogRefreshReport;

/// Render collection, validation, and catalog publication outcomes as text.
#[must_use]
pub fn subnet_catalog_refresh_report_text(report: &SubnetCatalogRefreshReport) -> String {
    [
        field("network", &report.network),
        field("catalog_path", &report.catalog_path),
        field("refresh_lock_path", &report.refresh_lock_path),
        field(
            "registry_canister_id",
            &(report.metadata.registry_canister_id),
        ),
        field("registry_version", &(report.metadata.registry_version)),
        field("assurance", &(report.metadata.assurance.as_str())),
        field(
            "source_endpoints",
            &(report
                .metadata
                .source_endpoints
                .iter()
                .map(|endpoint| terminal_text(endpoint))
                .collect::<Vec<_>>()
                .join(",")),
        ),
        field(
            "agreement_digest",
            &(report.metadata.agreement_digest.as_deref().unwrap_or("-")),
        ),
        field(
            "registry_query_call_count",
            &(report.metadata.registry_query_call_count),
        ),
        field("routing_source", &(report.metadata.routing_source.as_str())),
        field(
            "registry_record_count",
            &(report.metadata.registry_records.len()),
        ),
        field("catalog_digest", &(report.metadata.catalog_digest)),
        field("fetched_at", &report.metadata.fetched_at),
        field("fetched_by", &report.fetched_by),
        field("collector_version", &report.metadata.collector_version),
        field(
            "classification_schema_version",
            &(report.metadata.classification_schema_version),
        ),
        field(
            "classification_policy_digest",
            &(report.metadata.classification_policy_digest),
        ),
        field(
            "resolver_schema_version",
            &(report.metadata.resolver_schema_version),
        ),
        field("resolver_backend", &report.metadata.resolver_backend),
        field("dry_run", &(yes_no(report.dry_run))),
        field("wrote_catalog", &(yes_no(report.wrote_catalog))),
        field(
            "replaced_existing_catalog",
            &(yes_no(report.replaced_existing_catalog)),
        ),
        field("subnet_count", &(report.subnet_count)),
        field("routing_range_count", &(report.routing_range_count)),
    ]
    .join("\n")
}
