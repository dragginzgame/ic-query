use crate::text_value::field;
mod info;
mod list;
mod refresh;

pub use crate::subnet_catalog::info::subnet_catalog_info_report_text;
pub use crate::subnet_catalog::list::subnet_catalog_list_report_text;
pub use crate::subnet_catalog::refresh::subnet_catalog_refresh_report_text;

use ic_host_text::escape::terminal_text;
use ic_query::subnet_catalog::CacheDisposition;
use ic_query::subnet_catalog::SubnetCatalogReportMetadata;

pub(super) fn append_catalog_authority(
    lines: &mut Vec<String>,
    metadata: &SubnetCatalogReportMetadata,
    disposition: CacheDisposition,
) {
    lines.extend([
        field("assurance", &(metadata.assurance.as_str())),
        field("cache_disposition", &(disposition.as_str())),
        field("catalog_digest", &metadata.catalog_digest),
        field(
            "source_endpoints",
            &(metadata
                .source_endpoints
                .iter()
                .map(|endpoint| terminal_text(endpoint))
                .collect::<Vec<_>>()
                .join(",")),
        ),
        field(
            "agreement_digest",
            &(metadata.agreement_digest.as_deref().unwrap_or("-")),
        ),
        field(
            "registry_query_call_count",
            &(metadata.registry_query_call_count),
        ),
        field("routing_source", &(metadata.routing_source.as_str())),
        field("registry_record_count", &(metadata.registry_records.len())),
    ]);
}

pub(super) fn append_catalog_policy(
    lines: &mut Vec<String>,
    metadata: &SubnetCatalogReportMetadata,
) {
    lines.extend([
        field("collector_version", &metadata.collector_version),
        field(
            "classification_schema_version",
            &(metadata.classification_schema_version),
        ),
        field(
            "classification_policy_digest",
            &metadata.classification_policy_digest,
        ),
        field(
            "resolver_schema_version",
            &(metadata.resolver_schema_version),
        ),
        field("resolver_backend", &metadata.resolver_backend),
    ]);
}

fn compact_principal(value: &str) -> String {
    crate::text_value::compact_text(value, 5)
}
