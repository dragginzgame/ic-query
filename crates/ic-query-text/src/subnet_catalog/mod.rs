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
    lines.extend(crate::text_value::report_fields!(metadata;
        assurance = metadata.assurance.as_str(),
        cache_disposition = disposition.as_str(),
        catalog_digest,
        source_endpoints = metadata
                .source_endpoints
                .iter()
                .map(|endpoint| terminal_text(endpoint))
                .collect::<Vec<_>>()
                .join(","),
        agreement_digest = metadata.agreement_digest.as_deref().unwrap_or("-"),
        registry_query_call_count,
        routing_source = metadata.routing_source.as_str(),
        registry_record_count = metadata.registry_records.len(),
    ));
}

pub(super) fn append_catalog_policy(
    lines: &mut Vec<String>,
    metadata: &SubnetCatalogReportMetadata,
) {
    lines.extend(crate::text_value::report_fields!(metadata;
        collector_version,
        classification_schema_version,
        classification_policy_digest,
        resolver_schema_version,
        resolver_backend,
    ));
}

fn compact_principal(value: &str) -> String {
    crate::text_value::compact_text(value, 5)
}
