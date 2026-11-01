mod info;
mod list;
mod principal;
mod refresh;

pub use info::subnet_catalog_info_report_text;
pub use list::{subnet_catalog_list_report_text, subnet_catalog_list_report_verbose_text};
pub use refresh::subnet_catalog_refresh_report_text;

use crate::subnet_catalog::{CacheDisposition, SubnetCatalogReportMetadata};
use ic_host_text::escape::terminal_text;

pub(super) fn append_catalog_authority(
    lines: &mut Vec<String>,
    metadata: &SubnetCatalogReportMetadata,
    disposition: CacheDisposition,
) {
    lines.extend([
        format!("assurance: {}", metadata.assurance.as_str()),
        format!("cache_disposition: {}", disposition.as_str()),
        format!(
            "catalog_digest: {}",
            terminal_text(&metadata.catalog_digest)
        ),
        format!(
            "source_endpoints: {}",
            metadata
                .source_endpoints
                .iter()
                .map(|endpoint| terminal_text(endpoint))
                .collect::<Vec<_>>()
                .join(",")
        ),
        format!(
            "agreement_digest: {}",
            metadata.agreement_digest.as_deref().unwrap_or("-")
        ),
        format!(
            "registry_query_call_count: {}",
            metadata.registry_query_call_count
        ),
        format!("routing_source: {}", metadata.routing_source.as_str()),
        format!("registry_record_count: {}", metadata.registry_records.len()),
    ]);
}

pub(super) fn append_catalog_policy(
    lines: &mut Vec<String>,
    metadata: &SubnetCatalogReportMetadata,
) {
    lines.extend([
        format!(
            "collector_version: {}",
            terminal_text(&metadata.collector_version)
        ),
        format!(
            "classification_schema_version: {}",
            metadata.classification_schema_version
        ),
        format!(
            "classification_policy_digest: {}",
            terminal_text(&metadata.classification_policy_digest)
        ),
        format!(
            "resolver_schema_version: {}",
            metadata.resolver_schema_version
        ),
        format!(
            "resolver_backend: {}",
            terminal_text(&metadata.resolver_backend)
        ),
    ]);
}
